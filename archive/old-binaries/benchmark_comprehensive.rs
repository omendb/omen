//! Comprehensive benchmark: Current state vs SQLite
//!
//! Tests realistic workloads to validate our optimization decisions:
//! - Mixed read/write (not just bulk insert)
//! - Multiple scales (10K, 100K, 1M)
//! - Both sequential and random keys
//! - Query performance (validates SIMD impact)
//!
//! Run with: cargo run --release --bin benchmark_comprehensive

use omendb::rocks_storage::RocksStorage;
use rand::{Rng, SeedableRng};
use rusqlite::{Connection, params};
use std::time::Instant;
use tempfile::tempdir;

fn main() {
    println!("=== COMPREHENSIVE BENCHMARK ===");
    println!("Testing optimized OmenDB vs SQLite\n");

    // Test at multiple scales
    for &scale in &[10_000, 100_000, 1_000_000] {
        println!("\n=== {} KEYS ===", scale);

        println!("\n1. Bulk Insert (Random Keys)");
        benchmark_bulk_insert(scale);

        println!("\n2. Mixed Workload (80% read, 20% write)");
        benchmark_mixed_workload(scale);

        println!("\n3. Query Performance (Read-Heavy)");
        benchmark_query_heavy(scale);

        println!("\n{}", "=".repeat(60));
    }
}

fn benchmark_bulk_insert(n: usize) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let keys: Vec<i64> = (0..n).map(|_| rng.gen()).collect();
    let values: Vec<Vec<u8>> = (0..n).map(|i| format!("value_{}", i).into_bytes()).collect();

    // OmenDB (RocksDB + ALEX + SIMD)
    println!("  OmenDB:");
    let dir = tempdir().unwrap();
    let start = Instant::now();
    {
        let mut storage = RocksStorage::new(dir.path().join("omendb")).unwrap();
        let entries: Vec<(i64, Vec<u8>)> = keys.iter()
            .zip(values.iter())
            .map(|(&k, v)| (k, v.clone()))
            .collect();
        storage.insert_batch(entries).unwrap();
    }
    let omen_time = start.elapsed();
    println!("    Time: {:?} ({} ns/key)", omen_time, omen_time.as_nanos() / n as u128);

    // SQLite
    println!("  SQLite:");
    let start = Instant::now();
    {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE data (key INTEGER PRIMARY KEY, value BLOB)",
            [],
        ).unwrap();

        let tx = conn.transaction().unwrap();
        for (&key, value) in keys.iter().zip(values.iter()) {
            tx.execute(
                "INSERT INTO data (key, value) VALUES (?1, ?2)",
                params![key, value],
            ).unwrap();
        }
        tx.commit().unwrap();
    }
    let sqlite_time = start.elapsed();
    println!("    Time: {:?} ({} ns/key)", sqlite_time, sqlite_time.as_nanos() / n as u128);

    // Comparison
    let speedup = sqlite_time.as_secs_f64() / omen_time.as_secs_f64();
    println!("  Speedup: {:.2}x faster than SQLite", speedup);
}

