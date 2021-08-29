use anyhow::Result;
use axum::{
    handler::{get, post},
    response::IntoResponse,
    AddExtensionLayer, Router,
};
use std::{
    convert::Infallible,
    net::SocketAddr,
    sync::{Arc, RwLock},
    time::Duration,
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::response::{AppResponse, AppResponseError};
use crate::state::State;

mod db;
mod handlers;
mod response;
mod state;
mod vehicle;

/// A sample Rust backend app with Rest API and Scylla DB
#[derive(argh::FromArgs)]
struct CmdLineArgs {
    /// hostname or address of the ScyllaDB node (e.g. 172.17.0.2)
    #[argh(option)]
    addr: String,

    /// port of the ScyllaDB node (default: 9042)
    #[argh(option, default = "9042")]
    port: i32,
}

// Hint: start with RUST_LOG=hello=debug,tower_http=debug ./hello -- --help
#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line args
    let args: CmdLineArgs = argh::from_env();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Database
    let db = Arc::new(db::create_session(&args.addr, args.port).await?);

    // Shared state
    let shared_state = Arc::new(RwLock::new(State { count: 0 }));

    // Middlewares: Tower layer stack
    let middleware_stack = ServiceBuilder::new()
        .timeout(Duration::from_secs(1))
        .layer(TraceLayer::new_for_http())
        .into_inner();

    // Route
    let app = Router::new()
        .route("/", get(handlers::hello))
        .route("/timeout", get(handlers::timeout))
        .route("/vehicles", get(handlers::find_vehicle))
        .route("/vehicles", post(handlers::create_vehicle))
        .layer(middleware_stack)
        .layer(AddExtensionLayer::new(db))
        .layer(AddExtensionLayer::new(shared_state))
        .handle_error(|e| Ok::<_, Infallible>(convert_tower_error_into_response(e)));

    // Run our app with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    tracing::debug!("done");
    Ok(())
}

fn convert_tower_error_into_response(e: tower::BoxError) -> AppResponse {
    let response_error = if e.is::<tower::timeout::error::Elapsed>() {
        // Timeout
        match e.downcast() {
            Ok(e) => AppResponseError::TimeoutError(e),
            Err(e) => AppResponseError::StdError(e),
        }
    } else {
        // Generic error
        AppResponseError::StdError(e)
    };

    response_error.into_response()
}
