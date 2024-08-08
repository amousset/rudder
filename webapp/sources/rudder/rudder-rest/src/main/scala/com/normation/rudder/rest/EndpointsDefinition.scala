/*
 *************************************************************************************
 * Copyright 2017 Normation SAS
 *************************************************************************************
 *
 * This file is part of Rudder.
 *
 * Rudder is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * In accordance with OneParam with the terms of section 7 (7. Additional Terms.) of
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

package com.normation.rudder.rest

import com.normation.rudder.api.HttpAction.*
import com.normation.rudder.rest.EndpointSchema.syntax.*
import enumeratum.*
import sourcecode.Line

/*
 * This file contains the definition of all endpoint schema
 * in Rudder base.
 *
 * Any module wanting to contribute an API
 * More preciselly, it defines the data format of
 * an endpoint descriptor, an endpoint descriptor container,
 * and prefil all the known endpoints into the corner.
 *
 * It also defined interpretor for endpoint descriptor toward
 * Lift RestAPI objects.
 *
 */

// we need a marker trait to get endpoint in a sorted way. Bad, but nothing better
// safe rewriting sealerate
trait SortIndex {
  protected[rest] def z: Int
}

sealed trait CampaignApi extends EnumEntry with EndpointSchema with GeneralApi with SortIndex
object CampaignApi       extends Enum[CampaignApi] with ApiModuleProvider[CampaignApi] {
  case object GetCampaigns              extends CampaignApi with ZeroParam with StartsAtVersion16 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get all campaigns"
    val (action, path) = GET / "campaigns"
    val dataContainer: Some[String] = Some("campaigns")
  }
  case object GetCampaignEvents         extends CampaignApi with ZeroParam with StartsAtVersion16 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get all campaigns events"
    val (action, path) = GET / "campaigns" / "events"
    val dataContainer: Some[String] = Some("campaignEvents")
  }
  case object GetCampaignEventDetails   extends CampaignApi with OneParam with StartsAtVersion16 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Get a campaigns events details"
    val (action, path) = GET / "campaigns" / "events" / "{id}"
    val dataContainer: Some[String] = Some("campaignEvents")
  }
  case object SaveCampaign              extends CampaignApi with ZeroParam with StartsAtVersion16 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Save a campaign"
    val (action, path) = POST / "campaigns"
    val dataContainer: Some[String] = Some("campaigns")
  }
  case object ScheduleCampaign          extends CampaignApi with OneParam with StartsAtVersion16 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Schedule an event for a campaign"
    val (action, path) = POST / "campaigns" / "{id}" / "schedule"
    val dataContainer: Some[String] = Some("campaigns")
  }
  case object GetCampaignDetails        extends CampaignApi with OneParam with StartsAtVersion16 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Get a campaign"
    val (action, path) = GET / "campaigns" / "{id}"
    val dataContainer: Some[String] = Some("campaigns")
  }
  case object DeleteCampaign            extends CampaignApi with OneParam with StartsAtVersion16 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Delete a campaign"
    val (action, path) = DELETE / "campaigns" / "{id}"
    val dataContainer: Option[String] = None
  }
  case object GetCampaignEventsForModel extends CampaignApi with OneParam with StartsAtVersion16 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Get events for a campaign"
    val (action, path) = GET / "campaigns" / "{id}" / "events"
    val dataContainer: Some[String] = Some("campaignEvents")
  }
  case object SaveCampaignEvent         extends CampaignApi with OneParam with StartsAtVersion16 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Save a campaign event"
    val (action, path) = POST / "campaigns" / "events" / "{id}"
    val dataContainer: Some[String] = Some("campaignEvents")
  }
  case object DeleteCampaignEvent       extends CampaignApi with OneParam with StartsAtVersion16 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Delete a campaign event"
    val (action, path) = DELETE / "campaigns" / "events" / "{id}"
    val dataContainer: Option[String] = None
  }
  def endpoints: List[CampaignApi] = values.toList.sortBy(_.z)

  def values = findValues
}

sealed trait ComplianceApi extends EnumEntry with EndpointSchema with SortIndex
object ComplianceApi       extends Enum[ComplianceApi] with ApiModuleProvider[ComplianceApi] {

  case object GetRulesCompliance   extends ComplianceApi with GeneralApi with ZeroParam with StartsAtVersion7 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get compliance information for all rules"
    val (action, path) = GET / "compliance" / "rules"
    val dataContainer: Some[String] = Some("rules")
  }
  case object GetRulesComplianceId extends ComplianceApi with GeneralApi with OneParam with StartsAtVersion7 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Get compliance information for the given rule"
    val (action, path) = GET / "compliance" / "rules" / "{id}"
    val dataContainer: Some[String] = Some("rules")
  }
  case object GetNodesCompliance   extends ComplianceApi with GeneralApi with ZeroParam with StartsAtVersion7 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get compliance information for all nodes"
    val (action, path) = GET / "compliance" / "nodes"
    val dataContainer: Some[String] = Some("nodes")
  }

  case object GetNodeSystemCompliance extends ComplianceApi with InternalApi with OneParam with StartsAtVersion7 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Get compliance information for the given node"
    val (action, path) = GET / "compliance" / "nodes" / "{id}" / "system"
    val dataContainer: Some[String] = Some("nodes")
  }
  case object GetNodeComplianceId     extends ComplianceApi with GeneralApi with OneParam with StartsAtVersion7 with SortIndex   {
    val z: Int = implicitly[Line].value
    val description    = "Get compliance information for the given node"
    val (action, path) = GET / "compliance" / "nodes" / "{id}"
    val dataContainer: Some[String] = Some("nodes")
  }
  case object GetGlobalCompliance     extends ComplianceApi with GeneralApi with ZeroParam with StartsAtVersion10 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get the global compliance (alike what one has on Rudder main dashboard)"
    val (action, path) = GET / "compliance"
    val dataContainer: Some[String] = Some("globalCompliance")
  }

  case object GetDirectiveComplianceId extends ComplianceApi with GeneralApi with OneParam with StartsAtVersion17 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get a directive's compliance"
    val (action, path) = GET / "compliance" / "directives" / "{id}"
    val dataContainer: Some[String] = Some("directiveCompliance")
  }

  case object GetDirectivesCompliance extends ComplianceApi with GeneralApi with ZeroParam with StartsAtVersion17 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get all directive's compliance"
    val (action, path) = GET / "compliance" / "directives"
    val dataContainer: Some[String] = Some("directivesCompliance")
  }

  case object GetNodeGroupComplianceSummary
      extends ComplianceApi with GeneralApi with ZeroParam with StartsAtVersion17 with SortIndex {
    val z              = implicitly[Line].value
    val description    = "Get a node group's compliance summary"
    val (action, path) = GET / "compliance" / "summary" / "groups"
    val dataContainer: Some[String] = Some("groupCompliance")
  }

  case object GetNodeGroupComplianceId extends ComplianceApi with GeneralApi with OneParam with StartsAtVersion17 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get a node group's global compliance"
    val (action, path) = GET / "compliance" / "groups" / "{id}"
    val dataContainer: Some[String] = Some("groupCompliance")
  }

  case object GetNodeGroupComplianceTargetId
      extends ComplianceApi with GeneralApi with OneParam with StartsAtVersion17 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get a node group's targeted compliance"
    val (action, path) = GET / "compliance" / "groups" / "{id}" / "target"
    val dataContainer: Some[String] = Some("groupCompliance")
  }

  def endpoints: List[ComplianceApi] = values.toList.sortBy(_.z)

  def values = findValues
}

