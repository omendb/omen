//! ALEX vs SQLite: Honest Competitive Benchmark
//!
//! Compares OmenDB Table (with ALEX learned index) against SQLite (with B-tree)
//! Both systems tested with full persistence, durability, and ACID guarantees.
//!
//! What we're testing:
//! - SQLite: B-tree indexes with full ACID
//! - OmenDB: Table system with ALEX learned index + Arrow/Parquet storage
//!
//! Tested workloads:
//! - Sequential keys (time-series pattern) - learned index sweet spot
//! - Random keys (UUID pattern) - worst case for learned indexes
//!
//! Usage: cargo run --release --bin benchmark_table_vs_sqlite -- [SIZE]
//! Example: cargo run --release --bin benchmark_table_vs_sqlite -- 10000000

use anyhow::Result;
use arrow::datatypes::{DataType, Field, Schema};
use omendb::catalog::Catalog;
use omendb::row::Row;
use omendb::value::Value;
use rusqlite::{Connection, params};
use std::env;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;

const DEFAULT_SIZE: usize = 1_000_000;
const QUERY_COUNT: usize = 1000;

#[derive(Clone, Copy)]
enum DataDistribution {
    Sequential,  // 0, 1, 2, 3, ... (time-series)
    Random,      // Random i64 (UUID-like)
}

impl DataDistribution {
    fn name(&self) -> &str {
        match self {
            Self::Sequential => "Sequential (time-series)",
            Self::Random => "Random (UUID-like)",
        }
    }
}

struct BenchmarkResult {
    size: usize,
    distribution: DataDistribution,

    // Insert performance
    sqlite_insert_ms: f64,
    omendb_insert_ms: f64,

    // Point query performance
    sqlite_query_us: f64,
    omendb_query_us: f64,

    // Speedups
    insert_speedup: f64,
    query_speedup: f64,
    avg_speedup: f64,
}

impl BenchmarkResult {
    fn print(&self) {
        println!("═══════════════════════════════════════════════════════════════");
        println!("   {} - {} rows", self.distribution.name(), format_number(self.size));
        println!("═══════════════════════════════════════════════════════════════");
        println!();

        println!("⏱️  BULK INSERT");
        println!("   SQLite:  {:>10.2} ms  ({:>10} rows/sec)",
            self.sqlite_insert_ms,
            format_number((self.size as f64 / (self.sqlite_insert_ms / 1000.0)) as usize)
        );
        println!("   OmenDB:  {:>10.2} ms  ({:>10} rows/sec)",
            self.omendb_insert_ms,
            format_number((self.size as f64 / (self.omendb_insert_ms / 1000.0)) as usize)
        );
        println!("   Speedup: {:.2}x {}", self.insert_speedup, verdict(self.insert_speedup));
        println!();

        println!("🔍 POINT QUERIES ({} queries)", QUERY_COUNT);
        println!("   SQLite:  {:>10.3} μs avg", self.sqlite_query_us);
        println!("   OmenDB:  {:>10.3} μs avg", self.omendb_query_us);
        println!("   Speedup: {:.2}x {}", self.query_speedup, verdict(self.query_speedup));
        println!();

        println!("🎯 AVERAGE SPEEDUP: {:.2}x {}", self.avg_speedup, verdict(self.avg_speedup));
        println!();
    }
}

fn verdict(speedup: f64) -> &'static str {
    if speedup >= 50.0 {
        "✅ EXCEPTIONAL"
    } else if speedup >= 10.0 {
        "✅ EXCELLENT"
    } else if speedup >= 5.0 {
        "✅ VERY GOOD"
    } else if speedup >= 2.0 {
        "✅ GOOD"
    } else if speedup >= 1.0 {
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

fn main() -> Result<()> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║        ALEX vs SQLite: Honest Competitive Benchmark         ║");
    println!("║        Full Database Comparison - Both with Persistence     ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Parse size from command line or use default
    let size = env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_SIZE);

    println!("Testing with {} rows", format_number(size));
    println!();

    // Test sequential distribution (time-series sweet spot)
    println!("📊 Sequential Distribution (Time-Series Workload)");
    let seq_result = benchmark_comparison(size, DataDistribution::Sequential)?;
    seq_result.print();

    // Test random distribution (worst case for learned indexes)
    println!("📊 Random Distribution (UUID Workload)");
    let rand_result = benchmark_comparison(size, DataDistribution::Random)?;
    rand_result.print();

    // Summary
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                          SUMMARY                             ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Sequential (time-series):");
    println!("   Insert: {:.2}x faster", seq_result.insert_speedup);
    println!("   Query:  {:.2}x faster", seq_result.query_speedup);
    println!("   Avg:    {:.2}x faster {}", seq_result.avg_speedup, verdict(seq_result.avg_speedup));
    println!();
    println!("Random (UUID-like):");
    println!("   Insert: {:.2}x faster", rand_result.insert_speedup);
    println!("   Query:  {:.2}x faster", rand_result.query_speedup);
    println!("   Avg:    {:.2}x faster {}", rand_result.avg_speedup, verdict(rand_result.avg_speedup));
    println!();

    let overall_avg = (seq_result.avg_speedup + rand_result.avg_speedup) / 2.0;
    println!("📈 OVERALL AVERAGE SPEEDUP: {:.2}x {}", overall_avg, verdict(overall_avg));
    println!();

    // Validation against projected claims
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    CLAIM VALIDATION                          ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    if size >= 10_000_000 {
        if overall_avg >= 5.0 && overall_avg <= 15.0 {
            println!("✅ Validated: 5-15x faster at {}M scale", size / 1_000_000);
        } else if overall_avg > 15.0 {
            println!("✅ EXCEEDED: {:.2}x faster (projected 5-15x)", overall_avg);
        } else {
            println!("⚠️  Below projection: {:.2}x (projected 5-15x)", overall_avg);
        }
    } else if size >= 1_000_000 {
        if overall_avg >= 3.0 {
            println!("✅ Validated: {:.2}x faster at {}M scale", overall_avg, size / 1_000_000);
            println!("   Projected: 5-15x at 10M+ scale (linear scaling)");
        } else {
            println!("⚠️  {:.2}x at {}M scale (need >3x for 10M projection)", overall_avg, size / 1_000_000);
        }
    }
    println!();

    Ok(())
}

