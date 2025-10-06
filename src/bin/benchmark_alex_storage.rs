//! Benchmark AlexStorage vs RocksStorage
//!
//! Validates the 10x query improvement projection:
//! - RocksStorage: 3,902 ns/query (ALEX 218ns + RocksDB 3,684ns)
//! - AlexStorage: ~389 ns/query (ALEX 218ns + mmap 151ns)
//!
//! Run with: cargo run --release --bin benchmark_alex_storage

use omendb::alex_storage::AlexStorage;
use omendb::rocks_storage::RocksStorage;
use rand::{Rng, SeedableRng};
use std::time::Instant;
use tempfile::tempdir;

fn main() {
    println!("=== AlexStorage vs RocksStorage Benchmark ===");
    println!("Validating 10x query improvement\n");

    // Get scale from command line or use default
    let args: Vec<String> = std::env::args().collect();
    let scale = if args.len() > 1 {
        args[1].parse::<usize>().unwrap_or(100_000)
    } else {
        100_000
    };
    println!("Scale: {} keys\n", scale);

    println!("1. Bulk Insert Performance");
    benchmark_inserts(scale);

    println!("\n2. Query Performance (Critical Test)");
    benchmark_queries(scale);

    println!("\n3. Mixed Workload (80% read, 20% write)");
    benchmark_mixed(scale);
}

fn benchmark_inserts(n: usize) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let keys: Vec<i64> = (0..n).map(|_| rng.gen()).collect();
    let values: Vec<Vec<u8>> = (0..n).map(|i| format!("value_{}", i).into_bytes()).collect();

    let dir = tempdir().unwrap();

    // AlexStorage
    println!("  AlexStorage:");
    let start = Instant::now();
    {
        let alex_dir = dir.path().join("alex");
        std::fs::create_dir_all(&alex_dir).unwrap();
        let mut storage = AlexStorage::new(&alex_dir).unwrap();
        let entries: Vec<(i64, Vec<u8>)> = keys.iter()
            .zip(values.iter())
            .map(|(&k, v)| (k, v.clone()))
            .collect();
        storage.insert_batch(entries).unwrap();
    }
    let alex_time = start.elapsed();
    println!("    Time: {:?}", alex_time);
    println!("    Per-key: {} ns", alex_time.as_nanos() / n as u128);

    // RocksStorage
    println!("\n  RocksStorage:");
    let start = Instant::now();
    {
        let mut storage = RocksStorage::new(dir.path().join("rocks")).unwrap();
        let entries: Vec<(i64, Vec<u8>)> = keys.iter()
            .zip(values.iter())
            .map(|(&k, v)| (k, v.clone()))
            .collect();
        storage.insert_batch(entries).unwrap();
    }
    let rocks_time = start.elapsed();
    println!("    Time: {:?}", rocks_time);
    println!("    Per-key: {} ns", rocks_time.as_nanos() / n as u128);

    // Comparison
    let speedup = rocks_time.as_secs_f64() / alex_time.as_secs_f64();
    if speedup > 1.0 {
        println!("\n  Result: AlexStorage is {:.2}x faster", speedup);
    } else {
        println!("\n  Result: RocksStorage is {:.2}x faster", 1.0 / speedup);
    }
}

