# OmenDB Quick Start Guide

Get started with OmenDB in 5 minutes.

## Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
cd /path/to/omendb/core/omendb-rust
```

## 1. Run the Demo (30 seconds)

See OmenDB in action with a pre-built demo:

```bash
cargo run --bin sql_demo
```

**What you'll see:**
- Creating two tables (`users` and `metrics`)
- Inserting data into both tables
- Querying with learned indexes
- Database statistics

**Expected output:**
```
🚀 OmenDB - Multi-Table Database with Learned Indexes
============================================================

📊 Demo 1: Creating users table
✅ Table 'users' created

📊 Demo 2: Inserting user data
✅ Inserted 5 rows

📊 Demo 3: Querying users
✅ Retrieved 5 rows
...
```

## 2. Run Performance Benchmarks (2 minutes)

### Learned Index vs B-tree

```bash
cargo run --release --bin benchmark_vs_btree
```

**Results:**
- Sequential data: **20.79x faster**
- Average across 5 workloads: **9.85x faster**

### Full System Benchmark

```bash
cargo run --release --bin benchmark_full_system
```

**Results:**
- **102,270 ops/sec** average throughput
- **183.2μs** average latency
- Sub-millisecond P99 latency

## 3. Write Your First Program (3 minutes)

### Example 1: Simple Time-Series Database

Create `examples/my_first_db.rs`:

```rust
use omendb::catalog::Catalog;
use omendb::sql_engine::{SqlEngine, ExecutionResult};
use tempfile::TempDir;
use anyhow::Result;

fn main() -> Result<()> {
    // Create database
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;
    let mut engine = SqlEngine::new(catalog);

    // Create metrics table
    engine.execute(
        "CREATE TABLE metrics (
            timestamp BIGINT PRIMARY KEY,
            sensor_id BIGINT,
            temperature DOUBLE,
            status VARCHAR(50)
        )"
    )?;

    // Insert sensor data
    for i in 0..1000 {
        let sql = format!(
            "INSERT INTO metrics VALUES ({}, {}, {}, 'normal')",
            i * 1000,  // timestamp
            i % 10,    // sensor_id (10 sensors)
            20.0 + (i % 20) as f64 * 0.5  // temperature
        );
        engine.execute(&sql)?;
    }

    // Query all data
    let result = engine.execute("SELECT * FROM metrics")?;

    match result {
        ExecutionResult::Selected { rows, .. } => {
            println!("✅ Retrieved {} sensor readings", rows);
        }
        _ => {}
    }

    // Query with WHERE clause (uses learned index)
    let result = engine.execute("SELECT * FROM metrics WHERE timestamp > 500000")?;
    match result {
        ExecutionResult::Selected { rows, .. } => {
            println!("✅ Found {} readings with timestamp > 500000", rows);
        }
        _ => {}
    }

    // Show database stats
    let tables = engine.catalog().list_tables();
    println!("\nDatabase has {} table(s)", tables.len());

    for table_name in tables {
        let table = engine.catalog().get_table(&table_name)?;
        println!("  - {}: {} rows, using learned index",
                 table_name, table.row_count());
    }

    Ok(())
}
```

Run it:

```bash
cargo run --example my_first_db
```

### Example 2: Multi-Table Application

Create `examples/multi_table.rs`:

```rust
use omendb::catalog::Catalog;
use omendb::sql_engine::{SqlEngine, ExecutionResult};
use tempfile::TempDir;
use anyhow::Result;

fn main() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;
    let mut engine = SqlEngine::new(catalog);

    // Create multiple tables
    println!("Creating tables...");
    engine.execute(
        "CREATE TABLE users (
            id BIGINT PRIMARY KEY,
            name VARCHAR(255),
            created_at BIGINT
        )"
    )?;

    engine.execute(
        "CREATE TABLE sessions (
            id BIGINT PRIMARY KEY,
            user_id BIGINT,
            duration BIGINT
        )"
    )?;

    // Insert data
    println!("Inserting users...");
    engine.execute(
        "INSERT INTO users VALUES
            (1, 'Alice', 1000),
            (2, 'Bob', 2000),
            (3, 'Charlie', 3000)"
    )?;

    println!("Inserting sessions...");
    engine.execute(
        "INSERT INTO sessions VALUES
            (1, 1, 3600),
            (2, 1, 7200),
            (3, 2, 1800)"
    )?;

    // Query each table
    println!("\nQuerying users:");
    let result = engine.execute("SELECT * FROM users")?;
    if let ExecutionResult::Selected { rows, .. } = result {
        println!("  Found {} users", rows);
    }

    println!("\nQuerying sessions:");
    let result = engine.execute("SELECT * FROM sessions")?;
    if let ExecutionResult::Selected { rows, .. } = result {
        println!("  Found {} sessions", rows);
    }

    println!("\n✅ Multi-table database working!");
    Ok(())
}
```

Run it:

```bash
cargo run --example multi_table
```

## 4. Add OmenDB to Your Project

### Cargo.toml

```toml
[dependencies]
omendb = { path = "/path/to/omendb-rust" }
arrow = "45.0"
tempfile = "3.8"
anyhow = "1.0"
```

### Basic Usage

```rust
use omendb::catalog::Catalog;
use omendb::sql_engine::SqlEngine;

