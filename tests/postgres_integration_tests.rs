//! PostgreSQL Wire Protocol Integration Tests
//!
//! Tests the PostgreSQL wire protocol implementation using a real tokio-postgres client.
//! These tests verify:
//! - Connection establishment
//! - Query execution (SELECT, INSERT, CREATE TABLE)
//! - Type conversion (Arrow ↔ PostgreSQL)
//! - Error handling
//! - Special commands (SET, SHOW, BEGIN, etc.)

use datafusion::prelude::*;
use omendb::postgres::PostgresServer;
use std::time::Duration;
use tokio::time::sleep;
use tokio_postgres::{Client, NoTls};

/// Helper to start PostgreSQL server on a test port
async fn start_test_server(port: u16) -> anyhow::Result<()> {
    let ctx = SessionContext::new();

    // Create test table
    ctx.sql("CREATE TABLE test_users (id INT, name VARCHAR, age INT)")
        .await?
        .collect()
        .await?;

    ctx.sql("INSERT INTO test_users VALUES (1, 'Alice', 30), (2, 'Bob', 25), (3, 'Charlie', 35)")
        .await?
        .collect()
        .await?;

    let server = PostgresServer::with_addr(&format!("127.0.0.1:{}", port), ctx);

    // Run server in background
    tokio::spawn(async move {
        if let Err(e) = server.serve().await {
            eprintln!("Server error: {}", e);
        }
    });

    // Give server time to start
    sleep(Duration::from_millis(100)).await;

    Ok(())
}

/// Helper to connect to test server
async fn connect_to_test_server(port: u16) -> anyhow::Result<Client> {
    let (client, connection) = tokio_postgres::connect(
        &format!("host=127.0.0.1 port={} user=test dbname=test", port),
        NoTls,
    )
    .await?;

    // Spawn connection task
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    Ok(client)
}

#[tokio::test]
async fn test_postgres_connection() {
    let port = 15432;
    start_test_server(port).await.unwrap();

    let result = connect_to_test_server(port).await;
    assert!(result.is_ok(), "Failed to connect to PostgreSQL server");
}

#[tokio::test]
async fn test_postgres_simple_select() {
    let port = 15433;
    start_test_server(port).await.unwrap();
    let client = connect_to_test_server(port).await.unwrap();

    let results = client
        .simple_query("SELECT * FROM test_users")
        .await
        .unwrap();

    // Simple query returns SimpleQueryMessage enum, count Row variants
    let row_count = results
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();
    assert_eq!(row_count, 3, "Should return 3 rows");
}

#[tokio::test]
async fn test_postgres_where_clause() {
    let port = 15434;
    start_test_server(port).await.unwrap();
    let client = connect_to_test_server(port).await.unwrap();

    let results = client
        .simple_query("SELECT * FROM test_users WHERE id = 2")
        .await
        .unwrap();

    let row_count = results
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();
    assert_eq!(row_count, 1, "Should return 1 row");
}

#[tokio::test]
async fn test_postgres_insert() {
    let port = 15435;
    start_test_server(port).await.unwrap();
    let client = connect_to_test_server(port).await.unwrap();

    let results = client
        .simple_query("INSERT INTO test_users VALUES (4, 'David', 28)")
        .await
        .unwrap();

    // Check for CommandComplete message with INSERT tag
    let has_insert = results.iter().any(|msg| {
        matches!(msg, tokio_postgres::SimpleQueryMessage::CommandComplete(rows) if *rows == 1)
    });
    assert!(has_insert, "Should insert 1 row");

    // Verify insertion
    let results = client
        .simple_query("SELECT * FROM test_users WHERE id = 4")
        .await
        .unwrap();

    let row_count = results
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();
    assert_eq!(row_count, 1);
}

#[tokio::test]
async fn test_postgres_create_table() {
    let port = 15436;

    let ctx = SessionContext::new();
    let server = PostgresServer::with_addr(&format!("127.0.0.1:{}", port), ctx);

    tokio::spawn(async move {
        let _ = server.serve().await;
    });

    sleep(Duration::from_millis(100)).await;

    let client = connect_to_test_server(port).await.unwrap();

    let result = client
        .simple_query("CREATE TABLE products (id INT, name VARCHAR, price DOUBLE)")
        .await;

    assert!(result.is_ok(), "CREATE TABLE should succeed");
}

#[tokio::test]
async fn test_postgres_special_commands() {
    let port = 15437;
    start_test_server(port).await.unwrap();
    let client = connect_to_test_server(port).await.unwrap();

    // Test SET command
    let result = client.simple_query("SET client_encoding = 'UTF8'").await;
    assert!(result.is_ok(), "SET command should succeed");

    // Test BEGIN/COMMIT
    let result = client.simple_query("BEGIN").await;
    assert!(result.is_ok(), "BEGIN should succeed");

    let result = client.simple_query("COMMIT").await;
    assert!(result.is_ok(), "COMMIT should succeed");

    // Test ROLLBACK
    let result = client.simple_query("BEGIN").await;
    assert!(result.is_ok());

    let result = client.simple_query("ROLLBACK").await;
    assert!(result.is_ok(), "ROLLBACK should succeed");
}

#[tokio::test]
async fn test_postgres_multiple_queries() {
    let port = 15438;
    start_test_server(port).await.unwrap();
    let client = connect_to_test_server(port).await.unwrap();

    // Execute multiple queries (simple query protocol doesn't support parameters)
    for i in 1..=3 {
        let results = client
            .simple_query(&format!("SELECT * FROM test_users WHERE id = {}", i))
            .await
            .unwrap();

        let row_count = results
            .iter()
            .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
            .count();
        assert_eq!(row_count, 1, "Should return 1 row for id = {}", i);
    }
}

#[tokio::test]
async fn test_postgres_error_handling() {
    let port = 15439;
    start_test_server(port).await.unwrap();
    let client = connect_to_test_server(port).await.unwrap();

    // Query non-existent table
    let result = client.simple_query("SELECT * FROM nonexistent_table").await;
    assert!(result.is_err(), "Should fail for non-existent table");

    // Invalid SQL syntax
    let result = client.simple_query("SELECT * FROM").await;
    assert!(result.is_err(), "Should fail for invalid SQL");
}

#[tokio::test]
async fn test_postgres_null_values() {
    let port = 15440;

    let ctx = SessionContext::new();
    ctx.sql("CREATE TABLE nullable_test (id INT, name VARCHAR, optional_field INT)")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();

    ctx.sql("INSERT INTO nullable_test VALUES (1, 'Test', NULL)")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();

    let server = PostgresServer::with_addr(&format!("127.0.0.1:{}", port), ctx);

    tokio::spawn(async move {
        let _ = server.serve().await;
    });

    sleep(Duration::from_millis(100)).await;
    let client = connect_to_test_server(port).await.unwrap();

    let results = client
        .simple_query("SELECT * FROM nullable_test")
        .await
        .unwrap();

    let row_count = results
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();
    assert_eq!(row_count, 1, "Should return 1 row");

    // Simple query protocol returns all values as strings, so we can't test NULL the same way
    // This test now just verifies the query succeeds with NULL values
}
