//! Programmatic API Example
//! Using OmenDB without SQL - direct API calls

use omendb::catalog::Catalog;
use omendb::value::Value;
use omendb::row::Row;
use arrow::datatypes::{DataType, Field, Schema};
use std::sync::Arc;
use tempfile::TempDir;
use anyhow::Result;

fn main() -> Result<()> {
    println!("🚀 OmenDB Programmatic API Example\n");

    let temp_dir = TempDir::new()?;
    let mut catalog = Catalog::new(temp_dir.path().to_path_buf())?;

    // Define schema
    println!("📊 Creating table schema...");
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("score", DataType::Float64, false),
        Field::new("active", DataType::Boolean, false),
    ]));
    println!("  ✅ Schema defined: id, name, score, active\n");

    // Create table
    println!("📝 Creating 'players' table...");
    catalog.create_table(
        "players".to_string(),
        schema.clone(),
        "id".to_string(),  // Primary key
    )?;
    println!("  ✅ Table created with learned index on 'id'\n");

    // Insert rows directly
    println!("📝 Inserting player data...");
    let table = catalog.get_table_mut("players")?;

    let players = vec![
        (1, "Alice", 95.5, true),
        (2, "Bob", 87.3, true),
        (3, "Charlie", 92.1, false),
        (4, "Diana", 88.9, true),
        (5, "Eve", 94.7, true),
    ];

    for (id, name, score, active) in players {
        let row = Row::new(vec![
            Value::Int64(id),
            Value::Text(name.to_string()),
            Value::Float64(score),
            Value::Boolean(active),
        ]);
        table.insert(row)?;
    }
    println!("  ✅ Inserted {} players\n", table.row_count());

    // Point query using learned index
    println!("🔍 Point query: Get player with id=3");
    let result = table.get(&Value::Int64(3))?;

    if let Some(row) = result {
        println!("  ✅ Found player:");
        println!("     ID: {:?}", row.get(0)?);
        println!("     Name: {:?}", row.get(1)?);
        println!("     Score: {:?}", row.get(2)?);
        println!("     Active: {:?}", row.get(3)?);
    } else {
        println!("  ❌ Player not found");
    }

    // Range query
    println!("\n🔍 Range query: Get players with id 2-4");
    let results = table.range_query(&Value::Int64(2), &Value::Int64(4))?;

    println!("  ✅ Found {} players in range:", results.len());
    for row in results {
        println!(
            "     {:?} - {:?} (score: {:?})",
            row.get(0)?,
            row.get(1)?,
            row.get(2)?
        );
    }

    // Scan all
    println!("\n🔍 Full table scan:");
    let all_rows = table.scan_all()?;

    println!("  ✅ Total players: {}", all_rows.len());
    println!("\n  Leaderboard (by score):");

    let mut sorted_rows = all_rows.clone();
    sorted_rows.sort_by(|a, b| {
        let score_a = if let Value::Float64(s) = a.get(2).unwrap() {
            *s
        } else {
            0.0
        };
        let score_b = if let Value::Float64(s) = b.get(2).unwrap() {
            *s
        } else {
            0.0
        };
        score_b.partial_cmp(&score_a).unwrap()
    });

    for (i, row) in sorted_rows.iter().enumerate() {
        let active = if let Value::Boolean(a) = row.get(3)? {
            if *a {
                "🟢"
            } else {
                "🔴"
            }
        } else {
            "❓"
        };

        println!(
            "    {}. {:?} - Score: {:?} {}",
            i + 1,
            row.get(1)?,
            row.get(2)?,
            active
        );
    }

    // Table statistics
    println!("\n📊 Table Statistics:");
    println!("  Name: {}", table.name());
    println!("  Primary Key: {}", table.primary_key());
    println!("  Rows: {}", table.row_count());
    println!("  Columns: {}", table.schema().fields().len());
    println!("  Index Type: Learned (RMI)");

    // Catalog statistics
    println!("\n📊 Catalog Statistics:");
    let tables = catalog.list_tables();
    println!("  Total Tables: {}", tables.len());
    for table_name in tables {
        let t = catalog.get_table(&table_name)?;
        println!("    - {}: {} rows", table_name, t.row_count());
    }

    println!("\n✅ Demo complete!");
    println!("\n💡 API Features Demonstrated:");
    println!("   • Direct table creation with custom schema");
    println!("   • Inserting rows without SQL");
    println!("   • Point queries using learned index");
    println!("   • Range queries (start to end)");
    println!("   • Full table scans");
    println!("   • Catalog management");

    Ok(())
}