// Comprehensive aggregation tests for SQL engine (Phase 3)

use omendb::catalog::Catalog;
use omendb::sql_engine::SqlEngine;
use omendb::value::Value;
use tempfile::TempDir;

// Helper: Create test table with sample data
fn setup_test_data() -> (SqlEngine, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_str().unwrap()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Create sales table
    engine
        .execute("CREATE TABLE sales (id INT, product TEXT, category TEXT, quantity INT, price FLOAT, region TEXT)")
        .unwrap();

    // Insert test data
    engine
        .execute("INSERT INTO sales VALUES (1, 'Laptop', 'Electronics', 10, 999.99, 'North')")
        .unwrap();
    engine
        .execute("INSERT INTO sales VALUES (2, 'Mouse', 'Electronics', 50, 29.99, 'North')")
        .unwrap();
    engine
        .execute("INSERT INTO sales VALUES (3, 'Desk', 'Furniture', 5, 299.99, 'South')")
        .unwrap();
    engine
        .execute("INSERT INTO sales VALUES (4, 'Chair', 'Furniture', 20, 149.99, 'South')")
        .unwrap();
    engine
        .execute("INSERT INTO sales VALUES (5, 'Monitor', 'Electronics', 15, 399.99, 'East')")
        .unwrap();
    engine
        .execute("INSERT INTO sales VALUES (6, 'Keyboard', 'Electronics', 30, 79.99, 'East')")
        .unwrap();

    (engine, temp_dir)
}

#[test]
fn test_count_star() {
    let (mut engine, _temp_dir) = setup_test_data();

    let result = engine.execute("SELECT COUNT(*) FROM sales").unwrap();

    if let omendb::sql_engine::ExecutionResult::Selected { rows, data, .. } = result {
        assert_eq!(rows, 1);
        assert_eq!(data[0].get(0).unwrap(), &Value::Int64(6));
    } else {
        panic!("Expected Selected result");
    }
}

#[test]
fn test_count_column() {
    let (mut engine, _temp_dir) = setup_test_data();

    let result = engine
        .execute("SELECT COUNT(product) FROM sales")
        .unwrap();

    if let omendb::sql_engine::ExecutionResult::Selected { rows, data, .. } = result {
        assert_eq!(rows, 1);
        assert_eq!(data[0].get(0).unwrap(), &Value::Int64(6));
    } else {
        panic!("Expected Selected result");
    }
}

#[test]
fn test_sum_integer() {
    let (mut engine, _temp_dir) = setup_test_data();

    let result = engine
        .execute("SELECT SUM(quantity) FROM sales")
        .unwrap();

    if let omendb::sql_engine::ExecutionResult::Selected { rows, data, .. } = result {
        assert_eq!(rows, 1);
        // 10 + 50 + 5 + 20 + 15 + 30 = 130
        assert_eq!(data[0].get(0).unwrap(), &Value::Float64(130.0));
    } else {
        panic!("Expected Selected result");
    }
}

#[test]
fn test_sum_float() {
    let (mut engine, _temp_dir) = setup_test_data();

    let result = engine.execute("SELECT SUM(price) FROM sales").unwrap();

    if let omendb::sql_engine::ExecutionResult::Selected { rows, data, .. } = result {
        assert_eq!(rows, 1);
        // 999.99 + 29.99 + 299.99 + 149.99 + 399.99 + 79.99 = 1959.94
        if let Value::Float64(sum) = data[0].get(0).unwrap() {
            assert!((sum - 1959.94).abs() < 0.01);
        } else {
            panic!("Expected Float64");
        }
    } else {
        panic!("Expected Selected result");
    }
}

#[test]
fn test_avg_integer() {
    let (mut engine, _temp_dir) = setup_test_data();

    let result = engine
        .execute("SELECT AVG(quantity) FROM sales")
        .unwrap();

    if let omendb::sql_engine::ExecutionResult::Selected { rows, data, .. } = result {
        assert_eq!(rows, 1);
        // (10 + 50 + 5 + 20 + 15 + 30) / 6 = 21.666...
        if let Value::Float64(avg) = data[0].get(0).unwrap() {
            assert!((avg - 21.666666).abs() < 0.001);
        } else {
            panic!("Expected Float64");
        }
    } else {
        panic!("Expected Selected result");
    }
}

