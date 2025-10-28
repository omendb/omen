//! Direct comparison of multi-level ALEX vs SQLite at 50M scale
//!
//! Tests whether multi-level architecture fixes the performance
//! regression we saw at 50M+ rows.

use anyhow::Result;
use omendb::alex::MultiLevelAlexTree;
use rand::prelude::*;
use rusqlite::{Connection, params};
use std::time::Instant;

fn main() -> Result<()> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     Multi-Level ALEX vs SQLite at 50M Scale                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let size = 50_000_000;

    // Generate test data
    println!("🔄 Generating {} test keys...", size);
    let mut data = Vec::with_capacity(size);
    let mut rng = thread_rng();

    for _ in 0..size {
        data.push(rng.gen::<i64>());
    }

    // Sort for bulk loading
    data.sort();

    // Prepare data with values
    let data_with_values: Vec<(i64, Vec<u8>)> = data
        .iter()
        .map(|&k| (k, vec![0u8; 8]))
        .collect();

    // Build multi-level ALEX
    println!("\n📦 Building multi-level ALEX tree...");
    let start = Instant::now();
    let multi_tree = MultiLevelAlexTree::bulk_build(data_with_values.clone())?;
    let alex_build_time = start.elapsed();
    println!("  Build time: {:.2}s", alex_build_time.as_secs_f64());
    println!("  Height: {}", multi_tree.height());
    println!("  Leaves: {}", multi_tree.num_leaves());

    // Build SQLite
    println!("\n📦 Building SQLite database...");
    let conn = Connection::open_in_memory()?;

    conn.execute(
        "CREATE TABLE bench (
            key INTEGER PRIMARY KEY,
            value BLOB
        )",
        [],
    )?;

    let start = Instant::now();
    let tx = conn.unchecked_transaction()?;
    {
        let mut stmt = tx.prepare("INSERT INTO bench (key, value) VALUES (?1, ?2)")?;
        for (key, value) in &data_with_values {
            stmt.execute(params![key, value])?;
        }
    }
    tx.commit()?;
    let sqlite_build_time = start.elapsed();
    println!("  Build time: {:.2}s", sqlite_build_time.as_secs_f64());

    // Create index
    println!("  Creating index...");
    let start = Instant::now();
    conn.execute("CREATE INDEX idx_key ON bench(key)", [])?;
    let index_time = start.elapsed();
    println!("  Index time: {:.2}s", index_time.as_secs_f64());

    // Generate query keys
    let query_keys: Vec<i64> = data.choose_multiple(&mut rng, 10_000)
        .copied()
        .collect();

    // Benchmark multi-level ALEX queries
    println!("\n🔍 Testing multi-level ALEX queries...");
    let start = Instant::now();
    let mut alex_found = 0;
    for &key in &query_keys {
        if multi_tree.get(key)?.is_some() {
            alex_found += 1;
        }
    }
    let alex_query_time = start.elapsed();
    let alex_query_avg = alex_query_time.as_nanos() as f64 / query_keys.len() as f64;

    println!("  Total time: {:.2}ms", alex_query_time.as_millis());
    println!("  Avg per query: {:.1}ns", alex_query_avg);
    println!("  Found: {}/{}", alex_found, query_keys.len());

    // Benchmark SQLite queries
    println!("\n🔍 Testing SQLite queries...");
    let mut stmt = conn.prepare("SELECT value FROM bench WHERE key = ?1")?;

    let start = Instant::now();
    let mut sqlite_found = 0;
    for &key in &query_keys {
        if stmt.query_row(params![key], |_row| Ok(())).is_ok() {
            sqlite_found += 1;
        }
    }
    let sqlite_query_time = start.elapsed();
    let sqlite_query_avg = sqlite_query_time.as_nanos() as f64 / query_keys.len() as f64;

    println!("  Total time: {:.2}ms", sqlite_query_time.as_millis());
    println!("  Avg per query: {:.1}ns", sqlite_query_avg);
    println!("  Found: {}/{}", sqlite_found, query_keys.len());

    // Compare results
    println!("\n📈 Performance Comparison:");
    println!("  Build speedup: {:.2}x",
             (sqlite_build_time.as_secs_f64() + index_time.as_secs_f64()) / alex_build_time.as_secs_f64());
    println!("  Query speedup: {:.2}x",
             sqlite_query_avg / alex_query_avg);

    if alex_query_avg < sqlite_query_avg {
        println!("  ✅ Multi-level ALEX is FASTER by {:.1}%",
                 ((sqlite_query_avg - alex_query_avg) / sqlite_query_avg) * 100.0);
    } else {
        println!("  ⚠️ SQLite is faster by {:.1}%",
                 ((alex_query_avg - sqlite_query_avg) / alex_query_avg) * 100.0);
    }

    // Test memory usage
    println!("\n💾 Memory Comparison:");
    let alex_memory = multi_tree.num_leaves() * std::mem::size_of::<omendb::alex::GappedNode>();
    println!("  ALEX memory estimate: {:.2} MB", alex_memory as f64 / (1024.0 * 1024.0));
    // SQLite memory is harder to estimate accurately
    println!("  SQLite: In-memory database (size varies)");

    Ok(())
}