sealed trait GroupApi extends EnumEntry with EndpointSchema with SortIndex    {
  override def dataContainer: Some[String] = Some("groups")
}
object GroupApi       extends Enum[GroupApi] with ApiModuleProvider[GroupApi] {
  // API v2
  case object ListGroups               extends GroupApi with GeneralApi with ZeroParam with StartsAtVersion2 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "List all groups with their information"
    val (action, path) = GET / "groups"
  }
  case object CreateGroup              extends GroupApi with GeneralApi with ZeroParam with StartsAtVersion2 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Create a new group"
    val (action, path) = PUT / "groups"
  }
  case object GetGroupTree             extends GroupApi with GeneralApi with ZeroParam with StartsAtVersion6 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "List all group categories and group in a tree format"
    val (action, path) = GET / "groups" / "tree"
  }
  case object GroupDetails             extends GroupApi with GeneralApi with OneParam with StartsAtVersion2 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Get information about the given group"
    val (action, path) = GET / "groups" / "{id}"
  }
  case object DeleteGroup              extends GroupApi with GeneralApi with OneParam with StartsAtVersion2 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Delete given group"
    val (action, path) = DELETE / "groups" / "{id}"
  }
  case object UpdateGroup              extends GroupApi with GeneralApi with OneParam with StartsAtVersion2 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Update given group"
    val (action, path) = POST / "groups" / "{id}"
  }
  case object ReloadGroup              extends GroupApi with GeneralApi with OneParam with StartsAtVersion2 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Update given dynamic group node list"
    val (action, path) = GET / "groups" / "{id}" / "reload"
  }
  case object GroupInheritedProperties extends GroupApi with GeneralApi with OneParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get all proporeties for that group, included inherited ones"
    val (action, path) = GET / "groups" / "{id}" / "inheritedProperties"
  }
  // API v5 updates 'Create' methods but no new endpoints
  // API v6

  case object GroupDisplayInheritedProperties
      extends GroupApi with InternalApi with OneParam with StartsAtVersion13 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    =
      "Get all proporeties for that group, included inherited ones, for displaying in group property tab (internal)"
    val (action, path) = GET / "groups" / "{id}" / "displayInheritedProperties"
  }
  case object GetGroupCategoryDetails extends GroupApi with GeneralApi with OneParam with StartsAtVersion6 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get information about the given group category"
    val (action, path) = GET / "groups" / "categories" / "{id}"
    override def dataContainer: Some[String] = Some("groupCategories")
  }
  case object DeleteGroupCategory extends GroupApi with GeneralApi with OneParam with StartsAtVersion6 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Delete given group category"
    val (action, path) = DELETE / "groups" / "categories" / "{id}"
    override def dataContainer: Some[String] = Some("groupCategories")
  }
  case object UpdateGroupCategory extends GroupApi with GeneralApi with OneParam with StartsAtVersion6 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Update information for given group category"
    val (action, path) = POST / "groups" / "categories" / "{id}"
    override def dataContainer: Some[String] = Some("groupCategories")
  }
  case object CreateGroupCategory extends GroupApi with GeneralApi with ZeroParam with StartsAtVersion6 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Create a new group category"
    val (action, path) = PUT / "groups" / "categories"
    override def dataContainer: Some[String] = Some("groupCategories")
  }

  def endpoints: List[GroupApi] = values.toList.sortBy(_.z)

  def values = findValues
}

sealed trait DirectiveApi extends EnumEntry with EndpointSchema with GeneralApi with SortIndex {
  override def dataContainer: Some[String] = Some("directives")
}
object DirectiveApi       extends Enum[DirectiveApi] with ApiModuleProvider[DirectiveApi]      {

