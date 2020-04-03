// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2019-2020 Normation SAS

use crate::{error::Error, hashing::Hash};
use openssl::{stack::Stack, x509::X509};
use serde::{
    de::{Deserializer, Error as SerdeError, Visitor},
    Deserialize, Serialize,
};
use serde_json;
use std::{
    collections::{HashMap, HashSet},
    fmt,
    fs::{read, read_to_string},
    path::Path,
    str::FromStr,
};
use tracing::{error, info, trace, warn};

pub type NodeId = String;
pub type NodeIdRef = str;
pub type Host = String;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AgentFeature {
    CfengineRemoteRun,
}

// Not parsed for now
pub type AgentVersion = String;

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum AgentName {
    // Nova is not supposed to exist anymore
    CfengineCommunity,
    Dsc,
    Unknown(String),
}

impl AgentName {
    fn deserialize<'de, D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct V;

        impl<'de2> Visitor<'de2> for V {
            type Value = AgentName;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing the agent name")
            }

            fn visit_str<E>(self, v: &str) -> Result<AgentName, E>
            where
                E: SerdeError,
            {
                Ok(match v {
                    "cfengine-community" => AgentName::CfengineCommunity,
                    "dsc" => AgentName::Dsc,
                    _ => AgentName::Unknown(v.to_string()),
                })
            }
        }

        d.deserialize_str(V)
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct AgentInfo {
    #[serde(deserialize_with = "AgentName::deserialize")]
    name: AgentName,
    version: AgentVersion,
}

impl AgentInfo {
    pub fn has_feature(&self, feature: AgentFeature) -> bool {
        match feature {
            AgentFeature::CfengineRemoteRun => match self.name {
                // Remote-run on Windows would requires changes in relayd
                // so we know it is not possible at this point
                AgentName::Dsc => false,
                _ => true,
            },
        }
    }
}

#[derive(Deserialize, Default)]
struct Info {
    hostname: Host,
    #[serde(rename = "policy-server")]
    policy_server: NodeId,
    #[serde(rename = "key-hash")]
    key_hash: Hash,
    agents: Option<Vec<AgentInfo>>,
    #[serde(skip)]
    // Can be empty when not on a root server or no known certificates for
    // a node
    certificates: Option<Stack<X509>>,
}

impl Info {
    fn add_certificate(&mut self, cert: X509) -> Result<(), Error> {
        match self.certificates {
            Some(ref mut certs) => certs.push(cert)?,
            None => {
                let mut certs = Stack::new()?;
                certs.push(cert)?;
                self.certificates = Some(certs);
            }
        }
        Ok(())
    }

    // Node has feature = One the the agents has the feature
    pub fn has_feature(&self, feature: AgentFeature) -> bool {
        self.agents
            .as_ref()
            .map(|agents| agents.iter().any(|a| a.has_feature(feature)))
            // No agent info -> pre 6.1 server
            // Not supported except remote run
            .unwrap_or_else(|| feature == AgentFeature::CfengineRemoteRun)
    }
}

#[derive(Deserialize)]
#[serde(transparent)]
pub struct RawNodesList {
    data: HashMap<NodeId, Info>,
}

impl RawNodesList {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    fn add_certificate(&mut self, id: &NodeIdRef, cert: X509) -> Result<(), Error> {
        trace!("Adding certificate for node {}", id);
        self.data
            .get_mut(id)
            .ok_or_else(|| Error::CertificateForUnknownNode(id.to_string()))
            .and_then(|node| node.add_certificate(cert))
    }
}

pub struct NodesList {
    list: RawNodesList,
    my_id: NodeId,
}

impl NodesList {
    // Load nodes list from the nodeslist.json file
    pub fn new<P: AsRef<Path>>(
        my_id: NodeId,
        nodes_file: P,
        certificates_file: Option<P>,
    ) -> Result<Self, Error> {
        info!("Parsing nodes list from {:#?}", nodes_file.as_ref());

        let mut nodes = if nodes_file.as_ref().exists() {
            read_to_string(nodes_file)?.parse::<RawNodesList>()?
        } else {
            info!("Nodes list file does not exist, considering it as empty");
            RawNodesList::new()
        };

        if let Some(certificates_file) = certificates_file {
            if certificates_file.as_ref().exists() {
                // TODO PERF: stack_from_pem is mono threaded, could be parallelized if necessary,
                // by splitting the file before calling it
                for cert in X509::stack_from_pem(&read(certificates_file.as_ref())?)? {
                    Self::id_from_cert(&cert)
                        .and_then(|id| nodes.add_certificate(&id, cert))
                        .map_err(|e| warn!("{}", e))
                        // Skip node and continue
                        .unwrap_or(())
                }
            } else {
                info!("Certificates file does not exist, skipping");
            }
        }
        Ok(NodesList { list: nodes, my_id })
    }

    pub fn counts(&self) -> NodeCounts {
        NodeCounts {
            sub_nodes: self.list.data.len(),
            managed_nodes: self.my_neighbors().len(),
        }
    }

