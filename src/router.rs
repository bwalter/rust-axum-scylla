use axum::response::IntoResponse;
use axum::routing::BoxRoute;
use axum::{AddExtensionLayer, Router};
use std::{
    convert::Infallible,
    sync::{Arc, RwLock},
    time::Duration,
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::db::queries::VehicleQueries;
use crate::error::AppError;
use crate::handlers::vehicle_handlers;
use crate::response::AppResponse;
use crate::state::State;

#[tracing::instrument]
pub fn create_router(queries: Arc<dyn VehicleQueries>) -> Router<BoxRoute> {
    // Shared state
    let shared_state = Arc::new(RwLock::new(State {}));

    // Middlewares: Tower layer stack
    let middleware_stack = ServiceBuilder::new()
        .timeout(Duration::from_secs(5))
        .layer(TraceLayer::new_for_http())
        .into_inner();

    // Route
    use axum::handler::{get, post};
    Router::new()
        .route("/vehicle", post(vehicle_handlers::post_vehicle))
        .route(
            "/vehicle/:vin",
            get(vehicle_handlers::get_vehicle).delete(vehicle_handlers::delete_vehicle),
        )
        .layer(middleware_stack)
        .layer(AddExtensionLayer::new(queries))
        .layer(AddExtensionLayer::new(shared_state))
        .handle_error(|e| Ok::<_, Infallible>(convert_tower_error_into_response(e)))
        .boxed()
}

fn convert_tower_error_into_response(e: tower::BoxError) -> AppResponse {
    let response_error = if e.is::<tower::timeout::error::Elapsed>() {
        // Timeout
        match e.downcast() {
            Ok(e) => AppError::TimeoutError(e),
            Err(e) => AppError::StdError(e),
        }
    } else {
        // Unknown error
        AppError::StdError(e)
    };

    response_error.into_response()
}
