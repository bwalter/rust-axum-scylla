use async_trait::async_trait;

use crate::{error::AppError, vehicle::Vehicle};

pub type QueryResult<T> = Result<T, AppError>;

#[async_trait]
pub trait Queries: Send + Sync + 'static {
    async fn create_tables_if_not_exist(&self) -> QueryResult<()>;
    async fn create_vehicle(&self, vehicle: &Vehicle) -> QueryResult<()>;
    async fn find_one_vehicle(&self, vin: &str) -> QueryResult<Vehicle>;
}
