/*
 *************************************************************************************
 * Copyright 2020 Normation SAS
 *************************************************************************************
 *
 * This file is part of Rudder.
 *
 * Rudder is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * In accordance with the terms of section 7 (7. Additional Terms.) of
 * the GNU General Public License version 3, the copyright holders add
 * the following Additional permissions:
 * Notwithstanding to the terms of section 5 (5. Conveying Modified Source
 * Versions) and 6 (6. Conveying Non-Source Forms.) of the GNU General
 * Public License version 3, when you create a Related Module, this
 * Related Module is not considered as a part of the work and may be
 * distributed under the license agreement of your choice.
 * A "Related Module" means a set of sources files including their
 * documentation that, without modification of the Source Code, enables
 * supplementary functions or services in addition to those offered by
 * the Software.
 *
 * Rudder is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Rudder.  If not, see <http://www.gnu.org/licenses/>.

 *
 *************************************************************************************
 */

package com.normation.rudder

import better.files.File
import better.files.Resource
import com.normation.appconfig.ConfigRepository
import com.normation.appconfig.GenericConfigService
import com.normation.appconfig.ModifyGlobalPropertyInfo
import com.normation.cfclerk.domain.TechniqueVersionHelper
import com.normation.errors
import com.normation.errors.IOResult
import com.normation.errors.IOStream
import com.normation.eventlog.EventActor
import com.normation.eventlog.ModificationId
import com.normation.inventory.domain.FullInventory
import com.normation.inventory.domain.InventoryStatus
import com.normation.inventory.domain.NodeId
import com.normation.inventory.domain.Software
import com.normation.rudder.batch.AsyncWorkflowInfo
import com.normation.rudder.domain.Constants
import com.normation.rudder.domain.appconfig.RudderWebProperty
import com.normation.rudder.domain.archives.RuleArchiveId
import com.normation.rudder.domain.eventlog
import com.normation.rudder.domain.nodes.NodeGroup
import com.normation.rudder.domain.nodes.NodeGroupCategory
import com.normation.rudder.domain.nodes.NodeGroupCategoryId
import com.normation.rudder.domain.nodes.NodeGroupId
import com.normation.rudder.domain.nodes.NodeGroupUid
import com.normation.rudder.domain.policies.AddRuleDiff
import com.normation.rudder.domain.policies.DeleteRuleDiff
import com.normation.rudder.domain.policies.DirectiveId
import com.normation.rudder.domain.policies.FullGroupTarget
import com.normation.rudder.domain.policies.FullRuleTargetInfo
import com.normation.rudder.domain.policies.GlobalPolicyMode
import com.normation.rudder.domain.policies.GroupTarget
import com.normation.rudder.domain.policies.ModifyRuleDiff
import com.normation.rudder.domain.policies.PolicyMode
import com.normation.rudder.domain.policies.PolicyModeOverrides
import com.normation.rudder.domain.policies.PolicyTypeName
import com.normation.rudder.domain.policies.PolicyTypes
import com.normation.rudder.domain.policies.Rule
import com.normation.rudder.domain.policies.RuleId
import com.normation.rudder.domain.policies.RuleUid
import com.normation.rudder.domain.policies.TargetExclusion
import com.normation.rudder.domain.policies.TargetIntersection
import com.normation.rudder.domain.reports.ComplianceLevel
import com.normation.rudder.domain.reports.ComponentValueStatusReport
import com.normation.rudder.domain.reports.DirectiveStatusReport
import com.normation.rudder.domain.reports.MessageStatusReport
import com.normation.rudder.domain.reports.NodeConfigId
import com.normation.rudder.domain.reports.NodeExpectedReports
import com.normation.rudder.domain.reports.NodeModeConfig
import com.normation.rudder.domain.reports.NodeStatusReport
import com.normation.rudder.domain.reports.NodeStatusReport.*
import com.normation.rudder.domain.reports.OverriddenPolicy
import com.normation.rudder.domain.reports.ReportType
import com.normation.rudder.domain.reports.RuleNodeStatusReport
import com.normation.rudder.domain.reports.RunComplianceInfo
import com.normation.rudder.domain.reports.ValueStatusReport
import com.normation.rudder.facts.nodes.ChangeContext
import com.normation.rudder.facts.nodes.CoreNodeFact
import com.normation.rudder.facts.nodes.MinimalNodeFactInterface
import com.normation.rudder.facts.nodes.NodeFact
import com.normation.rudder.facts.nodes.NodeFactChangeEventCallback
import com.normation.rudder.facts.nodes.NodeFactChangeEventCC
import com.normation.rudder.facts.nodes.NodeFactRepository
import com.normation.rudder.facts.nodes.QueryContext
import com.normation.rudder.facts.nodes.SecurityTag
import com.normation.rudder.facts.nodes.SelectFacts
import com.normation.rudder.facts.nodes.SelectNodeStatus
import com.normation.rudder.reports.AgentRunInterval
import com.normation.rudder.reports.FullCompliance
import com.normation.rudder.reports.GlobalComplianceMode
import com.normation.rudder.repository.CategoryAndNodeGroup
import com.normation.rudder.repository.FullNodeGroupCategory
import com.normation.rudder.repository.RoNodeGroupRepository
import com.normation.rudder.repository.RoRuleRepository
import com.normation.rudder.repository.WoRuleRepository
import com.normation.rudder.rest.AuthorizationApiMapping
import com.normation.rudder.rest.ProviderRoleExtension
import com.normation.rudder.rest.RoleApiMapping
import com.normation.rudder.rest.lift.ComplianceAPIService
import com.normation.rudder.rule.category.RuleCategoryId
import com.normation.rudder.services.policies.NodeConfigData
import com.normation.rudder.services.policies.PolicyId
import com.normation.rudder.services.reports.ComputeCompliance
import com.normation.rudder.services.reports.NodeStatusReportInternal
import com.normation.rudder.services.reports.ReportingService
import com.normation.rudder.services.servers.AllowedNetwork
import com.normation.rudder.services.servers.PolicyServer
import com.normation.rudder.services.servers.PolicyServerManagementService
import com.normation.rudder.services.servers.PolicyServers
import com.normation.rudder.services.servers.PolicyServersUpdateCommand
import com.normation.rudder.services.workflows.WorkflowLevelService
import com.normation.rudder.tenants.TenantId
import com.normation.rudder.tenants.TenantService
import com.normation.rudder.users.EventTrace
import com.normation.rudder.users.FileUserDetailListProvider
import com.normation.rudder.users.PasswordEncoderDispatcher
import com.normation.rudder.users.SessionId
import com.normation.rudder.users.UserFile
import com.normation.rudder.users.UserInfo
import com.normation.rudder.users.UserManagementService
import com.normation.rudder.users.UserRepository
import com.normation.rudder.users.UserSession
import com.normation.rudder.users.UserStatus
import com.normation.zio.UnsafeRun
import com.typesafe.config.ConfigFactory
import java.io.InputStream
import java.nio.charset.StandardCharsets
import org.apache.commons.io.IOUtils
import org.joda.time.DateTime
import scala.collection.MapView
import scala.collection.immutable.SortedMap
import zio.*
import zio.System as _
import zio.Tag as _
import zio.ZIO
import zio.json.ast.Json
import zio.syntax.*

