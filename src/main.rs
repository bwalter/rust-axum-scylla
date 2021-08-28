use anyhow::Result;
use axum::{
    handler::{get, post},
    http::StatusCode,
    response::IntoResponse,
    AddExtensionLayer, Router,
};
use response::{AppResponse, AppResponseResult};
use std::{
    convert::Infallible,
    net::SocketAddr,
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::time::sleep;
use tower::{timeout::error::Elapsed, BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;

use crate::response::AppResponseError;
use crate::state::State;

mod db;
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
        .route("/", get(hello))
        .route("/timeout", get(timeout))
        .route("/vehicles", get(vehicle::find))
        .route("/vehicles", post(vehicle::create))
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

// basic handler that responds with a static string
async fn hello() -> AppResponseResult {
    sleep(Duration::from_secs(3)).await;
    Ok((StatusCode::OK, "Hello, World!").into_response())
}

// basic handler that will time out
async fn timeout() -> AppResponseResult {
    sleep(Duration::from_secs(3)).await;
    Ok((StatusCode::OK, "Unreachable").into_response())
}

fn convert_tower_error_into_response(e: BoxError) -> AppResponse {
    if e.is::<Elapsed>() {
        // Timeout
        AppResponseError::TimeoutError(e.downcast().unwrap()).into_response()
    } else {
        // Generic error
        AppResponseError::StdError(e).into_response()
    }
}
