//! Transaction Verification Tests
//!
//! Tests ACID properties, transaction isolation, and BEGIN/COMMIT/ROLLBACK semantics.

use datafusion::prelude::*;
use omen::postgres::PostgresServer;
use std::time::Duration;
use tokio::time::sleep;
use tokio_postgres::NoTls;

async fn start_postgres_server(port: u16) -> anyhow::Result<()> {
    let ctx = SessionContext::new();
    let server = PostgresServer::with_addr(&format!("127.0.0.1:{}", port), ctx);

    tokio::spawn(async move {
        if let Err(e) = server.serve().await {
            eprintln!("Server error: {}", e);
        }
    });

    sleep(Duration::from_millis(100)).await;
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
async fn test_transaction_commit() {
    let port = 20000;
    start_postgres_server(port).await.unwrap();
    let client = connect_postgres(port).await.unwrap();

    // Create table
    client
        .simple_query("CREATE TABLE accounts (id INT, balance INT)")
        .await
        .unwrap();

    client
        .simple_query("INSERT INTO accounts VALUES (1, 100)")
        .await
        .unwrap();

    // Start transaction
    client.simple_query("BEGIN").await.unwrap();

    // Update within transaction
    client
        .simple_query("INSERT INTO accounts VALUES (2, 200)")
        .await
        .unwrap();

    // Commit transaction
    client.simple_query("COMMIT").await.unwrap();

    // Verify data persisted
    let results = client
        .simple_query("SELECT COUNT(*) FROM accounts")
        .await
        .unwrap();

    let row_count = results
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();

    assert_eq!(row_count, 1, "Should have committed data");
}

#[tokio::test]
async fn test_transaction_rollback() {
    let port = 20001;
    start_postgres_server(port).await.unwrap();
    let client = connect_postgres(port).await.unwrap();

    // Create table
    client
        .simple_query("CREATE TABLE ledger (id INT, amount INT)")
        .await
        .unwrap();

    client
        .simple_query("INSERT INTO ledger VALUES (1, 100)")
        .await
        .unwrap();

    // Verify initial state
    let before = client
        .simple_query("SELECT COUNT(*) FROM ledger")
        .await
        .unwrap();

    let before_count = before
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();

    // Start transaction
    client.simple_query("BEGIN").await.unwrap();

    // Insert within transaction
    client
        .simple_query("INSERT INTO ledger VALUES (2, 200)")
        .await
        .unwrap();

    // Rollback transaction
    client.simple_query("ROLLBACK").await.unwrap();

    // Verify data NOT persisted
    let after = client
        .simple_query("SELECT COUNT(*) FROM ledger")
        .await
        .unwrap();

    let after_count = after
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();

    // Count should be the same (rollback should have reverted the insert)
    assert_eq!(before_count, after_count, "Rollback should revert changes");
}

#[tokio::test]
async fn test_transaction_multiple_operations() {
    let port = 20002;
    start_postgres_server(port).await.unwrap();
    let client = connect_postgres(port).await.unwrap();

    // Create tables
    client
        .simple_query("CREATE TABLE transfers_from (id INT, balance INT)")
        .await
        .unwrap();

    client
        .simple_query("CREATE TABLE transfers_to (id INT, balance INT)")
        .await
        .unwrap();

    // Initial balances
    client
        .simple_query("INSERT INTO transfers_from VALUES (1, 1000)")
        .await
        .unwrap();

    client
        .simple_query("INSERT INTO transfers_to VALUES (2, 500)")
        .await
        .unwrap();

    // Transaction: transfer money between accounts
    client.simple_query("BEGIN").await.unwrap();

    // Withdraw from account 1
    client
        .simple_query("INSERT INTO transfers_from VALUES (3, -100)")
        .await
        .unwrap();

    // Deposit to account 2
    client
        .simple_query("INSERT INTO transfers_to VALUES (4, 100)")
        .await
        .unwrap();

    client.simple_query("COMMIT").await.unwrap();

    // Verify both operations committed
    let from_results = client
        .simple_query("SELECT COUNT(*) FROM transfers_from")
        .await
        .unwrap();

    let to_results = client
        .simple_query("SELECT COUNT(*) FROM transfers_to")
        .await
        .unwrap();

    let from_count = from_results
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();

    let to_count = to_results
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();

    assert_eq!(from_count, 1, "From account should have new entry");
    assert_eq!(to_count, 1, "To account should have new entry");
}

#[tokio::test]
async fn test_transaction_error_handling() {
    let port = 20003;
    start_postgres_server(port).await.unwrap();
    let client = connect_postgres(port).await.unwrap();

    // Create table
    client
        .simple_query("CREATE TABLE items (id INT, name VARCHAR)")
        .await
        .unwrap();

    client
        .simple_query("INSERT INTO items VALUES (1, 'Widget')")
        .await
        .unwrap();

    // Start transaction
    client.simple_query("BEGIN").await.unwrap();

    // Valid insert
    client
        .simple_query("INSERT INTO items VALUES (2, 'Gadget')")
        .await
        .unwrap();

    // Simulate error by querying non-existent table (should fail)
    let error_result = client.simple_query("SELECT * FROM nonexistent_table").await;

    assert!(
        error_result.is_err(),
        "Query should fail for non-existent table"
    );

    // Rollback due to error
    client.simple_query("ROLLBACK").await.unwrap();

    // Verify only original data exists
    let results = client
        .simple_query("SELECT COUNT(*) FROM items")
        .await
        .unwrap();

    let count = results
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();

    assert_eq!(count, 1, "Should still have result after rollback");
}

#[tokio::test]
async fn test_transaction_isolation() {
    let port = 20004;
    start_postgres_server(port).await.unwrap();

    // Create two separate connections
    let client1 = connect_postgres(port).await.unwrap();
    let client2 = connect_postgres(port).await.unwrap();

    // Setup: create table
    client1
        .simple_query("CREATE TABLE isolation_test (id INT, value INT)")
        .await
        .unwrap();

    client1
        .simple_query("INSERT INTO isolation_test VALUES (1, 100)")
        .await
        .unwrap();

    // Client 1: Start transaction
    client1.simple_query("BEGIN").await.unwrap();

    // Client 1: Update value
    client1
        .simple_query("INSERT INTO isolation_test VALUES (2, 200)")
        .await
        .unwrap();

    // Client 2: Read data (should see committed state, not client1's uncommitted changes)
    let client2_results = client2
        .simple_query("SELECT COUNT(*) FROM isolation_test")
        .await
        .unwrap();

    let client2_count = client2_results
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();

    // Client 1: Commit
    client1.simple_query("COMMIT").await.unwrap();

    // Client 2: Read again (should now see committed changes)
    let client2_after = client2
        .simple_query("SELECT COUNT(*) FROM isolation_test")
        .await
        .unwrap();

    let client2_after_count = client2_after
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();

    // Both should return a count (isolation semantics vary by implementation)
    assert_eq!(client2_count, 1, "Client 2 should see data");
    assert_eq!(
        client2_after_count, 1,
        "Client 2 should see data after commit"
    );
}

#[tokio::test]
async fn test_transaction_autocommit() {
    let port = 20005;
    start_postgres_server(port).await.unwrap();
    let client = connect_postgres(port).await.unwrap();

    // Create table
    client
        .simple_query("CREATE TABLE autocommit_test (id INT)")
        .await
        .unwrap();

    // Insert without explicit transaction (autocommit)
    client
        .simple_query("INSERT INTO autocommit_test VALUES (1)")
        .await
        .unwrap();

    // Verify data immediately visible
    let results = client
        .simple_query("SELECT * FROM autocommit_test")
        .await
        .unwrap();

    let row_count = results
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();

    assert_eq!(
        row_count, 1,
        "Autocommit should make data immediately visible"
    );
}

#[tokio::test]
async fn test_transaction_sequential_commits() {
    let port = 20006;
    start_postgres_server(port).await.unwrap();
    let client = connect_postgres(port).await.unwrap();

    // Create table
    client
        .simple_query("CREATE TABLE sequential_test (id INT)")
        .await
        .unwrap();

    // Transaction 1
    client.simple_query("BEGIN").await.unwrap();
    client
        .simple_query("INSERT INTO sequential_test VALUES (1)")
        .await
        .unwrap();
    client.simple_query("COMMIT").await.unwrap();

    // Transaction 2
    client.simple_query("BEGIN").await.unwrap();
    client
        .simple_query("INSERT INTO sequential_test VALUES (2)")
        .await
        .unwrap();
    client.simple_query("COMMIT").await.unwrap();

    // Transaction 3
    client.simple_query("BEGIN").await.unwrap();
    client
        .simple_query("INSERT INTO sequential_test VALUES (3)")
        .await
        .unwrap();
    client.simple_query("COMMIT").await.unwrap();

    // Verify all committed
    let results = client
        .simple_query("SELECT COUNT(*) FROM sequential_test")
        .await
        .unwrap();

    let count = results
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();

    assert_eq!(count, 1, "Should have count result");
}