/*
 * Mock services for test, especially repositories, and provides
 * test data (nodes, directives, etc)
 */

class MockSettings(wfservice: WorkflowLevelService, asyncWF: AsyncWorkflowInfo) {

  val defaultPolicyServer: PolicyServers = PolicyServers(
    PolicyServer(Constants.ROOT_POLICY_SERVER_ID, AllowedNetwork("192.168.2.0/32", "root") :: Nil),
    Nil
  )

  object policyServerManagementService extends PolicyServerManagementService {
    val repo: Ref[PolicyServers] = Ref.make(defaultPolicyServer).runNow

    override def getPolicyServers(): IOResult[PolicyServers] = repo.get

    override def savePolicyServers(policyServers: PolicyServers): IOResult[PolicyServers] = ???

    override def updatePolicyServers(
        commands: List[PolicyServersUpdateCommand],
        modId:    ModificationId,
        actor:    EventActor
    ): IOResult[PolicyServers] = {
      for {
        servers <- repo.get
        updated <- PolicyServerManagementService.applyCommands(servers, commands)
        saved   <- repo.set(updated)
      } yield updated
    }

    override def deleteRelaySystemObjects(policyServerId: NodeId): IOResult[Unit] = {
      updatePolicyServers(
        PolicyServersUpdateCommand.Delete(policyServerId) :: Nil,
        ModificationId(s"clean-${policyServerId.value}"),
        eventlog.RudderEventActor
      ).unit
    }
  }

  // a mock service that keep information in memory only
  val configService: GenericConfigService = {

    object configRepo extends ConfigRepository {
      val configs = Ref.make(Map[String, RudderWebProperty]()).runNow
      override def getConfigParameters(): IOResult[Seq[RudderWebProperty]] = {
        configs.get.map(_.values.toList)
      }
      override def saveConfigParameter(
          parameter:                RudderWebProperty,
          modifyGlobalPropertyInfo: Option[ModifyGlobalPropertyInfo]
      ): IOResult[RudderWebProperty] = {
        configs.update(_.updated(parameter.name.value, parameter)).map(_ => parameter)
      }
    }

    new GenericConfigService(ConfigFactory.empty(), configRepo, asyncWF, wfservice)
  }

}

class MockCompliance(mockDirectives: MockDirectives) {
  self =>

  private val directives = mockDirectives.directives

