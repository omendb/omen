//! Tests for ORDER BY and LIMIT functionality

use omendb::catalog::Catalog;
use omendb::sql_engine::{SqlEngine, ExecutionResult};
use omendb::value::Value;
use tempfile::TempDir;

#[test]
fn test_order_by_asc() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE test (id BIGINT PRIMARY KEY, value BIGINT)").unwrap();

    // Insert in random order
    engine.execute("INSERT INTO test VALUES (3, 30), (1, 10), (5, 50), (2, 20), (4, 40)").unwrap();

    // ORDER BY id ASC
    let result = engine.execute("SELECT * FROM test ORDER BY id ASC").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 5);

            // Verify ascending order
            let ids: Vec<i64> = data.iter()
                .map(|row| {
                    if let Value::Int64(id) = row.get(0).unwrap() {
                        *id
                    } else {
                        panic!("Expected Int64")
                    }
                })
                .collect();

            assert_eq!(ids, vec![1, 2, 3, 4, 5], "IDs should be in ascending order");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_order_by_desc() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE test (id BIGINT PRIMARY KEY, value BIGINT)").unwrap();

    // Insert in random order
    engine.execute("INSERT INTO test VALUES (3, 30), (1, 10), (5, 50), (2, 20), (4, 40)").unwrap();

    // ORDER BY id DESC
    let result = engine.execute("SELECT * FROM test ORDER BY id DESC").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 5);

            // Verify descending order
            let ids: Vec<i64> = data.iter()
                .map(|row| {
                    if let Value::Int64(id) = row.get(0).unwrap() {
                        *id
                    } else {
                        panic!("Expected Int64")
                    }
                })
                .collect();

            assert_eq!(ids, vec![5, 4, 3, 2, 1], "IDs should be in descending order");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_order_by_with_where() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE test (id BIGINT PRIMARY KEY, value BIGINT)").unwrap();

    for i in 1..=10 {
        engine.execute(&format!("INSERT INTO test VALUES ({}, {})", i, i * 10)).unwrap();
    }

    // WHERE with ORDER BY
    let result = engine.execute("SELECT * FROM test WHERE id > 5 ORDER BY id DESC").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 5, "Should have 5 rows (6-10)");

            let ids: Vec<i64> = data.iter()
                .map(|row| {
                    if let Value::Int64(id) = row.get(0).unwrap() {
                        *id
                    } else {
                        panic!("Expected Int64")
                    }
                })
                .collect();

            assert_eq!(ids, vec![10, 9, 8, 7, 6]);
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_limit() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE test (id BIGINT PRIMARY KEY, value BIGINT)").unwrap();

    for i in 1..=10 {
        engine.execute(&format!("INSERT INTO test VALUES ({}, {})", i, i * 10)).unwrap();
    }

    // LIMIT
    let result = engine.execute("SELECT * FROM test LIMIT 3").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 3, "Should return exactly 3 rows");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_order_by_limit() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE test (id BIGINT PRIMARY KEY, value BIGINT)").unwrap();

    // Insert 10 rows
    for i in 1..=10 {
        engine.execute(&format!("INSERT INTO test VALUES ({}, {})", i, i * 10)).unwrap();
    }

    // Get top 3 by id DESC
    let result = engine.execute("SELECT * FROM test ORDER BY id DESC LIMIT 3").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 3, "Should return exactly 3 rows");

            let ids: Vec<i64> = data.iter()
                .map(|row| {
                    if let Value::Int64(id) = row.get(0).unwrap() {
                        *id
                    } else {
                        panic!("Expected Int64")
                    }
                })
                .collect();

            assert_eq!(ids, vec![10, 9, 8], "Should be top 3 in descending order");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_offset() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE test (id BIGINT PRIMARY KEY, value BIGINT)").unwrap();

    for i in 1..=10 {
        engine.execute(&format!("INSERT INTO test VALUES ({}, {})", i, i * 10)).unwrap();
    }

    // OFFSET 5
    let result = engine.execute("SELECT * FROM test OFFSET 5").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 5, "Should return 5 rows (after skipping 5)");

            let first_id = if let Value::Int64(id) = data[0].get(0).unwrap() {
                *id
            } else {
                panic!("Expected Int64")
            };

            assert_eq!(first_id, 6, "First row after OFFSET 5 should be id=6");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_limit_offset() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE test (id BIGINT PRIMARY KEY, value BIGINT)").unwrap();

    for i in 1..=10 {
        engine.execute(&format!("INSERT INTO test VALUES ({}, {})", i, i * 10)).unwrap();
    }

    // Pagination: LIMIT 3 OFFSET 3
    let result = engine.execute("SELECT * FROM test LIMIT 3 OFFSET 3").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 3, "Should return exactly 3 rows");

            let ids: Vec<i64> = data.iter()
                .map(|row| {
                    if let Value::Int64(id) = row.get(0).unwrap() {
                        *id
                    } else {
                        panic!("Expected Int64")
                    }
                })
                .collect();

            assert_eq!(ids, vec![4, 5, 6], "Should be rows 4-6 (offset 3, limit 3)");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_order_by_value_column() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE test (id BIGINT PRIMARY KEY, score BIGINT)").unwrap();

    // Insert with varying scores
    engine.execute("INSERT INTO test VALUES (1, 100), (2, 50), (3, 200), (4, 75)").unwrap();

    // ORDER BY score (not primary key)
    let result = engine.execute("SELECT * FROM test ORDER BY score ASC").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 4);

            let scores: Vec<i64> = data.iter()
                .map(|row| {
                    if let Value::Int64(score) = row.get(1).unwrap() {
                        *score
                    } else {
                        panic!("Expected Int64")
                    }
                })
                .collect();

            assert_eq!(scores, vec![50, 75, 100, 200], "Scores should be in ascending order");
        }
        _ => panic!("Expected Selected result"),
    }
}