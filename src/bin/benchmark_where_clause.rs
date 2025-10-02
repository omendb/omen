//! WHERE Clause Performance Benchmark
//! Validates that learned index optimization actually provides speedup

use anyhow::Result;
use omendb::catalog::Catalog;
use omendb::sql_engine::{ExecutionResult, SqlEngine};
use std::time::Instant;
use tempfile::TempDir;

fn main() -> Result<()> {
    println!("🔬 WHERE Clause Performance Benchmark");
    println!("{}", "=".repeat(60));
    println!();

    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;
    let mut engine = SqlEngine::new(catalog);

    // Create table
    println!("📊 Creating table with 100,000 rows...");
    engine.execute(
        "CREATE TABLE data (
            id BIGINT PRIMARY KEY,
            value DOUBLE,
            category VARCHAR(50)
        )",
    )?;

    // Insert 100K rows (sequential keys - optimal for learned index)
    let start = Instant::now();
    for i in 0..100_000 {
        let sql = format!(
            "INSERT INTO data VALUES ({}, {}, 'category_{}')",
            i,
            i as f64 * 1.5,
            i % 10
        );
        engine.execute(&sql)?;
    }
    let insert_time = start.elapsed();
    println!("  ✅ Inserted 100K rows in {:?}", insert_time);
    println!();

    // Benchmark 1: Point query (should use learned index)
    println!("🔍 Benchmark 1: Point Query (WHERE id = X)");
    println!("{}", "-".repeat(60));

    let queries = vec![1000, 25000, 50000, 75000, 99000];
    let mut total_time = std::time::Duration::ZERO;

    for &id in &queries {
        let start = Instant::now();
        let result = engine.execute(&format!("SELECT * FROM data WHERE id = {}", id))?;
        let duration = start.elapsed();
        total_time += duration;

        if let ExecutionResult::Selected { rows, .. } = result {
            println!("  Query id={}: {} row in {:?}", id, rows, duration);
        }
    }

    let avg_point_query = total_time / queries.len() as u32;
    println!("  Average point query: {:?}", avg_point_query);
    println!();

    // Benchmark 2: Small range query (should use learned index)
    println!("🔍 Benchmark 2: Small Range Query (WHERE id > X AND id < Y)");
    println!("{}", "-".repeat(60));

    let start = Instant::now();
    let result = engine.execute("SELECT * FROM data WHERE id > 50000 AND id < 50100")?;
    let range_time = start.elapsed();

    if let ExecutionResult::Selected { rows, .. } = result {
        println!("  Range [50000, 50100]: {} rows in {:?}", rows, range_time);
    }

    let start = Instant::now();
    let result = engine.execute("SELECT * FROM data WHERE id > 10000 AND id < 15000")?;
    let large_range_time = start.elapsed();

    if let ExecutionResult::Selected { rows, .. } = result {
        println!(
            "  Range [10000, 15000]: {} rows in {:?}",
            rows, large_range_time
        );
    }
    println!();

    // Benchmark 3: Greater than query
    println!("🔍 Benchmark 3: Greater Than Query (WHERE id > X)");
    println!("{}", "-".repeat(60));

    let start = Instant::now();
    let result = engine.execute("SELECT * FROM data WHERE id > 95000")?;
    let gt_time = start.elapsed();

    if let ExecutionResult::Selected { rows, .. } = result {
        println!("  id > 95000: {} rows in {:?}", rows, gt_time);
    }
    println!();

    // Benchmark 4: Less than query
    println!("🔍 Benchmark 4: Less Than Query (WHERE id < X)");
    println!("{}", "-".repeat(60));

    let start = Instant::now();
    let result = engine.execute("SELECT * FROM data WHERE id < 5000")?;
    let lt_time = start.elapsed();

    if let ExecutionResult::Selected { rows, .. } = result {
        println!("  id < 5000: {} rows in {:?}", rows, lt_time);
    }
    println!();

    // Benchmark 5: Non-primary-key query (full scan expected)
    println!("🔍 Benchmark 5: Non-Primary-Key Query (scan + filter)");
    println!("{}", "-".repeat(60));

    let start = Instant::now();
    let result = engine.execute("SELECT * FROM data WHERE category = 'category_5'")?;
    let scan_time = start.elapsed();

    if let ExecutionResult::Selected { rows, .. } = result {
        println!(
            "  category = 'category_5': {} rows in {:?}",
            rows, scan_time
        );
    }
    println!();

    // Benchmark 6: Full table scan for comparison
    println!("🔍 Benchmark 6: Full Table Scan (SELECT *)");
    println!("{}", "-".repeat(60));

    let start = Instant::now();
    let result = engine.execute("SELECT * FROM data")?;
    let full_scan_time = start.elapsed();

    if let ExecutionResult::Selected { rows, .. } = result {
        println!("  SELECT *: {} rows in {:?}", rows, full_scan_time);
    }
    println!();

    // Summary
    println!("{}", "=".repeat(60));
    println!("📈 Performance Summary");
    println!("{}", "=".repeat(60));
    println!();
    println!("Point queries (learned index): {:?} avg", avg_point_query);
    println!("Small range query: {:?}", range_time);
    println!("Large range query: {:?}", large_range_time);
    println!("Greater than query: {:?}", gt_time);
    println!("Less than query: {:?}", lt_time);
    println!("Non-PK scan + filter: {:?}", scan_time);
    println!("Full table scan: {:?}", full_scan_time);
    println!();

    // Analysis
    println!("🎯 Analysis:");

    // Point query should be MUCH faster than full scan
    let point_speedup = full_scan_time.as_micros() as f64 / avg_point_query.as_micros() as f64;
    println!("  Point query vs full scan: {:.2}x faster", point_speedup);

    if point_speedup > 10.0 {
        println!("  ✅ EXCELLENT: Point queries using learned index efficiently");
    } else if point_speedup > 3.0 {
        println!("  ✅ GOOD: Point queries faster than full scan");
    } else {
        println!("  ⚠️  WARNING: Point queries not much faster than full scan");
    }

    // Range queries should be faster than full scan
    let range_speedup = full_scan_time.as_micros() as f64 / range_time.as_micros() as f64;
    println!("  Range query vs full scan: {:.2}x faster", range_speedup);

    if range_speedup > 5.0 {
        println!("  ✅ EXCELLENT: Range queries using learned index efficiently");
    } else if range_speedup > 1.5 {
        println!("  ✅ GOOD: Range queries faster than full scan");
    } else {
        println!("  ⚠️  WARNING: Range queries not much faster than full scan");
    }

    println!();
    println!("✅ Benchmark complete!");

    Ok(())
}