  private def nodeGroupsRepo(nodeGroups: List[NodeGroup]) = new RoNodeGroupRepository {
    val nodesByGroup = nodeGroups
      .map(g => (g.id, Chunk.fromIterable(g.serverList)))
      .toMap

    override def getAllNodeIdsChunk(): IOResult[Map[NodeGroupId, Chunk[NodeId]]] = {
      nodesByGroup.succeed
    }

    override def getNodeGroupOpt(
        id: NodeGroupId
    )(implicit qc: QueryContext): IOResult[Option[(NodeGroup, NodeGroupCategoryId)]] = {
      nodeGroups.find(_.id == id).map((_, NodeGroupCategoryId("cat1"))).succeed
    }

    def getFullGroupLibrary(): IOResult[FullNodeGroupCategory] = {
      FullNodeGroupCategory(
        NodeGroupCategoryId("GroupRoot"),
        name = "GroupRoot",
        description = "root of group categories",
        subCategories = Nil,
        targetInfos = nodeGroups.map(g => {
          FullRuleTargetInfo(FullGroupTarget(GroupTarget(g.id), g), g.name, g.description, g.isEnabled, g.isSystem)
        }),
        isSystem = true
      ).succeed
    }

    def categoryExists(id:       NodeGroupCategoryId): IOResult[Boolean]           = ???
    def getNodeGroupCategory(id: NodeGroupId):         IOResult[NodeGroupCategory] = ???
    def getAll(): IOResult[Seq[NodeGroup]] = ???
    def getAllByIds(ids: Seq[NodeGroupId]): IOResult[Seq[NodeGroup]] = ???
    def getAllNodeIds(): IOResult[Map[NodeGroupId, Set[NodeId]]] = ???
    def getGroupsByCategory(includeSystem: Boolean)(implicit
        qc: QueryContext
    ): IOResult[SortedMap[List[NodeGroupCategoryId], CategoryAndNodeGroup]] = ???
    def findGroupWithAnyMember(nodeIds: Seq[NodeId]): IOResult[Seq[NodeGroupId]] = ???
    def findGroupWithAllMember(nodeIds: Seq[NodeId]): IOResult[Seq[NodeGroupId]] = ???
    def getRootCategory():     NodeGroupCategory                                                 = ???
    def getRootCategoryPure(): IOResult[NodeGroupCategory]                                       = ???
    def getCategoryHierarchy:  IOResult[SortedMap[List[NodeGroupCategoryId], NodeGroupCategory]] = ???
    def getAllGroupCategories(includeSystem: Boolean):             IOResult[Seq[NodeGroupCategory]]  = ???
    def getGroupCategory(id:                 NodeGroupCategoryId): IOResult[NodeGroupCategory]       = ???
    def getParentGroupCategory(id:           NodeGroupCategoryId): IOResult[NodeGroupCategory]       = ???
    def getParents_NodeGroupCategory(id:     NodeGroupCategoryId): IOResult[List[NodeGroupCategory]] = ???
    def getAllNonSystemCategories(): IOResult[Seq[NodeGroupCategory]] = ???
  }

  private object nodeFactRepo extends NodeFactRepository {
    override def getAll()(implicit qc: QueryContext, status: SelectNodeStatus): IOResult[MapView[NodeId, CoreNodeFact]] = {
      val _                                           = (qc, status) // ignore "unused" warning
      def build(id: String, mode: Option[PolicyMode]) = {
        val nodeFact = NodeConfigData.fact1
        nodeFact.copy(id = NodeId(id), rudderSettings = nodeFact.rudderSettings.copy(policyMode = mode))
      }
      Seq(
        build("n1", Some(PolicyMode.Enforce)),
        build("n2", Some(PolicyMode.Enforce)),
        build("n3", Some(PolicyMode.Enforce)),
        build("bn1", Some(PolicyMode.Enforce)),
        build("bn2", Some(PolicyMode.Enforce)),
        build("bn3", Some(PolicyMode.Enforce)),
        build("bn4", Some(PolicyMode.Enforce)),
        build("bn5", Some(PolicyMode.Enforce))
      ).map(n => (n.id, n)).toMap.view.succeed
    }

    def registerChangeCallbackAction(callback: NodeFactChangeEventCallback): IOResult[Unit] = ???
    def getStatus(id:                          NodeId)(implicit qc:   QueryContext): IOResult[InventoryStatus] = ???
    def get(nodeId:                            NodeId)(implicit qc:   QueryContext, status:    SelectNodeStatus): IOResult[Option[CoreNodeFact]]  = ???
    def slowGet(
        nodeId: NodeId
    )(implicit qc: QueryContext, status: SelectNodeStatus, attrs: SelectFacts): IOResult[Option[NodeFact]] = ???
    def getNodesBySoftwareName(softName:       String): IOResult[List[(NodeId, Software)]] = ???
    def slowGetAll()(implicit qc:              QueryContext, status:  SelectNodeStatus, attrs: SelectFacts):      errors.IOStream[NodeFact]       = ???
    def save(nodeFact:                         NodeFact)(implicit cc: ChangeContext, attrs:    SelectFacts):      IOResult[NodeFactChangeEventCC] = ???
    def setSecurityTag(nodeId: NodeId, tag: Option[SecurityTag])(implicit cc: ChangeContext): IOResult[NodeFactChangeEventCC] =
      ???
    def updateInventory(inventory: FullInventory, software: Option[Iterable[Software]])(implicit
        cc: ChangeContext
    ): IOResult[NodeFactChangeEventCC] = ???
    def changeStatus(nodeId: NodeId, into: InventoryStatus)(implicit cc: ChangeContext): IOResult[NodeFactChangeEventCC] = ???
    def delete(nodeId: NodeId)(implicit cc: ChangeContext): IOResult[NodeFactChangeEventCC] = ???

  }

