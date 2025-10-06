//! Fair comparison: OmenDB vs SQLite (both on disk)
//!
//! Previous benchmark used in-memory SQLite (unfair advantage).
//! This benchmark uses disk-based SQLite for honest comparison.
//!
//! Run with: cargo run --release --bin benchmark_fair_comparison

use omendb::rocks_storage::RocksStorage;
use rand::{Rng, SeedableRng};
use rusqlite::{Connection, params};
use std::time::Instant;
use tempfile::tempdir;

fn main() {
    println!("=== FAIR COMPARISON: Both on Disk ===\n");

    // Test at production scale
    let scale = 1_000_000;
    println!("Scale: {} keys\n", scale);

    println!("1. Bulk Insert (Write-Heavy)");
    benchmark_bulk_insert(scale);

    println!("\n2. Query Performance (Read-Heavy)");
    benchmark_queries(scale);

    println!("\n3. Mixed Workload (80% read, 20% write)");
    benchmark_mixed(scale);
}

fn benchmark_bulk_insert(n: usize) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let keys: Vec<i64> = (0..n).map(|_| rng.gen()).collect();
    let values: Vec<Vec<u8>> = (0..n).map(|i| format!("value_{}", i).into_bytes()).collect();

    let dir = tempdir().unwrap();

    // OmenDB (RocksDB on disk)
    println!("  OmenDB (RocksDB on disk):");
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
    println!("    Time: {:?}", omen_time);
    println!("    Per-key: {} ns", omen_time.as_nanos() / n as u128);
    println!("    Throughput: {:.0} inserts/sec", n as f64 / omen_time.as_secs_f64());

    // SQLite (disk-based)
    println!("\n  SQLite (disk-based):");
    let start = Instant::now();
    {
        let mut conn = Connection::open(dir.path().join("sqlite.db")).unwrap();

        // Optimize SQLite for bulk insert
        conn.execute_batch("
            PRAGMA journal_mode=WAL;
            PRAGMA synchronous=NORMAL;
        ").unwrap();

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
    println!("    Time: {:?}", sqlite_time);
    println!("    Per-key: {} ns", sqlite_time.as_nanos() / n as u128);
    println!("    Throughput: {:.0} inserts/sec", n as f64 / sqlite_time.as_secs_f64());

    // Comparison
    let speedup = sqlite_time.as_secs_f64() / omen_time.as_secs_f64();
    println!("\n  Result: OmenDB is {:.2}x faster than SQLite", speedup);
}

fn benchmark_queries(n: usize) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let keys: Vec<i64> = (0..n).map(|_| rng.gen()).collect();
    let values: Vec<Vec<u8>> = (0..n).map(|i| format!("value_{}", i).into_bytes()).collect();

    let dir = tempdir().unwrap();

    // Pre-populate OmenDB
    let mut omen_storage = RocksStorage::new(dir.path().join("omendb")).unwrap();
    let entries: Vec<(i64, Vec<u8>)> = keys.iter()
        .zip(values.iter())
        .map(|(&k, v)| (k, v.clone()))
        .collect();
    omen_storage.insert_batch(entries).unwrap();

    // Pre-populate SQLite
    let mut sqlite_conn = Connection::open(dir.path().join("sqlite.db")).unwrap();
    sqlite_conn.execute_batch("
        PRAGMA journal_mode=WAL;
        PRAGMA synchronous=NORMAL;
    ").unwrap();
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

    // Run queries
    let num_queries = 10_000;
    let mut rng = rand::rngs::StdRng::seed_from_u64(100);
    let query_keys: Vec<i64> = (0..num_queries)
        .map(|_| keys[rng.gen_range(0..keys.len())])
        .collect();

    // OmenDB queries (SIMD-optimized)
    println!("  OmenDB (SIMD-optimized ALEX + RocksDB disk):");
    let start = Instant::now();
    let mut hits = 0;
    for &key in &query_keys {
        if omen_storage.point_query(key).unwrap().is_some() {
            hits += 1;
        }
    }
    let omen_time = start.elapsed();
    println!("    Time: {:?}", omen_time);
    println!("    Per-query: {} ns", omen_time.as_nanos() / num_queries);
    println!("    Throughput: {:.0} queries/sec", num_queries as f64 / omen_time.as_secs_f64());
    println!("    Hit rate: {} / {}", hits, num_queries);

    // SQLite queries
    println!("\n  SQLite (B-tree index + disk):");
    let start = Instant::now();
    let mut hits = 0;
    let mut stmt = sqlite_conn.prepare_cached("SELECT value FROM data WHERE key = ?1").unwrap();
    for &key in &query_keys {
        if stmt.query_row(params![key], |_| Ok(())).is_ok() {
            hits += 1;
        }
    }
    let sqlite_time = start.elapsed();
    println!("    Time: {:?}", sqlite_time);
    println!("    Per-query: {} ns", sqlite_time.as_nanos() / num_queries);
    println!("    Throughput: {:.0} queries/sec", num_queries as f64 / sqlite_time.as_secs_f64());
    println!("    Hit rate: {} / {}", hits, num_queries);

    // Comparison
    let speedup = sqlite_time.as_secs_f64() / omen_time.as_secs_f64();
    if speedup > 1.0 {
        println!("\n  Result: OmenDB is {:.2}x faster than SQLite", speedup);
    } else {
        println!("\n  Result: SQLite is {:.2}x faster than OmenDB", 1.0 / speedup);
    }
}

fn benchmark_mixed(n: usize) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let keys: Vec<i64> = (0..n).map(|_| rng.gen()).collect();
    let values: Vec<Vec<u8>> = (0..n).map(|i| format!("value_{}", i).into_bytes()).collect();

    let dir = tempdir().unwrap();

    // Pre-populate OmenDB
    let mut omen_storage = RocksStorage::new(dir.path().join("omendb")).unwrap();
    let entries: Vec<(i64, Vec<u8>)> = keys.iter()
        .zip(values.iter())
        .map(|(&k, v)| (k, v.clone()))
        .collect();
    omen_storage.insert_batch(entries).unwrap();

    // Pre-populate SQLite
    let mut sqlite_conn = Connection::open(dir.path().join("sqlite.db")).unwrap();
    sqlite_conn.execute_batch("
        PRAGMA journal_mode=WAL;
        PRAGMA synchronous=NORMAL;
    ").unwrap();
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
            // Read (80%)
            let key = keys[rng.gen_range(0..keys.len())];
            if omen_storage.point_query(key).unwrap().is_some() {
                hits += 1;
            }
        } else {
            // Write (20%)
            let key = rng.gen::<i64>();
            let value = vec![1u8, 2u8, 3u8];
            omen_storage.insert(key, &value).unwrap();
        }
    }
    let omen_time = start.elapsed();
    println!("    Time: {:?}", omen_time);
    println!("    Per-op: {} ns", omen_time.as_nanos() / num_ops);
    println!("    Throughput: {:.0} ops/sec", num_ops as f64 / omen_time.as_secs_f64());

    // SQLite
    println!("\n  SQLite:");
    let mut rng = rand::rngs::StdRng::seed_from_u64(100); // Same seed
    let start = Instant::now();
    let mut hits = 0;
    for _ in 0..num_ops {
        if rng.gen::<f64>() < 0.8 {
            // Read (80%)
            let key = keys[rng.gen_range(0..keys.len())];
            let mut stmt = sqlite_conn.prepare_cached("SELECT value FROM data WHERE key = ?1").unwrap();
            if stmt.query_row(params![key], |_| Ok(())).is_ok() {
                hits += 1;
            }
        } else {
            // Write (20%)
            let key = rng.gen::<i64>();
            let value = vec![1u8, 2u8, 3u8];
            sqlite_conn.execute(
                "INSERT OR REPLACE INTO data (key, value) VALUES (?1, ?2)",
                params![key, value],
            ).unwrap();
        }
    }
    let sqlite_time = start.elapsed();
    println!("    Time: {:?}", sqlite_time);
    println!("    Per-op: {} ns", sqlite_time.as_nanos() / num_ops);
    println!("    Throughput: {:.0} ops/sec", num_ops as f64 / sqlite_time.as_secs_f64());

    // Comparison
    let speedup = sqlite_time.as_secs_f64() / omen_time.as_secs_f64();
    if speedup > 1.0 {
        println!("\n  Result: OmenDB is {:.2}x faster than SQLite", speedup);
    } else {
        println!("\n  Result: SQLite is {:.2}x faster than OmenDB", 1.0 / speedup);
    }
}
