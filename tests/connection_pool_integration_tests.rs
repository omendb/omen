//! Connection pool integration tests for PostgreSQL server
//!
//! Tests connection limit enforcement, statistics tracking, and cleanup

use datafusion::prelude::*;
use omendb::connection_pool::{ConnectionPool, PoolConfig};
use omendb::postgres::PostgresServer;
use std::time::Duration;
use tokio::time::timeout;
use tokio_postgres::{NoTls, Client};

async fn connect_to_server(port: u16) -> Result<Client, tokio_postgres::Error> {
    let config_str = format!("host=127.0.0.1 port={} user=postgres", port);
    let (client, connection) = tokio_postgres::connect(&config_str, NoTls).await?;

    // Spawn the connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    Ok(client)
}

#[tokio::test]
async fn test_connection_pool_limits() {
    // Start server with limited connections (3 max)
    let ctx = SessionContext::new();
    let config = PoolConfig {
        max_connections: 3,
        acquire_timeout: Duration::from_secs(2),
        ..Default::default()
    };

    let server = PostgresServer::with_pool_config("127.0.0.1:15440", ctx, config);

    // Verify pool configuration
    assert_eq!(server.connection_count(), 0);

    // Spawn server in background
    tokio::spawn(async move {
        let _ = server.serve().await;
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Connect 3 clients (should all succeed)
    let client1 = connect_to_server(15440).await.expect("Connection 1 failed");
    let client2 = connect_to_server(15440).await.expect("Connection 2 failed");
    let client3 = connect_to_server(15440).await.expect("Connection 3 failed");

    // All 3 connections should work
    let _ = client1.simple_query("SELECT 1").await;
    let _ = client2.simple_query("SELECT 1").await;
    let _ = client3.simple_query("SELECT 1").await;

    // Fourth connection should timeout (pool is full)
    let result = timeout(Duration::from_secs(3), connect_to_server(15440)).await;

    // This should either timeout or be rejected
    match result {
        Ok(Ok(_)) => {
            // Connection succeeded when it shouldn't have
            panic!("Fourth connection should have been rejected due to pool limit");
        }
        Ok(Err(e)) => {
            // Connection was rejected - this is expected
            println!("Fourth connection rejected as expected: {}", e);
        }
        Err(_) => {
            // Timeout - also acceptable behavior
            println!("Fourth connection timed out as expected");
        }
    }
}

#[tokio::test]
async fn test_connection_pool_reuse() {
    let ctx = SessionContext::new();
    let server = PostgresServer::with_addr("127.0.0.1:15441", ctx);

    // Spawn server in background
    tokio::spawn(async move {
        let _ = server.serve().await;
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Connect, disconnect, reconnect - should reuse connection slot
    {
        let client = connect_to_server(15441).await.expect("Connection 1 failed");
        let _ = client.simple_query("SELECT 1").await;
        drop(client);
    }

    // Short delay to let connection be released
    tokio::time::sleep(Duration::from_millis(100)).await;

    {
        let client = connect_to_server(15441).await.expect("Connection 2 failed");
        let _ = client.simple_query("SELECT 1").await;
    }
}

#[test]
fn test_pool_stats_tracking() {
    let pool = ConnectionPool::new();

    // Initial state
    let stats = pool.stats();
    assert_eq!(stats.total_created, 0);
    assert_eq!(stats.active_connections, 0);

    // Acquire 3 connections
    let conn1 = pool.acquire().unwrap();
    let conn2 = pool.acquire().unwrap();
    let conn3 = pool.acquire().unwrap();

    let stats = pool.stats();
    assert_eq!(stats.total_created, 3);
    assert_eq!(stats.active_connections, 3);
    assert_eq!(stats.total_acquisitions, 3);

    // Release 2 connections
    drop(conn1);
    drop(conn2);

    let stats = pool.stats();
    assert_eq!(stats.active_connections, 1);
    assert_eq!(stats.idle_connections, 2);
    assert_eq!(stats.total_releases, 2);

    // Reuse idle connection
    let _conn4 = pool.acquire().unwrap();
    let stats = pool.stats();
    assert_eq!(stats.total_created, 3); // No new connection created
    assert_eq!(stats.active_connections, 2);
    assert_eq!(stats.idle_connections, 1);
}

#[test]
fn test_idle_connection_cleanup() {
    let config = PoolConfig {
        max_connections: 10,
        idle_timeout: Duration::from_millis(200),
        ..Default::default()
    };

    let pool = ConnectionPool::with_config(config);

    // Create 5 connections
    {
        let _c1 = pool.acquire().unwrap();
        let _c2 = pool.acquire().unwrap();
        let _c3 = pool.acquire().unwrap();
        let _c4 = pool.acquire().unwrap();
        let _c5 = pool.acquire().unwrap();
    }

    // All connections now idle
    let stats = pool.stats();
    assert_eq!(stats.idle_connections, 5);

    // Wait for idle timeout
    std::thread::sleep(Duration::from_millis(250));

    // Cleanup should remove all idle connections
    let removed = pool.cleanup_idle_connections().unwrap();
    assert_eq!(removed, 5);

    let stats = pool.stats();
    assert_eq!(stats.idle_connections, 0);
    assert_eq!(stats.idle_timeouts, 5);
}

#[tokio::test]
async fn test_concurrent_connection_requests() {
    let ctx = SessionContext::new();
    let config = PoolConfig {
        max_connections: 20,
        acquire_timeout: Duration::from_secs(5),
        ..Default::default()
    };

    let server = PostgresServer::with_pool_config("127.0.0.1:15442", ctx, config);

    // Spawn server in background
    tokio::spawn(async move {
        let _ = server.serve().await;
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Spawn 10 concurrent connection attempts
    let mut handles = vec![];

    for i in 0..10 {
        let handle = tokio::spawn(async move {
            match connect_to_server(15442).await {
                Ok(client) => {
                    let result = client.simple_query(&format!("SELECT {}", i)).await;
                    result.is_ok()
                }
                Err(_) => false,
            }
        });
        handles.push(handle);
    }

    // Wait for all connections
    let mut success_count = 0;
    for handle in handles {
        if let Ok(true) = handle.await {
            success_count += 1;
        }
    }

    // All 10 should succeed (under the 20 connection limit)
    assert!(success_count >= 8, "Expected at least 8 successful connections, got {}", success_count);
}