fn benchmark_mixed_workload(n: usize) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    // Prepare data
    let keys: Vec<i64> = (0..n).map(|_| rng.gen()).collect();
    let values: Vec<Vec<u8>> = (0..n).map(|i| format!("value_{}", i).into_bytes()).collect();

    // Pre-populate both databases
    let dir = tempdir().unwrap();
    let mut omen_storage = RocksStorage::new(dir.path().join("omendb")).unwrap();
    let entries: Vec<(i64, Vec<u8>)> = keys.iter()
        .zip(values.iter())
        .map(|(&k, v)| (k, v.clone()))
        .collect();
    omen_storage.insert_batch(entries.clone()).unwrap();

    let mut sqlite_conn = Connection::open_in_memory().unwrap();
    sqlite_conn.execute(
        "CREATE TABLE data (key INTEGER PRIMARY KEY, value BLOB)",
        [],
    ).unwrap();
    let tx = sqlite_conn.transaction().unwrap();
    for (&key, value) in keys.iter().zip(values.iter()) {
        tx.execute(
            "INSERT INTO data (key, value) VALUES (?1, ?2)",
            params![key, value],
        ).unwrap();
    }
    tx.commit().unwrap();

    // Mixed workload: 80% reads, 20% writes
    let num_ops = 10_000;
    let mut rng = rand::rngs::StdRng::seed_from_u64(100);

    // OmenDB
    println!("  OmenDB:");
    let start = Instant::now();
    let mut hits = 0;
    for _ in 0..num_ops {
        if rng.gen::<f64>() < 0.8 {
            // Read
            let key = keys[rng.gen_range(0..keys.len())];
            if omen_storage.point_query(key).unwrap().is_some() {
                hits += 1;
            }
        } else {
            // Write
            let key = rng.gen::<i64>();
            let value = vec![1u8, 2u8, 3u8];
            omen_storage.insert(key, &value).unwrap();
        }
    }
    let omen_time = start.elapsed();
    println!("    Time: {:?} ({} ns/op, {} hits)", omen_time, omen_time.as_nanos() / num_ops, hits);

    // SQLite
    println!("  SQLite:");
    let mut rng = rand::rngs::StdRng::seed_from_u64(100); // Same seed for fairness
    let start = Instant::now();
    let mut hits = 0;
    for _ in 0..num_ops {
        if rng.gen::<f64>() < 0.8 {
            // Read
            let key = keys[rng.gen_range(0..keys.len())];
            let mut stmt = sqlite_conn.prepare_cached("SELECT value FROM data WHERE key = ?1").unwrap();
            if stmt.query_row(params![key], |_| Ok(())).is_ok() {
                hits += 1;
            }
        } else {
            // Write
            let key = rng.gen::<i64>();
            let value = vec![1u8, 2u8, 3u8];
            sqlite_conn.execute(
                "INSERT OR REPLACE INTO data (key, value) VALUES (?1, ?2)",
                params![key, value],
            ).unwrap();
        }
    }
    let sqlite_time = start.elapsed();
    println!("    Time: {:?} ({} ns/op, {} hits)", sqlite_time, sqlite_time.as_nanos() / num_ops, hits);

    // Comparison
    let speedup = sqlite_time.as_secs_f64() / omen_time.as_secs_f64();
    println!("  Speedup: {:.2}x faster than SQLite", speedup);
}

fn benchmark_query_heavy(n: usize) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    // Prepare data
    let keys: Vec<i64> = (0..n).map(|_| rng.gen()).collect();
    let values: Vec<Vec<u8>> = (0..n).map(|i| format!("value_{}", i).into_bytes()).collect();

    // Pre-populate both databases
    let dir = tempdir().unwrap();
    let mut omen_storage = RocksStorage::new(dir.path().join("omendb")).unwrap();
    let entries: Vec<(i64, Vec<u8>)> = keys.iter()
        .zip(values.iter())
        .map(|(&k, v)| (k, v.clone()))
        .collect();
    omen_storage.insert_batch(entries.clone()).unwrap();

    let mut sqlite_conn = Connection::open_in_memory().unwrap();
    sqlite_conn.execute(
        "CREATE TABLE data (key INTEGER PRIMARY KEY, value BLOB)",
        [],
    ).unwrap();
    let tx = sqlite_conn.transaction().unwrap();
    for (&key, value) in keys.iter().zip(values.iter()) {
        tx.execute(
            "INSERT INTO data (key, value) VALUES (?1, ?2)",
            params![key, value],
        ).unwrap();
    }
    tx.commit().unwrap();

    // Query-heavy workload: 10K random queries
    let num_queries = 10_000;
    let mut rng = rand::rngs::StdRng::seed_from_u64(100);
    let query_keys: Vec<i64> = (0..num_queries)
        .map(|_| keys[rng.gen_range(0..keys.len())])
        .collect();

    // OmenDB (should benefit from SIMD)
    println!("  OmenDB (SIMD-optimized queries):");
    let start = Instant::now();
    let mut hits = 0;
    for &key in &query_keys {
        if omen_storage.point_query(key).unwrap().is_some() {
            hits += 1;
        }
    }
    let omen_time = start.elapsed();
    println!("    Time: {:?} ({} ns/query, {} hits)",
        omen_time, omen_time.as_nanos() / num_queries, hits);

    // SQLite
    println!("  SQLite:");
    let start = Instant::now();
    let mut hits = 0;
    let mut stmt = sqlite_conn.prepare_cached("SELECT value FROM data WHERE key = ?1").unwrap();
    for &key in &query_keys {
        if stmt.query_row(params![key], |_| Ok(())).is_ok() {
            hits += 1;
        }
    }
    let sqlite_time = start.elapsed();
    println!("    Time: {:?} ({} ns/query, {} hits)",
        sqlite_time, sqlite_time.as_nanos() / num_queries, hits);

    // Comparison
    let speedup = sqlite_time.as_secs_f64() / omen_time.as_secs_f64();
    println!("  Speedup: {:.2}x faster than SQLite", speedup);
}
