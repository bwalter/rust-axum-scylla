use anyhow::Result;
use scylla::{Session, SessionBuilder};

use crate::vehicle;

pub async fn create_session(addr: &str, port: i32) -> Result<Session> {
    let session = SessionBuilder::new()
        .known_node(format!("{}:{}", addr, port))
        .build()
        .await?;

    session
        .query(
            "CREATE KEYSPACE IF NOT EXISTS hello WITH REPLICATION = \
            {'class' : 'SimpleStrategy', 'replication_factor' : 1}",
            &[],
        )
        .await?;

    session.use_keyspace("hello", false).await?;

    vehicle::create_table_if_not_exists(&session).await?;

    Ok(session)
}

pub trait WithDbConstants {
    const TABLE_NAME: &'static str;
}