  // We want to ignore rules that are defined in `MockRules` because they may target all nodes and pollute our compliance tests
  private def rulesRepo(rules: List[Rule]) = new RoRuleRepository with WoRuleRepository {
    override def getOpt(ruleId: RuleId):        IOResult[Option[Rule]] = {
      rules.find(_.id == ruleId).succeed
    }
    override def getAll(includeSytem: Boolean): IOResult[Seq[Rule]]    = {
      rules.succeed
    }

    override def getIds(includeSytem:             Boolean): IOResult[Set[RuleId]] = ???
    override def create(rule:                     Rule, modId:   ModificationId, actor: EventActor, reason: Option[String]): IOResult[AddRuleDiff] = ???
    override def update(
        rule:   Rule,
        modId:  ModificationId,
        actor:  EventActor,
        reason: Option[String]
    ): IOResult[Option[ModifyRuleDiff]] = ???
    override def load(rule:                       Rule, modId:   ModificationId, actor: EventActor, reason: Option[String]): IOResult[Unit]        = ???
    override def unload(ruleId:                   RuleId, modId: ModificationId, actor: EventActor, reason: Option[String]): IOResult[Unit]        = ???
    override def updateSystem(
        rule:   Rule,
        modId:  ModificationId,
        actor:  EventActor,
        reason: Option[String]
    ): IOResult[Option[ModifyRuleDiff]] = ???
    override def delete(id: RuleId, modId: ModificationId, actor: EventActor, reason: Option[String]): IOResult[DeleteRuleDiff] =
      ???
    override def deleteSystemRule(
        id:     RuleId,
        modId:  ModificationId,
        actor:  EventActor,
        reason: Option[String]
    ): IOResult[DeleteRuleDiff] = ???
    override def swapRules(newRules:              Seq[Rule]): IOResult[RuleArchiveId] = ???
    override def deleteSavedRuleArchiveId(saveId: RuleArchiveId): IOResult[Unit] = ???
  }

  def reportingService(statusReports: Map[NodeId, NodeStatusReport]): ReportingService = new ReportingService {
    def findRuleNodeStatusReports(nodeIds: Set[NodeId], filterByRules: Set[RuleId])(implicit
        qc: QueryContext
    ): IOResult[Map[NodeId, NodeStatusReport]] = {
      val filteredNodeReports = statusReports.view.filterKeys(nodeIds.contains(_)).toMap
      ReportingService.filterReportsByRules(filteredNodeReports, filterByRules).succeed
    }
    // used in node details API
    def getSystemAndUserCompliance(
        optNodeIds: Option[Set[NodeId]]
    )(implicit
        qc:         QueryContext
    ): IOResult[SystemUserComplianceRun] = {
      ZIO.succeed(SystemUserComplianceRun.empty)
    }

    def findDirectiveNodeStatusReports(
        nodeIds:            Set[NodeId],
        filterByDirectives: Set[DirectiveId]
    )(implicit qc: QueryContext): IOResult[Map[NodeId, NodeStatusReport]] = ???
    def findDirectiveRuleStatusReportsByRule(ruleId: RuleId)(implicit qc: QueryContext): IOResult[Map[NodeId, NodeStatusReport]] =
      ???
    def findNodeStatusReport(nodeId:            NodeId)(implicit qc: QueryContext): IOResult[NodeStatusReport] = ???
    def findUserNodeStatusReport(nodeId:        NodeId)(implicit qc: QueryContext): IOResult[NodeStatusReport] = ???
    def findSystemNodeStatusReport(nodeId:      NodeId)(implicit qc: QueryContext): IOResult[NodeStatusReport] = ???
    def getUserNodeStatusReports()(implicit qc: QueryContext): IOResult[Map[NodeId, NodeStatusReport]] = ???
    def findStatusReportsForDirective(directiveId: DirectiveId)(implicit
        qc: QueryContext
    ): IOResult[Map[NodeId, NodeStatusReport]] = ???
    def getGlobalUserCompliance()(implicit qc:  QueryContext): IOResult[Option[(ComplianceLevel, Long)]] = ???
  }

  val complianceAPIService: ComplianceAPIService = {
    import simpleExample.*
    import complexExample.*
    buildComplianceService(
      simpleCustomRules ++ complexCustomRules,
      simpleCustomNodeGroups ++ complexCustomNodeGroups,
      simpleStatusReports ++ complexStatusReports
    )
  }

  private object simpleExample {

    /*
    Nodes N1 and N2,

    Groups G1 (containing N1 and N2) and G2 (containing N2) and G3 containing N1

    Rules R1 (applying to G1), R2 (applying to G1 but excluding G2), and R3 (applying to G2 and G3)
     */