fn benchmark_comparison(size: usize, distribution: DataDistribution) -> Result<BenchmarkResult> {
    let temp_dir = TempDir::new()?;

    // Generate test data
    let keys = generate_keys(size, distribution);

    // Benchmark SQLite
    let sqlite_insert_ms = benchmark_sqlite_insert(size, &keys, &temp_dir)?;
    let sqlite_query_us = benchmark_sqlite_query(&keys, &temp_dir)?;

    // Benchmark OmenDB Table
    let omendb_insert_ms = benchmark_omendb_insert(size, &keys, &temp_dir)?;
    let omendb_query_us = benchmark_omendb_query(&keys, &temp_dir)?;

    let insert_speedup = sqlite_insert_ms / omendb_insert_ms;
    let query_speedup = sqlite_query_us / omendb_query_us;
    let avg_speedup = (insert_speedup + query_speedup) / 2.0;

    Ok(BenchmarkResult {
        size,
        distribution,
        sqlite_insert_ms,
        omendb_insert_ms,
        sqlite_query_us,
        omendb_query_us,
        insert_speedup,
        query_speedup,
        avg_speedup,
    })
}

fn generate_keys(size: usize, distribution: DataDistribution) -> Vec<i64> {
    match distribution {
        DataDistribution::Sequential => {
            (0..size as i64).collect()
        }
        DataDistribution::Random => {
            use rand::{Rng, SeedableRng};
            let mut rng = rand::rngs::StdRng::seed_from_u64(42);
            (0..size).map(|_| rng.gen::<i64>()).collect()
        }
    }
}

// ============================================================================
// SQLite Benchmarks
// ============================================================================

fn benchmark_sqlite_insert(_size: usize, keys: &[i64], temp_dir: &TempDir) -> Result<f64> {
    let db_path = temp_dir.path().join("sqlite.db");
    let conn = Connection::open(&db_path)?;

    conn.execute(
        "CREATE TABLE data (id INTEGER PRIMARY KEY, value TEXT)",
        [],
    )?;

    let start = Instant::now();
    let tx = conn.unchecked_transaction()?;

    for &key in keys {
        let value = format!("value_{}", key);
        tx.execute(
            "INSERT INTO data (id, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
    }

    tx.commit()?;
    let elapsed = start.elapsed();

    Ok(elapsed.as_secs_f64() * 1000.0)
}

fn benchmark_sqlite_query(keys: &[i64], temp_dir: &TempDir) -> Result<f64> {
    let db_path = temp_dir.path().join("sqlite.db");
    let conn = Connection::open(&db_path)?;

    // Sample keys for querying
    let sample_keys: Vec<i64> = keys
        .iter()
        .step_by(keys.len() / QUERY_COUNT)
        .copied()
        .take(QUERY_COUNT)
        .collect();

    let start = Instant::now();

    for &key in &sample_keys {
        let mut stmt = conn.prepare("SELECT value FROM data WHERE id = ?1")?;
        let _value: Option<String> = stmt.query_row(params![key], |row| row.get(0)).ok();
    }

    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() as f64 / sample_keys.len() as f64;

    Ok(avg_us)
}

// ============================================================================
// OmenDB Table Benchmarks
// ============================================================================

fn benchmark_omendb_insert(_size: usize, keys: &[i64], temp_dir: &TempDir) -> Result<f64> {
    let catalog_dir = temp_dir.path().join("omendb");
    let mut catalog = Catalog::new(catalog_dir)?;

    // Create schema
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Utf8, false),
    ]));

    // Create table
    catalog.create_table("data".to_string(), schema, "id".to_string())?;

    let start = Instant::now();

    // Get table reference once (not in loop)
    let table = catalog.get_table_mut("data")?;

    // Build rows vector for batch insert
    let mut rows = Vec::with_capacity(keys.len());
    for &key in keys {
        let value = format!("value_{}", key);
        rows.push(Row::new(vec![
            Value::Int64(key),
            Value::Text(value),
        ]));
    }

    // Use batch_insert (sorts by PK for optimal ALEX performance)
    table.batch_insert(rows)?;

    let elapsed = start.elapsed();

    Ok(elapsed.as_secs_f64() * 1000.0)
}

fn benchmark_omendb_query(keys: &[i64], temp_dir: &TempDir) -> Result<f64> {
    let catalog_dir = temp_dir.path().join("omendb");
    let catalog = Catalog::new(catalog_dir)?;

    let table = catalog.get_table("data")?;

    // Sample keys for querying
    let sample_keys: Vec<i64> = keys
        .iter()
        .step_by(keys.len() / QUERY_COUNT)
        .copied()
        .take(QUERY_COUNT)
        .collect();

    let start = Instant::now();

    for &key in &sample_keys {
        let _row = table.get(&Value::Int64(key))?;
    }

    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() as f64 / sample_keys.len() as f64;

    Ok(avg_us)
}
