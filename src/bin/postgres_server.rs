//! PostgreSQL wire protocol server for OmenDB
//!
//! Example usage:
//!   cargo run --bin postgres_server
//!   cargo run --bin postgres_server -- --cert certs/cert.pem --key certs/key.pem
//!   psql -h 127.0.0.1 -p 5433
//!   curl http://127.0.0.1:9090/metrics  # Prometheus metrics

use clap::Parser;
use datafusion::prelude::*;
use omen::postgres::{PostgresServer, serve_metrics};
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "postgres_server")]
#[command(about = "PostgreSQL wire protocol server for OmenDB", long_about = None)]
struct Args {
    /// Server address to bind to
    #[arg(short, long, default_value = "127.0.0.1:5433")]
    addr: String,

    /// Enable TLS/SSL with certificate file path
    #[arg(long)]
    cert: Option<String>,

    /// Private key file path (required if --cert is specified)
    #[arg(long)]
    key: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // Create DataFusion session context
    let ctx = SessionContext::new();

    // Register a sample in-memory table for testing
    ctx.sql("CREATE TABLE users (id INT, name VARCHAR)")
        .await?
        .collect()
        .await?;

    ctx.sql("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie')")
        .await?
        .collect()
        .await?;

    info!("Sample data loaded");

    // Start metrics HTTP server in background
    tokio::spawn(async {
        info!("Starting metrics HTTP server on 127.0.0.1:9090");
        info!("Metrics available at: http://127.0.0.1:9090/metrics");
        info!("Health check at: http://127.0.0.1:9090/health");
        if let Err(e) = serve_metrics("127.0.0.1:9090").await {
            eprintln!("Metrics server error: {}", e);
        }
    });

    // Create PostgreSQL server on port 5433 (to avoid conflict with system PostgreSQL)
    let mut server = PostgresServer::with_addr(&args.addr, ctx);

    // Enable TLS if certificate and key are provided
    if let (Some(cert), Some(key)) = (&args.cert, &args.key) {
        info!("Enabling TLS with cert: {}, key: {}", cert, key);
        server = server.with_tls(cert, key)?;
    } else if args.cert.is_some() || args.key.is_some() {
        anyhow::bail!("Both --cert and --key must be specified for TLS");
    }

    info!("Starting PostgreSQL server on {}", args.addr);
    if server.is_tls_enabled() {
        info!("TLS/SSL enabled - secure connections");
        info!("Connect with: psql 'host=127.0.0.1 port=5433 sslmode=require'");
    } else {
        info!("Connect with: psql -h 127.0.0.1 -p 5433");
    }

    server.serve().await?;

    Ok(())
}
