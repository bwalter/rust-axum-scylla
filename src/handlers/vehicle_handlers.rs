use std::sync::Arc;

use axum::{
    extract::{self, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{db::queries::VehicleQueries, model::vehicle::Vehicle, response::AppResponseResult};

#[tracing::instrument(err)]
pub async fn post_vehicle(
    Json(payload): Json<Vehicle>,
    queries: extract::Extension<Arc<dyn VehicleQueries>>,
) -> AppResponseResult {
    queries.create_vehicle(&payload).await?;

    Ok((StatusCode::CREATED, Json(payload)).into_response())
}

#[tracing::instrument(err)]
pub async fn get_vehicle(
    Path(vin): Path<String>,
    queries: extract::Extension<Arc<dyn VehicleQueries>>,
) -> AppResponseResult {
    let vehicle = queries.find_one_vehicle(&vin).await?;

    Ok((StatusCode::OK, Json(vehicle)).into_response())
}

#[tracing::instrument(err)]
pub async fn delete_vehicle(
    Path(vin): Path<String>,
    queries: extract::Extension<Arc<dyn VehicleQueries>>,
) -> AppResponseResult {
    queries.delete_one_vehicle(&vin).await?;

    Ok((StatusCode::OK, Json(())).into_response())
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use super::*;
    use crate::{db::queries, error::AppError, model::vehicle};

    #[tokio::test]
    async fn test_post_vehicle_ok() {
        let vehicle = Vehicle {
            vin: "vin".to_string(),
            engine: vehicle::Engine::Combustion,
            ev_data: None,
        };

        let mut mock_queries = queries::MockVehicleQueries::default();
        mock_queries
            .expect_create_vehicle()
            .with(eq(vehicle.clone()))
            .times(1)
            .returning(|_| Ok(()));

        let response = post_vehicle(
            Json(vehicle.clone()),
            extract::Extension(Arc::new(mock_queries)),
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
        let vehicle = Vehicle {
            vin: "vin".to_string(),
            engine: vehicle::Engine::Combustion,
            ev_data: None,
        };

        let mut mock_queries = queries::MockVehicleQueries::default();
        mock_queries
            .expect_create_vehicle()
            .with(eq(vehicle.clone()))
            .times(1)
            .returning(|_| Err(AppError::AlreadyExists("Vehicle")));

        let response = post_vehicle(
            Json(vehicle.clone()),
            extract::Extension(Arc::new(mock_queries)),
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
        let vehicle = Vehicle {
            vin: "vin".to_string(),
            engine: vehicle::Engine::Combustion,
            ev_data: None,
        };

        let mut mock_queries = queries::MockVehicleQueries::default();
        mock_queries
            .expect_create_vehicle()
            .with(eq(vehicle.clone()))
            .times(1)
            .returning(|_| Err("Test error".into()));

        let response = post_vehicle(Json(vehicle), extract::Extension(Arc::new(mock_queries)))
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

        let mut mock_queries = queries::MockVehicleQueries::default();
        mock_queries
            .expect_find_one_vehicle()
            .with(eq("vin"))
            .times(1)
            .returning(move |_| Ok(vehicle_clone.clone()));

        let response = get_vehicle(
            Path("vin".to_string()),
            extract::Extension(Arc::new(mock_queries)),
        )
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(to_bytes(response).await, to_bytes(Json(vehicle)).await);
    }

    #[tokio::test]
    async fn test_get_vehicle_not_found() {
        let mut mock_queries = queries::MockVehicleQueries::default();
        mock_queries
            .expect_find_one_vehicle()
            .with(eq("vin"))
            .times(1)
            .returning(|_| Err(AppError::NotFound("Vehicle")));

        let response = get_vehicle(
            Path("vin".to_string()),
            extract::Extension(Arc::new(mock_queries)),
        )
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
        let mut mock_queries = queries::MockVehicleQueries::default();
        mock_queries
            .expect_find_one_vehicle()
            .with(eq("vin"))
            .times(1)
            .returning(|_| Err("Test error".into()));

        let response = get_vehicle(
            Path("vin".to_string()),
            extract::Extension(Arc::new(mock_queries)),
        )
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            to_bytes(response).await,
            to_bytes(AppError::from("Test error")).await
        );
    }

    #[tokio::test]
    async fn test_delete_vehicle_ok() {
        let mut mock_queries = queries::MockVehicleQueries::default();
        mock_queries
            .expect_delete_one_vehicle()
            .with(eq("vin"))
            .times(1)
            .returning(move |_| Ok(()));

        let response = delete_vehicle(
            Path("vin".to_string()),
            extract::Extension(Arc::new(mock_queries)),
        )
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(to_bytes(response).await, to_bytes(Json(())).await);
    }

    #[tokio::test]
    async fn test_delete_vehicle_not_found() {
        let mut mock_queries = queries::MockVehicleQueries::default();
        mock_queries
            .expect_delete_one_vehicle()
            .with(eq("vin"))
            .times(1)
            .returning(|_| Err(AppError::NotFound("Vehicle")));

        let response = delete_vehicle(
            Path("vin".to_string()),
            extract::Extension(Arc::new(mock_queries)),
        )
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            to_bytes(response).await,
            to_bytes(AppError::NotFound("Vehicle")).await
        );
    }

    #[tokio::test]
    async fn test_delete_vehicle_error() {
        let mut mock_queries = queries::MockVehicleQueries::default();
        mock_queries
            .expect_delete_one_vehicle()
            .with(eq("vin"))
            .times(1)
            .returning(|_| Err("Test error".into()));

        let response = delete_vehicle(
            Path("vin".to_string()),
            extract::Extension(Arc::new(mock_queries)),
        )
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
