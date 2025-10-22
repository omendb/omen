//! PostgreSQL wire protocol server for OmenDB
//!
//! Example usage:
//!   cargo run --bin postgres_server
//!   psql -h 127.0.0.1 -p 5433
//!   curl http://127.0.0.1:9090/metrics  # Prometheus metrics

use datafusion::prelude::*;
use omendb::postgres::{PostgresServer, serve_metrics};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

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

    // Create and start PostgreSQL server on port 5433 (to avoid conflict with system PostgreSQL)
    let server = PostgresServer::with_addr("127.0.0.1:5433", ctx);
    info!("Starting PostgreSQL server on 127.0.0.1:5433");
    info!("Connect with: psql -h 127.0.0.1 -p 5433");

    server.serve().await?;

    Ok(())
}
