// Copyright 2019 Normation SAS
//
// This file is part of Rudder.
//
// Rudder is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// In accordance with the terms of section 7 (7. Additional Terms.) of
// the GNU General Public License version 3, the copyright holders add
// the following Additional permissions:
// Notwithstanding to the terms of section 5 (5. Conveying Modified Source
// Versions) and 6 (6. Conveying Non-Source Forms.) of the GNU General
// Public License version 3, when you create a Related Module, this
// Related Module is not considered as a part of the work and may be
// distributed under the license agreement of your choice.
// A "Related Module" means a set of sources files including their
// documentation that, without modification of the Source Code, enables
// supplementary functions or services in addition to those offered by
// the Software.
//
// Rudder is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Rudder.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
    configuration::{LogComponent, DatabaseConfig},
    data::reporting::{QueryableReport, RunLog},
    error::Error,
};
use diesel::{
    insert_into,
    pg::PgConnection,
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};
use slog::{slog_debug, slog_info, slog_trace};
use slog_scope::{debug, info, trace};

pub mod schema {
    table! {
        use diesel::sql_types::*;

        // Needs to be kept in sync with the database schema
        ruddersysevents {
            id -> BigInt,
            executiondate -> Timestamptz,
            ruleid -> Text,
            directiveid -> Text,
            component -> Text,
            keyvalue -> Nullable<Text>,
            eventtype -> Nullable<Text>,
            msg -> Nullable<Text>,
            policy -> Nullable<Text>,
            nodeid -> Text,
            executiontimestamp -> Nullable<Timestamptz>,
            serial -> Integer,
        }
    }
}

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

pub fn pg_pool(configuration: &DatabaseConfig) -> Result<PgPool, Error> {
    let manager = ConnectionManager::<PgConnection>::new(configuration.url.as_ref());
    Ok(Pool::builder()
        .max_size(configuration.max_pool_size)
        .build(manager)?)
}

pub fn insert_runlog(pool: &PgPool, runlog: &RunLog) -> Result<(), Error> {
    use self::schema::ruddersysevents::dsl::*;
    let connection = &*pool.get()?;

    // Non perfect as there could be race-conditions
    // but should avoid most duplicates

    let first_report = runlog
        .reports
        .first()
        .expect("a runlog should never be empty");

    trace!("Checking if first report {} is in the database", first_report; "component" => LogComponent::Database, "node" => &first_report.node_id);
    let new_runlog = ruddersysevents
        .filter(
            component
                .eq(&first_report.component)
                .and(nodeid.eq(&first_report.node_id))
                .and(keyvalue.eq(&first_report.key_value))
                .and(eventtype.eq(&first_report.event_type))
                .and(msg.eq(&first_report.msg))
                .and(policy.eq(&first_report.policy))
                .and(executiontimestamp.eq(&first_report.execution_datetime))
                .and(executiondate.eq(&first_report.start_datetime))
                .and(serial.eq(&first_report.serial))
                .and(ruleid.eq(&first_report.rule_id))
                .and(directiveid.eq(&first_report.directive_id)),
        )
        .limit(1)
        .load::<QueryableReport>(connection)
        .expect("Error loading reports")
        .is_empty();

    if new_runlog {
        trace!("Inserting runlog {:#?}", runlog; "component" => LogComponent::Database, "node" => &first_report.node_id);
        connection.transaction::<_, Error, _>(|| {
            for report in &runlog.reports {
                insert_into(ruddersysevents)
                    .values(report)
                    .execute(connection)?;
            }
            Ok(())
        })
    } else {
        info!("The {} runlog was already there, skipping insertion", runlog.info; "component" => LogComponent::Database, "node" => &first_report.node_id);
        debug!(
            "The report that was already present in database is: {}",
            first_report; "component" => LogComponent::Database, "node" => &first_report.node_id
        );
        Ok(())
    }
}