#[test]
fn test_avg_float() {
    let (mut engine, _temp_dir) = setup_test_data();

    let result = engine.execute("SELECT AVG(price) FROM sales").unwrap();

    if let omendb::sql_engine::ExecutionResult::Selected { rows, data, .. } = result {
        assert_eq!(rows, 1);
        // 1959.94 / 6 = 326.656666...
        if let Value::Float64(avg) = data[0].get(0).unwrap() {
            assert!((avg - 326.656666).abs() < 0.001);
        } else {
            panic!("Expected Float64");
        }
    } else {
        panic!("Expected Selected result");
    }
}

#[test]
fn test_min_integer() {
    let (mut engine, _temp_dir) = setup_test_data();

    let result = engine
        .execute("SELECT MIN(quantity) FROM sales")
        .unwrap();

    if let omendb::sql_engine::ExecutionResult::Selected { rows, data, .. } = result {
        assert_eq!(rows, 1);
        assert_eq!(data[0].get(0).unwrap(), &Value::Int64(5));
    } else {
        panic!("Expected Selected result");
    }
}

#[test]
fn test_max_integer() {
    let (mut engine, _temp_dir) = setup_test_data();

    let result = engine
        .execute("SELECT MAX(quantity) FROM sales")
        .unwrap();

    if let omendb::sql_engine::ExecutionResult::Selected { rows, data, .. } = result {
        assert_eq!(rows, 1);
        assert_eq!(data[0].get(0).unwrap(), &Value::Int64(50));
    } else {
        panic!("Expected Selected result");
    }
}

#[test]
fn test_min_float() {
    let (mut engine, _temp_dir) = setup_test_data();

    let result = engine.execute("SELECT MIN(price) FROM sales").unwrap();

    if let omendb::sql_engine::ExecutionResult::Selected { rows, data, .. } = result {
        assert_eq!(rows, 1);
        if let Value::Float64(min) = data[0].get(0).unwrap() {
            assert!((min - 29.99).abs() < 0.01);
        } else {
            panic!("Expected Float64");
        }
    } else {
        panic!("Expected Selected result");
    }
}

#[test]
fn test_max_float() {
    let (mut engine, _temp_dir) = setup_test_data();

    let result = engine.execute("SELECT MAX(price) FROM sales").unwrap();

    if let omendb::sql_engine::ExecutionResult::Selected { rows, data, .. } = result {
        assert_eq!(rows, 1);
        if let Value::Float64(max) = data[0].get(0).unwrap() {
            assert!((max - 999.99).abs() < 0.01);
        } else {
            panic!("Expected Float64");
        }
    } else {
        panic!("Expected Selected result");
    }
}

#[test]
fn test_multiple_aggregates() {
    let (mut engine, _temp_dir) = setup_test_data();

    let result = engine
        .execute("SELECT COUNT(*), SUM(quantity), AVG(price), MIN(price), MAX(price) FROM sales")
        .unwrap();

    if let omendb::sql_engine::ExecutionResult::Selected { rows, data, columns, .. } = result {
        assert_eq!(rows, 1);
        assert_eq!(columns.len(), 5);
        
        // COUNT(*)
        assert_eq!(data[0].get(0).unwrap(), &Value::Int64(6));
        
        // SUM(quantity)
        assert_eq!(data[0].get(1).unwrap(), &Value::Float64(130.0));
        
        // AVG(price)
        if let Value::Float64(avg) = data[0].get(2).unwrap() {
            assert!((avg - 326.656666).abs() < 0.001);
        }
        
        // MIN(price)
        if let Value::Float64(min) = data[0].get(3).unwrap() {
            assert!((min - 29.99).abs() < 0.01);
        }
        
        // MAX(price)
        if let Value::Float64(max) = data[0].get(4).unwrap() {
            assert!((max - 999.99).abs() < 0.01);
        }
    } else {
        panic!("Expected Selected result");
    }
}

#[test]
fn test_group_by_single_column() {
    let (mut engine, _temp_dir) = setup_test_data();

    let result = engine
        .execute("SELECT category, COUNT(*), SUM(quantity) FROM sales GROUP BY category")
        .unwrap();

    if let omendb::sql_engine::ExecutionResult::Selected { rows, data, columns, .. } = result {
        assert_eq!(rows, 2); // Electronics and Furniture
        assert_eq!(columns.len(), 3);
        
        // Results might be in any order, so check both possibilities
        let mut found_electronics = false;
        let mut found_furniture = false;
        
        for row in data {
            if let Value::Text(category) = row.get(0).unwrap() {
                if category == "Electronics" {
                    found_electronics = true;
                    assert_eq!(row.get(1).unwrap(), &Value::Int64(4)); // 4 electronics
                    assert_eq!(row.get(2).unwrap(), &Value::Float64(105.0)); // 10+50+15+30
                } else if category == "Furniture" {
                    found_furniture = true;
                    assert_eq!(row.get(1).unwrap(), &Value::Int64(2)); // 2 furniture
                    assert_eq!(row.get(2).unwrap(), &Value::Float64(25.0)); // 5+20
                }
            }
        }
        
        assert!(found_electronics, "Should find Electronics group");
        assert!(found_furniture, "Should find Furniture group");
    } else {
        panic!("Expected Selected result");
    }
}

