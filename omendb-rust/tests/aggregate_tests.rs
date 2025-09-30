use omendb::catalog::Catalog;
use omendb::sql_engine::{SqlEngine, ExecutionResult};
use omendb::value::Value;
use tempfile::TempDir;

#[test]
fn test_count_star() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE products (id BIGINT PRIMARY KEY, name TEXT, price FLOAT64)").unwrap();
    engine.execute("INSERT INTO products VALUES (1, 'Widget', 9.99)").unwrap();
    engine.execute("INSERT INTO products VALUES (2, 'Gadget', 19.99)").unwrap();
    engine.execute("INSERT INTO products VALUES (3, 'Doohickey', 29.99)").unwrap();
    engine.execute("INSERT INTO products VALUES (4, 'Thingamajig', 39.99)").unwrap();
    engine.execute("INSERT INTO products VALUES (5, 'Whatsit', 49.99)").unwrap();

    let result = engine.execute("SELECT COUNT(*) FROM products").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 1);
            assert_eq!(data[0].get(0).unwrap(), &Value::Int64(5));
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_count_column() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE orders (id BIGINT PRIMARY KEY, amount FLOAT64, discount FLOAT64)").unwrap();
    engine.execute("INSERT INTO orders VALUES (1, 100.0, 10.0)").unwrap();
    engine.execute("INSERT INTO orders VALUES (2, 200.0, NULL)").unwrap();
    engine.execute("INSERT INTO orders VALUES (3, 300.0, 20.0)").unwrap();
    engine.execute("INSERT INTO orders VALUES (4, 400.0, NULL)").unwrap();

    // COUNT(*) should count all rows
    let result = engine.execute("SELECT COUNT(*) FROM orders").unwrap();
    match result {
        ExecutionResult::Selected { data, .. } => {
            assert_eq!(data[0].get(0).unwrap(), &Value::Int64(4));
        }
        _ => panic!("Expected Selected result"),
    }

    // COUNT(discount) should count only non-NULL values
    let result = engine.execute("SELECT COUNT(discount) FROM orders").unwrap();
    match result {
        ExecutionResult::Selected { data, .. } => {
            assert_eq!(data[0].get(0).unwrap(), &Value::Int64(2));
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_sum() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE sales (id BIGINT PRIMARY KEY, amount FLOAT64)").unwrap();
    engine.execute("INSERT INTO sales VALUES (1, 100.0)").unwrap();
    engine.execute("INSERT INTO sales VALUES (2, 200.0)").unwrap();
    engine.execute("INSERT INTO sales VALUES (3, 300.0)").unwrap();

    let result = engine.execute("SELECT SUM(amount) FROM sales").unwrap();
    match result {
        ExecutionResult::Selected { data, .. } => {
            assert_eq!(data[0].get(0).unwrap(), &Value::Float64(600.0));
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_sum_with_nulls() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE payments (id BIGINT PRIMARY KEY, amount FLOAT64)").unwrap();
    engine.execute("INSERT INTO payments VALUES (1, 100.0)").unwrap();
    engine.execute("INSERT INTO payments VALUES (2, NULL)").unwrap();
    engine.execute("INSERT INTO payments VALUES (3, 300.0)").unwrap();

    let result = engine.execute("SELECT SUM(amount) FROM payments").unwrap();
    match result {
        ExecutionResult::Selected { data, .. } => {
            assert_eq!(data[0].get(0).unwrap(), &Value::Float64(400.0));
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_avg() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE scores (id BIGINT PRIMARY KEY, score FLOAT64)").unwrap();
    engine.execute("INSERT INTO scores VALUES (1, 80.0)").unwrap();
    engine.execute("INSERT INTO scores VALUES (2, 90.0)").unwrap();
    engine.execute("INSERT INTO scores VALUES (3, 70.0)").unwrap();

    let result = engine.execute("SELECT AVG(score) FROM scores").unwrap();
    match result {
        ExecutionResult::Selected { data, .. } => {
            assert_eq!(data[0].get(0).unwrap(), &Value::Float64(80.0));
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_avg_with_nulls() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE ratings (id BIGINT PRIMARY KEY, rating FLOAT64)").unwrap();
    engine.execute("INSERT INTO ratings VALUES (1, 100.0)").unwrap();
    engine.execute("INSERT INTO ratings VALUES (2, NULL)").unwrap();
    engine.execute("INSERT INTO ratings VALUES (3, 200.0)").unwrap();

    let result = engine.execute("SELECT AVG(rating) FROM ratings").unwrap();
    match result {
        ExecutionResult::Selected { data, .. } => {
            assert_eq!(data[0].get(0).unwrap(), &Value::Float64(150.0));
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_min_max() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE temps (id BIGINT PRIMARY KEY, temperature FLOAT64)").unwrap();
    engine.execute("INSERT INTO temps VALUES (1, 72.5)").unwrap();
    engine.execute("INSERT INTO temps VALUES (2, 85.3)").unwrap();
    engine.execute("INSERT INTO temps VALUES (3, 68.1)").unwrap();
    engine.execute("INSERT INTO temps VALUES (4, 79.8)").unwrap();

    let result = engine.execute("SELECT MIN(temperature) FROM temps").unwrap();
    match result {
        ExecutionResult::Selected { data, .. } => {
            assert_eq!(data[0].get(0).unwrap(), &Value::Float64(68.1));
        }
        _ => panic!("Expected Selected result"),
    }

    let result = engine.execute("SELECT MAX(temperature) FROM temps").unwrap();
    match result {
        ExecutionResult::Selected { data, .. } => {
            assert_eq!(data[0].get(0).unwrap(), &Value::Float64(85.3));
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_multiple_aggregates() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE metrics (id BIGINT PRIMARY KEY, value FLOAT64)").unwrap();
    engine.execute("INSERT INTO metrics VALUES (1, 10.0)").unwrap();
    engine.execute("INSERT INTO metrics VALUES (2, 20.0)").unwrap();
    engine.execute("INSERT INTO metrics VALUES (3, 30.0)").unwrap();
    engine.execute("INSERT INTO metrics VALUES (4, 40.0)").unwrap();

    let result = engine.execute("SELECT COUNT(*), SUM(value), AVG(value), MIN(value), MAX(value) FROM metrics").unwrap();
    match result {
        ExecutionResult::Selected { data, .. } => {
            assert_eq!(data[0].get(0).unwrap(), &Value::Int64(4));
            assert_eq!(data[0].get(1).unwrap(), &Value::Float64(100.0));
            assert_eq!(data[0].get(2).unwrap(), &Value::Float64(25.0));
            assert_eq!(data[0].get(3).unwrap(), &Value::Float64(10.0));
            assert_eq!(data[0].get(4).unwrap(), &Value::Float64(40.0));
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_group_by_single_column() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE sales (id BIGINT PRIMARY KEY, category TEXT, amount FLOAT64)").unwrap();
    engine.execute("INSERT INTO sales VALUES (1, 'Electronics', 100.0)").unwrap();
    engine.execute("INSERT INTO sales VALUES (2, 'Electronics', 200.0)").unwrap();
    engine.execute("INSERT INTO sales VALUES (3, 'Books', 50.0)").unwrap();
    engine.execute("INSERT INTO sales VALUES (4, 'Books', 75.0)").unwrap();
    engine.execute("INSERT INTO sales VALUES (5, 'Electronics', 150.0)").unwrap();

    let result = engine.execute("SELECT category, SUM(amount) FROM sales GROUP BY category").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 2);

            let mut found_electronics = false;
            let mut found_books = false;

            for row in data {
                match row.get(0).unwrap() {
                    Value::Text(cat) if cat == "Electronics" => {
                        assert_eq!(row.get(1).unwrap(), &Value::Float64(450.0));
                        found_electronics = true;
                    }
                    Value::Text(cat) if cat == "Books" => {
                        assert_eq!(row.get(1).unwrap(), &Value::Float64(125.0));
                        found_books = true;
                    }
                    _ => panic!("Unexpected category"),
                }
            }

            assert!(found_electronics && found_books);
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_group_by_with_count() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE orders (id BIGINT PRIMARY KEY, customer TEXT, order_id BIGINT, amount FLOAT64)").unwrap();
    engine.execute("INSERT INTO orders VALUES (1, 'Alice', 1, 100.0)").unwrap();
    engine.execute("INSERT INTO orders VALUES (2, 'Bob', 2, 200.0)").unwrap();
    engine.execute("INSERT INTO orders VALUES (3, 'Alice', 3, 150.0)").unwrap();
    engine.execute("INSERT INTO orders VALUES (4, 'Charlie', 4, 300.0)").unwrap();
    engine.execute("INSERT INTO orders VALUES (5, 'Alice', 5, 175.0)").unwrap();

    let result = engine.execute("SELECT customer, COUNT(*) FROM orders GROUP BY customer").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 3);

            let mut counts = std::collections::HashMap::new();
            for row in data {
                if let Value::Text(name) = row.get(0).unwrap() {
                    if let Value::Int64(count) = row.get(1).unwrap() {
                        counts.insert(name.clone(), *count);
                    }
                }
            }

            assert_eq!(counts.get("Alice"), Some(&3));
            assert_eq!(counts.get("Bob"), Some(&1));
            assert_eq!(counts.get("Charlie"), Some(&1));
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_group_by_multiple_aggregates() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE transactions (id BIGINT PRIMARY KEY, region TEXT, amount FLOAT64)").unwrap();
    engine.execute("INSERT INTO transactions VALUES (1, 'North', 100.0)").unwrap();
    engine.execute("INSERT INTO transactions VALUES (2, 'North', 200.0)").unwrap();
    engine.execute("INSERT INTO transactions VALUES (3, 'South', 150.0)").unwrap();
    engine.execute("INSERT INTO transactions VALUES (4, 'South', 250.0)").unwrap();
    engine.execute("INSERT INTO transactions VALUES (5, 'North', 300.0)").unwrap();

    let result = engine.execute("SELECT region, COUNT(*), SUM(amount), AVG(amount) FROM transactions GROUP BY region").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 2);

            let mut found_north = false;
            let mut found_south = false;

            for row in data {
                match row.get(0).unwrap() {
                    Value::Text(region) if region == "North" => {
                        assert_eq!(row.get(1).unwrap(), &Value::Int64(3));
                        assert_eq!(row.get(2).unwrap(), &Value::Float64(600.0));
                        assert_eq!(row.get(3).unwrap(), &Value::Float64(200.0));
                        found_north = true;
                    }
                    Value::Text(region) if region == "South" => {
                        assert_eq!(row.get(1).unwrap(), &Value::Int64(2));
                        assert_eq!(row.get(2).unwrap(), &Value::Float64(400.0));
                        assert_eq!(row.get(3).unwrap(), &Value::Float64(200.0));
                        found_south = true;
                    }
                    _ => panic!("Unexpected region"),
                }
            }

            assert!(found_north && found_south);
        }
        _ => panic!("Expected Selected result"),
    }
}