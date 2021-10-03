use async_trait::async_trait;

use crate::{model::vehicle::Vehicle, result::AppResult};

/// Define all the queries for DB abstraction
///
/// Will be implemented by concrete DB implementation, e.g.:
/// - Scylla client
/// - Mocked database (for tests)
#[mockall::automock]
#[async_trait]
pub trait VehicleQueries: std::fmt::Debug + Send + Sync + 'static {
    async fn create_vehicle(&self, vehicle: &Vehicle) -> AppResult<()>;
    async fn find_one_vehicle(&self, vin: &str) -> AppResult<Vehicle>;
    async fn delete_one_vehicle(&self, vin: &str) -> AppResult<()>;
}
