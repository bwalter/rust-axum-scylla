pub mod db;
pub mod error;
pub mod handlers;
pub mod response;
pub mod result;
pub mod state;
pub mod vehicle;

use anyhow::Result;
use axum::handler::{get, post};
use axum::response::IntoResponse;
use axum::routing::BoxRoute;
use axum::{AddExtensionLayer, Router};
use std::net::TcpListener;
use std::{
    convert::Infallible,
    sync::{Arc, RwLock},
    time::Duration,
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::db::queries::Queries;
use crate::error::AppError;
use crate::response::AppResponse;
use crate::state::State;

/// Start the server and wait (forever)
#[tracing::instrument]
pub async fn start(listener: TcpListener, queries: Arc<dyn Queries>) -> Result<()> {
    // Shared state
    let shared_state = Arc::new(RwLock::new(State {}));

    // Create app
    let app = app(shared_state, queries);

    // Run our app with hyper
    tracing::debug!("listening on {:?}", listener);
    axum::Server::from_tcp(listener)?
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

pub fn app(shared_state: Arc<RwLock<State>>, queries: Arc<dyn Queries>) -> Router<BoxRoute> {
    // Middlewares: Tower layer stack
    let middleware_stack = ServiceBuilder::new()
        .timeout(Duration::from_secs(5))
        .layer(TraceLayer::new_for_http())
        .into_inner();

    // Route
    Router::new()
        .route("/vehicle", get(handlers::get_vehicle))
        .route("/vehicle", post(handlers::post_vehicle))
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
