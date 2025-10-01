//! PostgreSQL wire protocol server for OmenDB
//!
//! Example usage:
//!   cargo run --bin postgres_server
//!   psql -h 127.0.0.1 -p 5432

use datafusion::prelude::*;
use omendb::postgres::PostgresServer;
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

    // Create and start PostgreSQL server
    let server = PostgresServer::new(ctx);
    info!("Starting PostgreSQL server on 127.0.0.1:5432");
    info!("Connect with: psql -h 127.0.0.1 -p 5432");

    server.serve().await?;

    Ok(())
}
