//! 10M scale benchmark for RedbStorage with learned indexes
//! Validates that optimizations scale linearly beyond 1M

use anyhow::Result;
use omen::redb_storage::RedbStorage;
use rusqlite::{Connection, params};
use std::time::Instant;
use tempfile::TempDir;

fn main() -> Result<()> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║        10M SCALE BENCHMARK: RedbStorage vs SQLite           ║");
    println!("║        Validating Linear Scaling of Optimizations          ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    benchmark_10m_sequential()?;

    Ok(())
}

fn benchmark_10m_sequential() -> Result<()> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("   Testing with 10,000,000 rows (Sequential)");
    println!("═══════════════════════════════════════════════════════════════");
    println!();

    let temp_dir = TempDir::new()?;

    // SQLite benchmark
    println!("🗄️  Benchmarking SQLite (10M sequential)...");
    let sqlite_insert_ms = benchmark_sqlite_insert(&temp_dir)?;
    let sqlite_point_query_us = benchmark_sqlite_queries(&temp_dir)?;

    // OmenDB benchmark
    println!("⚡ Benchmarking OmenDB (10M sequential)...");
    let omendb_insert_ms = benchmark_omendb_insert(&temp_dir)?;
    let omendb_point_query_us = benchmark_omendb_queries(&temp_dir)?;

    // Print results
    println!("\n📊 RESULTS (10,000,000 rows)");
    println!("─────────────────────────────────────────────────────────────");
    println!();

    let insert_speedup = sqlite_insert_ms / omendb_insert_ms;
    println!("⏱️  BULK INSERT PERFORMANCE");
    println!("   SQLite:  {:>10.2} ms  ({:>10} rows/sec)",
        sqlite_insert_ms,
        format_number((10_000_000.0 / (sqlite_insert_ms / 1000.0)) as usize)
    );
    println!("   OmenDB:  {:>10.2} ms  ({:>10} rows/sec)",
        omendb_insert_ms,
        format_number((10_000_000.0 / (omendb_insert_ms / 1000.0)) as usize)
    );
    println!("   Speedup: {:.2}x {}", insert_speedup, get_verdict(insert_speedup));
    println!();

    let query_speedup = sqlite_point_query_us / omendb_point_query_us;
    println!("🔍 POINT QUERY PERFORMANCE (1000 queries)");
    println!("   SQLite:  {:>8.3} μs avg", sqlite_point_query_us);
    println!("   OmenDB:  {:>8.3} μs avg", omendb_point_query_us);
    println!("   Speedup: {:.2}x {}", query_speedup, get_verdict(query_speedup));
    println!();

    let avg_speedup = (insert_speedup + query_speedup) / 2.0;
    println!("🎯 AVERAGE SPEEDUP: {:.2}x", avg_speedup);

    // Scaling analysis
    println!("\n📈 SCALING ANALYSIS");
    println!("─────────────────────────────────────────────────────────────");
    println!("Expected if linear scaling from 1M:");
    println!("  Insert: ~9000ms (actual: {:.0}ms)", omendb_insert_ms);
    println!("  Query: ~4.9μs (actual: {:.1}μs)", omendb_point_query_us);

    Ok(())
}

fn benchmark_sqlite_insert(temp_dir: &TempDir) -> Result<f64> {
    let db_path = temp_dir.path().join("sqlite_10m_insert.db");
    let conn = Connection::open(&db_path)?;

    conn.execute(
        "CREATE TABLE data (id INTEGER PRIMARY KEY, value BLOB)",
        [],
    )?;

    let start = Instant::now();
    let tx = conn.unchecked_transaction()?;

    for i in 0..10_000_000 {
        let value = format!("value_{}", i);
        tx.execute(
            "INSERT INTO data VALUES (?1, ?2)",
            params![i as i64, value.as_bytes()],
        )?;

        if i % 1_000_000 == 0 && i > 0 {
            println!("  SQLite: {} million rows inserted...", i / 1_000_000);
        }
    }

    tx.commit()?;
    let elapsed = start.elapsed();

    Ok(elapsed.as_secs_f64() * 1000.0)
}

fn benchmark_sqlite_queries(temp_dir: &TempDir) -> Result<f64> {
    let db_path = temp_dir.path().join("sqlite_10m_query.db");
    let conn = Connection::open(&db_path)?;

    conn.execute(
        "CREATE TABLE data (id INTEGER PRIMARY KEY, value BLOB)",
        [],
    )?;

    let tx = conn.unchecked_transaction()?;
    for i in 0..10_000_000 {
        let value = format!("value_{}", i);
        tx.execute(
            "INSERT INTO data VALUES (?1, ?2)",
            params![i as i64, value.as_bytes()],
        )?;
    }
    tx.commit()?;

    // Benchmark queries
    let num_queries = 1000;
    let start = Instant::now();

    for i in 0..num_queries {
        let key = (i * 10_000) as i64;
        let mut stmt = conn.prepare("SELECT value FROM data WHERE id = ?1")?;
        let _: Option<Vec<u8>> = stmt.query_row(params![key], |row| row.get(0)).ok();
    }

    let elapsed = start.elapsed();
    let us_per_query = (elapsed.as_secs_f64() * 1_000_000.0) / num_queries as f64;

    Ok(us_per_query)
}

fn benchmark_omendb_insert(temp_dir: &TempDir) -> Result<f64> {
    let db_path = temp_dir.path().join("omendb_10m_insert.db");
    let mut storage = RedbStorage::new(&db_path)?;

    let start = Instant::now();

    // Insert in batches of 1M for better progress tracking
    for batch_num in 0..10 {
        let batch_start = batch_num * 1_000_000;
        let entries: Vec<(i64, Vec<u8>)> = (batch_start..batch_start + 1_000_000)
            .map(|i| {
                let value = format!("value_{}", i);
                (i as i64, value.into_bytes())
            })
            .collect();

        storage.insert_batch(entries)?;
        println!("  OmenDB: {} million rows inserted...", batch_num + 1);
    }

    let elapsed = start.elapsed();

    Ok(elapsed.as_secs_f64() * 1000.0)
}

fn benchmark_omendb_queries(temp_dir: &TempDir) -> Result<f64> {
    let db_path = temp_dir.path().join("omendb_10m_query.db");
    let mut storage = RedbStorage::new(&db_path)?;

    // Insert 10M rows
    for batch_num in 0..10 {
        let batch_start = batch_num * 1_000_000;
        let entries: Vec<(i64, Vec<u8>)> = (batch_start..batch_start + 1_000_000)
            .map(|i| {
                let value = format!("value_{}", i);
                (i as i64, value.into_bytes())
            })
            .collect();

        storage.insert_batch(entries)?;
    }

    // Benchmark queries
    let num_queries = 1000;
    let start = Instant::now();

    for i in 0..num_queries {
        let key = (i * 10_000) as i64;
        let _ = storage.point_query(key)?;
    }

    let elapsed = start.elapsed();
    let us_per_query = (elapsed.as_secs_f64() * 1_000_000.0) / num_queries as f64;

    Ok(us_per_query)
}

fn get_verdict(speedup: f64) -> &'static str {
    if speedup >= 2.0 {
        "✅ EXCELLENT"
    } else if speedup >= 1.1 {
        "✅ GOOD"
    } else if speedup >= 0.9 {
        "➖ NEUTRAL"
    } else {
        "⚠️  SLOWER"
    }
}

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();

    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }

    result.chars().rev().collect()
}
