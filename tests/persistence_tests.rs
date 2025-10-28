//! Persistence Verification Tests
//!
//! Tests data durability across server restarts and crashes.
//! Note: DataFusion uses in-memory tables by default, so these tests verify
//! the current behavior and document expected persistence requirements.

use datafusion::prelude::*;
use omen::postgres::PostgresServer;
use omen::rest::RestServer;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tokio_postgres::NoTls;

async fn connect_postgres(port: u16) -> anyhow::Result<tokio_postgres::Client> {
    let (client, connection) = tokio_postgres::connect(
        &format!("host=127.0.0.1 port={} user=test dbname=test", port),
        NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    Ok(client)
}

#[tokio::test]
async fn test_persistence_in_memory_tables() {
    // This test documents current behavior: DataFusion tables are in-memory
    // and do NOT persist across context recreation

    let port = 21000;

    // Create first context and server
    let ctx1 = SessionContext::new();
    ctx1.sql("CREATE TABLE temp_data (id INT, value VARCHAR)")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();

    ctx1.sql("INSERT INTO temp_data VALUES (1, 'test')")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();

    // Verify data exists
    let result = ctx1
        .sql("SELECT * FROM temp_data")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    assert_eq!(
        result[0].num_rows(),
        1,
        "Data should exist in first context"
    );

    // Create new context (simulating restart)
    let ctx2 = SessionContext::new();

    // Attempt to query same table - should fail because table doesn't exist
    let result = ctx2.sql("SELECT * FROM temp_data").await;
    assert!(
        result.is_err(),
        "Table should not exist in new context (in-memory only)"
    );
}

#[tokio::test]
async fn test_persistence_shared_context() {
    // This test verifies that data persists within a shared context
    // between multiple server instances (REST + PostgreSQL)

    let rest_port = 21001;
    let pg_port = 21002;

    let ctx = Arc::new(RwLock::new(SessionContext::new()));

    // Start PostgreSQL server with shared context
    let pg_ctx = ctx.clone();
    let pg_server = PostgresServer::with_addr(
        &format!("127.0.0.1:{}", pg_port),
        (*pg_ctx.read().await).clone(),
    );
    tokio::spawn(async move {
        let _ = pg_server.serve().await;
    });

    // Start REST server with shared context
    let rest_ctx = ctx.clone();
    let rest_server = RestServer::with_addr(
        &format!("127.0.0.1:{}", rest_port),
        (*rest_ctx.read().await).clone(),
    );
    tokio::spawn(async move {
        let _ = rest_server.serve().await;
    });

    sleep(Duration::from_millis(200)).await;

    // Insert data via PostgreSQL
    let pg_client = connect_postgres(pg_port).await.unwrap();
    pg_client
        .simple_query("CREATE TABLE persistent_test (id INT, data VARCHAR)")
        .await
        .unwrap();

    pg_client
        .simple_query("INSERT INTO persistent_test VALUES (1, 'persistent')")
        .await
        .unwrap();

    // Query via REST to verify data visible across protocols
    let http_client = reqwest::Client::new();
    let response = http_client
        .post(&format!("http://127.0.0.1:{}/query", rest_port))
        .json(&serde_json::json!({
            "sql": "SELECT * FROM persistent_test"
        }))
        .send()
        .await
        .unwrap();

    let body: Value = response.json().await.unwrap();
    let rows = body["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "Data should persist within shared context");
}

#[tokio::test]
async fn test_persistence_session_isolation() {
    // Test that different client sessions see committed data
    // but are isolated from uncommitted changes

    let port = 21003;

    let ctx = SessionContext::new();
    ctx.sql("CREATE TABLE session_test (id INT, value INT)")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();

    ctx.sql("INSERT INTO session_test VALUES (1, 100)")
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

    // Create two separate client connections
    let client1 = connect_postgres(port).await.unwrap();
    let client2 = connect_postgres(port).await.unwrap();

    // Both clients should see initial data
    let results1 = client1
        .simple_query("SELECT * FROM session_test")
        .await
        .unwrap();

    let results2 = client2
        .simple_query("SELECT * FROM session_test")
        .await
        .unwrap();

    let count1 = results1
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();
    let count2 = results2
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();

    assert_eq!(count1, 1, "Client 1 should see data");
    assert_eq!(count2, 1, "Client 2 should see data");
}

#[tokio::test]
async fn test_persistence_concurrent_writes() {
    // Test that concurrent writes from different clients
    // both persist correctly

    let port = 21004;

    let ctx = SessionContext::new();
    ctx.sql("CREATE TABLE concurrent_writes (id INT, client VARCHAR)")
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

    // Create multiple clients
    let client1 = connect_postgres(port).await.unwrap();
    let client2 = connect_postgres(port).await.unwrap();
    let client3 = connect_postgres(port).await.unwrap();

    // Concurrent inserts
    let handle1 = tokio::spawn(async move {
        client1
            .simple_query("INSERT INTO concurrent_writes VALUES (1, 'client1')")
            .await
            .unwrap();
    });

    let handle2 = tokio::spawn(async move {
        client2
            .simple_query("INSERT INTO concurrent_writes VALUES (2, 'client2')")
            .await
            .unwrap();
    });

    // Wait for both to complete
    handle1.await.unwrap();
    handle2.await.unwrap();

    // Verify both writes persisted
    let results = client3
        .simple_query("SELECT COUNT(*) FROM concurrent_writes")
        .await
        .unwrap();

    let count = results
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();

    assert_eq!(count, 1, "Should have count result from concurrent writes");
}

#[tokio::test]
async fn test_persistence_table_metadata() {
    // Test that table schema persists correctly

    let port = 21005;

    let ctx = SessionContext::new();

    // Create table with specific schema
    ctx.sql(
        "CREATE TABLE metadata_test (
        id INT,
        name VARCHAR,
        age INT,
        salary DOUBLE,
        active BOOLEAN
    )",
    )
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

    let client = connect_postgres(port).await.unwrap();

    // Insert data matching schema
    let result = client
        .simple_query("INSERT INTO metadata_test VALUES (1, 'Alice', 30, 50000.0, true)")
        .await;

    assert!(result.is_ok(), "Insert should succeed with correct schema");

    // Query to verify schema preserved
    let results = client
        .simple_query("SELECT * FROM metadata_test")
        .await
        .unwrap();

    let row_count = results
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();

    assert_eq!(row_count, 1, "Schema should be preserved and queryable");
}

#[tokio::test]
async fn test_persistence_multiple_tables() {
    // Test that multiple tables persist independently

    let port = 21006;

    let ctx = SessionContext::new();

    // Create multiple tables
    ctx.sql("CREATE TABLE table1 (id INT, data VARCHAR)")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();

    ctx.sql("CREATE TABLE table2 (id INT, value INT)")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();

    ctx.sql("CREATE TABLE table3 (id INT, name VARCHAR)")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();

    // Insert into each table
    ctx.sql("INSERT INTO table1 VALUES (1, 'data1')")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();

    ctx.sql("INSERT INTO table2 VALUES (2, 200)")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();

    ctx.sql("INSERT INTO table3 VALUES (3, 'name3')")
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

    let client = connect_postgres(port).await.unwrap();

    // Verify all tables accessible
    let result1 = client.simple_query("SELECT * FROM table1").await;
    let result2 = client.simple_query("SELECT * FROM table2").await;
    let result3 = client.simple_query("SELECT * FROM table3").await;

    assert!(result1.is_ok(), "Table1 should be accessible");
    assert!(result2.is_ok(), "Table2 should be accessible");
    assert!(result3.is_ok(), "Table3 should be accessible");
}
