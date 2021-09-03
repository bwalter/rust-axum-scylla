use anyhow::Result;

use hello::{db, handler::AxumHandler};

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
    tracing_subscriber::fmt::init();

    // Start DB session and Queries
    let queries = db::start_db_session_and_create_queries(&args.addr, args.port).await?;

    hello::start::<AxumHandler>(queries).await
}
