//! My First OmenDB Database
//! Simple time-series metrics storage example

use omendb::catalog::Catalog;
use omendb::sql_engine::{SqlEngine, ExecutionResult};
use tempfile::TempDir;
use anyhow::Result;

fn main() -> Result<()> {
    println!("🚀 My First OmenDB Database\n");

    // Create database
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;
    let mut engine = SqlEngine::new(catalog);

    println!("📊 Creating metrics table...");
    // Create metrics table
    engine.execute(
        "CREATE TABLE metrics (
            timestamp BIGINT PRIMARY KEY,
            sensor_id BIGINT,
            temperature DOUBLE,
            status VARCHAR(50)
        )"
    )?;
    println!("✅ Table created\n");

    println!("📝 Inserting 1000 sensor readings...");
    // Insert sensor data
    for i in 0..1000 {
        let sql = format!(
            "INSERT INTO metrics VALUES ({}, {}, {}, 'normal')",
            i * 1000,  // timestamp (ms)
            i % 10,    // sensor_id (10 sensors)
            20.0 + (i % 20) as f64 * 0.5  // temperature (20-30°C)
        );
        engine.execute(&sql)?;
    }
    println!("✅ Inserted 1000 readings\n");

    println!("🔍 Querying all data with learned index...");
    // Query all data
    let result = engine.execute("SELECT * FROM metrics")?;

    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            println!("✅ Retrieved {} sensor readings\n", rows);

            // Show first 5 readings
            println!("First 5 readings:");
            for (i, row) in data.iter().take(5).enumerate() {
                println!(
                    "  {}. timestamp={:?}, sensor={:?}, temp={:?}, status={:?}",
                    i + 1,
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?
                );
            }
        }
        _ => {}
    }

    println!("\n📊 Database Statistics:");
    // Show database stats
    let tables = engine.catalog().list_tables();
    println!("  Tables: {}", tables.len());

    for table_name in tables {
        let table = engine.catalog().get_table(&table_name)?;
        println!("\n  Table: {}", table_name);
        println!("    - Rows: {}", table.row_count());
        println!("    - Primary Key: {}", table.primary_key());
        println!("    - Columns: {}", table.schema().fields().len());
        println!("    - Index: Learned (RMI)");
    }

    println!("\n✅ Demo complete!");
    Ok(())
}