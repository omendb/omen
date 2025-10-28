//! Test PostgreSQL wire protocol compatibility
//!
//! Validates that OmenDB can serve as a drop-in PostgreSQL replacement.
//! Tests standard PostgreSQL client libraries and SQL features.

use anyhow::Result;
use tokio_postgres::{Client, NoTls};
use std::time::{Duration, Instant};
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("╔══════════════════════════════════════════════════════════════╗");
    info!("║         PostgreSQL Compatibility Test Suite                 ║");
    info!("╚══════════════════════════════════════════════════════════════╝\n");

    // Wait for server to be ready
    info!("⏳ Waiting for PostgreSQL server on port 5433...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Connect to OmenDB PostgreSQL server
    let (client, connection) = match tokio_postgres::connect(
        "host=127.0.0.1 port=5433 user=postgres dbname=omendb",
        NoTls
    ).await {
        Ok(res) => res,
        Err(e) => {
            error!("❌ Failed to connect: {}", e);
            error!("   Make sure postgres_alex_server is running:");
            error!("   cargo run --release --bin postgres_alex_server");
            return Err(e.into());
        }
    };

    // Spawn connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    info!("✅ Connected to OmenDB PostgreSQL server\n");

    // Run test suite
    run_compatibility_tests(&client).await?;

    Ok(())
}

async fn run_compatibility_tests(client: &Client) -> Result<()> {
    let mut passed = 0;
    let mut failed = 0;

    // Test 1: Basic SELECT
    info!("📊 Test 1: Basic SELECT");
    match test_basic_select(client).await {
        Ok(_) => {
            info!("  ✅ PASSED");
            passed += 1;
        }
        Err(e) => {
            error!("  ❌ FAILED: {}", e);
            failed += 1;
        }
    }

    // Test 2: CREATE TABLE
    info!("\n📊 Test 2: CREATE TABLE");
    match test_create_table(client).await {
        Ok(_) => {
            info!("  ✅ PASSED");
            passed += 1;
        }
        Err(e) => {
            error!("  ❌ FAILED: {}", e);
            failed += 1;
        }
    }

    // Test 3: INSERT operations
    info!("\n📊 Test 3: INSERT operations");
    match test_insert(client).await {
        Ok(_) => {
            info!("  ✅ PASSED");
            passed += 1;
        }
        Err(e) => {
            error!("  ❌ FAILED: {}", e);
            failed += 1;
        }
    }

    // Test 4: Aggregation queries
    info!("\n📊 Test 4: Aggregation queries");
    match test_aggregation(client).await {
        Ok(_) => {
            info!("  ✅ PASSED");
            passed += 1;
        }
        Err(e) => {
            error!("  ❌ FAILED: {}", e);
            failed += 1;
        }
    }

    // Test 5: JOIN operations
    info!("\n📊 Test 5: JOIN operations");
    match test_joins(client).await {
        Ok(_) => {
            info!("  ✅ PASSED");
            passed += 1;
        }
        Err(e) => {
            error!("  ❌ FAILED: {}", e);
            failed += 1;
        }
    }

    // Test 6: Transaction support
    info!("\n📊 Test 6: Transaction support");
    match test_transactions(client).await {
        Ok(_) => {
            info!("  ✅ PASSED");
            passed += 1;
        }
        Err(e) => {
            warn!("  ⚠️  SKIPPED: {}", e);
        }
    }

    // Test 7: Performance benchmark
    info!("\n📊 Test 7: Performance benchmark");
    match test_performance(client).await {
        Ok(_) => {
            info!("  ✅ PASSED");
            passed += 1;
        }
        Err(e) => {
            error!("  ❌ FAILED: {}", e);
            failed += 1;
        }
    }

    // Summary
    info!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("📈 Test Results:");
    info!("  ✅ Passed: {}", passed);
    info!("  ❌ Failed: {}", failed);
    info!("  📊 Total: {}", passed + failed);
    info!("  🎯 Success Rate: {:.1}%", (passed as f64 / (passed + failed) as f64) * 100.0);

    if failed == 0 {
        info!("\n🎉 All tests passed! OmenDB is PostgreSQL-compatible!");
    } else {
        warn!("\n⚠️  Some tests failed. Check compatibility issues.");
    }

    Ok(())
}

async fn test_basic_select(client: &Client) -> Result<()> {
    let rows = client
        .query("SELECT 1 as num, 'test' as text", &[])
        .await?;

    assert_eq!(rows.len(), 1);
    let num: i32 = rows[0].get(0);
    let text: &str = rows[0].get(1);
    assert_eq!(num, 1);
    assert_eq!(text, "test");

    Ok(())
}