  case object ListDirectives     extends DirectiveApi with ZeroParam with StartsAtVersion2 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "List all directives"
    val (action, path) = GET / "directives"
  }
  case object DirectiveTree      extends DirectiveApi with ZeroParam with StartsAtVersion14 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get Directive tree"
    val (action, path) = GET / "directives" / "tree"
  }
  case object DirectiveDetails   extends DirectiveApi with OneParam with StartsAtVersion2 with SortIndex   {
    val z: Int = implicitly[Line].value
    val description    = "Get information about given directive"
    val (action, path) = GET / "directives" / "{id}"
  }
  case object DirectiveRevisions extends DirectiveApi with OneParam with StartsAtVersion14 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Get revisions for given directive"
    val (action, path) = GET / "directives" / "{id}" / "revisions"
  }
  case object CreateDirective    extends DirectiveApi with ZeroParam with StartsAtVersion2 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Create a new directive or clone an existing one"
    val (action, path) = PUT / "directives"
  }
  case object DeleteDirective    extends DirectiveApi with OneParam with StartsAtVersion2 with SortIndex   {
    val z: Int = implicitly[Line].value
    val description    = "Delete given directive"
    val (action, path) = DELETE / "directives" / "{id}"
  }
  case object CheckDirective     extends DirectiveApi with OneParam with StartsAtVersion2 with SortIndex   {
    val z: Int = implicitly[Line].value
    val description    = "Check if the given directive can be migrated to target technique version"
    val (action, path) = POST / "directives" / "{id}" / "check"
  }
  case object UpdateDirective    extends DirectiveApi with OneParam with StartsAtVersion2 with SortIndex   {
    val z: Int = implicitly[Line].value
    val description    = "Update given directive information"
    val (action, path) = POST / "directives" / "{id}"
  }

  def endpoints: List[DirectiveApi] = values.toList.sortBy(_.z)

  def values = findValues
}

sealed trait NodeApi extends EnumEntry with EndpointSchema with SortIndex  {
  override def dataContainer: Option[String] = Some("nodes")
}
object NodeApi       extends Enum[NodeApi] with ApiModuleProvider[NodeApi] {

  case object ListAcceptedNodes  extends NodeApi with GeneralApi with ZeroParam with StartsAtVersion2 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "List all accepted nodes with configurable details level"
    val (action, path) = GET / "nodes"
  }
  case object GetNodesStatus     extends NodeApi with GeneralApi with ZeroParam with StartsAtVersion13 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get the status (pending, accepted, unknown) of the comma separated list of nodes given by `ids` parameter"
    val (action, path) = GET / "nodes" / "status"
  }
  case object ListPendingNodes   extends NodeApi with GeneralApi with ZeroParam with StartsAtVersion2 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "List all pending nodes with configurable details level"
    val (action, path) = GET / "nodes" / "pending"
  }
  case object PendingNodeDetails extends NodeApi with GeneralApi with OneParam with StartsAtVersion2 with SortIndex   {
    val z: Int = implicitly[Line].value
    val description    = "Get information about the given pending node"
    val (action, path) = GET / "nodes" / "pending" / "{id}"
  }
  case object NodeDetails        extends NodeApi with GeneralApi with OneParam with StartsAtVersion2 with SortIndex   {
    val z: Int = implicitly[Line].value
    val description    = "Get information about the given accepted node"
    val (action, path) = GET / "nodes" / "{id}"
  }

  case object NodeInheritedProperties extends NodeApi with GeneralApi with OneParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get all properties for that node, included inherited ones"
    val (action, path) = GET / "nodes" / "{id}" / "inheritedProperties"
    override def dataContainer: Option[String] = None
  }

  case object NodeGlobalScore extends NodeApi with InternalApi with OneParam with StartsAtVersion19 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get global score for a Node"
    val (action, path) = GET / "nodes" / "{id}" / "score"
    override def dataContainer: Option[String] = None
  }

  case object NodeScoreDetails extends NodeApi with InternalApi with OneParam with StartsAtVersion19 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get all score details for a Node"
    val (action, path) = GET / "nodes" / "{id}" / "score" / "details"
    override def dataContainer: Option[String] = None
  }

  case object NodeScoreDetail extends NodeApi with InternalApi with TwoParam with StartsAtVersion19 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get a score details for a Node"
    val (action, path) = GET / "nodes" / "{id}" / "score" / "details" / "{name}"
    override def dataContainer: Some[String] = Some("score")
  }

  case object ApplyPolicyAllNodes     extends NodeApi with GeneralApi with ZeroParam with StartsAtVersion8 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Ask all nodes to start a run with the given policy"
    val (action, path) = POST / "nodes" / "applyPolicy"
    override def dataContainer: Option[String] = None
  }
  case object ChangePendingNodeStatus extends NodeApi with GeneralApi with ZeroParam with StartsAtVersion2 with SortIndex {
    val z: Int = implicitly[Line].value
    override val name  = "changePendingNodeStatus"
    val description    = "Accept or refuse pending nodes"
    val (action, path) = POST / "nodes" / "pending"
  }

  // WARNING: read_only user can access this endpoint
  //    No modifications are performed here
  //    POST over GET is required here because we can provide too many information to be passed as URL parameters
  case object NodeDisplayInheritedProperties
      extends NodeApi with InternalApi with OneParam with StartsAtVersion13 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get all properties for that node, included inherited ones, for displaying in node property tab (internal)"
    val (action, path) = GET / "nodes" / "{id}" / "displayInheritedProperties"
    override def dataContainer: Option[String] = None
  }
  case object NodeDetailsTable extends NodeApi with InternalApi with ZeroParam with StartsAtVersion13 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Getting data to build a Node table"
    val (action, path) = POST / "nodes" / "details"
  }
  case object NodeDetailsSoftware extends NodeApi with InternalApi with OneParam with StartsAtVersion13 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Getting a software version for a set of Nodes"
    val (action, path) = POST / "nodes" / "details" / "software" / "{software}"
  }
  case object NodeDetailsProperty extends NodeApi with InternalApi with OneParam with StartsAtVersion13 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Getting a property value for a set of Nodes"
    val (action, path) = POST / "nodes" / "details" / "property" / "{property}"
  }
  case object UpdateNode extends NodeApi with GeneralApi with OneParam with StartsAtVersion5 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Update given node information (node properties, policy mode...)"
    val (action, path) = POST / "nodes" / "{id}"
    override def dataContainer: Option[String] = None
  }
  case object DeleteNode extends NodeApi with GeneralApi with OneParam with StartsAtVersion2 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Delete given node"
    val (action, path) = DELETE / "nodes" / "{id}"
  }
  case object ChangePendingNodeStatus2 extends NodeApi with GeneralApi with OneParam with StartsAtVersion2 with SortIndex {
    val z: Int = implicitly[Line].value
    override val name  = "changePendingNodeStatus"
    val description    = "Accept or refuse given pending node"
    val (action, path) = POST / "nodes" / "pending" / "{id}"
  }
  case object ApplyPolicy extends NodeApi with GeneralApi with OneParam with StartsAtVersion8 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Ask given node to start a run with the given policy"
    val (action, path) = POST / "nodes" / "{id}" / "applyPolicy"
  }
  case object CreateNodes extends NodeApi with GeneralApi with ZeroParam with StartsAtVersion16 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Create one of more new nodes"
    val (action, path) = PUT / "nodes"

    override def dataContainer: Option[String] = None
  }
  def endpoints: List[NodeApi] = values.toList.sortBy(_.z)

  def values = findValues
}