fn benchmark_queries(n: usize) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let keys: Vec<i64> = (0..n).map(|_| rng.gen()).collect();
    let values: Vec<Vec<u8>> = (0..n).map(|i| format!("value_{}", i).into_bytes()).collect();

    let dir = tempdir().unwrap();

    // Pre-populate AlexStorage
    let alex_dir = dir.path().join("alex");
    std::fs::create_dir_all(&alex_dir).unwrap();
    let mut alex_storage = AlexStorage::new(&alex_dir).unwrap();
    let entries: Vec<(i64, Vec<u8>)> = keys.iter()
        .zip(values.iter())
        .map(|(&k, v)| (k, v.clone()))
        .collect();
    alex_storage.insert_batch(entries.clone()).unwrap();

    // Pre-populate RocksStorage
    let mut rocks_storage = RocksStorage::new(dir.path().join("rocks")).unwrap();
    rocks_storage.insert_batch(entries).unwrap();

    // Run queries
    let num_queries = 10_000;
    let mut rng = rand::rngs::StdRng::seed_from_u64(100);
    let query_keys: Vec<i64> = (0..num_queries)
        .map(|_| keys[rng.gen_range(0..keys.len())])
        .collect();

    // AlexStorage queries (mmap-based)
    println!("  AlexStorage (mmap-based):");
    let start = Instant::now();
    let mut hits = 0;
    for &key in &query_keys {
        if alex_storage.get(key).unwrap().is_some() {
            hits += 1;
        }
    }
    let alex_time = start.elapsed();
    println!("    Time: {:?}", alex_time);
    println!("    Per-query: {} ns", alex_time.as_nanos() / num_queries);
    println!("    Throughput: {:.2}M queries/sec", num_queries as f64 / alex_time.as_secs_f64() / 1_000_000.0);
    println!("    Hit rate: {} / {}", hits, num_queries);

    // RocksStorage queries (disk-based)
    println!("\n  RocksStorage (disk-based):");
    let start = Instant::now();
    let mut hits = 0;
    for &key in &query_keys {
        if rocks_storage.point_query(key).unwrap().is_some() {
            hits += 1;
        }
    }
    let rocks_time = start.elapsed();
    println!("    Time: {:?}", rocks_time);
    println!("    Per-query: {} ns", rocks_time.as_nanos() / num_queries);
    println!("    Throughput: {:.2}M queries/sec", num_queries as f64 / rocks_time.as_secs_f64() / 1_000_000.0);
    println!("    Hit rate: {} / {}", hits, num_queries);

    // Comparison
    let speedup = rocks_time.as_secs_f64() / alex_time.as_secs_f64();
    println!("\n  Result: AlexStorage is {:.2}x faster than RocksStorage", speedup);

    // Validation
    if speedup >= 8.0 {
        println!("  ✅ VALIDATED: Exceeded 8x target (projected: 10x)");
    } else if speedup >= 5.0 {
        println!("  ✅ GOOD: Above 5x (projected: 10x)");
    } else if speedup >= 2.0 {
        println!("  ⚠️  ACCEPTABLE: Above 2x but below projection");
    } else {
        println!("  ❌ FAILED: Did not achieve significant improvement");
    }
}

fn benchmark_mixed(n: usize) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let keys: Vec<i64> = (0..n).map(|_| rng.gen()).collect();
    let values: Vec<Vec<u8>> = (0..n).map(|i| format!("value_{}", i).into_bytes()).collect();

    let dir = tempdir().unwrap();

    // Pre-populate AlexStorage
    let alex_dir = dir.path().join("alex");
    std::fs::create_dir_all(&alex_dir).unwrap();
    let mut alex_storage = AlexStorage::new(&alex_dir).unwrap();
    let entries: Vec<(i64, Vec<u8>)> = keys.iter()
        .zip(values.iter())
        .map(|(&k, v)| (k, v.clone()))
        .collect();
    alex_storage.insert_batch(entries.clone()).unwrap();

    // Pre-populate RocksStorage
    let mut rocks_storage = RocksStorage::new(dir.path().join("rocks")).unwrap();
    rocks_storage.insert_batch(entries).unwrap();

    // Mixed workload: 80% reads, 20% writes
    let num_ops = 10_000;
    let mut rng = rand::rngs::StdRng::seed_from_u64(100);

    // AlexStorage
    println!("  AlexStorage:");
    let start = Instant::now();
    for _ in 0..num_ops {
        if rng.gen::<f64>() < 0.8 {
            // Read (80%)
            let key = keys[rng.gen_range(0..keys.len())];
            let _ = alex_storage.get(key).unwrap();
        } else {
            // Write (20%)
            let key = rng.gen::<i64>();
            let value = vec![1u8, 2u8, 3u8];
            alex_storage.insert(key, &value).unwrap();
        }
    }
    let alex_time = start.elapsed();
    println!("    Time: {:?}", alex_time);
    println!("    Per-op: {} ns", alex_time.as_nanos() / num_ops);

    // RocksStorage
    println!("\n  RocksStorage:");
    let mut rng = rand::rngs::StdRng::seed_from_u64(100); // Same seed
    let start = Instant::now();
    for _ in 0..num_ops {
        if rng.gen::<f64>() < 0.8 {
            // Read (80%)
            let key = keys[rng.gen_range(0..keys.len())];
            let _ = rocks_storage.point_query(key).unwrap();
        } else {
            // Write (20%)
            let key = rng.gen::<i64>();
            let value = vec![1u8, 2u8, 3u8];
            rocks_storage.insert(key, &value).unwrap();
        }
    }
    let rocks_time = start.elapsed();
    println!("    Time: {:?}", rocks_time);
    println!("    Per-op: {} ns", rocks_time.as_nanos() / num_ops);

    // Comparison
    let speedup = rocks_time.as_secs_f64() / alex_time.as_secs_f64();
    println!("\n  Result: AlexStorage is {:.2}x faster than RocksStorage", speedup);
}
