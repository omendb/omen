//! Benchmark compaction performance
//!
//! Measures compaction time and space reclamation at various scales and deletion rates.
//!
//! Tests:
//! 1. Compaction at different scales (10K, 100K, 1M)
//! 2. Different deletion rates (10%, 50%, 90%)
//! 3. Workload: insert → delete → compact → verify
//!
//! Run with: cargo run --release --bin benchmark_compaction

use omen::alex_storage::AlexStorage;
use rand::{Rng, SeedableRng};
use std::time::Instant;
use tempfile::tempdir;

fn main() {
    println!("=== Compaction Performance Benchmark ===\n");

    println!("1. Compaction at Different Scales (50% deletion rate)");
    benchmark_scale(10_000, 0.5);
    benchmark_scale(100_000, 0.5);
    benchmark_scale(1_000_000, 0.5);

    println!("\n2. Different Deletion Rates (100K scale)");
    benchmark_deletion_rate(100_000, 0.1);
    benchmark_deletion_rate(100_000, 0.3);
    benchmark_deletion_rate(100_000, 0.5);
    benchmark_deletion_rate(100_000, 0.7);
    benchmark_deletion_rate(100_000, 0.9);
}

fn benchmark_scale(n: usize, deletion_rate: f64) {
    let dir = tempdir().unwrap();
    let mut storage = AlexStorage::new(dir.path()).unwrap();

    println!("\n  Scale: {} keys, {:.0}% deletion", n, deletion_rate * 100.0);

    // Insert keys
    let insert_start = Instant::now();
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let keys: Vec<i64> = (0..n).map(|_| rng.gen()).collect();
    let values: Vec<Vec<u8>> = (0..n).map(|i| format!("value_{}", i).into_bytes()).collect();

    let entries: Vec<(i64, Vec<u8>)> = keys
        .iter()
        .zip(values.iter())
        .map(|(&k, v)| (k, v.clone()))
        .collect();
    storage.insert_batch(entries).unwrap();
    let insert_time = insert_start.elapsed();

    println!("    Insert time: {:.2}s", insert_time.as_secs_f64());

    let stats_before = storage.stats();
    let file_size_before = stats_before.file_size;
    println!("    File size before: {:.2} MB", file_size_before as f64 / 1_000_000.0);

    // Delete keys
    let num_deletes = (n as f64 * deletion_rate) as usize;
    let delete_start = Instant::now();
    for &key in keys.iter().take(num_deletes) {
        storage.delete(key).unwrap();
    }
    let delete_time = delete_start.elapsed();

    println!("    Delete time: {:.2}s ({} deletes)", delete_time.as_secs_f64(), num_deletes);

    // Compact
    let compact_start = Instant::now();
    let compact_stats = storage.compact().unwrap();
    let compact_time = compact_start.elapsed();

    println!("    Compaction time: {:.2}s", compact_time.as_secs_f64());
    println!("    Entries before: {}", compact_stats.entries_before);
    println!("    Entries after: {}", compact_stats.entries_after);
    println!("    Tombstones removed: {}", compact_stats.tombstones_removed);
    println!("    Bytes before: {:.2} MB", compact_stats.bytes_before as f64 / 1_000_000.0);
    println!("    Bytes after: {:.2} MB", compact_stats.bytes_after as f64 / 1_000_000.0);
    println!("    Space reclaimed: {:.2} MB ({:.1}%)",
        compact_stats.space_reclaimed as f64 / 1_000_000.0,
        (compact_stats.space_reclaimed as f64 / compact_stats.bytes_before as f64) * 100.0);

    // Verify live keys readable
    let verify_start = Instant::now();
    let mut verified = 0;
    for &key in keys.iter().skip(num_deletes).take(100) {
        if storage.get(key).unwrap().is_some() {
            verified += 1;
        }
    }
    let verify_time = verify_start.elapsed();

    println!("    Verification: {}/100 keys readable ({:.0}µs)", verified, verify_time.as_micros());

    // Throughput
    println!("    Throughput: {:.0} entries/sec", compact_stats.entries_before as f64 / compact_time.as_secs_f64());
}

fn benchmark_deletion_rate(n: usize, deletion_rate: f64) {
    let dir = tempdir().unwrap();
    let mut storage = AlexStorage::new(dir.path()).unwrap();

    println!("\n  {:.0}% deletion rate ({} keys)", deletion_rate * 100.0, n);

    // Insert keys
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let keys: Vec<i64> = (0..n).map(|_| rng.gen()).collect();
    let values: Vec<Vec<u8>> = (0..n).map(|i| format!("value_{}", i).into_bytes()).collect();

    let entries: Vec<(i64, Vec<u8>)> = keys
        .iter()
        .zip(values.iter())
        .map(|(&k, v)| (k, v.clone()))
        .collect();
    storage.insert_batch(entries).unwrap();

    let file_size_before = storage.stats().file_size;

    // Delete keys
    let num_deletes = (n as f64 * deletion_rate) as usize;
    for &key in keys.iter().take(num_deletes) {
        storage.delete(key).unwrap();
    }

    // Compact
    let compact_start = Instant::now();
    let compact_stats = storage.compact().unwrap();
    let compact_time = compact_start.elapsed();

    println!("    Compaction time: {:.2}s", compact_time.as_secs_f64());
    println!("    Space reclaimed: {:.2} MB ({:.1}%)",
        compact_stats.space_reclaimed as f64 / 1_000_000.0,
        (compact_stats.space_reclaimed as f64 / file_size_before as f64) * 100.0);
    println!("    Tombstones removed: {} ({:.1}%)",
        compact_stats.tombstones_removed,
        (compact_stats.tombstones_removed as f64 / compact_stats.entries_before as f64) * 100.0);
    println!("    Throughput: {:.0} entries/sec", compact_stats.entries_before as f64 / compact_time.as_secs_f64());
}
