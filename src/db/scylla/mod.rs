use anyhow::Result;
use std::sync::Arc;

use crate::db::queries::VehicleQueries;
use crate::db::scylla::vehicle_queries::ScyllaVehicleQueries;
use crate::error::AppError;

pub mod errors;
pub mod vehicle_queries;

pub async fn start_db_session_and_create_queries(
    addr: &str,
    port: u16,
) -> Result<Arc<dyn VehicleQueries + Send + Sync + 'static>, AppError> {
    // Database session
    let db_session = scylla::SessionBuilder::new()
        .known_node(format!("{}:{}", addr, port))
        .build()
        .await?;

    let db_session = Arc::new(db_session);

    // Create (lazily-prepared) queries
    let queries = ScyllaVehicleQueries::try_new(db_session, "hello".to_string()).await?;

    Ok(Arc::new(queries))
}
