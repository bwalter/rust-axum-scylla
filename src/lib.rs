pub mod db;
pub mod error;
pub mod handlers;
pub mod response;
pub mod state;
pub mod vehicle;

use anyhow::Result;
use axum::handler::{get, post};
use axum::response::IntoResponse;
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

pub async fn start(
    listener: TcpListener,
    queries: Arc<dyn Queries>,
    ready_tx: Option<tokio::sync::oneshot::Sender<()>>,
) -> Result<()> {
    // Shared state
    let shared_state = Arc::new(RwLock::new(State { count: 0 }));

    // Middlewares: Tower layer stack
    let middleware_stack = ServiceBuilder::new()
        .timeout(Duration::from_secs(15))
        .layer(TraceLayer::new_for_http())
        .into_inner();

    // Route
    let app = Router::new()
        .route("/", get(handlers::hello))
        .route("/timeout", get(handlers::timeout))
        .route("/vehicle", get(handlers::find_vehicle))
        .route("/vehicle", post(handlers::create_vehicle))
        .layer(middleware_stack)
        .layer(AddExtensionLayer::new(queries))
        .layer(AddExtensionLayer::new(shared_state))
        .handle_error(|e| Ok::<_, Infallible>(convert_tower_error_into_response(e)));

    // Run our app with hyper
    tracing::debug!("listening on {:?}", listener);
    let server = axum::Server::from_tcp(listener)?.serve(app.into_make_service());
    if let Some(tx) = ready_tx {
        tx.send(()).unwrap();
    }
    server.await?;

    tracing::debug!("done");
    Ok(())
}

fn convert_tower_error_into_response(e: tower::BoxError) -> AppResponse {
    let response_error = if e.is::<tower::timeout::error::Elapsed>() {
        // Timeout
        match e.downcast() {
            Ok(e) => AppError::TimeoutError(e),
            Err(e) => AppError::StdError(e),
        }
    } else {
        // Generic error
        AppError::StdError(e)
    };

    response_error.into_response()
}
