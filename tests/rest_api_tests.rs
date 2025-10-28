//! REST API Integration Tests
//!
//! Tests the REST API endpoints using real HTTP requests.

use datafusion::prelude::*;
use omen::rest::RestServer;
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;

/// Helper to start REST server on a test port
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

    let server = RestServer::with_addr(&format!("127.0.0.1:{}", port), ctx);

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

#[tokio::test]
async fn test_health_endpoint() {
    let port = 18080;
    start_test_server(port).await.unwrap();

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("http://127.0.0.1:{}/health", port))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.unwrap();
    assert_eq!(body["status"], "healthy");
    assert!(body["version"].is_string());
}

#[tokio::test]
async fn test_metrics_endpoint() {
    let port = 18081;
    start_test_server(port).await.unwrap();

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("http://127.0.0.1:{}/metrics", port))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.unwrap();
    assert!(body["uptime_seconds"].is_number());
    assert!(body["queries_executed"].is_number());
}

#[tokio::test]
async fn test_query_select() {
    let port = 18082;
    start_test_server(port).await.unwrap();

    let client = reqwest::Client::new();
    let response = client
        .post(&format!("http://127.0.0.1:{}/query", port))
        .json(&serde_json::json!({
            "sql": "SELECT * FROM test_users"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.unwrap();

    // Check columns
    let columns = body["columns"].as_array().unwrap();
    assert_eq!(columns.len(), 3);
    assert_eq!(columns[0], "id");
    assert_eq!(columns[1], "name");
    assert_eq!(columns[2], "age");

    // Check rows
    let rows = body["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 3);

    // Check first row
    let row0 = rows[0].as_array().unwrap();
    assert_eq!(row0[0], 1);
    assert_eq!(row0[1], "Alice");
    assert_eq!(row0[2], 30);
}

#[tokio::test]
async fn test_query_where_clause() {
    let port = 18083;
    start_test_server(port).await.unwrap();

    let client = reqwest::Client::new();
    let response = client
        .post(&format!("http://127.0.0.1:{}/query", port))
        .json(&serde_json::json!({
            "sql": "SELECT * FROM test_users WHERE id = 2"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.unwrap();
    let rows = body["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1);

    let row = rows[0].as_array().unwrap();
    assert_eq!(row[1], "Bob");
}

#[tokio::test]
async fn test_query_insert() {
    let port = 18084;
    start_test_server(port).await.unwrap();

    let client = reqwest::Client::new();

    // Insert new record
    let response = client
        .post(&format!("http://127.0.0.1:{}/query", port))
        .json(&serde_json::json!({
            "sql": "INSERT INTO test_users VALUES (4, 'David', 28)"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Verify insertion
    let response = client
        .post(&format!("http://127.0.0.1:{}/query", port))
        .json(&serde_json::json!({
            "sql": "SELECT * FROM test_users WHERE id = 4"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.unwrap();
    let rows = body["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1);

    let row = rows[0].as_array().unwrap();
    assert_eq!(row[1], "David");
    assert_eq!(row[2], 28);
}

#[tokio::test]
async fn test_query_error_handling() {
    let port = 18085;
    start_test_server(port).await.unwrap();

    let client = reqwest::Client::new();

    // Query non-existent table
    let response = client
        .post(&format!("http://127.0.0.1:{}/query", port))
        .json(&serde_json::json!({
            "sql": "SELECT * FROM nonexistent_table"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 400);

    let body: Value = response.json().await.unwrap();
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn test_query_aggregation() {
    let port = 18086;
    start_test_server(port).await.unwrap();

    let client = reqwest::Client::new();
    let response = client
        .post(&format!("http://127.0.0.1:{}/query", port))
        .json(&serde_json::json!({
            "sql": "SELECT COUNT(*) as count, AVG(age) as avg_age FROM test_users"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.unwrap();
    let rows = body["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1);

    let row = rows[0].as_array().unwrap();
    assert_eq!(row[0], 3); // count
    assert_eq!(row[1], 30.0); // avg age
}
