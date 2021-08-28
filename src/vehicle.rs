use std::sync::Arc;

use axum::{
    extract::{self, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use scylla::macros::{FromRow, FromUserType, IntoUserType};
use scylla::IntoTypedRows;
use scylla::{
    cql_to_rust::{FromCqlVal, FromRow},
    frame::value::MaybeUnset,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::string::ToString;
use strum_macros::{AsRefStr, EnumString, ToString};

use crate::{
    db::WithDbConstants,
    response::{AppResponseError, AppResponseResult},
};

#[derive(Serialize, Deserialize, Clone)]
pub struct Vehicle {
    vin: String,
    engine: Engine,
    ev_data: Option<EvData>,
}

#[derive(Deserialize, Serialize, ToString, AsRefStr, EnumString, Clone)]
#[serde(tag = "type")]
pub enum Engine {
    Combustion,
    Phev,
    Ev,
}

impl Engine {}

#[derive(Default, Serialize, Deserialize, FromUserType, IntoUserType, Clone)]
pub struct EvData {
    battery_capacity_in_kwh: i32,
    soc_in_percent: i32,
}

#[derive(FromRow)]
struct VehicleRow {
    vin: String,
    engine_type: String,
    ev_data: Option<EvData>,
}

impl VehicleRow {
    fn from_vehicle(vehicle: Vehicle) -> Self {
        VehicleRow {
            vin: vehicle.vin,
            engine_type: vehicle.engine.to_string(),
            ev_data: vehicle.ev_data,
        }
    }

    fn to_vehicle(self) -> Option<Vehicle> {
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

impl WithDbConstants for Vehicle {
    const TABLE_NAME: &'static str = "vehicle";
}

// Route handlers
// ---

#[derive(Debug, Deserialize)]
pub struct FindVehicle {
    vin: String,
}

pub async fn find(
    db: extract::Extension<Arc<scylla::Session>>,
    Query(payload): Query<FindVehicle>,
) -> AppResponseResult {
    let rows = db
        .query("SELECT * FROM vehicles WHERE vin = ?", (&payload.vin,))
        .await?
        .rows
        .ok_or_else(|| AppResponseError::NotFound())?;

    if rows.is_empty() {
        return Err(AppResponseError::NotFound());
    }

    let vehicle = rows
        .into_typed::<VehicleRow>()
        .next()
        .ok_or_else(|| AppResponseError::NotFound())??
        .to_vehicle();

    Ok((StatusCode::CREATED, Json(vehicle)).into_response())
}

pub async fn create(
    db: extract::Extension<Arc<scylla::Session>>,
    json_payload: Json<Vehicle>,
) -> AppResponseResult {
    let Json(vehicle) = json_payload;

    let row = VehicleRow::from_vehicle(vehicle.clone());
    let ev_data = if let Some(ref d) = vehicle.ev_data {
        MaybeUnset::Set(d)
    } else {
        MaybeUnset::Unset
    };

    db.query(
        "INSERT INTO vehicles (vin, engine_type, ev_data) VALUES (?, ?, ?)",
        (&row.vin, row.engine_type, ev_data),
    )
    .await?;

    Ok((StatusCode::CREATED, Json(vehicle)).into_response())
}

pub async fn create_table_if_not_exists(
    session: &scylla::Session,
) -> Result<(), scylla::transport::errors::NewSessionError> {
    session
        .query(
            "CREATE TYPE IF NOT EXISTS ev_data (battery_capacity_in_kwh int, soc_in_percent int)",
            &[],
        )
        .await?;

    session
        .query(
            "CREATE TABLE IF NOT EXISTS vehicles (vin text primary key, engine_type text, ev_data ev_data )",
            &[],
        )
        .await?;

    Ok(())
}