#[test]
fn test_group_by_multiple_columns() {
    let (mut engine, _temp_dir) = setup_test_data();

    let result = engine
        .execute("SELECT category, region, COUNT(*) FROM sales GROUP BY category, region")
        .unwrap();

    if let omendb::sql_engine::ExecutionResult::Selected { rows, columns, .. } = result {
        assert_eq!(rows, 4); // Electronics-North, Electronics-East, Furniture-South, etc
        assert_eq!(columns.len(), 3);
    } else {
        panic!("Expected Selected result");
    }
}

#[test]
fn test_empty_table_aggregates() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_str().unwrap()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine
        .execute("CREATE TABLE empty_table (id INT, value INT)")
        .unwrap();

    // COUNT should return 0
    let result = engine.execute("SELECT COUNT(*) FROM empty_table").unwrap();
    if let omendb::sql_engine::ExecutionResult::Selected { rows, data, .. } = result {
        assert_eq!(rows, 1);
        assert_eq!(data[0].get(0).unwrap(), &Value::Int64(0));
    }

    // SUM should return 0
    let result = engine.execute("SELECT SUM(value) FROM empty_table").unwrap();
    if let omendb::sql_engine::ExecutionResult::Selected { rows, data, .. } = result {
        assert_eq!(rows, 1);
        assert_eq!(data[0].get(0).unwrap(), &Value::Float64(0.0));
    }

    // AVG should return NULL
    let result = engine.execute("SELECT AVG(value) FROM empty_table").unwrap();
    if let omendb::sql_engine::ExecutionResult::Selected { rows, data, .. } = result {
        assert_eq!(rows, 1);
        assert_eq!(data[0].get(0).unwrap(), &Value::Null);
    }

    // MIN should return NULL
    let result = engine.execute("SELECT MIN(value) FROM empty_table").unwrap();
    if let omendb::sql_engine::ExecutionResult::Selected { rows, data, .. } = result {
        assert_eq!(rows, 1);
        assert_eq!(data[0].get(0).unwrap(), &Value::Null);
    }

    // MAX should return NULL
    let result = engine.execute("SELECT MAX(value) FROM empty_table").unwrap();
    if let omendb::sql_engine::ExecutionResult::Selected { rows, data, .. } = result {
        assert_eq!(rows, 1);
        assert_eq!(data[0].get(0).unwrap(), &Value::Null);
    }
}

#[test]
fn test_null_values_in_aggregates() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_str().unwrap()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine
        .execute("CREATE TABLE nullable (id INT, value INT)")
        .unwrap();
    
    // Insert some NULL values
    engine.execute("INSERT INTO nullable VALUES (1, 10)").unwrap();
    engine.execute("INSERT INTO nullable VALUES (2, 20)").unwrap();
    engine.execute("INSERT INTO nullable VALUES (3, NULL)").unwrap();
    engine.execute("INSERT INTO nullable VALUES (4, 30)").unwrap();

    // COUNT(*) should count all rows including NULLs
    let result = engine.execute("SELECT COUNT(*) FROM nullable").unwrap();
    if let omendb::sql_engine::ExecutionResult::Selected { data, .. } = result {
        assert_eq!(data[0].get(0).unwrap(), &Value::Int64(4));
    }

    // COUNT(column) should skip NULL values
    let result = engine.execute("SELECT COUNT(value) FROM nullable").unwrap();
    if let omendb::sql_engine::ExecutionResult::Selected { data, .. } = result {
        assert_eq!(data[0].get(0).unwrap(), &Value::Int64(3));
    }

    // SUM should skip NULL values
    let result = engine.execute("SELECT SUM(value) FROM nullable").unwrap();
    if let omendb::sql_engine::ExecutionResult::Selected { data, .. } = result {
        assert_eq!(data[0].get(0).unwrap(), &Value::Float64(60.0)); // 10+20+30
    }

    // AVG should skip NULL values
    let result = engine.execute("SELECT AVG(value) FROM nullable").unwrap();
    if let omendb::sql_engine::ExecutionResult::Selected { data, .. } = result {
        assert_eq!(data[0].get(0).unwrap(), &Value::Float64(20.0)); // 60/3
    }
}

