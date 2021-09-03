use anyhow::Result;
use std::sync::Arc;

use crate::error::AppError;

use crate::db::queries::Queries;
use crate::db::scylla_queries::ScyllaQueries;

pub mod queries;
pub mod scylla_queries;

pub async fn start_db_session_and_create_queries(
    addr: &str,
    port: u16,
) -> Result<Arc<dyn queries::Queries + Send + Sync + 'static>, AppError> {
    // Database session
    let db_session = scylla::SessionBuilder::new()
        .known_node(format!("{}:{}", addr, port))
        .build()
        .await?;

    let db_session = Arc::new(db_session);

    // Create (lazily-prepared) queries and tables
    let queries = Arc::new(ScyllaQueries::new(db_session));
    queries.create_tables_if_not_exist().await?;

    Ok(queries)
}