sealed trait ChangesApi extends EnumEntry with EndpointSchema with InternalApi with SortIndex {
  override def dataContainer: Option[String] = None
}
object ChangesApi       extends Enum[ChangesApi] with ApiModuleProvider[ChangesApi]           {

  case object GetRecentChanges extends ChangesApi with ZeroParam with StartsAtVersion14 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get changes for all Rules over the last 3 days (internal)"
    val (action, path) = GET / "changes"
  }

  case object GetRuleRepairedReports extends ChangesApi with OneParam with StartsAtVersion14 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get all repaired report for a Rule in a interval of time specified as parameter(internal)"
    val (action, path) = GET / "changes" / "{ruleId}"
  }

  def endpoints: List[ChangesApi] = values.toList.sortBy(_.z)

  def values = findValues
}
sealed trait ParameterApi extends EnumEntry with EndpointSchema with GeneralApi with SortIndex {
  override def dataContainer: Some[String] = Some("parameters")
}
object ParameterApi       extends Enum[ParameterApi] with ApiModuleProvider[ParameterApi]      {

  case object ListParameters   extends ParameterApi with ZeroParam with StartsAtVersion2 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "List all global parameters"
    val (action, path) = GET / "parameters"
  }
  case object CreateParameter  extends ParameterApi with ZeroParam with StartsAtVersion2 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Create a new parameter"
    val (action, path) = PUT / "parameters"
  }
  case object ParameterDetails extends ParameterApi with OneParam with StartsAtVersion2 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Get information about the given parameter"
    val (action, path) = GET / "parameters" / "{id}"
  }
  case object DeleteParameter  extends ParameterApi with OneParam with StartsAtVersion2 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Delete given parameter"
    val (action, path) = DELETE / "parameters" / "{id}"
  }
  case object UpdateParameter  extends ParameterApi with OneParam with StartsAtVersion2 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Update information about given parameter"
    val (action, path) = POST / "parameters" / "{id}"
  }

  def endpoints: List[ParameterApi] = values.toList.sortBy(_.z)

  def values = findValues
}

sealed trait SettingsApi extends EnumEntry with EndpointSchema with GeneralApi with SortIndex {
  override def dataContainer: Some[String] = Some("settings")
}
object SettingsApi       extends Enum[SettingsApi] with ApiModuleProvider[SettingsApi]        {
  case object GetAllSettings        extends SettingsApi with ZeroParam with StartsAtVersion6 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Get information about all Rudder settings"
    val (action, path) = GET / "settings"
  }
  case object GetAllAllowedNetworks extends SettingsApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "List all allowed networks"
    val (action, path) = GET / "settings" / "allowed_networks"
  }
  case object GetAllowedNetworks    extends SettingsApi with OneParam with StartsAtVersion11 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "List all allowed networks for one relay"
    val (action, path) = GET / "settings" / "allowed_networks" / "{nodeId}"
  }
  case object ModifyAllowedNetworks extends SettingsApi with OneParam with StartsAtVersion11 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Update all allowed networks for one relay"
    val (action, path) = POST / "settings" / "allowed_networks" / "{nodeId}"
  }

  case object ModifyDiffAllowedNetworks extends SettingsApi with OneParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Modify some allowed networks for one relay with a diff structure"
    val (action, path) = POST / "settings" / "allowed_networks" / "{nodeId}" / "diff"
  }

  case object GetSetting     extends SettingsApi with OneParam with StartsAtVersion6 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Get information about given Rudder setting"
    val (action, path) = GET / "settings" / "{key}"
  }
  case object ModifySettings extends SettingsApi with ZeroParam with StartsAtVersion6 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Update Rudder settings"
    val (action, path) = POST / "settings"
  }
  case object ModifySetting  extends SettingsApi with OneParam with StartsAtVersion6 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Update given Rudder setting"
    val (action, path) = POST / "settings" / "{key}"
  }

  def endpoints: List[SettingsApi] = values.toList.sortBy(_.z)

  def values = findValues
}

sealed trait PluginApi extends EnumEntry with EndpointSchema with GeneralApi with SortIndex {
  override def dataContainer: Option[String] = Some("plugins")
}
object PluginApi       extends Enum[PluginApi] with ApiModuleProvider[PluginApi]            {

  case object GetPluginsInfo        extends PluginApi with ZeroParam with StartsAtVersion14 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "List plugin information"
    val (action, path) = GET / "plugins" / "info"
  }
  case object GetPluginsSettings    extends PluginApi with ZeroParam with StartsAtVersion14 with SortIndex {
    val z:                      Int            = implicitly[Line].value
    override def dataContainer: Option[String] = None
    val description    = "List plugin system settings"
    val (action, path) = GET / "plugins" / "settings"
  }
  case object UpdatePluginsSettings extends PluginApi with ZeroParam with StartsAtVersion14 with SortIndex {
    val z:                      Int            = implicitly[Line].value
    override def dataContainer: Option[String] = None
    val description    = "Update plugin system settings"
    val (action, path) = POST / "plugins" / "settings"
  }
  def endpoints: List[PluginApi] = values.toList.sortBy(_.z)

  def values = findValues
}

sealed trait TechniqueApi     extends EnumEntry with EndpointSchema with SortIndex {
  override def dataContainer: Some[String] = Some("techniques")
}
sealed trait TechniqueApiPub  extends TechniqueApi with GeneralApi
sealed trait TechniqueApiPriv extends TechniqueApi with InternalApi

