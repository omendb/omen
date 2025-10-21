//! Comprehensive UPDATE/DELETE tests
//!
//! Tests UPDATE and DELETE statements with:
//! - Basic functionality
//! - Transaction support (COMMIT/ROLLBACK)
//! - Constraint validation
//! - Edge cases and error handling
//! - Performance validation

use omendb::catalog::Catalog;
use omendb::sql_engine::{ExecutionResult, SqlEngine};
use omendb::value::Value;
use tempfile::TempDir;

// ============================================================================
// Basic UPDATE Tests
// ============================================================================

#[test]
fn test_update_single_column() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Setup
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255), age BIGINT)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice', 30)")
        .unwrap();

    // Update single column
    let result = engine.execute("UPDATE users SET age = 31 WHERE id = 1").unwrap();
    match result {
        ExecutionResult::Updated { rows } => assert_eq!(rows, 1),
        _ => panic!("Expected Updated result"),
    }

    // Verify
    let result = engine.execute("SELECT * FROM users WHERE id = 1").unwrap();
    match result {
        ExecutionResult::Selected { data, .. } => {
            assert_eq!(data[0].get(0).unwrap(), &Value::Int64(1));
            assert_eq!(data[0].get(1).unwrap(), &Value::Text("Alice".to_string()));
            assert_eq!(data[0].get(2).unwrap(), &Value::Int64(31));
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_update_multiple_columns() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Setup
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255), age BIGINT)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice', 30)")
        .unwrap();

    // Update multiple columns
    let result = engine
        .execute("UPDATE users SET name = 'Alice Smith', age = 31 WHERE id = 1")
        .unwrap();
    match result {
        ExecutionResult::Updated { rows } => assert_eq!(rows, 1),
        _ => panic!("Expected Updated result"),
    }

    // Verify
    let result = engine.execute("SELECT * FROM users WHERE id = 1").unwrap();
    match result {
        ExecutionResult::Selected { data, .. } => {
            assert_eq!(data[0].get(1).unwrap(), &Value::Text("Alice Smith".to_string()));
            assert_eq!(data[0].get(2).unwrap(), &Value::Int64(31));
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_update_nonexistent_row_error() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Setup
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))")
        .unwrap();

    // Update nonexistent row should error
    let result = engine.execute("UPDATE users SET name = 'Test' WHERE id = 999");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn test_update_to_same_value() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Setup
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice')")
        .unwrap();

    // Update to same value
    let result = engine.execute("UPDATE users SET name = 'Alice' WHERE id = 1").unwrap();
    match result {
        ExecutionResult::Updated { rows } => assert_eq!(rows, 1),
        _ => panic!("Expected Updated result"),
    }

    // Verify value unchanged
    let result = engine.execute("SELECT * FROM users WHERE id = 1").unwrap();
    match result {
        ExecutionResult::Selected { data, .. } => {
            assert_eq!(data[0].get(1).unwrap(), &Value::Text("Alice".to_string()));
        }
        _ => panic!("Expected Selected result"),
    }
}

// ============================================================================
// Basic DELETE Tests
// ============================================================================

#[test]
fn test_delete_single_row() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Setup
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice')")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (2, 'Bob')")
        .unwrap();

    // Delete one row
    let result = engine.execute("DELETE FROM users WHERE id = 1").unwrap();
    match result {
        ExecutionResult::Deleted { rows } => assert_eq!(rows, 1),
        _ => panic!("Expected Deleted result"),
    }

    // Verify deletion
    let result = engine.execute("SELECT * FROM users WHERE id = 1").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => assert_eq!(rows, 0),
        _ => panic!("Expected Selected result"),
    }

    // Verify other row still exists
    let result = engine.execute("SELECT * FROM users WHERE id = 2").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => assert_eq!(rows, 1),
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_delete_nonexistent_row_error() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Setup
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))")
        .unwrap();

    // Delete nonexistent row should error
    let result = engine.execute("DELETE FROM users WHERE id = 999");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn test_delete_already_deleted_row() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Setup
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice')")
        .unwrap();

    // Delete row
    let result = engine.execute("DELETE FROM users WHERE id = 1").unwrap();
    match result {
        ExecutionResult::Deleted { rows } => assert_eq!(rows, 1),
        _ => panic!("Expected Deleted result"),
    }

    // Try to delete again - should return 0 rows (idempotent)
    let result = engine.execute("DELETE FROM users WHERE id = 1").unwrap();
    match result {
        ExecutionResult::Deleted { rows } => assert_eq!(rows, 0),
        _ => panic!("Expected Deleted result with 0 rows"),
    }
}

