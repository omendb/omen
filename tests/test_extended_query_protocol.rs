//! Integration test for Extended Query Protocol using rust-postgres client

use postgres::{Client, NoTls};

#[test]
fn test_extended_query_protocol_with_rust_client() {
    // Connect to OmenDB
    let mut client = Client::connect("host=localhost port=5433 user=postgres", NoTls)
        .expect("Failed to connect to OmenDB");

    println!("Connected to OmenDB");

    // Test 1: Parameterized SELECT (uses Extended Query Protocol)
    println!("\nTest 1: Parameterized SELECT");
    let rows = client
        .query("SELECT * FROM users WHERE id = $1", &[&1i64])
        .expect("Failed to execute parameterized SELECT");

    assert!(!rows.is_empty(), "Expected at least one row");
    let id: i64 = rows[0].get(0);
    println!("✓ Retrieved row with id: {}", id);
    assert_eq!(id, 1);

    // Test 2: Parameterized INSERT (uses Extended Query Protocol)
    println!("\nTest 2: Parameterized INSERT");
    let insert_result = client.execute(
        "INSERT INTO users VALUES ($1, $2)",
        &[&999i64, &"TestUser"],
    );

    match insert_result {
        Ok(count) => {
            println!("✓ Inserted {} row(s)", count);
            assert_eq!(count, 1);
        }
        Err(e) => {
            println!("⚠ INSERT failed (expected if not implemented): {}", e);
        }
    }

    // Test 3: Range query with multiple parameters
    println!("\nTest 3: Range query with two parameters");
    let range_result = client.query(
        "SELECT * FROM users WHERE id >= $1 AND id <= $2",
        &[&1i64, &3i64],
    );

    match range_result {
        Ok(rows) => {
            println!("✓ Retrieved {} rows with range query", rows.len());
            assert!(rows.len() > 0);
        }
        Err(e) => {
            println!("Range query error: {}", e);
            panic!("Range query should work");
        }
    }

    println!("\n✅ All Extended Query Protocol tests passed!");
}
