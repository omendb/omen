use anyhow::Result;
use clap::Parser;
use omendb_server::{config::Config, server::Server};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(name = "omendb-server")]
#[command(about = "High-performance vector database server")]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// HTTP port
    #[arg(long, default_value = "8080")]
    http_port: u16,

    /// gRPC port
    #[arg(long, default_value = "9090")]
    grpc_port: u16,

    /// Number of worker threads
    #[arg(short, long)]
    workers: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| args.log_level.clone().into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    info!("Starting OmenDB Server v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = Config::from_file(&args.config).unwrap_or_else(|e| {
        warn!("Failed to load config from {}: {}. Using defaults.", args.config, e);
        Config::default()
    });

    // Override with CLI args
    let mut config = config;
    config.server.http_port = args.http_port;
    config.server.grpc_port = args.grpc_port;
    if let Some(workers) = args.workers {
        config.server.worker_threads = workers;
    }

    // Initialize and start server
    let server = Server::new(config).await?;
    
    info!("Server initialized successfully");
    info!("HTTP server listening on port {}", args.http_port);
    info!("gRPC server listening on port {}", args.grpc_port);
    
    // Start server (this will block)
    server.run().await?;

    Ok(())
}