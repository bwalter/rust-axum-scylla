use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use hello::error::AppError;
use hello::vehicle::Vehicle;
use hello::{db::queries::Queries, result::AppResult};
use scylla::transport::errors::{DbError, QueryError};

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
    async fn create_tables_if_not_exist(&self) -> AppResult<()> {
        Ok(())
    }

    async fn create_vehicle(&self, vehicle: &Vehicle) -> AppResult<()> {
        let mut map = self.map.write().unwrap();
        match map.entry(vehicle.vin.to_string()) {
            Entry::Occupied(_) => {
                return Err(QueryError::DbError(
                    DbError::AlreadyExists {
                        keyspace: "hello".to_string(),
                        table: "vehicle".to_string(),
                    },
                    "Vehicle already exists".to_string(),
                )
                .into())
            }
            Entry::Vacant(e) => e.insert(vehicle.clone()),
        };

        Ok(())
    }

    async fn find_one_vehicle(&self, vin: &str) -> AppResult<Vehicle> {
        let map = self.map.read().unwrap();
        let vehicle = map.get(vin).ok_or_else(|| AppError::NotFound())?;

        Ok(vehicle.clone())
    }
}