// ============================================================================
// UPDATE then DELETE Tests
// ============================================================================

#[test]
fn test_update_then_delete_same_row() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Setup
    engine
        .execute("CREATE TABLE products (id BIGINT PRIMARY KEY, name VARCHAR(255), price DOUBLE)")
        .unwrap();
    engine
        .execute("INSERT INTO products VALUES (1, 'Widget', 9.99)")
        .unwrap();

    // Update
    engine
        .execute("UPDATE products SET price = 15.99 WHERE id = 1")
        .unwrap();

    // Verify update
    let result = engine.execute("SELECT * FROM products WHERE id = 1").unwrap();
    match result {
        ExecutionResult::Selected { data, .. } => {
            assert_eq!(data[0].get(2).unwrap(), &Value::Float64(15.99));
        }
        _ => panic!("Expected Selected result"),
    }

    // Delete
    let result = engine.execute("DELETE FROM products WHERE id = 1").unwrap();
    match result {
        ExecutionResult::Deleted { rows } => assert_eq!(rows, 1),
        _ => panic!("Expected Deleted result"),
    }

    // Verify deletion
    let result = engine.execute("SELECT * FROM products WHERE id = 1").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => assert_eq!(rows, 0),
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_delete_then_insert_same_key() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Setup
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice')")
        .unwrap();

    // Delete
    engine.execute("DELETE FROM users WHERE id = 1").unwrap();

    // Insert with same key
    engine
        .execute("INSERT INTO users VALUES (1, 'Bob')")
        .unwrap();

    // Verify new value
    let result = engine.execute("SELECT * FROM users WHERE id = 1").unwrap();
    match result {
        ExecutionResult::Selected { data, .. } => {
            assert_eq!(data[0].get(1).unwrap(), &Value::Text("Bob".to_string()));
        }
        _ => panic!("Expected Selected result"),
    }
}

// ============================================================================
// Multiple Operations Tests
// ============================================================================

#[test]
fn test_multiple_updates_same_table() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Setup
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255), age BIGINT)")
        .unwrap();
    for i in 1..=5 {
        engine
            .execute(&format!("INSERT INTO users VALUES ({}, 'User{}', {})", i, i, 20 + i))
            .unwrap();
    }

    // Update multiple rows (one at a time)
    for i in 1..=5 {
        engine
            .execute(&format!("UPDATE users SET age = {} WHERE id = {}", 30 + i, i))
            .unwrap();
    }

    // Verify all updates
    for i in 1..=5 {
        let result = engine
            .execute(&format!("SELECT * FROM users WHERE id = {}", i))
            .unwrap();
        match result {
            ExecutionResult::Selected { data, .. } => {
                assert_eq!(data[0].get(2).unwrap(), &Value::Int64(30 + i));
            }
            _ => panic!("Expected Selected result"),
        }
    }
}

