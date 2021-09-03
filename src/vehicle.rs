use scylla::cql_to_rust::{FromCqlVal, FromRow};
use scylla::macros::{FromRow, FromUserType, IntoUserType};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::string::ToString;
use strum_macros::{AsRefStr, EnumString, ToString};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Vehicle {
    pub vin: String,
    pub engine: Engine,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ev_data: Option<EvData>,
}

#[derive(Serialize, Deserialize, ToString, AsRefStr, EnumString, Clone, PartialEq, Debug)]
#[serde(tag = "type")]
pub enum Engine {
    Combustion,
    Phev,
    Ev,
}

impl Engine {}

#[derive(Default, Serialize, Deserialize, FromUserType, IntoUserType, Clone, PartialEq, Debug)]
pub struct EvData {
    pub battery_capacity_in_kwh: i32,
    pub soc_in_percent: i32,
}

#[derive(FromRow, Debug)]
pub struct VehicleRow {
    pub vin: String,
    pub engine_type: String,
    pub ev_data: Option<EvData>,
}

impl VehicleRow {
    pub fn from_vehicle(vehicle: Vehicle) -> Self {
        VehicleRow {
            vin: vehicle.vin,
            engine_type: vehicle.engine.to_string(),
            ev_data: vehicle.ev_data,
        }
    }

    pub fn to_vehicle(self) -> Option<Vehicle> {
        if let Some(engine) = Engine::from_str(&self.engine_type).ok() {
            Some(Vehicle {
                vin: self.vin,
                engine,
                ev_data: self.ev_data,
            })
        } else {
            None
        }
    }
}
