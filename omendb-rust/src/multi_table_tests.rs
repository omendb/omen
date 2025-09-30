//! Comprehensive integration tests for multi-table database functionality
//! Tests the full stack: Catalog, Table, SQL Engine, Storage, and Learned Index

use crate::catalog::Catalog;
use crate::sql_engine::{SqlEngine, ExecutionResult};
use crate::value::Value;
use anyhow::Result;
use tempfile::TempDir;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_multi_table_creation_and_queries() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        // Create users table
        engine.execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255), age BIGINT)").unwrap();

        // Create orders table
        engine.execute("CREATE TABLE orders (order_id BIGINT PRIMARY KEY, user_id BIGINT, amount DOUBLE)").unwrap();

        // Create logs table (time-series)
        engine.execute("CREATE TABLE logs (timestamp BIGINT PRIMARY KEY, level VARCHAR(50), message VARCHAR(500))").unwrap();

        // Verify all tables exist
        assert!(engine.catalog().table_exists("users"));
        assert!(engine.catalog().table_exists("orders"));
        assert!(engine.catalog().table_exists("logs"));
        assert_eq!(engine.catalog().list_tables().len(), 3);
    }

    #[test]
    fn test_multi_table_inserts_and_selects() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        // Create tables
        engine.execute("CREATE TABLE products (id BIGINT PRIMARY KEY, name VARCHAR(255), price DOUBLE)").unwrap();
        engine.execute("CREATE TABLE inventory (id BIGINT PRIMARY KEY, product_id BIGINT, quantity BIGINT)").unwrap();

        // Insert into products
        let result = engine.execute(
            "INSERT INTO products VALUES (1, 'Laptop', 999.99), (2, 'Mouse', 29.99), (3, 'Keyboard', 79.99)"
        ).unwrap();

        match result {
            ExecutionResult::Inserted { rows } => assert_eq!(rows, 3),
            _ => panic!("Expected Inserted result"),
        }

        // Insert into inventory
        let result = engine.execute(
            "INSERT INTO inventory VALUES (1, 1, 50), (2, 2, 200), (3, 3, 75)"
        ).unwrap();

        match result {
            ExecutionResult::Inserted { rows } => assert_eq!(rows, 3),
            _ => panic!("Expected Inserted result"),
        }

        // Query products
        let result = engine.execute("SELECT * FROM products").unwrap();
        match result {
            ExecutionResult::Selected { rows, data, .. } => {
                assert_eq!(rows, 3);
                assert_eq!(data[0].get(1).unwrap(), &Value::Text("Laptop".to_string()));
                assert_eq!(data[0].get(2).unwrap(), &Value::Float64(999.99));
            }
            _ => panic!("Expected Selected result"),
        }

        // Query inventory
        let result = engine.execute("SELECT * FROM inventory").unwrap();
        match result {
            ExecutionResult::Selected { rows, data, .. } => {
                assert_eq!(rows, 3);
                assert_eq!(data[0].get(2).unwrap(), &Value::Int64(50));
            }
            _ => panic!("Expected Selected result"),
        }
    }

    #[test]
    fn test_different_schemas_across_tables() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        // Table 1: Simple integers
        engine.execute("CREATE TABLE simple (id BIGINT PRIMARY KEY)").unwrap();

        // Table 2: Mixed types
        engine.execute(
            "CREATE TABLE mixed (id BIGINT PRIMARY KEY, name VARCHAR(100), value DOUBLE, active BOOLEAN)"
        ).unwrap();

        // Table 3: Text-heavy
        engine.execute(
            "CREATE TABLE documents (id BIGINT PRIMARY KEY, title VARCHAR(255), content VARCHAR(1000))"
        ).unwrap();

        // Insert different data types
        engine.execute("INSERT INTO simple VALUES (1), (2), (3)").unwrap();
        engine.execute("INSERT INTO mixed VALUES (1, 'test', 1.5, true)").unwrap();
        engine.execute("INSERT INTO documents VALUES (1, 'Title', 'Content here')").unwrap();

        // Verify each table has correct schema
        let simple = engine.catalog().get_table("simple").unwrap();
        assert_eq!(simple.schema().fields().len(), 1);

        let mixed = engine.catalog().get_table("mixed").unwrap();
        assert_eq!(mixed.schema().fields().len(), 4);

        let documents = engine.catalog().get_table("documents").unwrap();
        assert_eq!(documents.schema().fields().len(), 3);
    }

    #[test]
    fn test_large_scale_multi_table_operations() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        // Create tables
        engine.execute("CREATE TABLE events (timestamp BIGINT PRIMARY KEY, event_type VARCHAR(50))").unwrap();
        engine.execute("CREATE TABLE metrics (timestamp BIGINT PRIMARY KEY, value DOUBLE)").unwrap();

        // Insert 1000 records into each table
        for i in 0..1000 {
            let sql = format!("INSERT INTO events VALUES ({}, 'event_{}')", i, i % 10);
            engine.execute(&sql).unwrap();

            let sql = format!("INSERT INTO metrics VALUES ({}, {})", i + 10000, i as f64 * 1.5);
            engine.execute(&sql).unwrap();
        }

        // Verify counts
        let events = engine.catalog().get_table("events").unwrap();
        assert_eq!(events.row_count(), 1000);

        let metrics = engine.catalog().get_table("metrics").unwrap();
        assert_eq!(metrics.row_count(), 1000);

        // Query and verify data
        let result = engine.execute("SELECT * FROM events").unwrap();
        match result {
            ExecutionResult::Selected { rows, .. } => assert_eq!(rows, 1000),
            _ => panic!("Expected Selected result"),
        }

        let result = engine.execute("SELECT * FROM metrics").unwrap();
        match result {
            ExecutionResult::Selected { rows, .. } => assert_eq!(rows, 1000),
            _ => panic!("Expected Selected result"),
        }
    }

    #[test]
    fn test_table_isolation() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        // Create two tables with same schema
        engine.execute("CREATE TABLE table1 (id BIGINT PRIMARY KEY, value BIGINT)").unwrap();
        engine.execute("CREATE TABLE table2 (id BIGINT PRIMARY KEY, value BIGINT)").unwrap();

        // Insert different data
        engine.execute("INSERT INTO table1 VALUES (1, 100), (2, 200)").unwrap();
        engine.execute("INSERT INTO table2 VALUES (1, 999), (2, 888)").unwrap();

        // Query table1 - should only see table1 data
        let result = engine.execute("SELECT * FROM table1").unwrap();
        match result {
            ExecutionResult::Selected { data, .. } => {
                assert_eq!(data[0].get(1).unwrap(), &Value::Int64(100));
                assert_eq!(data[1].get(1).unwrap(), &Value::Int64(200));
            }
            _ => panic!("Expected Selected result"),
        }

        // Query table2 - should only see table2 data
        let result = engine.execute("SELECT * FROM table2").unwrap();
        match result {
            ExecutionResult::Selected { data, .. } => {
                assert_eq!(data[0].get(1).unwrap(), &Value::Int64(999));
                assert_eq!(data[1].get(1).unwrap(), &Value::Int64(888));
            }
            _ => panic!("Expected Selected result"),
        }
    }

    #[test]
    fn test_error_handling_table_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        // Try to insert into non-existent table
        let result = engine.execute("INSERT INTO nonexistent VALUES (1, 2)");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));

        // Try to select from non-existent table
        let result = engine.execute("SELECT * FROM nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_handling_duplicate_table() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        // Create table
        engine.execute("CREATE TABLE test (id BIGINT PRIMARY KEY)").unwrap();

        // Try to create again - should fail
        let result = engine.execute("CREATE TABLE test (id BIGINT PRIMARY KEY)");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exists"));
    }

    #[test]
    fn test_persistence_multi_table() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_path_buf();

        // Phase 1: Create tables and insert data
        {
            let catalog = Catalog::new(db_path.clone()).unwrap();
            let mut engine = SqlEngine::new(catalog);

            engine.execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))").unwrap();
            engine.execute("CREATE TABLE orders (id BIGINT PRIMARY KEY, amount DOUBLE)").unwrap();

            engine.execute("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')").unwrap();
            engine.execute("INSERT INTO orders VALUES (1, 99.99), (2, 149.99)").unwrap();

            // Tables exist before closing
            assert!(engine.catalog().table_exists("users"));
            assert!(engine.catalog().table_exists("orders"));
        }

        // Phase 2: Reload catalog and verify tables exist
        {
            let catalog = Catalog::new(db_path.clone()).unwrap();
            let engine = SqlEngine::new(catalog);

            // Tables should still exist
            assert!(engine.catalog().table_exists("users"));
            assert!(engine.catalog().table_exists("orders"));

            // Data should be persisted
            let users = engine.catalog().get_table("users").unwrap();
            assert_eq!(users.row_count(), 2);

            let orders = engine.catalog().get_table("orders").unwrap();
            assert_eq!(orders.row_count(), 2);
        }
    }

    #[test]
    fn test_learned_index_per_table() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        // Create tables with different primary keys
        engine.execute("CREATE TABLE sequential (id BIGINT PRIMARY KEY, value BIGINT)").unwrap();
        engine.execute("CREATE TABLE timestamps (ts BIGINT PRIMARY KEY, value BIGINT)").unwrap();

        // Insert sequential data
        for i in 0..100 {
            let sql = format!("INSERT INTO sequential VALUES ({}, {})", i, i * 2);
            engine.execute(&sql).unwrap();
        }

        // Insert timestamp data (sparse keys)
        for i in 0..100 {
            let sql = format!("INSERT INTO timestamps VALUES ({}, {})", i * 1000, i);
            engine.execute(&sql).unwrap();
        }

        // Both tables should have learned indexes
        let sequential = engine.catalog().get_table("sequential").unwrap();
        assert_eq!(sequential.row_count(), 100);

        let timestamps = engine.catalog().get_table("timestamps").unwrap();
        assert_eq!(timestamps.row_count(), 100);

        // Query using learned indexes
        let result = engine.execute("SELECT * FROM sequential").unwrap();
        match result {
            ExecutionResult::Selected { rows, .. } => assert_eq!(rows, 100),
            _ => panic!("Expected Selected result"),
        }

        let result = engine.execute("SELECT * FROM timestamps").unwrap();
        match result {
            ExecutionResult::Selected { rows, .. } => assert_eq!(rows, 100),
            _ => panic!("Expected Selected result"),
        }
    }

    #[test]
    fn test_table_get_by_primary_key() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        engine.execute("CREATE TABLE test (id BIGINT PRIMARY KEY, name VARCHAR(255))").unwrap();
        engine.execute("INSERT INTO test VALUES (1, 'Alice'), (10, 'Bob'), (100, 'Charlie')").unwrap();

        let table = engine.catalog().get_table("test").unwrap();

        // Test point queries using learned index
        let row = table.get(&Value::Int64(1)).unwrap();
        assert!(row.is_some());
        assert_eq!(row.unwrap().get(1).unwrap(), &Value::Text("Alice".to_string()));

        let row = table.get(&Value::Int64(10)).unwrap();
        assert!(row.is_some());
        assert_eq!(row.unwrap().get(1).unwrap(), &Value::Text("Bob".to_string()));

        let row = table.get(&Value::Int64(100)).unwrap();
        assert!(row.is_some());
        assert_eq!(row.unwrap().get(1).unwrap(), &Value::Text("Charlie".to_string()));

        // Non-existent key
        let row = table.get(&Value::Int64(999)).unwrap();
        assert!(row.is_none());
    }

    #[test]
    fn test_mixed_data_types_all_supported() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        engine.execute(
            "CREATE TABLE all_types (
                id BIGINT PRIMARY KEY,
                int_col BIGINT,
                float_col DOUBLE,
                text_col VARCHAR(255),
                bool_col BOOLEAN
            )"
        ).unwrap();

        engine.execute(
            "INSERT INTO all_types VALUES (1, 42, 3.14, 'test', true)"
        ).unwrap();

        let result = engine.execute("SELECT * FROM all_types").unwrap();
        match result {
            ExecutionResult::Selected { data, .. } => {
                let row = &data[0];
                assert_eq!(row.get(0).unwrap(), &Value::Int64(1));
                assert_eq!(row.get(1).unwrap(), &Value::Int64(42));
                assert_eq!(row.get(2).unwrap(), &Value::Float64(3.14));
                assert_eq!(row.get(3).unwrap(), &Value::Text("test".to_string()));
                assert_eq!(row.get(4).unwrap(), &Value::Boolean(true));
            }
            _ => panic!("Expected Selected result"),
        }
    }
}