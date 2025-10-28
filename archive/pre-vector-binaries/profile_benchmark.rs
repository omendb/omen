//! Profiling benchmark to identify bottlenecks
//!
//! This benchmark isolates specific operations to measure time spent in:
//! - RocksDB write operations
//! - ALEX tree operations (insert, search)
//! - Serialization/deserialization
//! - Memory allocations
//!
//! Run with: cargo flamegraph --bin profile_benchmark

use omen::alex::AlexTree;
use omen::rocks_storage::RocksStorage;
use rocksdb::{DB, Options, WriteBatch};
use std::time::Instant;
use tempfile::tempdir;

fn main() {
    println!("=== PROFILING BENCHMARK ===");
    println!("Generating flamegraph...");

    // Test configuration
    const SCALE: usize = 1_000_000; // 1M for realistic profiling

    // Test both sequential and random
    println!("\n1. Profiling SEQUENTIAL workload ({}  keys)...", SCALE);
    profile_sequential(SCALE);

    println!("\n2. Profiling RANDOM workload ({} keys)...", SCALE);
    profile_random(SCALE);

    println!("\n=== Profile complete. Check flamegraph.svg ===");
}

fn profile_sequential(n: usize) {
    let dir = tempdir().unwrap();

    // Phase 1: Isolate RocksDB write performance
    println!("  a) RocksDB-only writes...");
    let start = Instant::now();
    {
        let db_path = dir.path().join("rocksdb_only.db");
        let db = create_rocksdb(&db_path);

        let mut batch = WriteBatch::default();
        for i in 0..n {
            let key = (i as i64).to_be_bytes();
            let value = format!("value_{}", i).into_bytes();
            batch.put(key, &value);
        }
        db.write(batch).unwrap();
    }
    let rocksdb_time = start.elapsed();
    println!("     RocksDB: {:?}", rocksdb_time);

    // Phase 2: Isolate ALEX tree performance
    println!("  b) ALEX-only inserts...");
    let start = Instant::now();
    {
        let mut alex = AlexTree::new();
        let entries: Vec<(i64, Vec<u8>)> = (0..n)
            .map(|i| (i as i64, vec![1, 2, 3]))
            .collect();
        alex.insert_batch(entries).unwrap();
    }
    let alex_time = start.elapsed();
    println!("     ALEX: {:?}", alex_time);

    // Phase 3: Full RocksStorage (integrated)
    println!("  c) RocksStorage (RocksDB + ALEX)...");
    let start = Instant::now();
    {
        let db_path = dir.path().join("rocks_storage.db");
        let mut storage = RocksStorage::new(&db_path).unwrap();

        let entries: Vec<(i64, Vec<u8>)> = (0..n)
            .map(|i| (i as i64, format!("value_{}", i).into_bytes()))
            .collect();

        storage.insert_batch(entries).unwrap();
    }
    let full_time = start.elapsed();
    println!("     Full: {:?}", full_time);

    // Analysis
    println!("  d) Time breakdown:");
    println!("     RocksDB:  {:?} ({:.1}%)", rocksdb_time,
        100.0 * rocksdb_time.as_secs_f64() / full_time.as_secs_f64());
    println!("     ALEX:     {:?} ({:.1}%)", alex_time,
        100.0 * alex_time.as_secs_f64() / full_time.as_secs_f64());
    println!("     Full:     {:?} (100%)", full_time);

    let overhead = full_time.saturating_sub(rocksdb_time + alex_time);
    println!("     Overhead: {:?} ({:.1}%)", overhead,
        100.0 * overhead.as_secs_f64() / full_time.as_secs_f64());
}

fn profile_random(n: usize) {
    use rand::{Rng, SeedableRng};
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    let dir = tempdir().unwrap();

    // Generate random keys
    let random_keys: Vec<i64> = (0..n).map(|_| rng.gen::<i64>()).collect();

    // Phase 1: RocksDB-only writes (random)
    println!("  a) RocksDB-only writes (random)...");
    let start = Instant::now();
    {
        let db_path = dir.path().join("rocksdb_random.db");
        let db = create_rocksdb(&db_path);

        let mut batch = WriteBatch::default();
        for &key in &random_keys {
            let key_bytes = key.to_be_bytes();
            let value = format!("value_{}", key).into_bytes();
            batch.put(key_bytes, &value);
        }
        db.write(batch).unwrap();
    }
    let rocksdb_time = start.elapsed();
    println!("     RocksDB: {:?}", rocksdb_time);

    // Phase 2: ALEX-only inserts (random)
    println!("  b) ALEX-only inserts (random)...");
    let start = Instant::now();
    {
        let mut alex = AlexTree::new();
        let entries: Vec<(i64, Vec<u8>)> = random_keys.iter()
            .map(|&k| (k, vec![1, 2, 3]))
            .collect();
        alex.insert_batch(entries).unwrap();
    }
    let alex_time = start.elapsed();
    println!("     ALEX: {:?}", alex_time);

    // Phase 3: Full RocksStorage (random)
    println!("  c) RocksStorage (random)...");
    let start = Instant::now();
    {
        let db_path = dir.path().join("rocks_storage_random.db");
        let mut storage = RocksStorage::new(&db_path).unwrap();

        let entries: Vec<(i64, Vec<u8>)> = random_keys.iter()
            .map(|&k| (k, format!("value_{}", k).into_bytes()))
            .collect();

        storage.insert_batch(entries).unwrap();
    }
    let full_time = start.elapsed();
    println!("     Full: {:?}", full_time);

    // Analysis
    println!("  d) Time breakdown:");
    println!("     RocksDB:  {:?} ({:.1}%)", rocksdb_time,
        100.0 * rocksdb_time.as_secs_f64() / full_time.as_secs_f64());
    println!("     ALEX:     {:?} ({:.1}%)", alex_time,
        100.0 * alex_time.as_secs_f64() / full_time.as_secs_f64());
    println!("     Full:     {:?} (100%)", full_time);

    let overhead = full_time.saturating_sub(rocksdb_time + alex_time);
    println!("     Overhead: {:?} ({:.1}%)", overhead,
        100.0 * overhead.as_secs_f64() / full_time.as_secs_f64());
}

fn create_rocksdb(path: &std::path::Path) -> DB {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.set_write_buffer_size(64 * 1024 * 1024);
    opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
    DB::open(&opts, path).unwrap()
}
