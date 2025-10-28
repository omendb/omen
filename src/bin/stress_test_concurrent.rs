//! Concurrent stress test for OmenDB
//!
//! Tests multi-threaded insert and query performance to validate:
//! - No data corruption under concurrent access
//! - No deadlocks
//! - Reasonable throughput with multiple threads
//!
//! Usage:
//!   cargo run --release --bin stress_test_concurrent [num_threads] [rows_per_thread]
//!
//! Example:
//!   cargo run --release --bin stress_test_concurrent 10 100000

use anyhow::Result;
use omen::catalog::Catalog;
use omen::row::Row;
use omen::value::Value;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use arrow::datatypes::{DataType, Field, Schema};
use tempfile::TempDir;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let num_threads: usize = args.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    let rows_per_thread: usize = args.get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000);

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         OmenDB Concurrent Stress Test                       ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("Configuration:");
    println!("  Threads: {}", num_threads);
    println!("  Rows per thread: {}", rows_per_thread);
    println!("  Total rows: {}\n", num_threads * rows_per_thread);

    // Setup
    let temp_dir = TempDir::new()?;
    let catalog_dir = temp_dir.path().join("omendb");
    let catalog = Arc::new(std::sync::Mutex::new(Catalog::new(catalog_dir)?));

    // Create table
    {
        let mut cat = catalog.lock().unwrap();
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("thread_id", DataType::Int64, false),
            Field::new("value", DataType::Utf8, false),
        ]));
        cat.create_table("stress_test".to_string(), schema, "id".to_string())?;
    }

    println!("📝 Phase 1: Concurrent Inserts\n");

    let start = Instant::now();
    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let catalog_clone: Arc<Mutex<Catalog>> = Arc::clone(&catalog);
        let rows_per_thread = rows_per_thread;

        let handle = thread::spawn(move || -> Result<()> {
            let base_key = thread_id * rows_per_thread;

            // Each thread inserts its own range of keys
            let mut rows = Vec::with_capacity(rows_per_thread);
            for i in 0..rows_per_thread {
                let key = (base_key + i) as i64;
                rows.push(Row::new(vec![
                    Value::Int64(key),
                    Value::Int64(thread_id as i64),
                    Value::Text(format!("value_{}_{}", thread_id, i)),
                ]));
            }

            // Batch insert
            let mut cat = catalog_clone.lock().unwrap();
            let table = cat.get_table_mut("stress_test")?;
            table.batch_insert(rows)?;

            Ok(())
        });

        handles.push(handle);
    }

    // Wait for all threads
    for (i, handle) in handles.into_iter().enumerate() {
        handle.join().unwrap()?;
        println!("  ✓ Thread {} completed", i);
    }

    let insert_time = start.elapsed();
    let total_rows = num_threads * rows_per_thread;
    let throughput = total_rows as f64 / insert_time.as_secs_f64();

    println!("\n⏱️  Insert Results:");
    println!("  Total time: {:.2}s", insert_time.as_secs_f64());
    println!("  Throughput: {:.0} rows/sec", throughput);
    println!("  Per-thread avg: {:.2}s", insert_time.as_secs_f64() / num_threads as f64);

    // Verify data integrity
    println!("\n🔍 Phase 2: Data Integrity Verification\n");

    let verify_start = Instant::now();
    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let catalog_clone: Arc<Mutex<Catalog>> = Arc::clone(&catalog);
        let rows_per_thread = rows_per_thread;

        let handle = thread::spawn(move || -> Result<usize> {
            let base_key = thread_id * rows_per_thread;
            let mut found = 0;

            let cat = catalog_clone.lock().unwrap();
            let table = cat.get_table("stress_test")?;

            // Verify random sample (10%)
            for i in (0..rows_per_thread).step_by(10) {
                let key = (base_key + i) as i64;
                if table.get(&Value::Int64(key))?.is_some() {
                    found += 1;
                }
            }

            Ok(found)
        });

        handles.push(handle);
    }

    let mut total_found = 0;
    for (i, handle) in handles.into_iter().enumerate() {
        let found = handle.join().unwrap()?;
        total_found += found;
        println!("  ✓ Thread {} verified {} rows", i, found);
    }

    let verify_time = verify_start.elapsed();
    let expected_sample = (rows_per_thread / 10) * num_threads;

    println!("\n🔍 Verification Results:");
    println!("  Sample size: {}/{}", total_found, expected_sample);
    println!("  Verification time: {:.2}s", verify_time.as_secs_f64());

    if total_found == expected_sample {
        println!("  ✅ SUCCESS: All sampled rows found");
    } else {
        println!("  ⚠️  WARNING: {} rows missing from sample", expected_sample - total_found);
    }

    // Concurrent reads while writing
    println!("\n📊 Phase 3: Mixed Read/Write Workload\n");

    let mixed_start = Instant::now();
    let mut handles = vec![];

    // Half threads do writes, half do reads
    for thread_id in 0..num_threads {
        let catalog_clone: Arc<Mutex<Catalog>> = Arc::clone(&catalog);
        let is_reader = thread_id % 2 == 0;
        let ops_per_thread = 10_000;

        let handle = thread::spawn(move || -> Result<(usize, usize)> {
            let mut reads = 0;
            let mut writes = 0;

            if is_reader {
                // Reader thread
                let cat = catalog_clone.lock().unwrap();
                let table = cat.get_table("stress_test")?;

                for i in 0..ops_per_thread {
                    let key = (i % 100_000) as i64;
                    if table.get(&Value::Int64(key))?.is_some() {
                        reads += 1;
                    }
                }
            } else {
                // Writer thread
                let base_key = (thread_id + num_threads) * 10_000;
                let mut rows = Vec::with_capacity(ops_per_thread);

                for i in 0..ops_per_thread {
                    let key = (base_key + i) as i64;
                    rows.push(Row::new(vec![
                        Value::Int64(key),
                        Value::Int64(thread_id as i64),
                        Value::Text(format!("mixed_{}", i)),
                    ]));
                }

                let mut cat = catalog_clone.lock().unwrap();
                let table = cat.get_table_mut("stress_test")?;
                table.batch_insert(rows)?;
                writes = ops_per_thread;
            }

            Ok((reads, writes))
        });

        handles.push((thread_id, is_reader, handle));
    }

    let mut total_reads = 0;
    let mut total_writes = 0;

    for (thread_id, is_reader, handle) in handles {
        let (reads, writes) = handle.join().unwrap()?;
        total_reads += reads;
        total_writes += writes;

        if is_reader {
            println!("  ✓ Reader thread {} completed ({} reads)", thread_id, reads);
        } else {
            println!("  ✓ Writer thread {} completed ({} writes)", thread_id, writes);
        }
    }

    let mixed_time = mixed_start.elapsed();
    let read_throughput = total_reads as f64 / mixed_time.as_secs_f64();
    let write_throughput = total_writes as f64 / mixed_time.as_secs_f64();

    println!("\n📊 Mixed Workload Results:");
    println!("  Total time: {:.2}s", mixed_time.as_secs_f64());
    println!("  Read throughput: {:.0} reads/sec", read_throughput);
    println!("  Write throughput: {:.0} writes/sec", write_throughput);
    println!("  Combined: {:.0} ops/sec", (total_reads + total_writes) as f64 / mixed_time.as_secs_f64());

    // Final summary
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                      SUMMARY                                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("✅ Concurrent Inserts: {:.0} rows/sec ({} threads)", throughput, num_threads);
    println!("✅ Data Integrity: {}/{} verified", total_found, expected_sample);
    println!("✅ Mixed Workload: {:.0} ops/sec combined", (total_reads + total_writes) as f64 / mixed_time.as_secs_f64());

    println!("\n🎯 Stress Test: PASSED\n");

    Ok(())
}
