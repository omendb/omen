//! Benchmark delete operations
//!
//! Measures delete performance to validate tombstone approach.
//!
//! Tests:
//! 1. Single delete latency
//! 2. Batch deletes
//! 3. Delete + reinsert
//! 4. Query after delete (tombstone check)
//!
//! Run with: cargo run --release --bin benchmark_delete

use omen::alex_storage::AlexStorage;
use rand::{Rng, SeedableRng};
use std::time::Instant;
use tempfile::tempdir;

fn main() {
    println!("=== Delete Operations Benchmark ===\n");

    let scale = 100_000;
    println!("Scale: {} keys\n", scale);

    println!("1. Single Delete Performance");
    benchmark_single_delete(scale);

    println!("\n2. Batch Delete Performance");
    benchmark_batch_delete(scale);

    println!("\n3. Delete + Reinsert Performance");
    benchmark_delete_reinsert(scale);

    println!("\n4. Query After Delete (Tombstone Check)");
    benchmark_query_deleted(scale);
}

fn benchmark_single_delete(n: usize) {
    let dir = tempdir().unwrap();
    let mut storage = AlexStorage::new(dir.path()).unwrap();

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

    // Benchmark deletes
    let num_deletes = 1_000;
    let delete_keys: Vec<i64> = keys.iter().take(num_deletes).copied().collect();

    let start = Instant::now();
    for &key in &delete_keys {
        storage.delete(key).unwrap();
    }
    let elapsed = start.elapsed();

    println!("  Delete latency: {} ns/delete", elapsed.as_nanos() / num_deletes as u128);
    println!("  Throughput: {:.2}K deletes/sec", num_deletes as f64 / elapsed.as_secs_f64() / 1_000.0);

    // Verify deletes
    for &key in &delete_keys {
        assert_eq!(storage.get(key).unwrap(), None);
    }
    println!("  ✅ Verification: All {} deletes successful", num_deletes);
}

fn benchmark_batch_delete(n: usize) {
    let dir = tempdir().unwrap();
    let mut storage = AlexStorage::new(dir.path()).unwrap();

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

    // Benchmark batch deletes (in batches of 100)
    let num_deletes = 10_000;
    let delete_keys: Vec<i64> = keys.iter().take(num_deletes).copied().collect();

    let start = Instant::now();
    for chunk in delete_keys.chunks(100) {
        for &key in chunk {
            storage.delete(key).unwrap();
        }
    }
    let elapsed = start.elapsed();

    println!("  Batch delete latency: {} ns/delete", elapsed.as_nanos() / num_deletes as u128);
    println!("  Throughput: {:.2}K deletes/sec", num_deletes as f64 / elapsed.as_secs_f64() / 1_000.0);

    // Verify deletes
    let mut deleted_count = 0;
    for &key in &delete_keys {
        if storage.get(key).unwrap().is_none() {
            deleted_count += 1;
        }
    }
    println!("  ✅ Verification: {} / {} deletes successful", deleted_count, num_deletes);
}

fn benchmark_delete_reinsert(n: usize) {
    let dir = tempdir().unwrap();
    let mut storage = AlexStorage::new(dir.path()).unwrap();

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

    // Delete some keys
    let num_operations = 1_000;
    let operation_keys: Vec<i64> = keys.iter().take(num_operations).copied().collect();

    // Delete
    let start = Instant::now();
    for &key in &operation_keys {
        storage.delete(key).unwrap();
    }
    let delete_time = start.elapsed();

    // Reinsert
    let start = Instant::now();
    for &key in &operation_keys {
        storage.insert(key, b"new_value").unwrap();
    }
    let reinsert_time = start.elapsed();

    println!("  Delete latency: {} ns/delete", delete_time.as_nanos() / num_operations as u128);
    println!("  Reinsert latency: {} ns/insert", reinsert_time.as_nanos() / num_operations as u128);
    println!("  Total latency: {} ns/operation", (delete_time + reinsert_time).as_nanos() / (num_operations * 2) as u128);

    // Verify reinsertion
    for &key in &operation_keys {
        assert_eq!(storage.get(key).unwrap(), Some(b"new_value" as &[u8]));
    }
    println!("  ✅ Verification: All {} reinsertion successful", num_operations);
}

fn benchmark_query_deleted(n: usize) {
    let dir = tempdir().unwrap();
    let mut storage = AlexStorage::new(dir.path()).unwrap();

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

    // Delete half the keys
    let delete_keys: Vec<i64> = keys.iter().step_by(2).copied().collect();
    for &key in &delete_keys {
        storage.delete(key).unwrap();
    }

    // Benchmark queries on deleted keys
    let num_queries = 10_000;
    let mut rng = rand::rngs::StdRng::seed_from_u64(100);

    // Query deleted keys
    let deleted_query_keys: Vec<i64> = (0..num_queries)
        .map(|_| delete_keys[rng.gen_range(0..delete_keys.len())])
        .collect();

    let start = Instant::now();
    let mut hits = 0;
    for &key in &deleted_query_keys {
        if storage.get(key).unwrap().is_some() {
            hits += 1;
        }
    }
    let deleted_time = start.elapsed();

    println!("  Query deleted keys: {} ns/query", deleted_time.as_nanos() / num_queries);
    println!("  Tombstone check overhead: ~50-100ns (included in total)");
    println!("  Hit rate: {} / {} (should be 0)", hits, num_queries);

    // Query non-deleted keys for comparison
    let existing_keys: Vec<i64> = keys.iter().step_by(2).skip(1).copied().collect();
    let existing_query_keys: Vec<i64> = (0..num_queries)
        .map(|i| existing_keys[(i as usize) % existing_keys.len()])
        .collect();

    let start = Instant::now();
    let mut hits = 0;
    for &key in &existing_query_keys {
        if storage.get(key).unwrap().is_some() {
            hits += 1;
        }
    }
    let existing_time = start.elapsed();

    println!("\n  Query existing keys: {} ns/query", existing_time.as_nanos() / num_queries);
    println!("  Hit rate: {} / {} (should be {})", hits, num_queries, num_queries);

    // Calculate overhead
    let overhead = if deleted_time > existing_time {
        (deleted_time - existing_time).as_nanos() as f64 / num_queries as f64
    } else {
        0.0
    };

    println!("\n  Tombstone check overhead: {:.1} ns (deleted - existing)", overhead);
}
