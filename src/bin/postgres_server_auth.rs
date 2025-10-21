//! PostgreSQL wire protocol server with SCRAM-SHA-256 authentication
//!
//! Example usage:
//!   cargo run --bin postgres_server_auth
//!   psql -h 127.0.0.1 -p 5433 -U alice -d omendb
//!   Password: secret123

use arrow::array::{Int64Array, RecordBatch, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use datafusion::datasource::MemTable;
use datafusion::prelude::*;
use omendb::postgres::{OmenDbAuthSource, PostgresServer};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::RwLock;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create temporary directory for user storage (use persistent path in production)
    let temp_dir = TempDir::new()?;
    let data_dir = temp_dir.path();

    // Create authentication source and add users
    let auth_source = Arc::new(OmenDbAuthSource::new(data_dir)?);

    info!("Setting up authentication with SCRAM-SHA-256");
    auth_source.add_user("alice", "secret123").await?;
    auth_source.add_user("bob", "password456").await?;
    auth_source.add_user("postgres", "postgres").await?;

    info!(
        "Added {} users: alice (secret123), bob (password456), postgres (postgres)",
        auth_source.user_count().await
    );

    // Create DataFusion session context
    let ctx = SessionContext::new();

    // Create a sample users table
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("email", DataType::Utf8, true),
    ]));

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(Int64Array::from(vec![1, 2, 3])),
            Arc::new(StringArray::from(vec!["Alice", "Bob", "Charlie"])),
            Arc::new(StringArray::from(vec![
                "alice@example.com",
                "bob@example.com",
                "charlie@example.com",
            ])),
        ],
    )?;

    let provider = MemTable::try_new(schema, vec![vec![batch]])?;
    ctx.register_table("users", Arc::new(provider))?;

    info!("Sample data loaded");

    // Create PostgreSQL server with authentication
    let ctx = Arc::new(RwLock::new(ctx));
    let server = PostgresServer::with_auth("127.0.0.1:5433", ctx, auth_source);

    info!("Starting PostgreSQL server on 127.0.0.1:5433 with SCRAM-SHA-256 authentication");
    info!("Connect with: psql -h 127.0.0.1 -p 5433 -U alice -d omendb");
    info!("Password: secret123");
    info!("");
    info!("Available users:");
    info!("  alice / secret123");
    info!("  bob / password456");
    info!("  postgres / postgres");

    server.serve().await?;

    Ok(())
}