object TechniqueApi extends Enum[TechniqueApi] with ApiModuleProvider[TechniqueApi] {

  case object GetTechniques             extends TechniqueApiPub with ZeroParam with StartsAtVersion6 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Get all Techniques metadata"
    val (action, path) = GET / "techniques"
  }
  case object UpdateTechniques          extends TechniqueApiPub with ZeroParam with StartsAtVersion14 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "reload techniques metadata from file system"
    val (action, path) = POST / "techniques" / "reload"
  }
  case object GetAllTechniqueCategories extends TechniqueApiPub with ZeroParam with StartsAtVersion14 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get all technique categories"
    val (action, path) = GET / "techniques" / "categories"
  }
  case object ListTechniques            extends TechniqueApiPub with ZeroParam with StartsAtVersion14 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "List all techniques version"
    val (action, path) = GET / "techniques" / "versions"
  }
  case object ListTechniquesDirectives  extends TechniqueApiPub with OneParam with StartsAtVersion6 with SortIndex   {
    val z: Int = implicitly[Line].value
    val description    = "List directives derived from given technique"
    val (action, path) = GET / "techniques" / "{name}" / "directives"
    override def dataContainer: Some[String] = Some("directives")
  }
  case object ListTechniqueDirectives   extends TechniqueApiPub with TwoParam with StartsAtVersion6 with SortIndex   {
    val z: Int = implicitly[Line].value
    val description    = "List directives derived from given technique for given version"
    val (action, path) = GET / "techniques" / "{name}" / "{version}" / "directives"
    override def dataContainer: Some[String] = Some("directives")
  }
  case object TechniqueRevisions        extends TechniqueApiPub with TwoParam with StartsAtVersion14 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Get revisions for given technique"
    val (action, path) = GET / "techniques" / "{name}" / "{version}" / "revisions"
  }

  case object UpdateTechnique          extends TechniqueApiPub with TwoParam with StartsAtVersion14 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Update technique created with technique editor"
    val (action, path) = POST / "techniques" / "{techniqueId}" / "{version}"
  }
  case object CreateTechnique          extends TechniqueApiPub with ZeroParam with StartsAtVersion14 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Create a new technique in Rudder from a technique in the technique editor"
    val (action, path) = PUT / "techniques"
  }
  case object DeleteTechnique          extends TechniqueApiPub with TwoParam with StartsAtVersion14 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Delete a technique from technique editor"
    val (action, path) = DELETE / "techniques" / "{techniqueId}" / "{techniqueVersion}"
  }
  case object GetResources             extends TechniqueApiPub with TwoParam with StartsAtVersion14 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Get currently deployed resources of a technique"
    val (action, path) = GET / "techniques" / "{techniqueId}" / "{techniqueVersion}" / "resources"
  }
  case object GetNewResources          extends TechniqueApiPub with TwoParam with StartsAtVersion14 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Get resources of a technique draft"
    val (action, path) = GET / "drafts" / "{techniqueId}" / "{techniqueVersion}" / "resources"
  }
  case object CopyResourcesWhenCloning extends TechniqueApiPriv with TwoParam with StartsAtVersion14 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Copy resources from a technique to a technique draft"
    val (action, path) = POST / "drafts" / "{techniqueId}" / "{techniqueVersion}" / "resources" / "clone"
  }
  case object GetTechniqueAllVersion   extends TechniqueApiPub with OneParam with StartsAtVersion14 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Get all Techniques metadata"
    val (action, path) = GET / "techniques" / "{techniqueId}"
  }
  case object GetTechnique             extends TechniqueApiPub with TwoParam with StartsAtVersion14 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Get all Techniques metadata"
    val (action, path) = GET / "techniques" / "{techniqueId}" / "{techniqueVersion}"
  }
  /*
   * Method are returned sorted alpha-numericaly
   */
  case object GetMethods               extends TechniqueApiPub with ZeroParam with StartsAtVersion14 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get all methods metadata"
    val (action, path) = GET / "methods"
  }
  case object UpdateMethods            extends TechniqueApiPub with ZeroParam with StartsAtVersion14 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "reload methods metadata from file system"
    val (action, path) = POST / "methods" / "reload"
  }
  case object CheckTechnique           extends TechniqueApiPub with ZeroParam with StartsAtVersion16 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Check if a techniques is valid yaml, with rudderc compilation, with various output (json ? yaml ?)"
    val (action, path) = POST / "techniques" / "check"
  }

  def endpoints: List[TechniqueApi] = values.toList.sortBy(_.z)

  def values = findValues
}

sealed trait RuleApi extends EnumEntry with EndpointSchema with GeneralApi with SortIndex {
  override def dataContainer: Option[String] = Some("rules")
}
object RuleApi       extends Enum[RuleApi] with ApiModuleProvider[RuleApi]                {

