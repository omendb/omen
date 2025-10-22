//! Transaction rollback integration tests
//!
//! Tests that verify ACID compliance with true transaction support
//!
//! ## Running Tests
//!
//! These tests MUST be run sequentially due to shared TransactionContext:
//! ```bash
//! cargo test --test transaction_rollback_tests -- --test-threads=1
//! ```
//!
//! When run in parallel, tests interfere with each other's transaction state.
//! All 5 tests pass when run sequentially. Future work: per-connection transaction state.

use std::time::Duration;
use tokio::time::sleep;

/// Test basic transaction rollback
///
/// Verifies that INSERT within a transaction is NOT applied if ROLLBACK is called
#[tokio::test]
async fn test_basic_rollback() -> Result<(), Box<dyn std::error::Error>> {
    // Start PostgreSQL server in background
    let server_handle = tokio::spawn(async {
        // Server will run on port 5433
        // In a real test, we'd use a test-specific port
    });

    // Give server time to start
    sleep(Duration::from_millis(100)).await;

    // Connect to database
    let (client, connection) = tokio_postgres::connect(
        "host=localhost port=5433 user=postgres",
        tokio_postgres::NoTls,
    )
    .await?;

    // Connection runs in background
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Drop table if exists, then create fresh
    let _ = client.execute("DROP TABLE users", &[]).await; // Ignore error if doesn't exist

    client
        .execute(
            "CREATE TABLE users (id INT PRIMARY KEY, name TEXT)",
            &[],
        )
        .await?;

    // Insert initial data (auto-commit)
    client
        .execute("INSERT INTO users VALUES (1, 'Alice')", &[])
        .await?;

    // Verify initial data
    let rows = client.query("SELECT * FROM users WHERE id = 1", &[]).await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<_, i32>(0), 1);
    assert_eq!(rows[0].get::<_, String>(1), "Alice");

    // Start transaction
    client.execute("BEGIN", &[]).await?;

    // Insert data within transaction
    client
        .execute("INSERT INTO users VALUES (2, 'Bob')", &[])
        .await?;

    // Data should not be visible yet (buffered)
    // This is a limitation - we can't test visibility within same connection
    // But we can verify rollback works

    // ROLLBACK the transaction
    client.execute("ROLLBACK", &[]).await?;

    // Verify Bob was NOT inserted (rollback worked)
    let rows = client.query("SELECT * FROM users WHERE id = 2", &[]).await?;
    assert_eq!(rows.len(), 0, "Bob should NOT exist after ROLLBACK");

    // Verify Alice still exists
    let rows = client.query("SELECT * FROM users WHERE id = 1", &[]).await?;
    assert_eq!(rows.len(), 1);

    // Cleanup
    server_handle.abort();

    Ok(())
}

/// Test transaction commit
///
/// Verifies that INSERT within a transaction IS applied after COMMIT
#[tokio::test]
async fn test_basic_commit() -> Result<(), Box<dyn std::error::Error>> {
    // Give server time to start
    sleep(Duration::from_millis(100)).await;

    // Connect to database
    let (client, connection) = tokio_postgres::connect(
        "host=localhost port=5433 user=postgres",
        tokio_postgres::NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Drop table if exists, then create fresh
    let _ = client.execute("DROP TABLE test_commit", &[]).await;

    client
        .execute(
            "CREATE TABLE test_commit (id INT PRIMARY KEY, value TEXT)",
            &[],
        )
        .await?;

    // Start transaction
    client.execute("BEGIN", &[]).await?;

    // Insert data
    client
        .execute("INSERT INTO test_commit VALUES (1, 'test')", &[])
        .await?;

    // COMMIT the transaction
    client.execute("COMMIT", &[]).await?;

    // Verify data was inserted
    let rows = client
        .query("SELECT * FROM test_commit WHERE id = 1", &[])
        .await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<_, i32>(0), 1);
    assert_eq!(rows[0].get::<_, String>(1), "test");

    Ok(())
}

/// Test multiple operations in transaction
#[tokio::test]
async fn test_multiple_operations_rollback() -> Result<(), Box<dyn std::error::Error>> {
    sleep(Duration::from_millis(100)).await;

    let (client, connection) = tokio_postgres::connect(
        "host=localhost port=5433 user=postgres",
        tokio_postgres::NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Drop table if exists, then create fresh
    let _ = client.execute("DROP TABLE multi_op", &[]).await;

    client
        .execute(
            "CREATE TABLE multi_op (id INT PRIMARY KEY, value TEXT)",
            &[],
        )
        .await?;

    // Start transaction
    client.execute("BEGIN", &[]).await?;

    // Insert multiple rows
    client
        .execute("INSERT INTO multi_op VALUES (1, 'one')", &[])
        .await?;
    client
        .execute("INSERT INTO multi_op VALUES (2, 'two')", &[])
        .await?;
    client
        .execute("INSERT INTO multi_op VALUES (3, 'three')", &[])
        .await?;

    // Rollback all
    client.execute("ROLLBACK", &[]).await?;

    // Verify none were inserted
    let rows = client.query("SELECT * FROM multi_op", &[]).await?;
    assert_eq!(rows.len(), 0, "No rows should exist after ROLLBACK");

    Ok(())
}

/// Test transaction with error handling
#[tokio::test]
async fn test_transaction_error_rollback() -> Result<(), Box<dyn std::error::Error>> {
    sleep(Duration::from_millis(100)).await;

    let (client, connection) = tokio_postgres::connect(
        "host=localhost port=5433 user=postgres",
        tokio_postgres::NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Drop table if exists, then create fresh
    let _ = client.execute("DROP TABLE error_test", &[]).await;

    client
        .execute(
            "CREATE TABLE error_test (id INT PRIMARY KEY, value TEXT)",
            &[],
        )
        .await?;

    // Insert initial data
    client
        .execute("INSERT INTO error_test VALUES (1, 'exists')", &[])
        .await?;

    // Start transaction
    client.execute("BEGIN", &[]).await?;

    // Insert valid data
    client
        .execute("INSERT INTO error_test VALUES (2, 'new')", &[])
        .await?;

    // Try to insert duplicate key (should fail)
    let result = client
        .execute("INSERT INTO error_test VALUES (1, 'duplicate')", &[])
        .await;

    assert!(result.is_err(), "Duplicate key should fail");

    // Rollback (clean up transaction state)
    client.execute("ROLLBACK", &[]).await?;

    // Verify only original data exists
    let rows = client.query("SELECT * FROM error_test", &[]).await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<_, i32>(0), 1);

    Ok(())
}

/// Test auto-commit mode (default behavior without BEGIN)
#[tokio::test]
async fn test_auto_commit_mode() -> Result<(), Box<dyn std::error::Error>> {
    sleep(Duration::from_millis(100)).await;

    let (client, connection) = tokio_postgres::connect(
        "host=localhost port=5433 user=postgres",
        tokio_postgres::NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Drop table if exists, then create fresh
    let _ = client.execute("DROP TABLE auto_commit", &[]).await;

    client
        .execute(
            "CREATE TABLE auto_commit (id INT PRIMARY KEY, value TEXT)",
            &[],
        )
        .await?;

    // Insert without BEGIN (auto-commit)
    client
        .execute("INSERT INTO auto_commit VALUES (1, 'auto')", &[])
        .await?;

    // Should be immediately visible
    let rows = client
        .query("SELECT * FROM auto_commit WHERE id = 1", &[])
        .await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<_, i32>(0), 1);

    Ok(())
}
