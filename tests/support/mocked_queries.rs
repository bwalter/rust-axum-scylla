use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use hello::error::AppError;
use hello::vehicle::Vehicle;
use hello::{db::queries::Queries, result::AppResult};

pub struct MockedQueries {
    map: Arc<RwLock<HashMap<String, Vehicle>>>,
}

impl MockedQueries {
    pub fn new() -> Self {
        MockedQueries {
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
impl Queries for MockedQueries {
    async fn create_tables_if_not_exist(&self) -> AppResult<()> {
        Ok(())
    }

    async fn create_vehicle(&self, vehicle: &Vehicle) -> AppResult<()> {
        let mut map = self.map.write().unwrap();
        match map.entry(vehicle.vin.to_string()) {
            Entry::Occupied(_) => return Err(AppError::AlreadyExists()),
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
