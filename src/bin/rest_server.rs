//! REST API server binary

use datafusion::prelude::*;
use omendb::rest::RestServer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("omendb=info".parse()?))
        .init();

    // Create DataFusion context
    let ctx = SessionContext::new();

    // Create demo table
    ctx.sql("CREATE TABLE users (id INT, name VARCHAR, age INT)")
        .await?
        .collect()
        .await?;

    ctx.sql("INSERT INTO users VALUES (1, 'Alice', 30), (2, 'Bob', 25), (3, 'Charlie', 35)")
        .await?
        .collect()
        .await?;

    println!("Starting OmenDB REST API server on 0.0.0.0:8080");
    println!("Endpoints:");
    println!("  GET  /health  - Health check");
    println!("  GET  /metrics - Metrics");
    println!("  POST /query   - Execute SQL query");

    // Start server
    let server = RestServer::new(ctx);
    server.serve().await?;

    Ok(())
}
