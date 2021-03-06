use std::{
    net::{SocketAddr, TcpListener},
    sync::Arc,
};

use anyhow::Result;

use hello::{app::App, db};

const KEYSPACE: &str = "hello";

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

    // TCP listener
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(&addr)?;

    // DB session and queries
    let session = db::scylla::create_session(&args.addr, args.port).await?;
    let queries = db::scylla::queries::ScyllaQueries::new(session, KEYSPACE).await?;

    // Create app
    let app = App::new(Arc::new(queries));

    // Start server
    tracing::debug!("listening on {:?}", listener);
    axum::Server::from_tcp(listener)?
        .serve(app.router.into_make_service())
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen for signal");
            tracing::error!("Ctrl-C received!");
        })
        .await?;

    Ok(())
}
