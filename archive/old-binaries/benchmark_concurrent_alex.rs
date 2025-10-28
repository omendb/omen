//! Benchmark ConcurrentAlexStorage
//!
//! Measures multi-threaded performance to validate concurrency implementation.
//!
//! Tests:
//! 1. Concurrent reads (multiple reader threads)
//! 2. Concurrent writes (multiple writer threads)
//! 3. Mixed workload (80% read, 20% write with multiple threads)
//!
//! Run with: cargo run --release --bin benchmark_concurrent_alex

use omen::alex_storage_concurrent::ConcurrentAlexStorage;
use rand::{Rng, SeedableRng};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use tempfile::tempdir;

fn main() {
    println!("=== ConcurrentAlexStorage Benchmark ===\n");

    let scale = 100_000;
    println!("Scale: {} keys\n", scale);

    println!("1. Single-threaded Baseline");
    benchmark_single_threaded(scale);

    println!("\n2. Concurrent Reads (2, 4, 8 threads)");
    benchmark_concurrent_reads(scale, 2);
    benchmark_concurrent_reads(scale, 4);
    benchmark_concurrent_reads(scale, 8);

    println!("\n3. Concurrent Writes (2, 4 threads)");
    benchmark_concurrent_writes(scale, 2);
    benchmark_concurrent_writes(scale, 4);

    println!("\n4. Mixed Workload (80% read, 20% write - 2, 4 threads)");
    benchmark_mixed_workload(scale, 2);
    benchmark_mixed_workload(scale, 4);
}

fn benchmark_single_threaded(n: usize) {
    let dir = tempdir().unwrap();
    let storage = Arc::new(ConcurrentAlexStorage::new(dir.path()).unwrap());

    // Pre-populate
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let keys: Vec<i64> = (0..n).map(|_| rng.gen()).collect();
    let values: Vec<Vec<u8>> = (0..n).map(|i| format!("value_{}", i).into_bytes()).collect();

    let entries: Vec<(i64, Vec<u8>)> = keys
        .iter()
        .zip(values.iter())
        .map(|(&k, v)| (k, v.clone()))
        .collect();
    storage.insert_batch(entries).unwrap();

    // Benchmark queries
    let num_queries = 10_000;
    let mut rng = rand::rngs::StdRng::seed_from_u64(100);
    let query_keys: Vec<i64> = (0..num_queries)
        .map(|_| keys[rng.gen_range(0..keys.len())])
        .collect();

    let start = Instant::now();
    let mut hits = 0;
    for &key in &query_keys {
        if storage.get(key).unwrap().is_some() {
            hits += 1;
        }
    }
    let elapsed = start.elapsed();

    println!("  Queries: {} ns/query", elapsed.as_nanos() / num_queries);
    println!("  Throughput: {:.2}M queries/sec", num_queries as f64 / elapsed.as_secs_f64() / 1_000_000.0);
    println!("  Hit rate: {} / {}", hits, num_queries);
}

fn benchmark_concurrent_reads(n: usize, num_threads: usize) {
    let dir = tempdir().unwrap();
    let storage = Arc::new(ConcurrentAlexStorage::new(dir.path()).unwrap());

    // Pre-populate
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let keys: Vec<i64> = (0..n).map(|_| rng.gen()).collect();
    let values: Vec<Vec<u8>> = (0..n).map(|i| format!("value_{}", i).into_bytes()).collect();

    let entries: Vec<(i64, Vec<u8>)> = keys
        .iter()
        .zip(values.iter())
        .map(|(&k, v)| (k, v.clone()))
        .collect();
    storage.insert_batch(entries).unwrap();

    // Benchmark concurrent reads
    let queries_per_thread = 10_000 / num_threads;
    let keys = Arc::new(keys);

    let start = Instant::now();

    let mut handles = vec![];
    for thread_id in 0..num_threads {
        let storage_clone = storage.clone();
        let keys_clone = keys.clone();

        let handle = thread::spawn(move || {
            let mut rng = rand::rngs::StdRng::seed_from_u64(100 + thread_id as u64);
            let mut hits = 0;

            for _ in 0..queries_per_thread {
                let key = keys_clone[rng.gen_range(0..keys_clone.len())];
                if storage_clone.get(key).unwrap().is_some() {
                    hits += 1;
                }
            }

            hits
        });

        handles.push(handle);
    }

    let mut total_hits = 0;
    for handle in handles {
        total_hits += handle.join().unwrap();
    }

    let elapsed = start.elapsed();
    let total_queries = queries_per_thread * num_threads;

    println!("\n  {} threads:", num_threads);
    println!("    Queries: {} ns/query", elapsed.as_nanos() / total_queries as u128);
    println!("    Throughput: {:.2}M queries/sec", total_queries as f64 / elapsed.as_secs_f64() / 1_000_000.0);
    println!("    Hit rate: {} / {}", total_hits, total_queries);
    println!("    Speedup: {:.2}x (ideal: {}x)", 10_000.0 / (elapsed.as_nanos() as f64 / total_queries as f64), num_threads);
}