// HAVING clause tests

#[test]
fn test_having_count() {
    let (mut engine, _temp_dir) = setup_test_data();

    let result = engine
        .execute("SELECT category, COUNT(*) FROM sales GROUP BY category HAVING COUNT(*) > 1")
        .unwrap();

    if let omendb::sql_engine::ExecutionResult::Selected { rows, data, .. } = result {
        // Both Electronics (4) and Furniture (2) have count > 1
        assert_eq!(rows, 2);
        
        for row in data {
            if let Value::Int64(count) = row.get(1).unwrap() {
                assert!(*count > 1, "HAVING COUNT(*) > 1 should filter correctly");
            }
        }
    } else {
        panic!("Expected Selected result");
    }
}

#[test]
fn test_having_sum() {
    let (mut engine, _temp_dir) = setup_test_data();

    let result = engine
        .execute("SELECT category, SUM(quantity) FROM sales GROUP BY category HAVING SUM(quantity) >= 50")
        .unwrap();

    if let omendb::sql_engine::ExecutionResult::Selected { rows, data, .. } = result {
        // Only Electronics has SUM(quantity) >= 50 (10+50+15+30 = 105)
        assert_eq!(rows, 1);
        
        if let Value::Text(category) = data[0].get(0).unwrap() {
            assert_eq!(category, "Electronics");
        }
        
        if let Value::Float64(sum) = data[0].get(1).unwrap() {
            assert!(*sum >= 50.0);
        }
    } else {
        panic!("Expected Selected result");
    }
}

#[test]
fn test_having_avg() {
    let (mut engine, _temp_dir) = setup_test_data();

    let result = engine
        .execute("SELECT category, AVG(price) FROM sales GROUP BY category HAVING AVG(price) > 300")
        .unwrap();

    if let omendb::sql_engine::ExecutionResult::Selected { rows, data, .. } = result {
        // Only Electronics should pass (avg price > 300)
        assert_eq!(rows, 1);
        
        if let Value::Text(category) = data[0].get(0).unwrap() {
            assert_eq!(category, "Electronics");
        }
    } else {
        panic!("Expected Selected result");
    }
}

#[test]
fn test_having_with_multiple_conditions() {
    let (mut engine, _temp_dir) = setup_test_data();

    // Test HAVING with AND condition
    let result = engine
        .execute("SELECT category, COUNT(*), SUM(quantity) FROM sales GROUP BY category HAVING COUNT(*) > 1 AND SUM(quantity) > 30")
        .unwrap();

    if let omendb::sql_engine::ExecutionResult::Selected { rows, .. } = result {
        // Electronics: count=4, sum=105 ✓
        // Furniture: count=2, sum=25 ✗ (sum not > 30)
        assert_eq!(rows, 1);
    } else {
        panic!("Expected Selected result");
    }
}

#[test]
fn test_having_all_filtered_out() {
    let (mut engine, _temp_dir) = setup_test_data();

    // Test HAVING that filters out everything
    let result = engine
        .execute("SELECT category, COUNT(*) FROM sales GROUP BY category HAVING COUNT(*) > 100")
        .unwrap();

    if let omendb::sql_engine::ExecutionResult::Selected { rows, .. } = result {
        assert_eq!(rows, 0, "HAVING should filter out all rows");
    } else {
        panic!("Expected Selected result");
    }
}

#[test]
fn test_having_without_group_by() {
    let (mut engine, _temp_dir) = setup_test_data();

    // HAVING with aggregates but no GROUP BY (single group)
    let result = engine
        .execute("SELECT COUNT(*) FROM sales HAVING COUNT(*) > 5")
        .unwrap();

    if let omendb::sql_engine::ExecutionResult::Selected { rows, data, .. } = result {
        assert_eq!(rows, 1);
        assert_eq!(data[0].get(0).unwrap(), &Value::Int64(6));
    } else {
        panic!("Expected Selected result");
    }

    // Test HAVING that filters out the single group
    let result = engine
        .execute("SELECT COUNT(*) FROM sales HAVING COUNT(*) > 100")
        .unwrap();

    if let omendb::sql_engine::ExecutionResult::Selected { rows, .. } = result {
        assert_eq!(rows, 0, "HAVING should filter out single group");
    } else {
        panic!("Expected Selected result");
    }
}