    val simpleStatusReports: Map[NodeId, NodeStatusReport] = Map(
      // R1, R2, R3 apply on N1
      nodeId(1) -> simpleNodeStatusReport(
        nodeId(1),
        Set(
          simpleRuleNodeStatusReport(nodeId(1), ruleId(1), directives.fileTemplateDirecive1.id, ReportType.EnforceSuccess),
          simpleRuleNodeStatusReport(nodeId(1), ruleId(2), directives.fileTemplateVariables2.id, ReportType.EnforceRepaired),
          simpleRuleNodeStatusReport(nodeId(1), ruleId(3), directives.rpmDirective.id, ReportType.EnforceError)
        )
      ),
      // R1, R3 apply on N2
      nodeId(2) -> simpleNodeStatusReport(
        nodeId(2),
        Set(
          simpleRuleNodeStatusReport(nodeId(2), ruleId(1), directives.fileTemplateDirecive1.id, ReportType.EnforceSuccess),
          simpleRuleNodeStatusReport(nodeId(2), ruleId(3), directives.rpmDirective.id, ReportType.EnforceError)
        )
      )
    )

    val g1: NodeGroup = NodeGroup(
      nodeGroupId(1),
      name = "G1",
      description = "",
      properties = Nil,
      query = None,
      isDynamic = false,
      serverList = (1 to 2).map(nodeId).toSet,
      _isEnabled = true
    )

    val g2: NodeGroup = NodeGroup(
      nodeGroupId(2),
      name = "G2",
      description = "",
      properties = Nil,
      query = None,
      isDynamic = false,
      serverList = Set(nodeId(2)),
      _isEnabled = true
    )

    val g3: NodeGroup = NodeGroup(
      nodeGroupId(3),
      name = "G3",
      description = "",
      properties = Nil,
      query = None,
      isDynamic = false,
      serverList = Set(nodeId(1)),
      _isEnabled = true
    )

    val r1: Rule = Rule(
      ruleId(1),
      "R1",
      RuleCategoryId("rulecat1"),
      Set(GroupTarget(g1.id)),
      Set(directives.fileTemplateDirecive1.id)
    )

    val r2: Rule = Rule(
      ruleId(2),
      "R2",
      RuleCategoryId("rulecat1"),
      Set(
        TargetExclusion(TargetIntersection(Set(GroupTarget(g1.id))), TargetIntersection(Set(GroupTarget(g2.id))))
      ), // include G1 but not G2
      Set(directives.fileTemplateDirecive1.id)
    )

    val r3: Rule = Rule(
      ruleId(3),
      "R3",
      RuleCategoryId("rulecat1"),
      Set(GroupTarget(g2.id), GroupTarget(g3.id)),
      Set(directives.fileTemplateDirecive1.id)
    )

    val simpleCustomRules: List[Rule] = List(r1, r2, r3)

    val simpleCustomNodeGroups: List[NodeGroup] = List(g1, g2, g3)

    private def nodeId(id:      Int): NodeId      = NodeId("n" + id)
    private def ruleId(id:      Int): RuleId      = RuleId(RuleUid("r" + id))
    private def nodeGroupId(id: Int): NodeGroupId = NodeGroupId(NodeGroupUid("g" + id))
  }

  private object complexExample {

    /*
    Nodes N1, N2, N3, N4, N5

    Groups G1 (with N1, N2), G2 (N2), G3 (N1), G4 (N3, N4, N5), G5 (N2, N3), G6(N4, N5)

    Directives D1, D2, D3, D4, D5

    Rules

    - R1, which applies D1 to G1
    - R2, which applies D2 to G1 but not G2
    - R3, which applies D3 to G2 and G3
    - R4, which applies D4 to G4 and G5
    - R5, which applies D5 to G6 but not G4 (so nothing)
    - R6, which applies D4 to G6 (so we will have skipped on N4 and N5)
     */

    val g1: NodeGroup = NodeGroup(
      nodeGroupId(1),
      name = "G1",
      description = "",
      properties = Nil,
      query = None,
      isDynamic = false,
      serverList = Set(nodeId(1), nodeId(2)),
      _isEnabled = true
    )
    val g2: NodeGroup = NodeGroup(
      nodeGroupId(2),
      name = "G2",
      description = "",
      properties = Nil,
      query = None,
      isDynamic = false,
      serverList = Set(nodeId(2)),
      _isEnabled = true
    )
    val g3: NodeGroup = NodeGroup(
      nodeGroupId(3),
      name = "G3",
      description = "",
      properties = Nil,
      query = None,
      isDynamic = false,
      serverList = Set(nodeId(1)),
      _isEnabled = true
    )
    val g4: NodeGroup = NodeGroup(
      nodeGroupId(4),
      name = "G4",
      description = "",
      properties = Nil,
      query = None,
      isDynamic = false,
      serverList = Set(nodeId(3), nodeId(4), nodeId(5)),
      _isEnabled = true
    )
    val g5: NodeGroup = NodeGroup(
      nodeGroupId(5),
      name = "G5",
      description = "",
      properties = Nil,
      query = None,
      isDynamic = false,
      serverList = Set(nodeId(2), nodeId(3)),
      _isEnabled = true
    )
    val g6: NodeGroup = NodeGroup(
      nodeGroupId(6),
      name = "G6",
      description = "",
      properties = Nil,
      query = None,
      isDynamic = false,
      serverList = Set(nodeId(4), nodeId(5)),
      _isEnabled = true
    )

