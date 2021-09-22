use std::sync::Arc;

use axum::{
    extract::{self, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::{db::queries::Queries, response::AppResponseResult, vehicle::Vehicle};

#[tracing::instrument(err)]
pub async fn post_vehicle(
    queries: extract::Extension<Arc<dyn Queries>>,
    Json(payload): Json<Vehicle>,
) -> AppResponseResult {
    queries.create_vehicle(&payload).await?;

    Ok((StatusCode::CREATED, Json(payload)).into_response())
}

#[tracing::instrument(err)]
pub async fn get_vehicle(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db::queries, error::AppError, vehicle};

    #[tokio::test]
    async fn test_post_vehicle_ok() {
        let mut mock_queries = queries::MockQueries::default();
        mock_queries
            .expect_create_vehicle()
            .times(1)
            .returning(|_| Ok(()));

        let vehicle = Vehicle {
            vin: "vin".to_string(),
            engine: vehicle::Engine::Combustion,
            ev_data: None,
        };

        let response = post_vehicle(
            extract::Extension(Arc::new(mock_queries)),
            Json(vehicle.clone()),
        )
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(
            to_bytes(response).await,
            to_bytes(Json(vehicle).into_response()).await
        )
    }

    #[tokio::test]
    async fn test_post_vehicle_already_exists() {
        let mut mock_queries = queries::MockQueries::default();
        mock_queries
            .expect_create_vehicle()
            .times(1)
            .returning(|_| Err(AppError::AlreadyExists("Vehicle")));

        let vehicle = Vehicle {
            vin: "vin".to_string(),
            engine: vehicle::Engine::Combustion,
            ev_data: None,
        };

        let response = post_vehicle(
            extract::Extension(Arc::new(mock_queries)),
            Json(vehicle.clone()),
        )
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        assert_eq!(
            to_bytes(response).await,
            to_bytes(AppError::AlreadyExists("Vehicle")).await
        )
    }

    #[tokio::test]
    async fn test_post_vehicle_error() {
        let mut mock_queries = queries::MockQueries::default();
        mock_queries
            .expect_create_vehicle()
            .times(1)
            .returning(|_| Err("Test error".into()));

        let vehicle = Vehicle {
            vin: "vin".to_string(),
            engine: vehicle::Engine::Combustion,
            ev_data: None,
        };

        let response = post_vehicle(extract::Extension(Arc::new(mock_queries)), Json(vehicle))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            to_bytes(response).await,
            to_bytes(AppError::from("Test error")).await
        );
    }

    #[tokio::test]
    async fn test_get_vehicle_ok() {
        let vehicle = Vehicle {
            vin: "vin".to_string(),
            engine: vehicle::Engine::Combustion,
            ev_data: None,
        };
        let vehicle_clone = vehicle.clone();

        let mut mock_queries = queries::MockQueries::default();
        mock_queries
            .expect_find_one_vehicle()
            .times(1)
            .returning(move |_| Ok(vehicle_clone.clone()));

        let params = FindVehicle {
            vin: "vin".to_string(),
        };

        let response = get_vehicle(extract::Extension(Arc::new(mock_queries)), Query(params))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(to_bytes(response).await, to_bytes(Json(vehicle)).await);
    }

    #[tokio::test]
    async fn test_get_vehicle_not_found() {
        let mut mock_queries = queries::MockQueries::default();
        mock_queries
            .expect_find_one_vehicle()
            .times(1)
            .returning(|_| Err(AppError::NotFound("Vehicle")));

        let params = FindVehicle {
            vin: "vin".to_string(),
        };

        let response = get_vehicle(extract::Extension(Arc::new(mock_queries)), Query(params))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        assert_eq!(
            to_bytes(response).await,
            to_bytes(AppError::NotFound("Vehicle")).await
        );
    }

    #[tokio::test]
    async fn test_get_vehicle_error() {
        let mut mock_queries = queries::MockQueries::default();
        mock_queries
            .expect_find_one_vehicle()
            .times(1)
            .returning(|_| Err("Test error".into()));

        let params = FindVehicle {
            vin: "vin".to_string(),
        };

        let response = get_vehicle(extract::Extension(Arc::new(mock_queries)), Query(params))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            to_bytes(response).await,
            to_bytes(AppError::from("Test error")).await
        );
    }

    async fn to_bytes<R>(response: R) -> axum::body::Bytes
    where
        R: IntoResponse,
    {
        hyper::body::to_bytes(response.into_response().into_body())
            .await
            .map_err(Into::into)
            .unwrap()
    }
}
