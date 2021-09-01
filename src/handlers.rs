use std::{sync::Arc, time::Duration};

use axum::{
    extract::{self, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use scylla::{frame::value::MaybeUnset, IntoTypedRows};
use serde::Deserialize;

use crate::{
    response::{AppResponseError, AppResponseResult},
    vehicle::{Vehicle, VehicleRow},
};

pub async fn hello() -> AppResponseResult {
    Ok((StatusCode::OK, "Hello, World!").into_response())
}

// Will time out because of the timeout middleware!
pub async fn timeout() -> AppResponseResult {
    tokio::time::sleep(Duration::from_secs(100)).await;
    Ok((StatusCode::OK, "Unreachable").into_response())
}

#[derive(Debug, Deserialize)]
pub struct FindVehicle {
    vin: String,
}

pub async fn find_vehicle(
    db_session: extract::Extension<Arc<scylla::Session>>,
    Query(payload): Query<FindVehicle>,
) -> AppResponseResult {
    let rows = db_session
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

pub async fn create_vehicle(
    db_session: extract::Extension<Arc<scylla::Session>>,
    json_payload: Json<Vehicle>,
) -> AppResponseResult {
    let Json(vehicle) = json_payload;

    let row = VehicleRow::from_vehicle(vehicle.clone());
    let ev_data = if let Some(ref d) = vehicle.ev_data {
        MaybeUnset::Set(d)
    } else {
        MaybeUnset::Unset
    };

    db_session
        .query(
            "INSERT INTO vehicles (vin, engine_type, ev_data) VALUES (?, ?, ?)",
            (&row.vin, row.engine_type, ev_data),
        )
        .await?;

    Ok((StatusCode::CREATED, Json(vehicle)).into_response())
}
