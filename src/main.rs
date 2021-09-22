use std::net::{SocketAddr, TcpListener};

use anyhow::Result;

use hello::db;

/// A sample Rust backend app with Rest API and Scylla DB
#[derive(argh::FromArgs)]
struct CmdLineArgs {
    /// hostname or address of the ScyllaDB node (e.g. 172.17.0.2)
    #[argh(option)]
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

    // Start server
    hello::start(listener, queries).await
}
