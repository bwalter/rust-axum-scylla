use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use hello::error::AppError;
use hello::vehicle::Vehicle;
use hello::{db::queries::Queries, result::AppResult};

#[derive(Debug)]
pub struct TestQueries {
    map: Arc<RwLock<HashMap<String, Vehicle>>>,
}

impl TestQueries {
    pub fn new() -> Self {
        TestQueries {
            map: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Queries for TestQueries {
    async fn create_vehicle(&self, vehicle: &Vehicle) -> AppResult<()> {
        let mut map = self.map.write().unwrap();
        match map.entry(vehicle.vin.to_string()) {
            Entry::Occupied(_) => return Err(AppError::AlreadyExists("Vehicle")),
            Entry::Vacant(e) => e.insert(vehicle.clone()),
        };

        Ok(())
    }

    async fn find_one_vehicle(&self, vin: &str) -> AppResult<Vehicle> {
        let map = self.map.read().unwrap();
        let vehicle = map.get(vin).ok_or(AppError::NotFound("Vehicle"))?;

        Ok(vehicle.clone())
    }
}
