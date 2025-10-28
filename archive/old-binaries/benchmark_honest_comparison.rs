//! HONEST Benchmark: OmenDB vs SQLite
//!
//! Full database comparison - both systems have:
//! - Disk persistence
//! - Transaction commits
//! - Durability guarantees
//! - Index maintenance
//!
//! What we're testing:
//! - SQLite: B-tree indexes with full ACID
//! - OmenDB: RocksDB (LSM-tree) + ALEX learned indexes with full ACID
//!
//! Tested workloads:
//! - Sequential keys (time-series pattern) - our sweet spot
//! - Random keys (UUID pattern) - worst case for learned indexes
//!
//! Usage: cargo run --release --bin benchmark_honest_comparison

use anyhow::Result;
use omendb::rocks_storage::RocksStorage;
use rusqlite::{Connection, params};
use std::time::Instant;
use tempfile::TempDir;
use tracing_subscriber::EnvFilter;

const SIZES: &[usize] = &[10_000, 100_000, 1_000_000, 10_000_000];

#[derive(Clone, Copy)]
enum DataDistribution {
    Sequential,  // 0, 1, 2, 3, ... (time-series, auto-increment)
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

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("omendb=info".parse().unwrap()))
        .init();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║        HONEST BENCHMARK: OmenDB vs SQLite                   ║");
    println!("║        Full Database Comparison - Both with Persistence     ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    for &size in SIZES {
        println!("═══════════════════════════════════════════════════════════════");
        println!("   Testing with {} rows", format_number(size));
        println!("═══════════════════════════════════════════════════════════════");
        println!();

        // Test both data distributions
        for distribution in &[DataDistribution::Sequential, DataDistribution::Random] {
            println!("📊 Data Distribution: {}", distribution.name());
            println!("─────────────────────────────────────────────────────────────");

            let result = benchmark_comparison(size, *distribution)?;
            result.print();
            println!();
        }
    }

    Ok(())
}

struct BenchmarkResult {
    size: usize,
    distribution: DataDistribution,
    sqlite_insert_ms: f64,
    omendb_insert_ms: f64,
    sqlite_point_query_us: f64,
    omendb_point_query_us: f64,
}

impl BenchmarkResult {
    fn print(&self) {
        let insert_speedup = self.sqlite_insert_ms / self.omendb_insert_ms;
        let query_speedup = self.sqlite_point_query_us / self.omendb_point_query_us;

        println!("⏱️  BULK INSERT PERFORMANCE");
        println!("   SQLite:  {:>10.2} ms  ({:>10} rows/sec)",
            self.sqlite_insert_ms,
            format_number((self.size as f64 / (self.sqlite_insert_ms / 1000.0)) as usize)
        );
        println!("   OmenDB:  {:>10.2} ms  ({:>10} rows/sec)",
            self.omendb_insert_ms,
            format_number((self.size as f64 / (self.omendb_insert_ms / 1000.0)) as usize)
        );
        println!("   Speedup: {:.2}x {}", insert_speedup, get_verdict(insert_speedup));
        println!();

        println!("🔍 POINT QUERY PERFORMANCE (1000 queries)");
        println!("   SQLite:  {:>10.3} μs avg", self.sqlite_point_query_us);
        println!("   OmenDB:  {:>10.3} μs avg", self.omendb_point_query_us);
        println!("   Speedup: {:.2}x {}", query_speedup, get_verdict(query_speedup));
        println!();

        let avg_speedup = (insert_speedup + query_speedup) / 2.0;
        println!("🎯 AVERAGE SPEEDUP: {:.2}x", avg_speedup);
        println!();
    }
}

fn get_verdict(speedup: f64) -> &'static str {
    if speedup >= 50.0 {
        "✅ EXCEPTIONAL"
    } else if speedup >= 10.0 {
        "✅ EXCELLENT"
    } else if speedup >= 5.0 {
        "✅ VERY GOOD"
    } else if speedup >= 2.0 {
        "✅ GOOD"
    } else if speedup >= 1.0 {
        "➖ NEUTRAL (tie)"
    } else {
        "⚠️  SLOWER"
    }
}

fn benchmark_comparison(size: usize, distribution: DataDistribution) -> Result<BenchmarkResult> {
    let temp_dir = TempDir::new()?;

    // Generate test data
    let keys = generate_keys(size, distribution);

    // Benchmark SQLite
    let sqlite_insert_ms = benchmark_sqlite_insert(size, &keys, &temp_dir)?;
    let sqlite_point_query_us = benchmark_sqlite_point_query(&keys, &temp_dir)?;

    // Benchmark OmenDB
    let omendb_insert_ms = benchmark_omendb_insert(size, &keys, &temp_dir)?;
    let omendb_point_query_us = benchmark_omendb_point_query(&keys, &temp_dir)?;

    Ok(BenchmarkResult {
        size,
        distribution,
        sqlite_insert_ms,
        omendb_insert_ms,
        sqlite_point_query_us,
        omendb_point_query_us,
    })
}