    /// Nodes list file only contains sub-nodes, so we only have to check for
    /// node presence.
    pub fn is_subnode(&self, id: &NodeIdRef) -> bool {
        self.list.data.get(id).is_some()
    }

    pub fn is_my_neighbor(&self, id: &NodeIdRef) -> Result<bool, ()> {
        self.list
            .data
            .get(id)
            .ok_or(())
            .map(|n| n.policy_server == self.my_id)
    }

    pub fn key_hash(&self, id: &NodeIdRef) -> Option<Hash> {
        self.list.data.get(id).map(|s| s.key_hash.clone())
    }

    pub fn hostname(&self, id: &NodeIdRef) -> Option<Host> {
        self.list.data.get(id).map(|s| s.hostname.clone())
    }

    pub fn certs(&self, id: &NodeIdRef) -> Option<&Stack<X509>> {
        self.list
            .data
            .get(id)
            .and_then(|node| node.certificates.as_ref())
    }

    fn id_from_cert(cert: &X509) -> Result<NodeId, Error> {
        Ok(cert
            .subject_name()
            .entries()
            // Rudder node id uses "userId"
            .find(|c| c.object().to_string() == "userId")
            .ok_or(Error::MissingIdInCertificate)?
            .data()
            .as_utf8()?
            .to_string())
    }

    /// Some(Next hop) if any, None if directly connected, error if not found
    fn next_hop(&self, node_id: &NodeIdRef) -> Result<Option<NodeId>, ()> {
        // nodeslist should not contain loops but just in case
        // 20 levels of relays should be more than enough
        const MAX_RELAY_LEVELS: u8 = 20;

        if self.is_my_neighbor(node_id)? {
            return Ok(None);
        }

        let mut current_id = node_id;
        let mut current = self.list.data.get(current_id).ok_or(())?;
        let mut next_hop = Err(());

        for level in 0..MAX_RELAY_LEVELS {
            if current.policy_server == self.my_id {
                next_hop = Ok(Some(current_id.to_string()));
                break;
            }
            current_id = &current.policy_server;
            current = self.list.data.get(current_id).ok_or(())?;

            if level == MAX_RELAY_LEVELS {
                warn!(
                    "Reached maximum level of relay ({}) for {}, there is probably a loop",
                    MAX_RELAY_LEVELS, node_id
                );
            }
        }

        next_hop
    }

    // NOTE: Following methods could be made faster by pre-computing a graph in cache

    pub fn my_neighbors(&self) -> Vec<Host> {
        self.list
            .data
            .values()
            .filter(|k| k.policy_server == self.my_id)
            .map(|k| k.hostname.clone())
            .collect()
    }

    pub fn neighbors_from(&self, server: &NodeIdRef, nodes: &[NodeId]) -> Vec<Host> {
        nodes
            .iter()
            .filter_map(|n| self.list.data.get::<str>(n))
            .filter(|n| n.policy_server == server)
            .map(|n| n.hostname.clone())
            .collect()
    }

    pub fn my_neighbors_from(&self, nodes: &[NodeId]) -> Vec<Host> {
        self.neighbors_from(&self.my_id, nodes)
    }

    pub fn my_sub_relays(&self) -> Vec<Host> {
        let mut relays = HashSet::new();
        for policy_server in self
            .list
            .data
            .values()
            .filter_map(|v| self.list.data.get(&v.policy_server))
            .filter(|v| v.policy_server == self.my_id)
            .map(|v| v.hostname.clone())
        {
            let _ = relays.insert(policy_server);
        }
        relays.into_iter().collect()
    }

    /// Relays to contact to trigger given nodes, with the matching nodes
    /// Logs and ignores unknown nodes
    pub fn my_sub_relays_from(&self, nodes: &[NodeId]) -> Vec<(Host, Vec<NodeId>)> {
        let mut relays: HashMap<Host, Vec<NodeId>> = HashMap::new();
        for node in nodes.iter() {
            let hostname = match self.next_hop(node) {
                Ok(Some(ref next_hop)) => self
                    .list
                    .data
                    .get::<str>(next_hop)
                    .map(|n| n.hostname.clone())
                    // We are sure it is there at this point
                    .unwrap(),
                Ok(None) => continue,
                Err(()) => {
                    error!("Unknown node {}", node);
                    continue;
                }
            };

            if let Some(nodes) = relays.get_mut(&hostname) {
                nodes.push(node.clone());
            } else {
                relays.insert(hostname, vec![node.clone()]);
            }
        }

        relays.into_iter().collect()
    }
}

