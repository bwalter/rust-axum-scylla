use async_trait::async_trait;

use crate::{result::AppResult, vehicle::Vehicle};

/// Define all the queries for DB abstraction
///
/// Will be implemented by concrete DB implementation, e.g.:
/// - Scylla client
/// - Mocked database (for tests)
#[async_trait]
pub trait Queries: Send + Sync + 'static {
    async fn create_tables_if_not_exist(&self) -> AppResult<()>;
    async fn create_vehicle(&self, vehicle: &Vehicle) -> AppResult<()>;
    async fn find_one_vehicle(&self, vin: &str) -> AppResult<Vehicle>;
}
