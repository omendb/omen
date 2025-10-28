//! Profiling benchmark: 10M row query performance
//!
//! Focused on profiling the query path to identify bottlenecks.
//! Run with: cargo build --release && perf record -F 999 -g ./target/release/profile_10m_queries

use anyhow::Result;
use omen::rocks_storage::RocksStorage;
use tempfile::TempDir;
use std::time::Instant;

fn main() -> Result<()> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║            10M Query Performance Profiling                  ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("profile.db");

    // Build database with 10M sequential keys
    println!("📦 Building database with 10M sequential keys...");
    let mut storage = RocksStorage::new(&db_path)?;

    let start = Instant::now();
    let entries: Vec<(i64, Vec<u8>)> = (0..10_000_000)
        .map(|i| (i, format!("value_{}", i).into_bytes()))
        .collect();
    storage.insert_batch(entries)?;
    let build_time = start.elapsed();

    println!("  ✅ Build time: {:.2}s\n", build_time.as_secs_f64());

    // Run 100K queries for profiling (enough samples)
    println!("🔍 Running 100,000 point queries for profiling...");
    println!("  (Run under perf to capture profile data)\n");

    let num_queries = 100_000;
    let start = Instant::now();

    for i in 0..num_queries {
        let key = (i * 100) % 10_000_000; // Spread across dataset
        let _ = storage.point_query(key)?;
    }

    let elapsed = start.elapsed();
    let avg_latency_ns = (elapsed.as_nanos() as f64) / num_queries as f64;
    let avg_latency_us = avg_latency_ns / 1000.0;

    println!("✅ Profiling run complete:");
    println!("  Total time: {:.2}s", elapsed.as_secs_f64());
    println!("  Queries: {}", num_queries);
    println!("  Avg latency: {:.2}μs ({:.0}ns)", avg_latency_us, avg_latency_ns);
    println!("  Throughput: {:.0} queries/sec", num_queries as f64 / elapsed.as_secs_f64());

    Ok(())
}
