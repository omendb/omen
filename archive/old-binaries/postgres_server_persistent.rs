//! PostgreSQL wire protocol server for OmenDB with persistent storage
//!
//! Example usage:
//!   cargo run --bin postgres_server_persistent
//!   psql -h 127.0.0.1 -p 5433

use arrow::datatypes::{DataType, Field, Schema};
use datafusion::prelude::*;
use omen::catalog::Catalog;
use omen::datafusion::arrow_table_provider::ArrowTableProvider;
use omen::postgres::PostgresServer;
use omen::row::Row;
use omen::value::Value;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create OmenDB catalog with persistent storage
    let data_dir = PathBuf::from("./omendb_data");
    let mut catalog = Catalog::new(data_dir)?;

    // Create DataFusion session context
    let ctx = SessionContext::new();

    // Create a sample users table with persistent storage
    let users_schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("email", DataType::Utf8, true),
    ]));

    // Create table in catalog if it doesn't exist
    if !catalog.table_exists("users") {
        catalog.create_table("users".to_string(), users_schema.clone(), "id".to_string())?;

        // Insert sample data using OmenDB's native API
        {
            let table = catalog.get_table_mut("users")?;
            table.insert(Row::new(vec![
                Value::Int64(1),
                Value::Text("Alice".to_string()),
                Value::Text("alice@example.com".to_string()),
            ]))?;
            table.insert(Row::new(vec![
                Value::Int64(2),
                Value::Text("Bob".to_string()),
                Value::Text("bob@example.com".to_string()),
            ]))?;
            table.insert(Row::new(vec![
                Value::Int64(3),
                Value::Text("Charlie".to_string()),
                Value::Text("charlie@example.com".to_string()),
            ]))?;

            table.persist()?;
            info!("Created and populated 'users' table with persistent storage");
        }
    } else {
        info!("Loading existing 'users' table from persistent storage");
    }

    // Load table independently for DataFusion (ArrowTableProvider needs Arc<RwLock<Table>>)
    use omen::table::Table;
    let df_table = Table::load(
        "users".to_string(),
        PathBuf::from("./omendb_data/users"),
    )?;
    let table_ref = Arc::new(RwLock::new(df_table));
    let provider = Arc::new(ArrowTableProvider::new(table_ref, "users"));
    ctx.register_table("users", provider)?;

    info!("Sample data loaded from persistent storage");

    // Create PostgreSQL server with persistent storage
    let server = PostgresServer::with_addr("127.0.0.1:5433", ctx);
    info!("Starting PostgreSQL server on 127.0.0.1:5433");
    info!("Connect with: psql -h 127.0.0.1 -p 5433");
    info!("Data persisted to: ./omendb_data");

    server.serve().await?;

    Ok(())
}
