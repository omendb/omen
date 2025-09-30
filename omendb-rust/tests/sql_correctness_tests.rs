//! SQL Correctness Tests - Verify SQL operations return exact correct results
//! Compare query results against expected values to ensure correctness

use omendb::catalog::Catalog;
use omendb::sql_engine::{SqlEngine, ExecutionResult};
use omendb::value::Value;
use tempfile::TempDir;

#[test]
fn test_insert_correctness() {
    // Verify INSERT actually stores the exact values we give it
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE data (id BIGINT PRIMARY KEY, value DOUBLE)").unwrap();
    engine.execute("INSERT INTO data VALUES (100, 3.14159)").unwrap();

    // Query it back and verify exact values
    let result = engine.execute("SELECT * FROM data WHERE id = 100").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 1, "Should return exactly 1 row");
            assert_eq!(data.len(), 1, "Data should have 1 row");

            let row = &data[0];
            assert_eq!(row.get(0).unwrap(), &Value::Int64(100), "ID should be exactly 100");
            assert_eq!(row.get(1).unwrap(), &Value::Float64(3.14159), "Value should be exactly 3.14159");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_select_where_equals_correctness() {
    // Verify WHERE id = X returns only rows with id = X, nothing else
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE test (id BIGINT PRIMARY KEY, name VARCHAR(50))").unwrap();

    // Insert test data
    for i in 0..20 {
        engine.execute(&format!("INSERT INTO test VALUES ({}, 'name_{}')", i, i)).unwrap();
    }

    // Test WHERE id = 10 (should return only row with id=10)
    let result = engine.execute("SELECT * FROM test WHERE id = 10").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 1, "WHERE id = 10 should return exactly 1 row");

            let row = &data[0];
            assert_eq!(row.get(0).unwrap(), &Value::Int64(10), "Row should have id=10");
            assert_eq!(row.get(1).unwrap(), &Value::Text("name_10".to_string()), "Row should have name_10");
        }
        _ => panic!("Expected Selected result"),
    }

    // Test WHERE id = 999 (should return 0 rows)
    let result = engine.execute("SELECT * FROM test WHERE id = 999").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 0, "WHERE id = 999 should return 0 rows (non-existent key)");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_range_query_correctness() {
    // Verify range queries return ALL and ONLY rows in the range
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE range_test (id BIGINT PRIMARY KEY, value BIGINT)").unwrap();

    // Insert 0..100
    for i in 0..100 {
        engine.execute(&format!("INSERT INTO range_test VALUES ({}, {})", i, i * 10)).unwrap();
    }

    // Test: WHERE id > 50 AND id < 60 should return [51, 52, ..., 59]
    let result = engine.execute("SELECT * FROM range_test WHERE id > 50 AND id < 60").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 9, "Should return exactly 9 rows (51-59)");

            // Verify each returned row is in correct range
            let ids: Vec<i64> = data.iter()
                .map(|row| {
                    if let Value::Int64(id) = row.get(0).unwrap() {
                        *id
                    } else {
                        panic!("Expected Int64")
                    }
                })
                .collect();

            // Should be [51, 52, 53, 54, 55, 56, 57, 58, 59]
            let expected: Vec<i64> = (51..60).collect();
            assert_eq!(ids, expected, "Should return exactly ids 51-59 in order");

            // Verify no id <= 50 or id >= 60
            for id in ids {
                assert!(id > 50 && id < 60, "All returned ids should be in range (50, 60)");
            }
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_range_query_inclusive_correctness() {
    // Test >= and <= operators
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE incl_test (id BIGINT PRIMARY KEY, value BIGINT)").unwrap();

    for i in 0..20 {
        engine.execute(&format!("INSERT INTO incl_test VALUES ({}, {})", i, i)).unwrap();
    }

    // WHERE id >= 10 AND id <= 15 should include both 10 and 15
    let result = engine.execute("SELECT * FROM incl_test WHERE id >= 10 AND id <= 15").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 6, "Should return exactly 6 rows [10, 11, 12, 13, 14, 15]");

            let ids: Vec<i64> = data.iter()
                .map(|row| {
                    if let Value::Int64(id) = row.get(0).unwrap() {
                        *id
                    } else {
                        panic!("Expected Int64")
                    }
                })
                .collect();

            assert_eq!(ids, vec![10, 11, 12, 13, 14, 15], "Should include boundaries");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_data_types_correctness() {
    // Verify each data type stores and retrieves correctly
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE types (
        id BIGINT PRIMARY KEY,
        int_val BIGINT,
        float_val DOUBLE,
        text_val VARCHAR(100),
        bool_val BOOLEAN
    )").unwrap();

    // Insert specific values
    engine.execute("INSERT INTO types VALUES (
        1,
        42,
        2.71828,
        'hello world',
        true
    )").unwrap();

    // Retrieve and verify exact values
    let result = engine.execute("SELECT * FROM types WHERE id = 1").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, columns } => {
            assert_eq!(rows, 1);
            assert_eq!(columns.len(), 5);

            let row = &data[0];
            assert_eq!(row.get(0).unwrap(), &Value::Int64(1), "id should be 1");
            assert_eq!(row.get(1).unwrap(), &Value::Int64(42), "int_val should be 42");
            assert_eq!(row.get(2).unwrap(), &Value::Float64(2.71828), "float_val should be 2.71828");
            assert_eq!(row.get(3).unwrap(), &Value::Text("hello world".to_string()), "text_val should be 'hello world'");
            assert_eq!(row.get(4).unwrap(), &Value::Boolean(true), "bool_val should be true");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_multi_row_insert_correctness() {
    // Verify INSERT with multiple VALUES works correctly
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE multi (id BIGINT PRIMARY KEY, value BIGINT)").unwrap();

    // Insert 5 rows in one statement
    let result = engine.execute("INSERT INTO multi VALUES
        (1, 10),
        (2, 20),
        (3, 30),
        (4, 40),
        (5, 50)
    ").unwrap();

    match result {
        ExecutionResult::Inserted { rows } => {
            assert_eq!(rows, 5, "Should report 5 rows inserted");
        }
        _ => panic!("Expected Inserted result"),
    }

    // Verify all 5 rows are actually there
    let result = engine.execute("SELECT * FROM multi").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 5, "Should have 5 rows");
            assert_eq!(data.len(), 5, "Data should have 5 rows");

            // Verify each row has correct values
            for (i, row) in data.iter().enumerate() {
                let expected_id = (i + 1) as i64;
                let expected_value = expected_id * 10;

                assert_eq!(row.get(0).unwrap(), &Value::Int64(expected_id), "Row {} id incorrect", i);
                assert_eq!(row.get(1).unwrap(), &Value::Int64(expected_value), "Row {} value incorrect", i);
            }
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_full_table_scan_correctness() {
    // Verify SELECT * returns ALL rows (no WHERE clause)
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE scan_test (id BIGINT PRIMARY KEY, value BIGINT)").unwrap();

    // Insert exactly 50 rows
    for i in 0..50 {
        engine.execute(&format!("INSERT INTO scan_test VALUES ({}, {})", i, i * 2)).unwrap();
    }

    // Full scan
    let result = engine.execute("SELECT * FROM scan_test").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 50, "Full scan should return all 50 rows");
            assert_eq!(data.len(), 50, "Data should have all 50 rows");

            // Verify rows are in order (sorted by primary key)
            for (i, row) in data.iter().enumerate() {
                assert_eq!(row.get(0).unwrap(), &Value::Int64(i as i64), "Row {} should have id={}", i, i);
            }
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_where_no_matches_correctness() {
    // Verify WHERE that matches nothing returns empty result (not an error)
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE nomatch (id BIGINT PRIMARY KEY, value BIGINT)").unwrap();

    for i in 0..10 {
        engine.execute(&format!("INSERT INTO nomatch VALUES ({}, {})", i, i)).unwrap();
    }

    // Query for non-existent range
    let result = engine.execute("SELECT * FROM nomatch WHERE id > 100 AND id < 200").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 0, "Should return 0 rows");
            assert_eq!(data.len(), 0, "Data should be empty");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_where_all_matches_correctness() {
    // Verify WHERE that matches everything returns all rows
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE allmatch (id BIGINT PRIMARY KEY, value BIGINT)").unwrap();

    for i in 0..10 {
        engine.execute(&format!("INSERT INTO allmatch VALUES ({}, {})", i, i)).unwrap();
    }

    // Query that includes all rows
    let result = engine.execute("SELECT * FROM allmatch WHERE id >= 0 AND id < 100").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 10, "Should return all 10 rows");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_string_correctness() {
    // Verify string values stored and retrieved exactly (with special characters)
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE strings (id BIGINT PRIMARY KEY, text VARCHAR(255))").unwrap();

    // Test various string values
    let test_strings = vec![
        (1, "simple"),
        (2, "with spaces"),
        (3, "with-dashes"),
        (4, "with_underscores"),
        (5, "with.dots"),
        (6, "numbers123"),
        (7, ""),  // empty string
    ];

    for (id, text) in &test_strings {
        engine.execute(&format!("INSERT INTO strings VALUES ({}, '{}')", id, text)).unwrap();
    }

    // Verify each string is stored exactly
    for (id, expected_text) in test_strings {
        let result = engine.execute(&format!("SELECT * FROM strings WHERE id = {}", id)).unwrap();
        match result {
            ExecutionResult::Selected { rows, data, .. } => {
                assert_eq!(rows, 1, "Should find string with id={}", id);
                assert_eq!(
                    data[0].get(1).unwrap(),
                    &Value::Text(expected_text.to_string()),
                    "String for id={} should be exactly '{}'",
                    id,
                    expected_text
                );
            }
            _ => panic!("Expected Selected result"),
        }
    }
}

#[test]
fn test_zero_values_correctness() {
    // Verify zero values (0, 0.0, false) work correctly
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE zeros (
        id BIGINT PRIMARY KEY,
        zero_int BIGINT,
        zero_float DOUBLE,
        zero_bool BOOLEAN
    )").unwrap();

    engine.execute("INSERT INTO zeros VALUES (1, 0, 0.0, false)").unwrap();

    let result = engine.execute("SELECT * FROM zeros WHERE id = 1").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 1);

            let row = &data[0];
            assert_eq!(row.get(1).unwrap(), &Value::Int64(0), "zero_int should be 0");
            assert_eq!(row.get(2).unwrap(), &Value::Float64(0.0), "zero_float should be 0.0");
            assert_eq!(row.get(3).unwrap(), &Value::Boolean(false), "zero_bool should be false");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_column_order_correctness() {
    // Verify columns are returned in schema order
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE order_test (
        col_a BIGINT PRIMARY KEY,
        col_b VARCHAR(50),
        col_c DOUBLE,
        col_d BOOLEAN
    )").unwrap();

    engine.execute("INSERT INTO order_test VALUES (1, 'text', 3.14, true)").unwrap();

    let result = engine.execute("SELECT * FROM order_test WHERE col_a = 1").unwrap();
    match result {
        ExecutionResult::Selected { columns, data, .. } => {
            // Verify column names in correct order
            assert_eq!(columns, vec!["col_a", "col_b", "col_c", "col_d"]);

            // Verify values in correct order
            let row = &data[0];
            assert_eq!(row.get(0).unwrap(), &Value::Int64(1));
            assert_eq!(row.get(1).unwrap(), &Value::Text("text".to_string()));
            assert_eq!(row.get(2).unwrap(), &Value::Float64(3.14));
            assert_eq!(row.get(3).unwrap(), &Value::Boolean(true));
        }
        _ => panic!("Expected Selected result"),
    }
}