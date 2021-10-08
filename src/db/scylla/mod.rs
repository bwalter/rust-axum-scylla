use crate::result::AppResult;
use std::sync::Arc;

use crate::error::AppError;
use crate::register_db_error;

pub mod queries;
pub mod vehicle_queries;

pub async fn create_session(addr: &str, port: u16) -> AppResult<scylla::Session> {
    // Database session
    let session = scylla::SessionBuilder::new()
        .known_node(format!("{}:{}", addr, port))
        .build()
        .await?;

    Ok(session)
}

register_db_error!(scylla::transport::errors::NewSessionError);
register_db_error!(scylla::transport::errors::QueryError);
register_db_error!(Arc<scylla::transport::errors::QueryError>);
register_db_error!(scylla::cql_to_rust::FromRowError);
