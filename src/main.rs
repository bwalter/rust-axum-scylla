use std::net::{SocketAddr, TcpListener};

use anyhow::Result;

use hello::{db, router};

/// A sample Rust backend app with Rest API and Scylla DB
#[derive(argh::FromArgs)]
struct CmdLineArgs {
    /// hostname or address of the ScyllaDB node (default: localhost)
    #[argh(option, default = "\"localhost\".to_string()")]
    addr: String,

    /// port of the ScyllaDB node (default: 9042)
    #[argh(option, default = "9042")]
    port: u16,
}
// Hint: start with RUST_LOG=hello=debug,tower_http=debug ./hello -- --help
#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line args
    let args: CmdLineArgs = argh::from_env();

    // Initialize tracing
    //console_subscriber::init();
    tracing_subscriber::fmt::init();

    // Start DB session and Queries
    let queries = db::start_db_session_and_create_queries(&args.addr, args.port).await?;

    // TCP listener
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(&addr)?;

    // Create router
    let router = router::create_router(queries);

    // Start server
    tracing::debug!("listening on {:?}", listener);
    axum::Server::from_tcp(listener)?
        .serve(router.into_make_service())
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen for signal");
            tracing::error!("Ctrl-C received!");
        })
        .await?;

    Ok(())
}