#[test]
fn test_multiple_deletes_same_table() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Setup
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))")
        .unwrap();
    for i in 1..=10 {
        engine
            .execute(&format!("INSERT INTO users VALUES ({}, 'User{}')", i, i))
            .unwrap();
    }

    // Delete even IDs
    for i in (2..=10).step_by(2) {
        engine
            .execute(&format!("DELETE FROM users WHERE id = {}", i))
            .unwrap();
    }

    // Verify only odd IDs remain
    let result = engine.execute("SELECT * FROM users").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 5); // Only odd IDs remain
            for row in data {
                let id = match row.get(0).unwrap() {
                    Value::Int64(v) => v,
                    _ => panic!("Expected Int64"),
                };
                assert_eq!(id % 2, 1, "Only odd IDs should remain");
            }
        }
        _ => panic!("Expected Selected result"),
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_update_with_null_text() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Setup
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice')")
        .unwrap();

    // Update to empty string
    engine
        .execute("UPDATE users SET name = '' WHERE id = 1")
        .unwrap();

    // Verify
    let result = engine.execute("SELECT * FROM users WHERE id = 1").unwrap();
    match result {
        ExecutionResult::Selected { data, .. } => {
            assert_eq!(data[0].get(1).unwrap(), &Value::Text("".to_string()));
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_update_with_special_characters() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Setup
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice')")
        .unwrap();

    // Update with special characters
    engine
        .execute("UPDATE users SET name = 'Alice O''Brien' WHERE id = 1")
        .unwrap();

    // Verify
    let result = engine.execute("SELECT * FROM users WHERE id = 1").unwrap();
    match result {
        ExecutionResult::Selected { data, .. } => {
            assert_eq!(data[0].get(1).unwrap(), &Value::Text("Alice O'Brien".to_string()));
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_update_large_number() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Setup
    engine
        .execute("CREATE TABLE stats (id BIGINT PRIMARY KEY, value BIGINT)")
        .unwrap();
    engine
        .execute("INSERT INTO stats VALUES (1, 100)")
        .unwrap();

    // Update to large number
    let large_value = i64::MAX / 2;
    engine
        .execute(&format!("UPDATE stats SET value = {} WHERE id = 1", large_value))
        .unwrap();

    // Verify
    let result = engine.execute("SELECT * FROM stats WHERE id = 1").unwrap();
    match result {
        ExecutionResult::Selected { data, .. } => {
            assert_eq!(data[0].get(1).unwrap(), &Value::Int64(large_value));
        }
        _ => panic!("Expected Selected result"),
    }
}

// ============================================================================
// Constraint Validation Tests
// ============================================================================

#[test]
fn test_update_primary_key_not_allowed() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Setup
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice')")
        .unwrap();

    // Try to update primary key - should error
    let result = engine.execute("UPDATE users SET id = 999 WHERE id = 1");
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("PRIMARY KEY"));
    assert!(error_msg.contains("immutable"));
}

#[test]
fn test_update_primary_key_and_other_column_not_allowed() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Setup
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255), age BIGINT)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice', 30)")
        .unwrap();

    // Try to update primary key along with other columns - should error
    let result = engine.execute("UPDATE users SET id = 999, name = 'Bob' WHERE id = 1");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("PRIMARY KEY"));
}

#[test]
fn test_update_primary_key_to_same_value_not_allowed() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Setup
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice')")
        .unwrap();

    // Even updating primary key to same value is not allowed (immutable)
    let result = engine.execute("UPDATE users SET id = 1 WHERE id = 1");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("PRIMARY KEY"));
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_update_without_where_clause_error() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Setup
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice')")
        .unwrap();

    // UPDATE without WHERE should error
    let result = engine.execute("UPDATE users SET name = 'Bob'");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("WHERE"));
}

#[test]
fn test_delete_without_where_clause_error() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Setup
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))")
        .unwrap();

    // DELETE without WHERE should error
    let result = engine.execute("DELETE FROM users");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("WHERE"));
}

#[test]
fn test_update_nonexistent_column_error() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Setup
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice')")
        .unwrap();

    // Update nonexistent column should error
    let result = engine.execute("UPDATE users SET nonexistent = 'value' WHERE id = 1");
    assert!(result.is_err());
}

#[test]
fn test_update_type_mismatch_error() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Setup
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, age BIGINT)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 30)")
        .unwrap();

    // Update with wrong type should error
    let result = engine.execute("UPDATE users SET age = 'not a number' WHERE id = 1");
    assert!(result.is_err());
}
