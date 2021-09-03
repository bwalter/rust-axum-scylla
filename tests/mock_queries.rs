use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use hello::db::queries::{Queries, QueryResult};
use hello::error::AppError;
use hello::vehicle::Vehicle;

pub struct MockQueries {
    map: Arc<RwLock<HashMap<String, Vehicle>>>,
}

impl MockQueries {
    pub fn new() -> Self {
        MockQueries {
            map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_vehicle(&self, vin: &str) -> Option<Vehicle> {
        self.map.read().unwrap().get(vin).map(Vehicle::clone)
    }

    pub fn insert_vehicle(&self, vehicle: Vehicle) {
        self.map
            .write()
            .unwrap()
            .insert(vehicle.vin.clone(), vehicle);
    }
}

#[async_trait]
impl Queries for MockQueries {
    async fn create_tables_if_not_exist(&self) -> QueryResult<()> {
        Ok(())
    }

    async fn create_vehicle(&self, vehicle: &Vehicle) -> QueryResult<()> {
        let mut map = self.map.write().unwrap();
        map.insert(vehicle.vin.to_string(), vehicle.clone());

        Ok(())
    }

    async fn find_one_vehicle(&self, vin: &str) -> QueryResult<Vehicle> {
        let map = self.map.read().unwrap();
        let vehicle = map.get(vin).ok_or_else(|| AppError::NotFound())?;

        Ok(vehicle.clone())
    }
}