// Initialize database
let catalog = Catalog::new(std::path::PathBuf::from("./data"))?;
let mut engine = SqlEngine::new(catalog);

// Create table
engine.execute(
    "CREATE TABLE events (id BIGINT PRIMARY KEY, name VARCHAR(255))"
)?;

// Insert data
engine.execute("INSERT INTO events VALUES (1, 'login'), (2, 'logout')")?;

// Query
engine.execute("SELECT * FROM events")?;
```

## 5. Common Operations

### Creating Tables

```rust
// Simple table
engine.execute(
    "CREATE TABLE simple (id BIGINT PRIMARY KEY)"
)?;

// Complex table with multiple types
engine.execute(
    "CREATE TABLE complex (
        id BIGINT PRIMARY KEY,
        count BIGINT,
        ratio DOUBLE,
        label VARCHAR(100),
        active BOOLEAN
    )"
)?;
```

### Inserting Data

```rust
// Single insert
engine.execute("INSERT INTO table VALUES (1, 'value')")?;

// Batch insert
engine.execute(
    "INSERT INTO table VALUES
        (1, 'first'),
        (2, 'second'),
        (3, 'third')"
)?;

// Programmatic insert
for i in 0..1000 {
    engine.execute(&format!("INSERT INTO table VALUES ({}, 'item_{}')", i, i))?;
}
```

### Querying Data

```rust
// Select all
let result = engine.execute("SELECT * FROM table")?;

// Process results
match result {
    ExecutionResult::Selected { columns, rows, data } => {
        println!("Columns: {:?}", columns);
        println!("Rows: {}", rows);

        for row in data {
            println!("{:?}", row);
        }
    }
    _ => {}
}
```

### Checking Database State

```rust
// List all tables
let tables = engine.catalog().list_tables();
println!("Tables: {:?}", tables);

// Get table info
let table = engine.catalog().get_table("my_table")?;
println!("Rows: {}", table.row_count());
println!("Primary key: {}", table.primary_key());

// Check if table exists
if engine.catalog().table_exists("my_table") {
    println!("Table exists!");
}
```

## 6. Performance Tips

### Disable WAL for Maximum Speed

For benchmarking or bulk loads:

```rust
// Disable WAL (3-5x faster writes, no durability)
let catalog = Catalog::new_with_wal(data_dir, false)?;
```

### Batch Inserts

```rust
// Good: Single SQL with multiple values
engine.execute(
    "INSERT INTO table VALUES (1, 'a'), (2, 'b'), (3, 'c')"
)?;

// Avoid: Multiple single inserts
for i in 0..100 {
    engine.execute(&format!("INSERT INTO table VALUES ({}, 'item')", i))?;
}
```

### Learned Index Optimization

```rust
// Best performance: Sequential or near-sequential keys
// Good: Timestamps, auto-increment IDs
engine.execute("INSERT INTO table VALUES (1000, ...), (2000, ...), (3000, ...)")?;

// Lower performance: Random keys
// Still 2x faster than B-trees on average
```

## 7. Testing Your Code

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_my_database() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        engine.execute("CREATE TABLE test (id BIGINT PRIMARY KEY)").unwrap();
        engine.execute("INSERT INTO test VALUES (1)").unwrap();

        let result = engine.execute("SELECT * FROM test").unwrap();
        match result {
            ExecutionResult::Selected { rows, .. } => {
                assert_eq!(rows, 1);
            }
            _ => panic!("Expected SELECT result"),
        }
    }
}
```

Run tests:

```bash
cargo test
```

## 8. Troubleshooting

### Build Issues

```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update
```

### Performance Issues

```bash
# Always use --release for benchmarks
cargo run --release --bin benchmark_full_system

# Check if WAL is enabled (disabling improves write speed)
let catalog = Catalog::new_with_wal(data_dir, false)?;
```

### Data Issues

```bash
# Delete and recreate database
rm -rf /path/to/data
# Database will be recreated on next run
```

## 9. What's Next?

### Learn More
- Read the [full README](README.md) for architecture details
- Explore source code in `src/` directory
- Run benchmarks to understand performance characteristics

### Contribute
- Add new SQL operations (WHERE, JOIN, etc.)
- Improve learned index algorithms
- Write more examples

### Production Use
- Implement PostgreSQL wire protocol for standard client access
- Add Docker deployment
- Set up monitoring and metrics

## 10. Getting Help

- **Issues**: Open GitHub issue
- **Questions**: Check README.md and code comments
- **Email**: nijaru7@gmail.com

---

**Next Steps:**
1. ✅ Run the demo
2. ✅ Run benchmarks
3. ✅ Write your first program
4. 📚 Read the full README
5. 🚀 Build something amazing!

Welcome to OmenDB - where learned indexes meet production databases!