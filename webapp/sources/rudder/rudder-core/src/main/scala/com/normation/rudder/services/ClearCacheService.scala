package com.normation.rudder.services

import com.normation.box._
import com.normation.eventlog.EventActor
import com.normation.eventlog.EventLog
import com.normation.eventlog.EventLogDetails
import com.normation.eventlog.ModificationId
import com.normation.rudder.batch.AsyncDeploymentActor
import com.normation.rudder.batch.AutomaticStartDeployment
import com.normation.rudder.batch.ManualStartDeployment
import com.normation.rudder.domain.eventlog.ClearCacheEventLog
import com.normation.rudder.repository.CachedRepository
import com.normation.rudder.repository.EventLogRepository
import com.normation.rudder.services.policies.nodeconfig.NodeConfigurationHashRepository
import com.normation.utils.StringUuidGenerator
import com.normation.zio._
import net.liftweb.common.Box
import net.liftweb.common.EmptyBox
import net.liftweb.common.Full
import net.liftweb.common.Loggable
import net.liftweb.http.S

trait ClearCacheService {

  /*
   * This method only clear the "node configuration" cache. That will
   * force a full regeneration of all policies
   */
  def clearNodeConfigurationCache(storeEvent: Boolean, actor: EventActor): Box[Unit]

  /*
   * This method clear all caches, which are:
   * - node configurations (force full regen)
   * - cachedAgentRunRepository
   * - recentChangesService
   * - reportingServiceImpl
   */
  def action(actor: EventActor): Box[String]
}

class ClearCacheServiceImpl(
    nodeConfigurationService: NodeConfigurationHashRepository,
    asyncDeploymentAgent:     AsyncDeploymentActor,
    eventLogRepository:       EventLogRepository,
    uuidGen:                  StringUuidGenerator,
    clearableCache:           Seq[CachedRepository]
) extends ClearCacheService with Loggable {

  def clearNodeConfigurationCache(storeEvent: Boolean = true, actor: EventActor) = {
    nodeConfigurationService.deleteAllNodeConfigurations() match {
      case eb: EmptyBox =>
        (eb ?~! "Error while clearing node configuration cache")
      case Full(set) =>
        if (storeEvent) {
          val modId = ModificationId(uuidGen.newUuid)
          eventLogRepository
            .saveEventLog(
              modId,
              ClearCacheEventLog(
                EventLogDetails(
                  modificationId = Some(modId),
                  principal = actor,
                  details = EventLog.emptyDetails,
                  reason = Some("Node configuration cache deleted on user request")
                )
              )
            )
            .runNowLogError(err => logger.error(s"Error when logging cache event: ${err.fullMsg}"))
          logger.debug("Deleting node configurations on user clear cache request")
          asyncDeploymentAgent ! ManualStartDeployment(
            modId,
            actor,
            "Trigger policy generation after clearing configuration cache"
          )
        }
        Full(set)
    }
  }

  def action(actor: EventActor) = {

    S.clearCurrentNotices
    val modId = ModificationId(uuidGen.newUuid)

    // clear agentRun cache
    clearableCache.foreach(_.clearCache())

    // clear node configuration cache
    (for {
      set <- clearNodeConfigurationCache(storeEvent = false, actor)
      _   <- eventLogRepository
               .saveEventLog(
                 modId,
                 ClearCacheEventLog(
                   EventLogDetails(
                     modificationId = Some(modId),
                     principal = actor,
                     details = EventLog.emptyDetails,
                     reason =
                       Some("Clearing cache for: node configuration, recent changes, compliance and node info at user request")
                   )
                 )
               )
               .toBox
    } yield {
      set
    }) match {
      case eb: EmptyBox =>
        val e = eb ?~! "Error when clearing caches"
        logger.error(e.messageChain)
        logger.debug(e.exceptionChain)
        e
      case Full(set) => // ok
        logger.debug("Deleting node configurations on user clear cache request")
        asyncDeploymentAgent ! AutomaticStartDeployment(modId, actor)
        Full("ok")
    }
  }
}
