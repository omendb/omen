//! Standalone test for Extended Query Protocol using rust-postgres

use postgres::{Client, NoTls};

fn main() {
    println!("Testing Extended Query Protocol with rust-postgres client\n");

    // Connect to OmenDB
    let mut client = match Client::connect("host=localhost port=5433 user=postgres", NoTls) {
        Ok(c) => {
            println!("✓ Connected to OmenDB");
            c
        }
        Err(e) => {
            eprintln!("❌ Failed to connect: {}", e);
            std::process::exit(1);
        }
    };

    // Test 1: Parameterized SELECT (uses Extended Query Protocol)
    println!("\nTest 1: Parameterized SELECT with $1");
    match client.query("SELECT * FROM users WHERE id = $1", &[&1i64]) {
        Ok(rows) => {
            if !rows.is_empty() {
                let id: i64 = rows[0].get(0);
                println!("✓ Retrieved row with id: {}", id);
            } else {
                println!("⚠ No rows returned");
            }
        }
        Err(e) => {
            eprintln!("❌ Failed: {}", e);
        }
    }

    // Test 2: Range query with two parameters
    println!("\nTest 2: Range query with $1 and $2");
    match client.query(
        "SELECT * FROM users WHERE id >= $1 AND id <= $2",
        &[&1i64, &3i64],
    ) {
        Ok(rows) => {
            println!("✓ Retrieved {} rows with range query", rows.len());
            for row in rows.iter().take(3) {
                let id: i64 = row.get(0);
                println!("  - id: {}", id);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed: {}", e);
        }
    }

    // Test 3: Execute with parameter binding
    println!("\nTest 3: Prepared statement execution");
    match client.execute("SELECT * FROM users WHERE id = $1", &[&2i64]) {
        Ok(count) => {
            println!("✓ Executed successfully, affected {} rows", count);
        }
        Err(e) => {
            eprintln!("❌ Failed: {}", e);
        }
    }

    println!("\n" + &"=".repeat(60));
    println!("✅ Extended Query Protocol test completed!");
    println!("=".repeat(60));
}