fn generate_keys(size: usize, distribution: DataDistribution) -> Vec<i64> {
    match distribution {
        DataDistribution::Sequential => {
            (0..size as i64).collect()
        }
        DataDistribution::Random => {
            use rand::{Rng, SeedableRng};
            let mut rng = rand::rngs::StdRng::seed_from_u64(42); // Deterministic
            (0..size).map(|_| rng.gen::<i64>()).collect()
        }
    }
}

// SQLite Benchmarks
fn benchmark_sqlite_insert(size: usize, keys: &[i64], temp_dir: &TempDir) -> Result<f64> {
    let db_path = temp_dir.path().join("sqlite_insert.db");
    let conn = Connection::open(&db_path)?;

    conn.execute(
        "CREATE TABLE data (id INTEGER PRIMARY KEY, value BLOB)",
        [],
    )?;

    let start = Instant::now();
    let tx = conn.unchecked_transaction()?;

    for &key in keys {
        let value = format!("value_{}", key);
        tx.execute(
            "INSERT INTO data (id, value) VALUES (?1, ?2)",
            params![key, value.as_bytes()],
        )?;
    }

    tx.commit()?;
    let elapsed = start.elapsed();

    Ok(elapsed.as_secs_f64() * 1000.0)
}

fn benchmark_sqlite_point_query(keys: &[i64], temp_dir: &TempDir) -> Result<f64> {
    let db_path = temp_dir.path().join("sqlite_query.db");
    let conn = Connection::open(&db_path)?;

    conn.execute(
        "CREATE TABLE data (id INTEGER PRIMARY KEY, value BLOB)",
        [],
    )?;

    let tx = conn.unchecked_transaction()?;
    for &key in keys {
        let value = format!("value_{}", key);
        tx.execute(
            "INSERT INTO data VALUES (?1, ?2)",
            params![key, value.as_bytes()],
        )?;
    }
    tx.commit()?;

    // Benchmark queries
    let num_queries = 1000.min(keys.len());
    let query_keys: Vec<i64> = keys.iter()
        .step_by(keys.len() / num_queries)
        .copied()
        .take(num_queries)
        .collect();

    let start = Instant::now();

    for &key in &query_keys {
        let mut stmt = conn.prepare("SELECT value FROM data WHERE id = ?1")?;
        let _: Option<Vec<u8>> = stmt.query_row(params![key], |row| row.get(0)).ok();
    }

    let elapsed = start.elapsed();
    let us_per_query = (elapsed.as_secs_f64() * 1_000_000.0) / num_queries as f64;

    Ok(us_per_query)
}

// OmenDB Benchmarks
fn benchmark_omendb_insert(size: usize, keys: &[i64], temp_dir: &TempDir) -> Result<f64> {
    let db_path = temp_dir.path().join("omendb_insert.db");
    let mut storage = RocksStorage::new(&db_path)?;

    let start = Instant::now();

    let entries: Vec<(i64, Vec<u8>)> = keys.iter()
        .map(|&key| {
            let value = format!("value_{}", key);
            (key, value.into_bytes())
        })
        .collect();

    storage.insert_batch(entries)?;

    let elapsed = start.elapsed();

    Ok(elapsed.as_secs_f64() * 1000.0)
}

fn benchmark_omendb_point_query(keys: &[i64], temp_dir: &TempDir) -> Result<f64> {
    let db_path = temp_dir.path().join("omendb_query.db");
    let mut storage = RocksStorage::new(&db_path)?;

    let entries: Vec<(i64, Vec<u8>)> = keys.iter()
        .map(|&key| {
            let value = format!("value_{}", key);
            (key, value.into_bytes())
        })
        .collect();

    storage.insert_batch(entries)?;

    // Benchmark queries
    let num_queries = 1000.min(keys.len());
    let query_keys: Vec<i64> = keys.iter()
        .step_by(keys.len() / num_queries)
        .copied()
        .take(num_queries)
        .collect();

    let start = Instant::now();

    for &key in &query_keys {
        let _ = storage.point_query(key)?;
    }

    let elapsed = start.elapsed();
    let us_per_query = (elapsed.as_secs_f64() * 1_000_000.0) / num_queries as f64;

    // Print cache statistics
    let (cache_hits, cache_misses, hit_rate) = storage.cache_stats();
    if cache_hits + cache_misses > 0 {
        println!("   📊 Cache Stats: {:.1}% hit rate ({} hits, {} misses)",
            hit_rate * 100.0, format_number(cache_hits as usize), format_number(cache_misses as usize));
    }

    Ok(us_per_query)
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
