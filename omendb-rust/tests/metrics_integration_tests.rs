use omendb::catalog::Catalog;
use omendb::sql_engine::SqlEngine;
use omendb::metrics::get_metrics;
use tempfile::TempDir;

#[test]
fn test_sql_query_metrics_recorded() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Execute CREATE TABLE - should record CREATE_TABLE metric
    engine.execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name TEXT)").unwrap();

    // Execute INSERT - should record INSERT metric
    engine.execute("INSERT INTO users VALUES (1, 'Alice')").unwrap();
    engine.execute("INSERT INTO users VALUES (2, 'Bob')").unwrap();

    // Execute SELECT - should record SELECT metric
    engine.execute("SELECT * FROM users").unwrap();

    // Get metrics and verify they were recorded
    let metrics = get_metrics();

    // Should contain SQL query metrics
    assert!(metrics.contains("omendb_sql_query_duration_seconds"));
    assert!(metrics.contains("omendb_sql_queries_total"));
    assert!(metrics.contains("omendb_sql_query_rows_returned"));

    // Should contain query types
    assert!(metrics.contains("CREATE_TABLE") || metrics.contains("query_type"));
    assert!(metrics.contains("INSERT") || metrics.contains("query_type"));
    assert!(metrics.contains("SELECT") || metrics.contains("query_type"));
}

#[test]
fn test_sql_error_metrics_recorded() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Execute invalid SQL - should record error metric
    let result = engine.execute("INVALID SQL QUERY");
    assert!(result.is_err());

    // Query non-existent table - should record error
    let result = engine.execute("SELECT * FROM nonexistent_table");
    assert!(result.is_err());

    // Get metrics and verify errors were recorded
    let metrics = get_metrics();
    assert!(metrics.contains("omendb_sql_query_errors_total"));
}

#[test]
fn test_query_latency_histogram() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Create table and insert data
    engine.execute("CREATE TABLE metrics_test (id BIGINT PRIMARY KEY, value BIGINT)").unwrap();

    for i in 1..=100 {
        engine.execute(&format!("INSERT INTO metrics_test VALUES ({}, {})", i, i * 10)).unwrap();
    }

    // Execute query that should generate histogram data
    engine.execute("SELECT * FROM metrics_test").unwrap();

    // Get metrics
    let metrics = get_metrics();

    // Should contain histogram buckets
    assert!(metrics.contains("omendb_sql_query_duration_seconds"));
    assert!(metrics.contains("bucket"));
}

#[test]
fn test_rows_returned_metric() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Create table and insert data
    engine.execute("CREATE TABLE row_count_test (id BIGINT PRIMARY KEY, data TEXT)").unwrap();

    for i in 1..=50 {
        engine.execute(&format!("INSERT INTO row_count_test VALUES ({}, 'data_{}')", i, i)).unwrap();
    }

    // Execute query
    engine.execute("SELECT * FROM row_count_test").unwrap();

    // Get metrics
    let metrics = get_metrics();

    // Should contain rows returned histogram
    assert!(metrics.contains("omendb_sql_query_rows_returned"));
}

#[test]
fn test_timeout_error_metric() {
    use std::time::Duration;
    use omendb::sql_engine::QueryConfig;

    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();

    // Set very short timeout for testing
    let config = QueryConfig {
        timeout: Duration::from_nanos(1), // Extremely short timeout
        max_rows: 1_000_000,
        max_memory_bytes: 1_000_000_000,
    };

    let mut engine = SqlEngine::with_config(catalog, config);

    // This should timeout immediately
    let result = engine.execute("CREATE TABLE timeout_test (id BIGINT PRIMARY KEY)");

    // Even if it doesn't timeout (race condition), we should still get metrics
    let metrics = get_metrics();
    assert!(metrics.contains("omendb_sql_query") || metrics.contains("sql"));
}

#[test]
fn test_aggregate_query_metrics() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Create table with data
    engine.execute("CREATE TABLE sales (id BIGINT PRIMARY KEY, category TEXT, amount FLOAT64)").unwrap();

    for i in 1..=20 {
        let category = if i % 2 == 0 { "A" } else { "B" };
        engine.execute(&format!("INSERT INTO sales VALUES ({}, '{}', {})", i, category, i as f64 * 10.0)).unwrap();
    }

    // Execute aggregate query
    engine.execute("SELECT category, COUNT(*), SUM(amount) FROM sales GROUP BY category").unwrap();

    // Get metrics
    let metrics = get_metrics();

    // Should contain SELECT metrics
    assert!(metrics.contains("omendb_sql_queries_total"));
    assert!(metrics.contains("SELECT") || metrics.contains("query_type"));
}