impl FromStr for RawNodesList {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(serde_json::from_str(s)?)
    }
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct NodeCounts {
    // Total nodes under this relays
    pub sub_nodes: usize,
    // Nodes directly managed by this relay
    pub managed_nodes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_nodeslist() {
        let nodeslist =
            NodesList::new("root".to_string(), "tests/files/nodeslist.json", None).unwrap();
        assert_eq!(
            nodeslist.list.data["e745a140-40bc-4b86-b6dc-084488fc906b"].hostname,
            "node1.rudder.local"
        );
        assert_eq!(nodeslist.list.data.len(), 6);
    }

    #[test]
    fn it_parses_absent_nodeslist() {
        let nodeslist =
            NodesList::new("root".to_string(), "tests/files/notthere.json", None).unwrap();
        assert_eq!(nodeslist.list.data.len(), 0);
    }

    #[test]
    fn it_parses_big_nodeslist() {
        assert!(NodesList::new(
            "root".to_string(),
            "benches/files/nodeslist.json",
            Some("benches/files/allnodescerts.pem")
        )
        .is_ok())
    }

    #[test]
    fn it_parses_agent_info() {
        let nodeslist =
            NodesList::new("root".to_string(), "tests/files/nodeslist.json", None).unwrap();
        assert_eq!(
            nodeslist.list.data["c745a140-40bc-4b86-b6dc-084488fc906b"].agents,
            Some(vec![AgentInfo {
                name: AgentName::CfengineCommunity,
                version: "6.0.3".to_string()
            }])
        )
    }

    #[test]
    fn it_parses_certificates() {
        let nodeslist = NodesList::new(
            "root".to_string(),
            "tests/files/nodeslist.json",
            Some("tests/files/keys/nodescerts.pem"),
        )
        .unwrap();
        assert_eq!(nodeslist.list.data.len(), 6);
        assert_eq!(
            nodeslist.list.data["37817c4d-fbf7-4850-a985-50021f4e8f41"]
                .certificates
                .as_ref()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            nodeslist.list.data["e745a140-40bc-4b86-b6dc-084488fc906b"]
                .certificates
                .as_ref()
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn if_gets_subrelays() {
        assert!(
            NodesList::new("root".to_string(), "tests/files/nodeslist.json", None)
                .unwrap()
                .is_subnode("37817c4d-fbf7-4850-a985-50021f4e8f41")
        );
        assert!(
            !NodesList::new("root".to_string(), "tests/files/nodeslist.json", None)
                .unwrap()
                .is_subnode("unknown")
        );
    }

    #[test]
    fn if_gets_my_neighbors() {
        assert!(
            NodesList::new("root".to_string(), "tests/files/nodeslist.json", None)
                .unwrap()
                .is_my_neighbor("37817c4d-fbf7-4850-a985-50021f4e8f41")
                .unwrap()
        );
        assert!(
            !NodesList::new("root".to_string(), "tests/files/nodeslist.json", None)
                .unwrap()
                .is_my_neighbor("b745a140-40bc-4b86-b6dc-084488fc906b")
                .unwrap()
        );
        assert!(
            NodesList::new("root".to_string(), "tests/files/nodeslist.json", None)
                .unwrap()
                .is_my_neighbor("unknown")
                .is_err()
        );
    }

    #[test]
    fn it_filters_neighbors() {
        let mut reference = vec![
            "node1.rudder.local",
            "node2.rudder.local",
            "server.rudder.local",
        ];
        reference.sort();

        let mut actual = NodesList::new("root".to_string(), "tests/files/nodeslist.json", None)
            .unwrap()
            .my_neighbors();
        actual.sort();

        assert_eq!(reference, actual);
    }

    #[test]
    fn it_gets_neighbors() {
        let mut reference = vec![
            "node1.rudder.local",
            "node2.rudder.local",
            "server.rudder.local",
        ];
        reference.sort();

        let mut actual = NodesList::new("root".to_string(), "tests/files/nodeslist.json", None)
            .unwrap()
            .my_neighbors();
        actual.sort();

        assert_eq!(reference, actual);
    }

    #[test]
    fn it_gets_sub_relays() {
        let mut reference = vec![
            "node1.rudder.local",
            "node2.rudder.local",
            "server.rudder.local",
        ];
        reference.sort();

        let mut actual = NodesList::new("root".to_string(), "tests/files/nodeslist.json", None)
            .unwrap()
            .my_sub_relays();
        actual.sort();

        assert_eq!(reference, actual);
    }

    #[test]
    fn it_filters_sub_relays() {
        let mut reference = vec![(
            "node1.rudder.local".to_string(),
            vec![
                "b745a140-40bc-4b86-b6dc-084488fc906b".to_string(),
                "a745a140-40bc-4b86-b6dc-084488fc906b".to_string(),
            ],
        )];
        reference.sort();

        let mut actual = NodesList::new("root".to_string(), "tests/files/nodeslist.json", None)
            .unwrap()
            .my_sub_relays_from(&[
                "b745a140-40bc-4b86-b6dc-084488fc906b".to_string(),
                "a745a140-40bc-4b86-b6dc-084488fc906b".to_string(),
                "root".to_string(),
                "37817c4d-fbf7-4850-a985-50021f4e8f41".to_string(),
                "e745a140-40bc-4b86-b6dc-084488fc906b".to_string(),
            ]);
        actual.sort();

        assert_eq!(reference, actual);
    }
}