fn benchmark_concurrent_writes(n: usize, num_threads: usize) {
    let dir = tempdir().unwrap();
    let storage = Arc::new(ConcurrentAlexStorage::new(dir.path()).unwrap());

    // Pre-populate
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let keys: Vec<i64> = (0..n).map(|_| rng.gen()).collect();
    let values: Vec<Vec<u8>> = (0..n).map(|i| format!("value_{}", i).into_bytes()).collect();

    let entries: Vec<(i64, Vec<u8>)> = keys
        .iter()
        .zip(values.iter())
        .map(|(&k, v)| (k, v.clone()))
        .collect();
    storage.insert_batch(entries).unwrap();

    // Benchmark concurrent writes
    let writes_per_thread = 1_000 / num_threads;

    let start = Instant::now();

    let mut handles = vec![];
    for thread_id in 0..num_threads {
        let storage_clone = storage.clone();

        let handle = thread::spawn(move || {
            let mut rng = rand::rngs::StdRng::seed_from_u64(200 + thread_id as u64);

            for _ in 0..writes_per_thread {
                let key = rng.gen::<i64>();
                let value = vec![1u8, 2u8, 3u8];
                storage_clone.insert(key, &value).unwrap();
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed = start.elapsed();
    let total_writes = writes_per_thread * num_threads;

    println!("\n  {} threads:", num_threads);
    println!("    Writes: {} ns/write", elapsed.as_nanos() / total_writes as u128);
    println!("    Throughput: {:.2}K writes/sec", total_writes as f64 / elapsed.as_secs_f64() / 1_000.0);
    println!("    Note: Writes are serialized by lock, no speedup expected");
}

fn benchmark_mixed_workload(n: usize, num_threads: usize) {
    let dir = tempdir().unwrap();
    let storage = Arc::new(ConcurrentAlexStorage::new(dir.path()).unwrap());

    // Pre-populate
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let keys: Vec<i64> = (0..n).map(|_| rng.gen()).collect();
    let values: Vec<Vec<u8>> = (0..n).map(|i| format!("value_{}", i).into_bytes()).collect();

    let entries: Vec<(i64, Vec<u8>)> = keys
        .iter()
        .zip(values.iter())
        .map(|(&k, v)| (k, v.clone()))
        .collect();
    storage.insert_batch(entries).unwrap();

    // Benchmark mixed workload (80% read, 20% write)
    let ops_per_thread = 10_000 / num_threads;
    let keys = Arc::new(keys);

    let start = Instant::now();

    let mut handles = vec![];
    for thread_id in 0..num_threads {
        let storage_clone = storage.clone();
        let keys_clone = keys.clone();

        let handle = thread::spawn(move || {
            let mut rng = rand::rngs::StdRng::seed_from_u64(300 + thread_id as u64);
            let mut reads = 0;
            let mut writes = 0;

            for _ in 0..ops_per_thread {
                if rng.gen::<f64>() < 0.8 {
                    // Read (80%)
                    let key = keys_clone[rng.gen_range(0..keys_clone.len())];
                    let _ = storage_clone.get(key).unwrap();
                    reads += 1;
                } else {
                    // Write (20%)
                    let key = rng.gen::<i64>();
                    let value = vec![1u8, 2u8, 3u8];
                    storage_clone.insert(key, &value).unwrap();
                    writes += 1;
                }
            }

            (reads, writes)
        });

        handles.push(handle);
    }

    let mut total_reads = 0;
    let mut total_writes = 0;
    for handle in handles {
        let (reads, writes) = handle.join().unwrap();
        total_reads += reads;
        total_writes += writes;
    }

    let elapsed = start.elapsed();
    let total_ops = ops_per_thread * num_threads;

    println!("\n  {} threads:", num_threads);
    println!("    Total ops: {} ({} reads, {} writes)", total_ops, total_reads, total_writes);
    println!("    Latency: {} ns/op", elapsed.as_nanos() / total_ops as u128);
    println!("    Throughput: {:.2}K ops/sec", total_ops as f64 / elapsed.as_secs_f64() / 1_000.0);
    println!("    Speedup: {:.2}x (ideal for 80% read: {:.2}x)",
        10_000.0 / (elapsed.as_nanos() as f64 / total_ops as f64),
        1.0 + (num_threads as f64 - 1.0) * 0.8);
}
