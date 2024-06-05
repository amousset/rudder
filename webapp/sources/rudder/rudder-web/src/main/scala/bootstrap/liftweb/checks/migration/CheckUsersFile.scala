/*
 *************************************************************************************
 * Copyright 2024 Normation SAS
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

package bootstrap.liftweb.checks.migration

import bootstrap.liftweb.BootstrapChecks
import bootstrap.liftweb.BootstrapLogger
import com.normation.errors.*
import com.normation.rudder.users.RudderPasswordEncoder.SecurityLevel
import com.normation.rudder.users.UserFileProcessing
import com.normation.rudder.users.UserFileSecurityLevelMigration
import com.normation.rudder.users.UserManagementIO
import com.normation.zio.UnsafeRun
import zio.*

class CheckUsersFile(migration: UserFileSecurityLevelMigration) extends BootstrapChecks {

  override def description: String = "Check if hash algorithm for user password is a modern one, if not enable unsafe_hashes"

  def allChecks(currentSecurityLevel: SecurityLevel): IOResult[Unit] = {
    for {
      _ <- currentSecurityLevel match {
             case SecurityLevel.Modern => ZIO.unit
             case SecurityLevel.Legacy => migration.migrateToModern(UserManagementIO.getUserFilePath(migration.file))
           }
    } yield {}
  }

  override def checks(): Unit = {
    // at startup we need to read the file that may need to be migrated
    (for {
      xml        <- UserFileProcessing.readUserFile(migration.file)
      parsedHash <- UserFileProcessing.parseXmlHash(xml)

      // invalid hash also need to be renamed to modern one
      securityLevel = parsedHash
                        .map(SecurityLevel.fromPasswordEncoderType)
                        .getOrElse(
                          SecurityLevel.Legacy
                        )
      _            <- allChecks(securityLevel)
    } yield {})
      .catchAll(err => BootstrapLogger.error("Error when trying to check users file"))
      .forkDaemon
      .runNow
  }

}