  case object ListRules              extends RuleApi with ZeroParam with StartsAtVersion2 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "List all rules with their information"
    val (action, path) = GET / "rules"
  }
  case object CreateRule             extends RuleApi with ZeroParam with StartsAtVersion2 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Create a new rule"
    val (action, path) = PUT / "rules"
  }
  // must be before rule details, else it is never reached
  case object GetRuleTree            extends RuleApi with ZeroParam with StartsAtVersion6 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get rule categories and rule structured in a tree format"
    val (action, path) = GET / "rules" / "tree"
    override def dataContainer: Option[String] = None
  }
  case object RuleDetails            extends RuleApi with OneParam with StartsAtVersion2 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Get information about given rule"
    val (action, path) = GET / "rules" / "{id}"
  }
  case object DeleteRule             extends RuleApi with OneParam with StartsAtVersion2 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Delete given rule"
    val (action, path) = DELETE / "rules" / "{id}"
  }
  case object UpdateRule             extends RuleApi with OneParam with StartsAtVersion2 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Update information about given rule"
    val (action, path) = POST / "rules" / "{id}"
  }
  case object GetRuleCategoryDetails extends RuleApi with OneParam with StartsAtVersion6 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Get information about given rule category"
    val (action, path) = GET / "rules" / "categories" / "{id}"
    override def dataContainer: Option[String] = None
  }
  case object DeleteRuleCategory     extends RuleApi with OneParam with StartsAtVersion6 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Delete given category"
    val (action, path) = DELETE / "rules" / "categories" / "{id}"
    override def dataContainer: Some[String] = Some("rulesCategories")
  }
  case object UpdateRuleCategory     extends RuleApi with OneParam with StartsAtVersion6 with SortIndex  {
    val z: Int = implicitly[Line].value
    val description    = "Update information about given rule category"
    val (action, path) = POST / "rules" / "categories" / "{id}"
    override def dataContainer: Option[String] = None
  }
  case object CreateRuleCategory     extends RuleApi with ZeroParam with StartsAtVersion6 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Create a new rule category"
    val (action, path) = PUT / "rules" / "categories"
    override def dataContainer: Option[String] = None
  }

  // internal, because non definitive, API to load/unload a specific revision from git to ldap
  case object LoadRuleRevisionForGeneration   extends RuleApi with OneParam with StartsAtVersion14 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Load a revision of a rule from config-repo to ldap, ready for next generation"
    val (action, path) = POST / "rules" / "revision" / "load" / "{id}"
    override def dataContainer: Option[String] = None
  }
  case object UnloadRuleRevisionForGeneration extends RuleApi with OneParam with StartsAtVersion14 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    =
      "Unload a revision of a rule from ldap, it will not be used in next generation. Only rule with a revision can be unloaded"
    val (action, path) = POST / "rules" / "revision" / "unload" / "{id}"
    override def dataContainer: Option[String] = None
  }

  def endpoints: List[RuleApi] = values.toList.sortBy(_.z)

  def values = findValues
}

sealed trait RuleInternalApi extends EnumEntry with EndpointSchema with InternalApi with SortIndex {
  override def dataContainer: Option[String] = Some("rulesinternal")
}
object RuleInternalApi       extends Enum[RuleInternalApi] with ApiModuleProvider[RuleInternalApi] {
  // For the rule detail page
  case object GetRuleNodesAndDirectives extends RuleInternalApi with OneParam with StartsAtVersion14 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get the list of nodes and directives of a rule"
    val (action, path) = GET / "rulesinternal" / "nodesanddirectives" / "{id}"
    override def dataContainer: Option[String] = None
  }

  // For group page
  case object GetGroupRelatedRules extends RuleInternalApi with ZeroParam with StartsAtVersion14 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "List all info of rules in a tree format"
    val (action, path) = GET / "rulesinternal" / "relatedtree"
    override def dataContainer: Option[String] = None
  }

  def endpoints: List[RuleInternalApi] = values.toList.sortBy(_.z)

  def values = findValues
}

sealed trait GroupInternalApi extends EnumEntry with EndpointSchema with InternalApi with SortIndex {
  override def dataContainer: Option[String] = Some("groupsinternal")
}

object GroupInternalApi extends Enum[GroupInternalApi] with ApiModuleProvider[GroupInternalApi] {
  case object GetGroupCategoryTree extends GroupInternalApi with ZeroParam with StartsAtVersion14 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get the tree of groups with bare minimum group information"
    val (action, path) = GET / "groupsinternal" / "categorytree"
    override def dataContainer: Option[String] = None
  }

  def endpoints: List[GroupInternalApi] = values.toList.sortBy(_.z)

  def values = findValues
}

sealed trait ScoreApi extends EnumEntry with EndpointSchema with InternalApi with SortIndex {
  override def dataContainer: Option[String] = Some("scores")
}

object ScoreApi extends Enum[ScoreApi] with ApiModuleProvider[ScoreApi] {

  case object GetScoreList extends ScoreApi with ZeroParam with StartsAtVersion19 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "List all info of all available scores"
    val (action, path) = GET / "scores" / "list"

    override def dataContainer: Option[String] = None
  }
  def endpoints: List[ScoreApi] = values.toList.sortBy(_.z)
  def values = findValues
}

sealed trait SystemApi extends EnumEntry with EndpointSchema with GeneralApi with SortIndex {
  override def dataContainer: Option[String] = None // nothing normalized here ?
}
object SystemApi       extends Enum[SystemApi] with ApiModuleProvider[SystemApi]            {

