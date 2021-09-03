use std::{sync::Arc, time::Duration};

use axum::{
    extract::{self, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::{db::queries::Queries, response::AppResponseResult, vehicle::Vehicle};

pub async fn hello() -> AppResponseResult {
    Ok((StatusCode::OK, "Hello, World!").into_response())
}

// Will time out because of the timeout middleware!
pub async fn timeout() -> AppResponseResult {
    tokio::time::sleep(Duration::from_secs(100)).await;
    Ok((StatusCode::OK, "Unreachable").into_response())
}

pub async fn create_vehicle(
    queries: extract::Extension<Arc<dyn Queries>>,
    Json(payload): Json<Vehicle>,
) -> AppResponseResult {
    queries.create_vehicle(&payload).await?;

    Ok((StatusCode::CREATED, Json(payload)).into_response())
}

pub async fn find_vehicle(
    queries: extract::Extension<Arc<dyn Queries>>,
    Query(payload): Query<FindVehicle>,
) -> AppResponseResult {
    let vehicle = queries.find_one_vehicle(&payload.vin).await?;

    Ok((StatusCode::OK, Json(vehicle)).into_response())
}

#[derive(Debug, Deserialize)]
pub struct FindVehicle {
    vin: String,
}