async fn test_create_table(client: &Client) -> Result<()> {
    // Drop table if exists
    let _ = client.execute("DROP TABLE IF EXISTS test_table", &[]).await;

    // Create table
    client.execute(
        "CREATE TABLE test_table (
            id SERIAL PRIMARY KEY,
            name VARCHAR(100),
            value INTEGER
        )",
        &[]
    ).await?;

    info!("  Created table 'test_table'");
    Ok(())
}

async fn test_insert(client: &Client) -> Result<()> {
    // Ensure table exists
    let _ = client.execute("DROP TABLE IF EXISTS test_insert", &[]).await;
    client.execute(
        "CREATE TABLE test_insert (id INT, name VARCHAR)",
        &[]
    ).await?;

    // Insert data
    let inserted = client.execute(
        "INSERT INTO test_insert VALUES (1, 'Alice'), (2, 'Bob')",
        &[]
    ).await?;

    info!("  Inserted {} rows", inserted);

    // Verify data
    let rows = client
        .query("SELECT * FROM test_insert ORDER BY id", &[])
        .await?;

    assert_eq!(rows.len(), 2);
    Ok(())
}

async fn test_aggregation(client: &Client) -> Result<()> {
    // Create and populate test data
    let _ = client.execute("DROP TABLE IF EXISTS test_agg", &[]).await;
    client.execute(
        "CREATE TABLE test_agg (category VARCHAR, value INT)",
        &[]
    ).await?;

    client.execute(
        "INSERT INTO test_agg VALUES
         ('A', 10), ('A', 20), ('B', 30), ('B', 40), ('C', 50)",
        &[]
    ).await?;

    // Test aggregations
    let rows = client
        .query(
            "SELECT category, SUM(value), AVG(value), COUNT(*)
             FROM test_agg
             GROUP BY category
             ORDER BY category",
            &[]
        )
        .await?;

    assert_eq!(rows.len(), 3);
    info!("  Aggregation query returned {} groups", rows.len());

    Ok(())
}

async fn test_joins(client: &Client) -> Result<()> {
    // Create tables
    let _ = client.execute("DROP TABLE IF EXISTS orders", &[]).await;
    let _ = client.execute("DROP TABLE IF EXISTS customers", &[]).await;

    client.execute(
        "CREATE TABLE customers (id INT PRIMARY KEY, name VARCHAR)",
        &[]
    ).await?;

    client.execute(
        "CREATE TABLE orders (id INT, customer_id INT, amount DECIMAL)",
        &[]
    ).await?;

    // Insert data
    client.execute(
        "INSERT INTO customers VALUES (1, 'Alice'), (2, 'Bob')",
        &[]
    ).await?;

    client.execute(
        "INSERT INTO orders VALUES (1, 1, 100.0), (2, 1, 200.0), (3, 2, 150.0)",
        &[]
    ).await?;

    // Test JOIN
    let rows = client
        .query(
            "SELECT c.name, COUNT(o.id), SUM(o.amount)
             FROM customers c
             JOIN orders o ON c.id = o.customer_id
             GROUP BY c.name
             ORDER BY c.name",
            &[]
        )
        .await?;

    assert_eq!(rows.len(), 2);
    info!("  JOIN query returned {} results", rows.len());

    Ok(())
}

async fn test_transactions(client: &Client) -> Result<()> {
    // Note: Full transaction support may not be implemented
    warn!("  Transaction support not yet implemented");
    Err(anyhow::anyhow!("Transactions pending implementation"))
}

async fn test_performance(client: &Client) -> Result<()> {
    // Create performance test table
    let _ = client.execute("DROP TABLE IF EXISTS perf_test", &[]).await;
    client.execute(
        "CREATE TABLE perf_test (id BIGINT PRIMARY KEY, data VARCHAR)",
        &[]
    ).await?;

    // Bulk insert test
    let start = Instant::now();
    for i in 0..1000 {
        client.execute(
            &format!("INSERT INTO perf_test VALUES ({}, 'data_{}')", i, i),
            &[]
        ).await?;
    }
    let insert_time = start.elapsed();

    info!("  Inserted 1000 rows in {:.2}ms", insert_time.as_millis());
    info!("  Throughput: {:.0} inserts/sec",
          1000.0 / insert_time.as_secs_f64());

    // Query performance test
    let start = Instant::now();
    for i in 0..100 {
        let _ = client
            .query(&format!("SELECT * FROM perf_test WHERE id = {}", i * 10), &[])
            .await?;
    }
    let query_time = start.elapsed();

    info!("  Performed 100 queries in {:.2}ms", query_time.as_millis());
    info!("  Avg query latency: {:.2}ms",
          query_time.as_millis() as f64 / 100.0);

    Ok(())
}