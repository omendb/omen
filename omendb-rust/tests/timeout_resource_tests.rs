use omendb::catalog::Catalog;
use omendb::sql_engine::{SqlEngine, ExecutionResult, QueryConfig};
use omendb::value::Value;
use tempfile::TempDir;
use std::time::Duration;

#[test]
fn test_query_timeout_config() {
    // Test that custom timeout config works
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();

    let config = QueryConfig {
        timeout: Duration::from_secs(60),
        max_rows: 5000,
        max_memory_bytes: 500_000_000,
    };

    let mut engine = SqlEngine::with_config(catalog, config);

    engine.execute("CREATE TABLE test (id BIGINT PRIMARY KEY, value BIGINT)").unwrap();
    engine.execute("INSERT INTO test VALUES (1, 100)").unwrap();

    let result = engine.execute("SELECT * FROM test").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 1);
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_max_rows_limit() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();

    // Set very low max_rows for testing
    let config = QueryConfig {
        timeout: Duration::from_secs(30),
        max_rows: 10,
        max_memory_bytes: 1_000_000_000,
    };

    let mut engine = SqlEngine::with_config(catalog, config);

    engine.execute("CREATE TABLE test (id BIGINT PRIMARY KEY, value BIGINT)").unwrap();

    // Insert 15 rows
    for i in 1..=15 {
        engine.execute(&format!("INSERT INTO test VALUES ({}, {})", i, i * 10)).unwrap();
    }

    // Query without LIMIT should fail (exceeds max_rows)
    let result = engine.execute("SELECT * FROM test");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("exceeds maximum row limit"));

    // Query with LIMIT should succeed
    let result = engine.execute("SELECT * FROM test LIMIT 5").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 5);
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_query_size_limit() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Create a query that exceeds 10MB size limit
    let huge_query = "SELECT ".to_string() + &"x".repeat(11_000_000);

    let result = engine.execute(&huge_query);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Query size exceeds limit"));
}

#[test]
fn test_default_limits_are_reasonable() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Default config should allow normal operations
    engine.execute("CREATE TABLE test (id BIGINT PRIMARY KEY, value BIGINT)").unwrap();

    // Insert 1000 rows (well under default max_rows of 1M)
    for i in 1..=1000 {
        engine.execute(&format!("INSERT INTO test VALUES ({}, {})", i, i * 10)).unwrap();
    }

    let result = engine.execute("SELECT * FROM test").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 1000);
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_aggregates_respect_limits() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();

    let config = QueryConfig {
        timeout: Duration::from_secs(30),
        max_rows: 5, // Very low limit
        max_memory_bytes: 1_000_000_000,
    };

    let mut engine = SqlEngine::with_config(catalog, config);

    engine.execute("CREATE TABLE sales (id BIGINT PRIMARY KEY, category TEXT, amount FLOAT64)").unwrap();

    for i in 1..=20 {
        let category = if i % 2 == 0 { "A" } else { "B" };
        engine.execute(&format!("INSERT INTO sales VALUES ({}, '{}', {})", i, category, i as f64 * 10.0)).unwrap();
    }

    // Aggregate query that returns few rows should work
    let result = engine.execute("SELECT category, COUNT(*), SUM(amount) FROM sales GROUP BY category").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 2); // Only 2 groups, under limit
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_order_by_with_limits() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();

    let config = QueryConfig {
        timeout: Duration::from_secs(30),
        max_rows: 100,
        max_memory_bytes: 1_000_000_000,
    };

    let mut engine = SqlEngine::with_config(catalog, config);

    engine.execute("CREATE TABLE test (id BIGINT PRIMARY KEY, value BIGINT)").unwrap();

    for i in 1..=50 {
        engine.execute(&format!("INSERT INTO test VALUES ({}, {})", i, 100 - i)).unwrap();
    }

    // ORDER BY should respect limits
    let result = engine.execute("SELECT * FROM test ORDER BY value DESC").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 50);
            // Verify descending order
            assert_eq!(data[0].get(1).unwrap(), &Value::Int64(99));
            assert_eq!(data[49].get(1).unwrap(), &Value::Int64(50));
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_empty_query_error() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Empty query should return error
    let result = engine.execute("");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No SQL statement found"));
}

#[test]
fn test_multiple_statements_error() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Multiple statements not supported
    let result = engine.execute("SELECT 1; SELECT 2;");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Multiple statements not supported"));
}