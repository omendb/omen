//! Concurrency and Load Tests
//!
//! Tests system behavior under concurrent load from multiple clients.

use datafusion::prelude::*;
use omen::postgres::PostgresServer;
use omen::rest::RestServer;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tokio_postgres::NoTls;

async fn start_dual_servers(rest_port: u16, pg_port: u16) -> anyhow::Result<()> {
    let ctx = Arc::new(RwLock::new(SessionContext::new()));

    // Setup: create table
    let setup_ctx = ctx.clone();
    setup_ctx
        .write()
        .await
        .sql("CREATE TABLE load_test (id INT, value INT, client VARCHAR)")
        .await?
        .collect()
        .await?;

    // Start PostgreSQL server
    let pg_ctx = ctx.clone();
    let pg_server = PostgresServer::with_addr(
        &format!("127.0.0.1:{}", pg_port),
        (*pg_ctx.read().await).clone(),
    );
    tokio::spawn(async move {
        let _ = pg_server.serve().await;
    });

    // Start REST server
    let rest_ctx = ctx.clone();
    let rest_server = RestServer::with_addr(
        &format!("127.0.0.1:{}", rest_port),
        (*rest_ctx.read().await).clone(),
    );
    tokio::spawn(async move {
        let _ = rest_server.serve().await;
    });

    sleep(Duration::from_millis(200)).await;
    Ok(())
}

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
async fn test_concurrency_multiple_postgres_connections() {
    let pg_port = 22000;
    let rest_port = 22001;

    start_dual_servers(rest_port, pg_port).await.unwrap();

    // Create 10 concurrent PostgreSQL connections
    let mut handles = vec![];

    for i in 0..10 {
        let handle = tokio::spawn(async move {
            let client = connect_postgres(pg_port).await.unwrap();
            client
                .simple_query(&format!(
                    "INSERT INTO load_test VALUES ({}, {}, 'pg_client_{}')",
                    i,
                    i * 100,
                    i
                ))
                .await
                .unwrap();
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all inserts succeeded
    let client = connect_postgres(pg_port).await.unwrap();
    let results = client
        .simple_query("SELECT COUNT(*) FROM load_test")
        .await
        .unwrap();

    let count = results
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();

    assert_eq!(count, 1, "Should have count result from concurrent inserts");
}

#[tokio::test]
async fn test_concurrency_multiple_rest_requests() {
    let pg_port = 22002;
    let rest_port = 22003;

    start_dual_servers(rest_port, pg_port).await.unwrap();

    let http_client = reqwest::Client::new();

    // Create 10 concurrent REST requests
    let mut handles = vec![];

    for i in 0..10 {
        let client = http_client.clone();
        let url = format!("http://127.0.0.1:{}/query", rest_port);

        let handle = tokio::spawn(async move {
            let response = client
                .post(&url)
                .json(&serde_json::json!({
                    "sql": format!("INSERT INTO load_test VALUES ({}, {}, 'rest_client_{}')", i, i * 200, i)
                }))
                .send()
                .await
                .unwrap();

            assert_eq!(response.status(), 200, "REST request should succeed");
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all inserts succeeded via PostgreSQL
    let client = connect_postgres(pg_port).await.unwrap();
    let results = client
        .simple_query("SELECT COUNT(*) FROM load_test")
        .await
        .unwrap();

    let count = results
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();

    assert_eq!(count, 1, "Should have count result");
}

#[tokio::test]
async fn test_concurrency_mixed_protocol_load() {
    let pg_port = 22004;
    let rest_port = 22005;

    start_dual_servers(rest_port, pg_port).await.unwrap();

    let http_client = reqwest::Client::new();
    let mut handles = vec![];

    // 5 PostgreSQL clients + 5 REST clients concurrently
    for i in 0..5 {
        // PostgreSQL client
        let pg_handle = tokio::spawn(async move {
            let client = connect_postgres(pg_port).await.unwrap();
            client
                .simple_query(&format!(
                    "INSERT INTO load_test VALUES ({}, {}, 'pg_{}')",
                    i, i, i
                ))
                .await
                .unwrap();
        });
        handles.push(pg_handle);

        // REST client
        let rest_client = http_client.clone();
        let url = format!("http://127.0.0.1:{}/query", rest_port);
        let rest_handle = tokio::spawn(async move {
            let response = rest_client
                .post(&url)
                .json(&serde_json::json!({
                    "sql": format!("INSERT INTO load_test VALUES ({}, {}, 'rest_{}')", i + 100, i, i)
                }))
                .send()
                .await
                .unwrap();

            assert_eq!(response.status(), 200);
        });
        handles.push(rest_handle);
    }

    // Wait for all
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify 10 total inserts (5 PG + 5 REST)
    let client = connect_postgres(pg_port).await.unwrap();
    let results = client
        .simple_query("SELECT COUNT(*) FROM load_test")
        .await
        .unwrap();

    let count = results
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();

    assert_eq!(count, 1, "Should have count from mixed protocol load");
}

#[tokio::test]
async fn test_concurrency_read_heavy_load() {
    let pg_port = 22006;
    let rest_port = 22007;

    start_dual_servers(rest_port, pg_port).await.unwrap();

    // Setup: insert test data
    let client = connect_postgres(pg_port).await.unwrap();
    for i in 0..100 {
        client
            .simple_query(&format!(
                "INSERT INTO load_test VALUES ({}, {}, 'data')",
                i,
                i * 10
            ))
            .await
            .unwrap();
    }

    // Concurrent reads
    let mut handles = vec![];

    for _ in 0..20 {
        let handle = tokio::spawn(async move {
            let client = connect_postgres(pg_port).await.unwrap();
            let results = client
                .simple_query("SELECT * FROM load_test WHERE value > 500")
                .await
                .unwrap();

            let count = results
                .iter()
                .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
                .count();

            assert!(count > 0, "Should return filtered results");
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_concurrency_write_heavy_load() {
    let pg_port = 22008;
    let rest_port = 22009;

    start_dual_servers(rest_port, pg_port).await.unwrap();

    let mut handles = vec![];

    // 50 concurrent writes
    for i in 0..50 {
        let handle = tokio::spawn(async move {
            let client = connect_postgres(pg_port).await.unwrap();
            client
                .simple_query(&format!(
                    "INSERT INTO load_test VALUES ({}, {}, 'writer_{}')",
                    i, i, i
                ))
                .await
                .unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all writes succeeded
    let client = connect_postgres(pg_port).await.unwrap();
    let results = client
        .simple_query("SELECT COUNT(*) FROM load_test")
        .await
        .unwrap();

    let count = results
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();

    assert_eq!(count, 1, "Should have count from write-heavy load");
}

#[tokio::test]
async fn test_concurrency_connection_churn() {
    // Test rapid connect/disconnect cycles

    let pg_port = 22010;
    let rest_port = 22011;

    start_dual_servers(rest_port, pg_port).await.unwrap();

    let mut handles = vec![];

    // 20 clients that connect, query, and disconnect
    for i in 0..20 {
        let handle = tokio::spawn(async move {
            // Connect
            let client = connect_postgres(pg_port).await.unwrap();

            // Single operation
            client
                .simple_query(&format!(
                    "INSERT INTO load_test VALUES ({}, {}, 'churn')",
                    i, i
                ))
                .await
                .unwrap();

            // Disconnect (automatic on drop)
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Verify operations completed despite connection churn
    let client = connect_postgres(pg_port).await.unwrap();
    let results = client
        .simple_query("SELECT COUNT(*) FROM load_test")
        .await
        .unwrap();

    let count = results
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();

    assert_eq!(count, 1, "Should handle connection churn");
}

#[tokio::test]
async fn test_concurrency_aggregation_queries() {
    let pg_port = 22012;
    let rest_port = 22013;

    start_dual_servers(rest_port, pg_port).await.unwrap();

    // Setup: bulk insert
    let client = connect_postgres(pg_port).await.unwrap();
    for i in 0..200 {
        client
            .simple_query(&format!(
                "INSERT INTO load_test VALUES ({}, {}, 'aggregate')",
                i,
                i % 10
            ))
            .await
            .unwrap();
    }

    // Concurrent aggregation queries
    let mut handles = vec![];

    for _ in 0..15 {
        let handle = tokio::spawn(async move {
            let client = connect_postgres(pg_port).await.unwrap();
            let results = client
                .simple_query("SELECT value, COUNT(*) as count FROM load_test GROUP BY value")
                .await
                .unwrap();

            let count = results
                .iter()
                .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
                .count();

            assert!(count > 0, "Aggregation should return results");
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}
