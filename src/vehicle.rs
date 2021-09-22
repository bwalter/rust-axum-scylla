use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumString, ToString};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Vehicle {
    pub vin: String,

    #[serde(rename = "engine_type")]
    pub engine: Engine,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ev_data: Option<EvData>,
}

#[derive(Serialize, Deserialize, ToString, AsRefStr, EnumString, Clone, PartialEq, Debug)]
pub enum Engine {
    Combustion,
    Phev,
    Ev,
}

impl Engine {}

#[derive(Default, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct EvData {
    pub battery_capacity_in_kwh: i32,
    pub soc_in_percent: i32,
}
