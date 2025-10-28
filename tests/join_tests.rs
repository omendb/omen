use omen::catalog::Catalog;
use omen::sql_engine::{ExecutionResult, SqlEngine};
use omen::value::Value;
use tempfile::TempDir;

#[test]
fn test_basic_inner_join() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Create users table
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255), age BIGINT)")
        .unwrap();

    // Create orders table
    engine
        .execute("CREATE TABLE orders (id BIGINT PRIMARY KEY, user_id BIGINT, total DOUBLE)")
        .unwrap();

    // Insert users
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice', 30)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (2, 'Bob', 25)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (3, 'Charlie', 35)")
        .unwrap();

    // Insert orders
    engine
        .execute("INSERT INTO orders VALUES (100, 1, 50.00)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (101, 1, 75.50)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (102, 2, 100.00)")
        .unwrap();

    // Execute INNER JOIN
    let result = engine
        .execute("SELECT * FROM users INNER JOIN orders ON users.id = orders.user_id")
        .unwrap();

    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 3, "Expected 3 joined rows");

            // Check first row: Alice's first order
            assert_eq!(
                data[0].get(0).unwrap(),
                &Value::Int64(1),
                "users.id should be 1"
            );
            assert_eq!(
                data[0].get(1).unwrap(),
                &Value::Text("Alice".to_string()),
                "users.name should be Alice"
            );
            assert_eq!(
                data[0].get(2).unwrap(),
                &Value::Int64(30),
                "users.age should be 30"
            );
            assert_eq!(
                data[0].get(3).unwrap(),
                &Value::Int64(100),
                "orders.id should be 100"
            );
            assert_eq!(
                data[0].get(4).unwrap(),
                &Value::Int64(1),
                "orders.user_id should be 1"
            );
            assert_eq!(
                data[0].get(5).unwrap(),
                &Value::Float64(50.00),
                "orders.total should be 50.00"
            );

            // Check second row: Alice's second order
            assert_eq!(
                data[1].get(0).unwrap(),
                &Value::Int64(1),
                "users.id should be 1"
            );
            assert_eq!(
                data[1].get(3).unwrap(),
                &Value::Int64(101),
                "orders.id should be 101"
            );

            // Check third row: Bob's order
            assert_eq!(
                data[2].get(0).unwrap(),
                &Value::Int64(2),
                "users.id should be 2"
            );
            assert_eq!(
                data[2].get(1).unwrap(),
                &Value::Text("Bob".to_string()),
                "users.name should be Bob"
            );
            assert_eq!(
                data[2].get(3).unwrap(),
                &Value::Int64(102),
                "orders.id should be 102"
            );
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_inner_join_with_column_projection() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Create tables
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255), age BIGINT)")
        .unwrap();
    engine
        .execute("CREATE TABLE orders (id BIGINT PRIMARY KEY, user_id BIGINT, total DOUBLE)")
        .unwrap();

    // Insert data
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice', 30)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (100, 1, 50.00)")
        .unwrap();

    // Execute INNER JOIN with specific columns
    let result = engine
        .execute("SELECT users.name, orders.total FROM users INNER JOIN orders ON users.id = orders.user_id")
        .unwrap();

    match result {
        ExecutionResult::Selected {
            rows,
            data,
            columns,
        } => {
            assert_eq!(rows, 1);
            assert_eq!(columns, vec!["name", "total"]);

            assert_eq!(data[0].get(0).unwrap(), &Value::Text("Alice".to_string()));
            assert_eq!(data[0].get(1).unwrap(), &Value::Float64(50.00));
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_inner_join_no_matches() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Create tables
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255), age BIGINT)")
        .unwrap();
    engine
        .execute("CREATE TABLE orders (id BIGINT PRIMARY KEY, user_id BIGINT, total DOUBLE)")
        .unwrap();

    // Insert users
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice', 30)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (2, 'Bob', 25)")
        .unwrap();

    // Insert orders with non-matching user_ids
    engine
        .execute("INSERT INTO orders VALUES (100, 99, 50.00)")
        .unwrap();

    // Execute INNER JOIN - should return 0 rows
    let result = engine
        .execute("SELECT * FROM users INNER JOIN orders ON users.id = orders.user_id")
        .unwrap();

    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 0, "Expected 0 rows when no matches");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_left_join_basic() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Create tables
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255), age BIGINT)")
        .unwrap();
    engine
        .execute("CREATE TABLE orders (id BIGINT PRIMARY KEY, user_id BIGINT, total DOUBLE)")
        .unwrap();

    // Insert users
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice', 30)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (2, 'Bob', 25)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (3, 'Charlie', 35)")
        .unwrap();

    // Insert orders (only for Alice and Bob)
    engine
        .execute("INSERT INTO orders VALUES (100, 1, 50.00)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (101, 2, 100.00)")
        .unwrap();

    // Execute LEFT JOIN - should return 3 rows (all users, Charlie with NULLs)
    let result = engine
        .execute("SELECT * FROM users LEFT JOIN orders ON users.id = orders.user_id")
        .unwrap();

    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 3, "Expected 3 rows (all users)");

            // Check Alice (has order)
            assert_eq!(data[0].get(0).unwrap(), &Value::Int64(1));
            assert_eq!(
                data[0].get(1).unwrap(),
                &Value::Text("Alice".to_string())
            );
            assert_eq!(data[0].get(3).unwrap(), &Value::Int64(100));

            // Check Bob (has order)
            assert_eq!(data[1].get(0).unwrap(), &Value::Int64(2));
            assert_eq!(data[1].get(1).unwrap(), &Value::Text("Bob".to_string()));
            assert_eq!(data[1].get(3).unwrap(), &Value::Int64(101));

            // Check Charlie (no order - NULLs for order columns)
            assert_eq!(data[2].get(0).unwrap(), &Value::Int64(3));
            assert_eq!(
                data[2].get(1).unwrap(),
                &Value::Text("Charlie".to_string())
            );
            assert_eq!(data[2].get(3).unwrap(), &Value::Null, "orders.id should be NULL");
            assert_eq!(data[2].get(4).unwrap(), &Value::Null, "orders.user_id should be NULL");
            assert_eq!(data[2].get(5).unwrap(), &Value::Null, "orders.total should be NULL");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_inner_join_with_where_clause() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Create tables
    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255), age BIGINT)")
        .unwrap();
    engine
        .execute("CREATE TABLE orders (id BIGINT PRIMARY KEY, user_id BIGINT, total DOUBLE)")
        .unwrap();

    // Insert data
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice', 30)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (2, 'Bob', 25)")
        .unwrap();

    engine
        .execute("INSERT INTO orders VALUES (100, 1, 50.00)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (101, 1, 150.00)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (102, 2, 75.00)")
        .unwrap();

    // Execute INNER JOIN with WHERE clause
    let result = engine
        .execute(
            "SELECT users.name, orders.total \
             FROM users INNER JOIN orders ON users.id = orders.user_id \
             WHERE orders.total > 80.0",
        )
        .unwrap();

    match result {
        ExecutionResult::Selected {
            rows,
            data,
            columns,
        } => {
            assert_eq!(rows, 1, "Expected 1 row (only Alice's 150.00 order)");
            assert_eq!(columns, vec!["name", "total"]);

            assert_eq!(data[0].get(0).unwrap(), &Value::Text("Alice".to_string()));
            assert_eq!(data[0].get(1).unwrap(), &Value::Float64(150.00));
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_inner_join_one_to_many() {
    // One user with multiple orders
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255), age BIGINT)")
        .unwrap();
    engine
        .execute("CREATE TABLE orders (id BIGINT PRIMARY KEY, user_id BIGINT, total DOUBLE)")
        .unwrap();

    engine
        .execute("INSERT INTO users VALUES (1, 'Alice', 30)")
        .unwrap();

    // Alice has 3 orders
    engine
        .execute("INSERT INTO orders VALUES (100, 1, 50.00)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (101, 1, 75.00)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (102, 1, 100.00)")
        .unwrap();

    let result = engine
        .execute("SELECT users.name, orders.total FROM users INNER JOIN orders ON users.id = orders.user_id")
        .unwrap();

    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 3, "Expected 3 rows (one user, three orders)");

            // All rows should have Alice
            for i in 0..3 {
                assert_eq!(data[i].get(0).unwrap(), &Value::Text("Alice".to_string()));
            }

            // Check order totals
            assert_eq!(data[0].get(1).unwrap(), &Value::Float64(50.00));
            assert_eq!(data[1].get(1).unwrap(), &Value::Float64(75.00));
            assert_eq!(data[2].get(1).unwrap(), &Value::Float64(100.00));
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_inner_join_all_rows_match() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255), age BIGINT)")
        .unwrap();
    engine
        .execute("CREATE TABLE orders (id BIGINT PRIMARY KEY, user_id BIGINT, total DOUBLE)")
        .unwrap();

    // Every user has exactly one order
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice', 30)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (2, 'Bob', 25)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (3, 'Charlie', 35)")
        .unwrap();

    engine
        .execute("INSERT INTO orders VALUES (100, 1, 50.00)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (101, 2, 75.00)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (102, 3, 100.00)")
        .unwrap();

    let result = engine
        .execute("SELECT * FROM users INNER JOIN orders ON users.id = orders.user_id")
        .unwrap();

    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 3, "Expected 3 rows (all match)");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_left_join_all_rows_match() {
    // LEFT JOIN where all left rows have matches - same as INNER JOIN
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255), age BIGINT)")
        .unwrap();
    engine
        .execute("CREATE TABLE orders (id BIGINT PRIMARY KEY, user_id BIGINT, total DOUBLE)")
        .unwrap();

    engine
        .execute("INSERT INTO users VALUES (1, 'Alice', 30)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (2, 'Bob', 25)")
        .unwrap();

    engine
        .execute("INSERT INTO orders VALUES (100, 1, 50.00)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (101, 2, 75.00)")
        .unwrap();

    let result = engine
        .execute("SELECT users.name, orders.total FROM users LEFT JOIN orders ON users.id = orders.user_id")
        .unwrap();

    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 2, "Expected 2 rows");

            // No NULLs - all rows matched
            for i in 0..2 {
                assert!(!matches!(data[i].get(1).unwrap(), Value::Null));
            }
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_left_join_no_matches() {
    // LEFT JOIN where no left rows have matches - all right columns NULL
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255), age BIGINT)")
        .unwrap();
    engine
        .execute("CREATE TABLE orders (id BIGINT PRIMARY KEY, user_id BIGINT, total DOUBLE)")
        .unwrap();

    engine
        .execute("INSERT INTO users VALUES (1, 'Alice', 30)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (2, 'Bob', 25)")
        .unwrap();

    // No orders
    let result = engine
        .execute("SELECT users.name, orders.total FROM users LEFT JOIN orders ON users.id = orders.user_id")
        .unwrap();

    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 2, "Expected 2 rows (all users, no orders)");

            // All order columns should be NULL
            assert_eq!(data[0].get(0).unwrap(), &Value::Text("Alice".to_string()));
            assert_eq!(data[0].get(1).unwrap(), &Value::Null);

            assert_eq!(data[1].get(0).unwrap(), &Value::Text("Bob".to_string()));
            assert_eq!(data[1].get(1).unwrap(), &Value::Null);
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_inner_join_empty_left_table() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255), age BIGINT)")
        .unwrap();
    engine
        .execute("CREATE TABLE orders (id BIGINT PRIMARY KEY, user_id BIGINT, total DOUBLE)")
        .unwrap();

    // No users, but has orders
    engine
        .execute("INSERT INTO orders VALUES (100, 1, 50.00)")
        .unwrap();

    let result = engine
        .execute("SELECT * FROM users INNER JOIN orders ON users.id = orders.user_id")
        .unwrap();

    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 0, "Expected 0 rows (empty left table)");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_inner_join_empty_right_table() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255), age BIGINT)")
        .unwrap();
    engine
        .execute("CREATE TABLE orders (id BIGINT PRIMARY KEY, user_id BIGINT, total DOUBLE)")
        .unwrap();

    engine
        .execute("INSERT INTO users VALUES (1, 'Alice', 30)")
        .unwrap();

    // No orders
    let result = engine
        .execute("SELECT * FROM users INNER JOIN orders ON users.id = orders.user_id")
        .unwrap();

    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 0, "Expected 0 rows (empty right table)");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_left_join_empty_left_table() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255), age BIGINT)")
        .unwrap();
    engine
        .execute("CREATE TABLE orders (id BIGINT PRIMARY KEY, user_id BIGINT, total DOUBLE)")
        .unwrap();

    // No users
    engine
        .execute("INSERT INTO orders VALUES (100, 1, 50.00)")
        .unwrap();

    let result = engine
        .execute("SELECT * FROM users LEFT JOIN orders ON users.id = orders.user_id")
        .unwrap();

    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 0, "Expected 0 rows (empty left table)");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_left_join_one_to_many() {
    // One user with multiple orders, one user with no orders
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255), age BIGINT)")
        .unwrap();
    engine
        .execute("CREATE TABLE orders (id BIGINT PRIMARY KEY, user_id BIGINT, total DOUBLE)")
        .unwrap();

    engine
        .execute("INSERT INTO users VALUES (1, 'Alice', 30)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (2, 'Bob', 25)")
        .unwrap();

    // Alice has 2 orders, Bob has none
    engine
        .execute("INSERT INTO orders VALUES (100, 1, 50.00)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (101, 1, 75.00)")
        .unwrap();

    let result = engine
        .execute("SELECT users.name, orders.total FROM users LEFT JOIN orders ON users.id = orders.user_id")
        .unwrap();

    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 3, "Expected 3 rows (Alice twice, Bob once)");

            // Alice's orders
            assert_eq!(data[0].get(0).unwrap(), &Value::Text("Alice".to_string()));
            assert_eq!(data[0].get(1).unwrap(), &Value::Float64(50.00));

            assert_eq!(data[1].get(0).unwrap(), &Value::Text("Alice".to_string()));
            assert_eq!(data[1].get(1).unwrap(), &Value::Float64(75.00));

            // Bob with NULL
            assert_eq!(data[2].get(0).unwrap(), &Value::Text("Bob".to_string()));
            assert_eq!(data[2].get(1).unwrap(), &Value::Null);
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_inner_join_non_primary_key_column() {
    // Join on a non-primary key column
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine
        .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255), age BIGINT)")
        .unwrap();
    engine
        .execute("CREATE TABLE profiles (id BIGINT PRIMARY KEY, user_age BIGINT, bio VARCHAR(255))")
        .unwrap();

    engine
        .execute("INSERT INTO users VALUES (1, 'Alice', 30)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (2, 'Bob', 30)")
        .unwrap();

    engine
        .execute("INSERT INTO profiles VALUES (100, 30, 'Profile for age 30')")
        .unwrap();

    // Join on age (non-primary key)
    let result = engine
        .execute("SELECT users.name, profiles.bio FROM users INNER JOIN profiles ON users.age = profiles.user_age")
        .unwrap();

    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 2, "Expected 2 rows (both Alice and Bob match age 30)");

            // Both should have the same bio
            assert_eq!(data[0].get(1).unwrap(), &Value::Text("Profile for age 30".to_string()));
            assert_eq!(data[1].get(1).unwrap(), &Value::Text("Profile for age 30".to_string()));
        }
        _ => panic!("Expected Selected result"),
    }
}