    val d1 = directives.fileTemplateDirecive1
    val d2 = directives.fileTemplateVariables2
    val d3 = directives.rpmDirective
    val d4 = directives.fileTemplateDirecive1
    val d5 = directives.fileTemplateVariables2

    val r1: Rule = Rule(
      ruleId(1),
      "R1",
      RuleCategoryId("rulecat1"),
      Set(GroupTarget(g1.id)),
      Set(d1.id)
    )

    val r2: Rule = Rule(
      ruleId(2),
      "R2",
      RuleCategoryId("rulecat1"),
      Set(
        TargetExclusion(TargetIntersection(Set(GroupTarget(g1.id))), TargetIntersection(Set(GroupTarget(g2.id))))
      ), // include G1 but not G2
      Set(d2.id)
    )

    val r3: Rule = Rule(
      ruleId(3),
      "R3",
      RuleCategoryId("rulecat1"),
      Set(GroupTarget(g2.id), GroupTarget(g3.id)),
      Set(d3.id)
    )

    val r4: Rule = Rule(
      ruleId(4),
      "R4",
      RuleCategoryId("rulecat1"),
      Set(GroupTarget(g4.id), GroupTarget(g5.id)),
      Set(d4.id)
    )

    val r5: Rule = Rule(
      ruleId(5),
      "R5",
      RuleCategoryId("rulecat1"),
      Set(
        TargetExclusion(TargetIntersection(Set(GroupTarget(g6.id))), TargetIntersection(Set(GroupTarget(g4.id))))
      ), // include G6 but not G4 (no node at all)
      Set(d5.id)
    )

    val r6: Rule = Rule(
      ruleId(6),
      "R6",
      RuleCategoryId("rulecat1"),
      Set(GroupTarget(g6.id)),
      Set(d4.id)
    )

    val complexStatusReports: Map[NodeId, NodeStatusReport] = Map(
      // R1, R2, R3 apply on N1
      nodeId(1) -> simpleNodeStatusReport(
        nodeId(1),
        Set(
          simpleRuleNodeStatusReport(nodeId(1), ruleId(1), d1.id, ReportType.EnforceSuccess),
          simpleRuleNodeStatusReport(nodeId(1), ruleId(2), d2.id, ReportType.EnforceRepaired),
          simpleRuleNodeStatusReport(nodeId(1), ruleId(3), d3.id, ReportType.EnforceError)
        )
      ),
      // R1, R3, R4 apply on N2
      nodeId(2) -> simpleNodeStatusReport(
        nodeId(2),
        Set(
          simpleRuleNodeStatusReport(nodeId(2), ruleId(1), d1.id, ReportType.EnforceSuccess),
          simpleRuleNodeStatusReport(nodeId(2), ruleId(3), d3.id, ReportType.EnforceError),
          simpleRuleNodeStatusReport(nodeId(2), ruleId(4), d4.id, ReportType.EnforceSuccess)
        )
      ),
      // R4 applies on N3
      nodeId(3) -> simpleNodeStatusReport(
        nodeId(3),
        Set(
          simpleRuleNodeStatusReport(nodeId(3), ruleId(4), d4.id, ReportType.EnforceSuccess)
        )
      ),
      // R4 applies on N4
      nodeId(4) -> simpleNodeStatusReport(
        nodeId(4),
        Set(
          simpleRuleNodeStatusReport(nodeId(4), ruleId(4), d4.id, ReportType.EnforceSuccess)
        )
      ),
      // R4 applies on N5
      nodeId(5) -> simpleNodeStatusReport(
        nodeId(5),
        Set(
          simpleRuleNodeStatusReport(nodeId(5), ruleId(4), d4.id, ReportType.EnforceSuccess)
        )
      ),
      // R5 targets nothing at all because no node is targeted
      // R6 is skipped, there are some reports with 'overrides'
      nodeId(6) -> simpleNodeStatusReport(
        nodeId(6),
        Set(
          simpleRuleNodeStatusReport(nodeId(6), ruleId(4), d4.id, ReportType.NoAnswer),
          simpleRuleNodeStatusReport(nodeId(6), ruleId(5), d4.id, ReportType.NoAnswer)
        ),
        List(
          OverriddenPolicy(
            PolicyId(ruleId(6), d4.id, TechniqueVersionHelper("1.0")),
            PolicyId(ruleId(4), d4.id, TechniqueVersionHelper("1.0"))
          ),
          OverriddenPolicy(
            PolicyId(ruleId(6), d4.id, TechniqueVersionHelper("1.0")),
            PolicyId(ruleId(5), d4.id, TechniqueVersionHelper("1.0"))
          )
        )
      )
    )

    val complexCustomRules: List[Rule] = List(r1, r2, r3, r4, r5, r6)

    val complexCustomNodeGroups: List[NodeGroup] = List(g1, g2, g3, g4, g5, g6)