  case object Info extends SystemApi with ZeroParam with StartsAtVersion10 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get information about system installation (version, etc)"
    val (action, path) = GET / "system" / "info"
  }

  case object Status extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get Api status"
    val (action, path) = GET / "system" / "status"
  }

  case object DebugInfo extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Launch the support info script and get the result"
    val (action, path) = GET / "system" / "debug" / "info"

  }

  // For now, the techniques reload endpoint is implemented in the System API
  // but moving it inside the Techniques API should be discussed.

  case object ReloadAll extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "reload both techniques and dynamic groups"
    val (action, path) = POST / "system" / "reload"
  }

  case object TechniquesReload extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "reload all techniques" // automatically done every 5 minutes
    val (action, path) = POST / "system" / "reload" / "techniques"
  }

  case object DyngroupsReload extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "reload all dynamic groups"
    val (action, path) = POST / "system" / "reload" / "groups"
  }

  case object PoliciesUpdate extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "update policies"
    val (action, path) = POST / "system" / "update" / "policies"
  }

  case object PoliciesRegenerate extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "regenerate all policies"
    val (action, path) = POST / "system" / "regenerate" / "policies"
  }

  // Archive list endpoints

  case object ArchivesGroupsList extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "list groups archives"
    val (action, path) = GET / "system" / "archives" / "groups"
  }

  case object ArchivesDirectivesList extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "list directives archives"
    val (action, path) = GET / "system" / "archives" / "directives"
  }

  case object ArchivesRulesList extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "list rules archives"
    val (action, path) = GET / "system" / "archives" / "rules"
  }

  case object ArchivesParametersList extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "list parameters archives"
    val (action, path) = GET / "system" / "archives" / "parameters"
  }

  case object ArchivesFullList extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "list all archives"
    val (action, path) = GET / "system" / "archives" / "full"
  }

  // Archive restore endpoints

  // Latest archive

  case object RestoreGroupsLatestArchive extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "restore groups latest archive"
    val (action, path) = POST / "system" / "archives" / "groups" / "restore" / "latestArchive"
  }

  case object RestoreDirectivesLatestArchive extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "restore directives latest archive"
    val (action, path) = POST / "system" / "archives" / "directives" / "restore" / "latestArchive"
  }

  case object RestoreRulesLatestArchive extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "restore rules latest archive"
    val (action, path) = POST / "system" / "archives" / "rules" / "restore" / "latestArchive"
  }

  case object RestoreParametersLatestArchive extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "restore parameters latest archive"
    val (action, path) = POST / "system" / "archives" / "parameters" / "restore" / "latestArchive"
  }

  case object RestoreFullLatestArchive extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "restore all latest archive"
    val (action, path) = POST / "system" / "archives" / "full" / "restore" / "latestArchive"
  }

  // Latest commit
  case object RestoreGroupsLatestCommit extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "restore groups latest commit"
    val (action, path) = POST / "system" / "archives" / "groups" / "restore" / "latestCommit"
  }

  case object RestoreDirectivesLatestCommit extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "restore directives latest commit"
    val (action, path) = POST / "system" / "archives" / "directives" / "restore" / "latestCommit"
  }

  case object RestoreRulesLatestCommit extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "restore rules latest commit"
    val (action, path) = POST / "system" / "archives" / "rules" / "restore" / "latestCommit"
  }

  case object RestoreParametersLatestCommit extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "restore parameters latest commit"
    val (action, path) = POST / "system" / "archives" / "parameters" / "restore" / "latestCommit"
  }

  case object RestoreFullLatestCommit extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "restore full latest commit"
    val (action, path) = POST / "system" / "archives" / "full" / "restore" / "latestCommit"
  }

  // Restore a particular entity base on its datetime

  case object ArchiveGroupDateRestore extends SystemApi with OneParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "restore a group archive created on date passed as parameter"
    val (action, path) = POST / "system" / "archives" / "groups" / "restore" / "{dateTime}"
  }

  case object ArchiveDirectiveDateRestore extends SystemApi with OneParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "restore a directive archive created on date passed as parameter"
    val (action, path) = POST / "system" / "archives" / "directives" / "restore" / "{dateTime}"
  }

  case object ArchiveRuleDateRestore extends SystemApi with OneParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "restore a rule archive created on date passed as parameter"
    val (action, path) = POST / "system" / "archives" / "rules" / "restore" / "{dateTime}"
  }

  case object ArchiveParameterDateRestore extends SystemApi with OneParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "restore a parameter archive created on date passed as parameter"
    val (action, path) = POST / "system" / "archives" / "parameters" / "restore" / "{dateTime}"
  }

  case object ArchiveFullDateRestore extends SystemApi with OneParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "restore a full archive created on date passed as parameter"
    val (action, path) = POST / "system" / "archives" / "full" / "restore" / "{dateTime}"
  }

  // Archive endpoints

  case object ArchiveGroups extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "archive groups"
    val (action, path) = POST / "system" / "archives" / "groups"
  }

  case object ArchiveDirectives extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "archive directives"
    val (action, path) = POST / "system" / "archives" / "directives"
  }

  case object ArchiveRules extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "archive rules"
    val (action, path) = POST / "system" / "archives" / "rules"
  }

  case object ArchiveParameters extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "archive parameters"
    val (action, path) = POST / "system" / "archives" / "parameters"
  }

  case object ArchiveFull extends SystemApi with ZeroParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "archive full"
    val (action, path) = POST / "system" / "archives" / "full"
  }

  // ZIP Archive endpoints

  case object GetGroupsZipArchive extends SystemApi with OneParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get a groups zip archive based on its commit id"
    val (action, path) = GET / "system" / "archives" / "groups" / "zip" / "{commitId}"
  }

  case object GetDirectivesZipArchive extends SystemApi with OneParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get a directives zip archive based on its commit id"
    val (action, path) = GET / "system" / "archives" / "directives" / "zip" / "{commitId}"
  }

  case object GetRulesZipArchive extends SystemApi with OneParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get a rules zip archive based on its commit id"
    val (action, path) = GET / "system" / "archives" / "rules" / "zip" / "{commitId}"
  }

  case object GetParametersZipArchive extends SystemApi with OneParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get a parameters zip archive based on its commit id"
    val (action, path) = GET / "system" / "archives" / "parameters" / "zip" / "{commitId}"
  }

  case object GetAllZipArchive extends SystemApi with OneParam with StartsAtVersion11 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get a full zip archive based on its commit id"
    val (action, path) = GET / "system" / "archives" / "full" / "zip" / "{commitId}"
  }

  // Health check endpoints

  // This endpoint run all checks to return the result
  case object GetHealthcheckResult extends SystemApi with ZeroParam with StartsAtVersion13 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Result of a health check run"
    val (action, path) = GET / "system" / "healthcheck"
  }

  case object PurgeSoftware extends SystemApi with ZeroParam with StartsAtVersion13 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Trigger an async purge of softwares"
    val (action, path) = POST / "system" / "maintenance" / "purgeSoftware"
  }

  def endpoints: List[SystemApi] = values.toList.sortBy(_.z)

  def values = findValues
}

sealed trait InfoApi extends EnumEntry with EndpointSchema with GeneralApi with SortIndex {
  override def dataContainer: Option[String] = None
}
object InfoApi       extends Enum[InfoApi] with ApiModuleProvider[InfoApi]                {

  case object ApiGeneralInformations extends InfoApi with ZeroParam with StartsAtVersion6 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get information about Rudder public API"
    val (action, path) = GET / "info"
  }

  case object ApiInformations extends InfoApi with OneParam with StartsAtVersion10 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get detailed information about Rudder public API with the given name"
    val (action, path) = GET / "info" / "details" / "{id}"
  }

  case object ApiSubInformations extends InfoApi with OneParam with StartsAtVersion10 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get information about Rudder public API starting with given path"
    val (action, path) = GET / "info" / "{id}"
  }

  def endpoints: List[InfoApi] = values.toList.sortBy(_.z)

  def values = findValues
}

