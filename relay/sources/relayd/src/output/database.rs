use crate::{configuration::DatabaseConfig, data::reporting::RunLog, error::Error};
use diesel::{
    insert_into,
    pg::PgConnection,
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};

pub mod schema {
    table! {
        use diesel::sql_types::*;

        // Needs to be kept in sync with the database schema
        ruddersysevents {
            id -> BigInt,
            executiondate -> Timestamptz,
            nodeid -> Text,
            directiveid -> Text,
            ruleid -> Text,
            serial -> Integer,
            component -> Text,
            keyvalue -> Nullable<Text>,
            executiontimestamp -> Nullable<Timestamptz>,
            eventtype -> Nullable<Text>,
            policy -> Nullable<Text>,
            msg -> Nullable<Text>,
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

    // TODO test presence of runlog before inserting

    let connection = &*pool.get()?;
    connection.transaction::<_, Error, _>(|| {
        for report in &runlog.reports {
            insert_into(ruddersysevents)
                .values(report)
                .execute(connection)?;
        }
        Ok(())
    })
}