    // prefix all ids with "b" to avoid id collision with simpleExample
    private def nodeId(id:      Int): NodeId      = NodeId("bn" + id)
    private def ruleId(id:      Int): RuleId      = RuleId(RuleUid("br" + id))
    private def nodeGroupId(id: Int): NodeGroupId = NodeGroupId(NodeGroupUid("bg" + id))
  }

  private def buildComplianceService(
      customRules:      List[Rule],
      customNodeGroups: List[NodeGroup],
      statusesReports:  Map[NodeId, NodeStatusReport]
  ): ComplianceAPIService = {
    new ComplianceAPIService(
      rulesRepo(customRules),
      nodeFactRepo,
      nodeGroupsRepo(customNodeGroups),
      reportingService(statusesReports),
      mockDirectives.directiveRepo,
      GlobalComplianceMode(FullCompliance, 0).succeed,
      GlobalPolicyMode(PolicyMode.Enforce, PolicyModeOverrides.Always).succeed
    )
  }

  private def simpleRuleNodeStatusReport(
      nodeId:      NodeId,
      ruleId:      RuleId,
      directiveId: DirectiveId,
      reportType:  ReportType
  ): RuleNodeStatusReport = {
    RuleNodeStatusReport(
      nodeId,
      ruleId,
      PolicyTypeName.rudderBase,
      None,
      None,
      Map(
        directiveId -> DirectiveStatusReport(
          directiveId,
          PolicyTypes.rudderBase,
          List(
            ValueStatusReport(
              s"${directiveId.serialize}-component-${ruleId.serialize}-${nodeId.value}",
              s"${directiveId.serialize}-component-${ruleId.serialize}-${nodeId.value}",
              List(
                ComponentValueStatusReport(
                  s"${directiveId.serialize}-component-value-${ruleId.serialize}-${nodeId.value}",
                  s"${directiveId.serialize}-component-value-${ruleId.serialize}-${nodeId.value}",
                  s"report-${ruleId.serialize}-${nodeId.value}",
                  List(MessageStatusReport(reportType, None))
                )
              )
            )
          )
        )
      ),
      DateTime.parse("2100-01-01T00:00:00.000Z")
    )
  }

  private def simpleNodeStatusReport(
      nodeId:          NodeId,
      ruleNodeReports: Set[RuleNodeStatusReport],
      overrides:       List[OverriddenPolicy] = List.empty
  ): NodeStatusReport = {
    NodeStatusReportInternal
      .buildWith(
        nodeId,
        ComputeCompliance(
          DateTime.parse("2023-01-01T00:00:00.000Z"),
          NodeExpectedReports(
            nodeId,
            NodeConfigId(s"${nodeId.value}-config"),
            DateTime.parse("2023-01-01T00:00:00.000Z"),
            None,
            NodeModeConfig(
              GlobalComplianceMode(FullCompliance, 0),
              None,
              AgentRunInterval(None, 1, 0, 0, 0),
              None,
              GlobalPolicyMode(PolicyMode.Enforce, PolicyModeOverrides.Unoverridable),
              None
            ),
            List.empty,
            List.empty
          ),
          DateTime.parse("2024-01-01T00:00:00.000Z")
        ),
        RunComplianceInfo.OK,
        overrides,
        ruleNodeReports
      )
      .toNodeStatusReport()
  }

}

class MockUserManagement(userInfos: List[UserInfo], userSessions: List[UserSession], usersConfigFile: File) {

  object userRepo extends UserRepository {

    override def logStartSession(
        userId:            String,
        permissions:       List[String],
        authz:             List[String],
        tenants:           String,
        sessionId:         SessionId,
        authenticatorName: String,
        date:              DateTime
    ): IOResult[Unit] = ???

    override def logCloseSession(userId: String, date: DateTime, cause: String): IOResult[Unit] = ???

    override def closeAllOpenSession(endDate: DateTime, endCause: String): IOResult[Unit] = ???

    override def getLastPreviousLogin(userId: String, closedSessionsOnly: Boolean): IOResult[Option[UserSession]] = {
      userSessions.find(us => us.userId == userId && (!closedSessionsOnly || !us.isOpen)).succeed
    }

    override def deleteOldSessions(olderThan: DateTime): IOResult[Unit] = ???

    override def setExistingUsers(
        origin:          String,
        users:           List[String],
        trace:           EventTrace,
        isCaseSensitive: Boolean
    ): IOResult[Set[String]] = ???

    override def disable(
        userId:            List[String],
        notLoggedSince:    Option[DateTime],
        excludeFromOrigin: List[String],
        trace:             EventTrace
    ): IOResult[List[String]] = {
      userId.succeed
    }

    override def delete(
        userId:            List[String],
        notLoggedSince:    Option[DateTime],
        excludeFromOrigin: List[String],
        initialStatus:     Option[UserStatus],
        trace:             EventTrace
    ): IOResult[List[String]] = {
      userId.succeed
    }

    override def purge(
        userId:            List[String],
        deletedSince:      Option[DateTime],
        excludeFromOrigin: List[String],
        trace:             EventTrace
    ): IOResult[List[String]] = ???

    override def setActive(userId: List[String], trace: EventTrace): IOResult[Unit] = {
      ZIO.unit
    }