sealed trait HookApi extends EnumEntry with EndpointSchema with InternalApi with SortIndex {
  override def dataContainer: Option[String] = None // nothing normalized here ?
}
object HookApi       extends Enum[HookApi] with ApiModuleProvider[HookApi]                 {
  case object GetHooks extends HookApi with ZeroParam with StartsAtVersion16 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get all hooks"
    val (action, path) = GET / "hooks"
  }

  def endpoints: List[HookApi] = values.toList.sortBy(_.z)

  def values = findValues
}

/*
 * Porting the old "inventory endpoints" APIs to rudder.
 * You have an endpoint for inventory processing status,
 * one to send inventory if you don't want to use file watcher parsing,
 * and control start/stop/restart of file watcher.
 */
sealed trait InventoryApi extends EnumEntry with EndpointSchema with GeneralApi with SortIndex {
  override def dataContainer: Option[String] = None
}
object InventoryApi       extends Enum[InventoryApi] with ApiModuleProvider[InventoryApi]      {

  case object QueueInformation extends InventoryApi with ZeroParam with StartsAtVersion12 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get information about inventory current processing status"
    val (action, path) = GET / "inventories" / "info"
  }

  case object UploadInventory extends InventoryApi with ZeroParam with StartsAtVersion12 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    =
      "Upload an inventory (parameter 'file' and its signature (parameter 'signature') with 'content-disposition:file' attachement format"
    val (action, path) = POST / "inventories" / "upload"
  }

  case object FileWatcherStart extends InventoryApi with ZeroParam with StartsAtVersion12 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Start inventory file watcher (inotify)"
    val (action, path) = POST / "inventories" / "watcher" / "start"
  }

  case object FileWatcherStop extends InventoryApi with ZeroParam with StartsAtVersion12 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Stop inventory file watcher (inotify)"
    val (action, path) = POST / "inventories" / "watcher" / "stop"
  }

  case object FileWatcherRestart extends InventoryApi with ZeroParam with StartsAtVersion12 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Restart inventory file watcher (inotify)"
    val (action, path) = POST / "inventories" / "watcher" / "restart"
  }

  def values:    IndexedSeq[InventoryApi] = findValues
  def endpoints: List[InventoryApi]       = values.toList.sortBy(_.z)
}

/*
 * This API definition need to be in Rudder core because it defines specific
 * user API rights, which is a special thing. The actual implementation will
 * be defined in the API Authorization plugin.
 * Note that these endpoint don't have token ID has parameter because an user can only manage
 * its own token, and Rudder will make the mapping server side.
 */
sealed trait UserApi extends EnumEntry with EndpointSchema with InternalApi with SortIndex {
  override def dataContainer: Option[String] = None
}
object UserApi       extends Enum[UserApi] with ApiModuleProvider[UserApi]                 {
  case object GetApiToken    extends UserApi with ZeroParam with StartsAtVersion10 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Get information about user personal UserApi token"
    val (action, path) = GET / "user" / "api" / "token"
  }
  case object CreateApiToken extends UserApi with ZeroParam with StartsAtVersion10 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Create user personal UserApi token"
    val (action, path) = PUT / "user" / "api" / "token"
  }
  case object DeleteApiToken extends UserApi with ZeroParam with StartsAtVersion10 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Delete user personal UserApi token"
    val (action, path) = DELETE / "user" / "api" / "token"
  }

  case object UpdateApiToken extends UserApi with ZeroParam with StartsAtVersion10 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Update user personal UserApi token"
    val (action, path) = POST / "user" / "api" / "token"
  }

  def endpoints: List[UserApi] = values.toList.sortBy(_.z)

  def values = findValues
}

/*
 * An API for import & export of archives of objects with their dependencies
 */
sealed trait ArchiveApi extends EnumEntry with EndpointSchema with GeneralApi with SortIndex {
  override def dataContainer: Option[String] = None
}
object ArchiveApi       extends Enum[ArchiveApi] with ApiModuleProvider[ArchiveApi]          {
  /*
   * Request format:
   *   ../archives/export?rules=rule_ids&directives=dir_ids&techniques=tech_ids&groups=group_ids&include=scope
   * Where:
   * - rule_ids = xxxx-xxxx-xxx-xxx[,other ids]
   * - dir_ids = xxxx-xxxx-xxx-xxx[,other ids]
   * - group_ids = xxxx-xxxx-xxx-xxx[,other ids]
   * - tech_ids = techniqueName/1.0[,other tech ids]
   * - scope = all (default), none, directives, techniques (implies directive), groups
   */
  case object ExportSimple extends ArchiveApi with ZeroParam with StartsAtVersion16 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Export the list of objects with their dependencies in a policy archive"
    val (action, path) = GET / "archives" / "export"
  }
  case object Import       extends ArchiveApi with ZeroParam with StartsAtVersion16 with SortIndex {
    val z: Int = implicitly[Line].value
    val description    = "Import policy archive"
    val (action, path) = POST / "archives" / "import"
  }

  def endpoints: List[ArchiveApi] = values.toList.sortBy(_.z)

  def values = findValues
}

/*
 * All API.
 */
object AllApi {
  val api: List[EndpointSchema] = {
    ComplianceApi.endpoints :::
    GroupApi.endpoints :::
    DirectiveApi.endpoints :::
    NodeApi.endpoints :::
    ParameterApi.endpoints :::
    SettingsApi.endpoints :::
    SystemApi.endpoints :::
    TechniqueApi.endpoints :::
    RuleApi.endpoints :::
    InventoryApi.endpoints :::
    ArchiveApi.endpoints :::
    InfoApi.endpoints :::
    HookApi.endpoints :::
    // UserApi is not declared here, it will be contributed by plugin
    Nil
  }
}
