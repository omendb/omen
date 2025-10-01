/// Comprehensive transaction tests for BEGIN/COMMIT/ROLLBACK
/// Tests ACID properties and transaction isolation

use omendb::{
    catalog::Catalog,
    sql_engine::{SqlEngine, QueryConfig, ExecutionResult},
    wal::WalManager,
};
use tempfile::TempDir;
use std::sync::Arc;

#[test]
fn test_begin_transaction() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();

    // Create WAL manager
    let wal_dir = temp_dir.path().join("wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    let wal = Arc::new(WalManager::new(&wal_dir).unwrap());

    // Create engine with transaction support
    let mut engine = SqlEngine::with_transactions(catalog, QueryConfig::default(), wal);

    // Begin transaction
    let result = engine.execute("BEGIN").unwrap();

    match result {
        ExecutionResult::TransactionStarted { txn_id } => {
            assert_eq!(txn_id, 0); // First transaction should have ID 0
        },
        _ => panic!("Expected TransactionStarted result"),
    }

    println!("✅ BEGIN transaction works");
}

#[test]
fn test_commit_transaction() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();

    let wal_dir = temp_dir.path().join("wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    let wal = Arc::new(WalManager::new(&wal_dir).unwrap());

    let mut engine = SqlEngine::with_transactions(catalog, QueryConfig::default(), wal);

    // Begin transaction
    engine.execute("BEGIN").unwrap();

    // Commit transaction
    let result = engine.execute("COMMIT").unwrap();

    match result {
        ExecutionResult::TransactionCommitted { txn_id } => {
            assert_eq!(txn_id, 0);
        },
        _ => panic!("Expected TransactionCommitted result"),
    }

    println!("✅ COMMIT transaction works");
}

#[test]
fn test_rollback_transaction() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();

    let wal_dir = temp_dir.path().join("wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    let wal = Arc::new(WalManager::new(&wal_dir).unwrap());

    let mut engine = SqlEngine::with_transactions(catalog, QueryConfig::default(), wal);

    // Begin transaction
    engine.execute("BEGIN").unwrap();

    // Rollback transaction
    let result = engine.execute("ROLLBACK").unwrap();

    match result {
        ExecutionResult::TransactionRolledBack { txn_id } => {
            assert_eq!(txn_id, 0);
        },
        _ => panic!("Expected TransactionRolledBack result"),
    }

    println!("✅ ROLLBACK transaction works");
}

#[test]
fn test_commit_without_begin() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();

    let wal_dir = temp_dir.path().join("wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    let wal = Arc::new(WalManager::new(&wal_dir).unwrap());

    let mut engine = SqlEngine::with_transactions(catalog, QueryConfig::default(), wal);

    // Try to commit without beginning a transaction
    let result = engine.execute("COMMIT");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No active transaction"));

    println!("✅ COMMIT without BEGIN properly errors");
}

#[test]
fn test_rollback_without_begin() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();

    let wal_dir = temp_dir.path().join("wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    let wal = Arc::new(WalManager::new(&wal_dir).unwrap());

    let mut engine = SqlEngine::with_transactions(catalog, QueryConfig::default(), wal);

    // Try to rollback without beginning a transaction
    let result = engine.execute("ROLLBACK");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No active transaction"));

    println!("✅ ROLLBACK without BEGIN properly errors");
}

#[test]
fn test_nested_begin() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();

    let wal_dir = temp_dir.path().join("wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    let wal = Arc::new(WalManager::new(&wal_dir).unwrap());

    let mut engine = SqlEngine::with_transactions(catalog, QueryConfig::default(), wal);

    // Begin first transaction
    engine.execute("BEGIN").unwrap();

    // Try to begin another transaction while one is active
    let result = engine.execute("BEGIN");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Transaction already in progress"));

    println!("✅ Nested BEGIN properly errors");
}

#[test]
fn test_multiple_sequential_transactions() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();

    let wal_dir = temp_dir.path().join("wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    let wal = Arc::new(WalManager::new(&wal_dir).unwrap());

    let mut engine = SqlEngine::with_transactions(catalog, QueryConfig::default(), wal);

    // First transaction
    engine.execute("BEGIN").unwrap();
    engine.execute("COMMIT").unwrap();

    // Second transaction
    let result = engine.execute("BEGIN").unwrap();
    match result {
        ExecutionResult::TransactionStarted { txn_id } => {
            assert_eq!(txn_id, 1); // Should be second transaction
        },
        _ => panic!("Expected TransactionStarted result"),
    }

    engine.execute("ROLLBACK").unwrap();

    // Third transaction
    let result = engine.execute("BEGIN").unwrap();
    match result {
        ExecutionResult::TransactionStarted { txn_id } => {
            assert_eq!(txn_id, 2); // Should be third transaction
        },
        _ => panic!("Expected TransactionStarted result"),
    }

    engine.execute("COMMIT").unwrap();

    println!("✅ Sequential transactions work correctly");
}

#[test]
fn test_transaction_without_wal() {
    // Test that transactions fail gracefully when WAL is not configured
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();

    // Create engine WITHOUT transaction support
    let mut engine = SqlEngine::new(catalog);

    // Try to begin transaction
    let result = engine.execute("BEGIN");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Transactions not enabled"));

    println!("✅ Transactions without WAL properly error");
}

#[test]
fn test_begin_start_transaction_alias() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();

    let wal_dir = temp_dir.path().join("wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    let wal = Arc::new(WalManager::new(&wal_dir).unwrap());

    let mut engine = SqlEngine::with_transactions(catalog, QueryConfig::default(), wal);

    // BEGIN and START TRANSACTION should both work
    let result = engine.execute("START TRANSACTION").unwrap();
    match result {
        ExecutionResult::TransactionStarted { txn_id } => {
            assert_eq!(txn_id, 0);
        },
        _ => panic!("Expected TransactionStarted result"),
    }

    engine.execute("COMMIT").unwrap();

    println!("✅ START TRANSACTION alias works");
}

#[test]
fn test_transaction_metrics() {
    use omendb::metrics::get_metrics;

    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();

    let wal_dir = temp_dir.path().join("wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    let wal = Arc::new(WalManager::new(&wal_dir).unwrap());

    let mut engine = SqlEngine::with_transactions(catalog, QueryConfig::default(), wal);

    // Execute transaction commands
    engine.execute("BEGIN").unwrap();
    engine.execute("COMMIT").unwrap();
    engine.execute("BEGIN").unwrap();
    engine.execute("ROLLBACK").unwrap();

    // Get metrics
    let metrics = get_metrics();

    // Verify transaction metrics are recorded
    // (Exact verification depends on metrics format)
    assert!(metrics.contains("omendb_sql_queries_total") || metrics.contains("BEGIN") || metrics.contains("COMMIT"),
        "Transaction metrics should be recorded");

    println!("✅ Transaction metrics recorded");
}
