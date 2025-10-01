use omendb::{LogConfig, init_logging, catalog::Catalog, sql_engine::SqlEngine};
use tempfile::TempDir;

#[test]
fn test_logging_initialization() {
    // Test JSON format initialization
    let config = LogConfig {
        level: "info".to_string(),
        json_format: true,
        log_queries: false,
        log_spans: true,
        log_file: None,
    };

    // Should not panic
    let _ = init_logging(config);
}

#[test]
fn test_pretty_format_initialization() {
    // Test pretty format initialization
    let config = LogConfig {
        level: "debug".to_string(),
        json_format: false,
        log_queries: true,
        log_spans: true,
        log_file: None,
    };

    // Should not panic
    let _ = init_logging(config);
}

#[test]
fn test_sql_execution_with_logging() {
    // Initialize logging for this test
    let config = LogConfig {
        level: "debug".to_string(),
        json_format: false,
        log_queries: true,
        log_spans: false,
        log_file: None,
    };
    let _ = init_logging(config);

    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // These operations should generate log output
    engine.execute("CREATE TABLE test (id BIGINT PRIMARY KEY, name TEXT)").unwrap();
    engine.execute("INSERT INTO test VALUES (1, 'Alice')").unwrap();
    engine.execute("SELECT * FROM test").unwrap();

    // No assertions on log output - just verify execution doesn't fail
}

#[test]
fn test_error_logging() {
    // Initialize logging
    let config = LogConfig {
        level: "warn".to_string(),
        json_format: false,
        log_queries: false,
        log_spans: false,
        log_file: None,
    };
    let _ = init_logging(config);

    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // This should generate error logs
    let result = engine.execute("INVALID SQL QUERY");
    assert!(result.is_err());

    // Query non-existent table
    let result = engine.execute("SELECT * FROM nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_connection_pool_logging() {
    use omendb::connection_pool::ConnectionPool;

    // Initialize logging
    let config = LogConfig {
        level: "debug".to_string(),
        json_format: false,
        log_queries: false,
        log_spans: false,
        log_file: None,
    };
    let _ = init_logging(config);

    let pool = ConnectionPool::new();

    // These operations should generate log output
    let conn1 = pool.acquire().unwrap();
    let conn2 = pool.acquire().unwrap();

    drop(conn1);
    drop(conn2);

    // Cleanup idle connections (should log if any removed)
    let _ = pool.cleanup_idle_connections();
}

#[test]
fn test_production_config() {
    let config = LogConfig::production();
    assert_eq!(config.level, "info");
    assert!(config.json_format);
    assert!(!config.log_queries);
    assert_eq!(config.log_file, Some("omendb.log".to_string()));
}

#[test]
fn test_development_config() {
    let config = LogConfig::development();
    assert_eq!(config.level, "debug");
    assert!(!config.json_format);
    assert!(config.log_queries);
}

#[test]
fn test_verbose_config() {
    let config = LogConfig::verbose();
    assert_eq!(config.level, "trace");
    assert!(config.log_queries);
}

#[test]
fn test_log_levels() {
    // Test different log levels
    let levels = vec!["trace", "debug", "info", "warn", "error"];

    for level in levels {
        let config = LogConfig {
            level: level.to_string(),
            json_format: false,
            log_queries: false,
            log_spans: false,
            log_file: None,
        };

        // Should not panic
        let _ = init_logging(config);
    }
}

#[test]
fn test_query_logging_flag() {
    // With query logging enabled
    let config = LogConfig {
        level: "debug".to_string(),
        json_format: false,
        log_queries: true,
        log_spans: false,
        log_file: None,
    };
    let _ = init_logging(config);

    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // This should log the full query
    engine.execute("CREATE TABLE users (id BIGINT PRIMARY KEY)").unwrap();
}

#[test]
fn test_span_events_flag() {
    // With span events enabled
    let config = LogConfig {
        level: "trace".to_string(),
        json_format: false,
        log_queries: false,
        log_spans: true,
        log_file: None,
    };
    let _ = init_logging(config);

    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // This should generate span events
    engine.execute("CREATE TABLE test (id BIGINT PRIMARY KEY)").unwrap();
}
