//! Multi-Table Database Example
//! Demonstrates creating and querying multiple tables

use omendb::catalog::Catalog;
use omendb::sql_engine::{SqlEngine, ExecutionResult};
use tempfile::TempDir;
use anyhow::Result;

fn main() -> Result<()> {
    println!("🚀 Multi-Table Database Example\n");

    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;
    let mut engine = SqlEngine::new(catalog);

    // Create multiple tables
    println!("📊 Creating tables...");
    engine.execute(
        "CREATE TABLE users (
            id BIGINT PRIMARY KEY,
            name VARCHAR(255),
            created_at BIGINT
        )"
    )?;
    println!("  ✅ users table created");

    engine.execute(
        "CREATE TABLE sessions (
            id BIGINT PRIMARY KEY,
            user_id BIGINT,
            duration BIGINT
        )"
    )?;
    println!("  ✅ sessions table created");

    engine.execute(
        "CREATE TABLE events (
            id BIGINT PRIMARY KEY,
            user_id BIGINT,
            event_type VARCHAR(100),
            timestamp BIGINT
        )"
    )?;
    println!("  ✅ events table created\n");

    // Insert data into users
    println!("📝 Inserting users...");
    engine.execute(
        "INSERT INTO users VALUES
            (1, 'Alice', 1000),
            (2, 'Bob', 2000),
            (3, 'Charlie', 3000),
            (4, 'Diana', 4000),
            (5, 'Eve', 5000)"
    )?;
    println!("  ✅ Inserted 5 users\n");

    // Insert data into sessions
    println!("📝 Inserting sessions...");
    engine.execute(
        "INSERT INTO sessions VALUES
            (1, 1, 3600),
            (2, 1, 7200),
            (3, 2, 1800),
            (4, 3, 5400),
            (5, 2, 9000)"
    )?;
    println!("  ✅ Inserted 5 sessions\n");

    // Insert data into events
    println!("📝 Inserting events...");
    engine.execute(
        "INSERT INTO events VALUES
            (1, 1, 'login', 1000),
            (2, 1, 'page_view', 1100),
            (3, 2, 'login', 2000),
            (4, 3, 'login', 3000),
            (5, 2, 'logout', 3800)"
    )?;
    println!("  ✅ Inserted 5 events\n");

    // Query each table
    println!("🔍 Querying users:");
    let result = engine.execute("SELECT * FROM users")?;
    if let ExecutionResult::Selected { rows, data, .. } = result {
        println!("  Found {} users", rows);
        for (i, row) in data.iter().enumerate() {
            println!(
                "    {}. id={:?}, name={:?}, created={:?}",
                i + 1,
                row.get(0)?,
                row.get(1)?,
                row.get(2)?
            );
        }
    }

    println!("\n🔍 Querying sessions:");
    let result = engine.execute("SELECT * FROM sessions")?;
    if let ExecutionResult::Selected { rows, data, .. } = result {
        println!("  Found {} sessions", rows);
        for (i, row) in data.iter().enumerate() {
            println!(
                "    {}. id={:?}, user_id={:?}, duration={:?}s",
                i + 1,
                row.get(0)?,
                row.get(1)?,
                row.get(2)?
            );
        }
    }

    println!("\n🔍 Querying events:");
    let result = engine.execute("SELECT * FROM events")?;
    if let ExecutionResult::Selected { rows, data, .. } = result {
        println!("  Found {} events", rows);
        for (i, row) in data.iter().enumerate() {
            println!(
                "    {}. id={:?}, user_id={:?}, type={:?}, time={:?}",
                i + 1,
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?
            );
        }
    }

    // Show database statistics
    println!("\n📊 Database Statistics:");
    let tables = engine.catalog().list_tables();
    println!("  Total tables: {}", tables.len());

    for table_name in tables {
        let table = engine.catalog().get_table(&table_name)?;
        println!("\n  Table: {}", table_name);
        println!("    - Rows: {}", table.row_count());
        println!("    - Primary Key: {}", table.primary_key());
        println!("    - Index: Learned (RMI)");
    }

    println!("\n✅ Multi-table database working perfectly!");
    println!("\n💡 Key Features Demonstrated:");
    println!("   • Multiple tables with different schemas");
    println!("   • Each table has its own learned index");
    println!("   • Standard SQL interface for all operations");
    println!("   • Columnar storage with Apache Arrow");

    Ok(())
}