    override def updateInfo(
        id:        String,
        name:      Option[Option[String]],
        email:     Option[Option[String]],
        otherInfo: Option[Json.Obj]
    ): IOResult[Unit] = {
      ZIO.unit
    }

    override def getAll(): IOResult[List[UserInfo]] = userInfos.succeed

    override def get(userId: String, isCaseSensitive: Boolean): IOResult[Option[UserInfo]] = {
      userInfos.find(_.id == userId).succeed
    }

    override def getAllStatuses(): IOResult[Map[String, UserStatus]] = ???
  }

  val usersInputStream: () => InputStream = () => IOUtils.toInputStream(usersConfigFile.contentAsString, StandardCharsets.UTF_8)

  val passwordEncoderDispatcher = new PasswordEncoderDispatcher(0)

  val userService: FileUserDetailListProvider = {
    val usersFile = UserFile(usersConfigFile.pathAsString, usersInputStream)

    val roleApiMapping = new RoleApiMapping(AuthorizationApiMapping.Core)

    val res = new FileUserDetailListProvider(roleApiMapping, usersFile, passwordEncoderDispatcher)
    res.reload()
    res
  }

  val userManagementService: UserManagementService = {
    new UserManagementService(
      userRepo,
      userService,
      passwordEncoderDispatcher,
      UserFile(usersConfigFile.pathAsString, usersInputStream).succeed
    )
  }

  val providerRoleExtension: Map[String, ProviderRoleExtension] = Map("file" -> ProviderRoleExtension.WithOverride)
  val authBackendProviders:  Set[String]                        = Set("file")

  object tenantsService extends TenantService {
    override def tenantsEnabled: Boolean            = false
    override def getTenants():   UIO[Set[TenantId]] = ???
    override def updateTenants(ids:                             Set[TenantId]): IOResult[Unit] = ???
    override def nodeFilter[A <: MinimalNodeFactInterface](opt: Option[A])(implicit qc:          QueryContext): UIO[Option[A]]     = ???
    override def nodeFilterStream(s:                            IOStream[NodeFact])(implicit qc: QueryContext): IOStream[NodeFact] = ???
    override def nodeFilterMapView(nodes: Ref[Map[NodeId, CoreNodeFact]])(implicit
        qc: QueryContext
    ): IOResult[MapView[NodeId, CoreNodeFact]] = ???
    override def nodeGetMapView(nodes: Ref[Map[NodeId, CoreNodeFact]], nodeId: NodeId)(implicit
        qc: QueryContext
    ): IOResult[Option[CoreNodeFact]] = ???
    override def manageUpdate[A](existing: Option[CoreNodeFact], updated: NodeFact, cc: ChangeContext)(
        action: NodeFact => IOResult[A]
    ): IOResult[A] = ???
    override def checkDelete(
        existing:         CoreNodeFact,
        cc:               ChangeContext,
        availableTenants: Set[TenantId]
    ): Either[errors.RudderError, CoreNodeFact] = ???
  }
}

object MockUserManagement {
  // Default mock with fake values and returning the temporary directory for cleanup
  def apply(): (File, MockUserManagement) = {
    val tmpDir = File.newTemporaryDirectory("rudder-users")
    (tmpDir, new MockUserManagement(fakeUsers, fakeUserSessions, fakeUserFile(tmpDir)))
  }

  val fakeUsers:        List[UserInfo]    = {
    List(
      UserInfo( // user3 not in the file will get empty permissions and authz
        "user3",
        DateTime.parse("2024-02-01T01:01:01Z"),
        UserStatus.Disabled,
        "manager",
        Some("User 3"),
        Some("user3@example.com"),
        None,
        List.empty,
        Json.Obj("some" -> Json.Str("value"))
      ),
      UserInfo(
        "user2",
        DateTime.parse("2024-02-01T01:01:01Z"),
        UserStatus.Active,
        "file",
        None,
        None,
        None,
        List.empty,
        Json.Obj()
      ),
      UserInfo(
        "user1",
        DateTime.parse("2024-02-01T01:01:01Z"),
        UserStatus.Active,
        "file",
        None,
        None,
        None,
        List.empty,
        Json.Obj()
      )
    )
  }
  val fakeUserSessions: List[UserSession] = {
    List(
      UserSession(
        "user2",
        SessionId("s2-2"),
        DateTime.parse("2024-02-29T00:00:00Z"),
        "file",
        List.empty,
        List.empty,
        None,
        None,
        None
      ),
      UserSession(
        "user2",
        SessionId("s2-1"),
        DateTime.parse("2024-02-28T12:34:00Z"),
        "file",
        List.empty,
        List.empty,
        Some("a-previous-tenant-zone"),
        Some(DateTime.parse("2024-02-28T12:34:00Z")),
        None
      )
    )
  }

  // copy the resource file to a temporary directory
  def fakeUserFile(tmpDir: File) = Resource
    .asStream("test-users.xml")
    .map(IOUtils.toString(_, StandardCharsets.UTF_8))
    .map(File(tmpDir, "test-users.xml").writeText(_))
    .getOrElse(throw new Exception("Cannot find test-users.xml in test resources"))
}
