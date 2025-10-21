use omendb::catalog::Catalog;
use omendb::sql_engine::{ExecutionResult, SqlEngine};
use omendb::value::Value;
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
