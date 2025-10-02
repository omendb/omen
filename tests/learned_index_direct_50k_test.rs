//! Direct RedbStorage Test - 50K Rows
//!
//! Tests learned index performance on 50K rows WITHOUT DataFusion overhead.
//! This bypasses the TableProvider layer to measure pure learned index performance.

use omendb::redb_storage::RedbStorage;
use std::time::Instant;
use tempfile::tempdir;

#[test]
fn test_50k_rows_direct_redbstorage() {
    println!("\n=== DIRECT RedbStorage Test: 50K Rows ===");
    println!("(Bypassing DataFusion to measure pure learned index performance)");

    let dir = tempdir().unwrap();
    let db_path = dir.path().join("direct_50k.redb");
    let mut storage = RedbStorage::new(&db_path).unwrap();

    // Insert 50K rows using insert_batch
    println!("\nInserting 50K rows...");
    let start = Instant::now();

    let batch_size = 10_000;
    for batch_start in (0..50_000).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(50_000);

        let entries: Vec<(i64, Vec<u8>)> = (batch_start..batch_end)
            .map(|i| (i as i64, format!("value_{}", i).into_bytes()))
            .collect();

        storage.insert_batch(entries).unwrap();

        if batch_start > 0 {
            let elapsed = start.elapsed().as_secs_f64();
            let rate = batch_start as f64 / elapsed;
            println!("  {} / 50000 rows ({:.0} rows/sec)", batch_start, rate);
        }
    }

    let insert_time = start.elapsed();
    println!("✓ Inserted 50,000 rows in {:.2}s ({:.0} rows/sec)",
             insert_time.as_secs_f64(),
             50_000.0 / insert_time.as_secs_f64());

    // Warm up
    let _ = storage.point_query(25000).unwrap();

    // Test point query performance (average of 100 queries)
    println!("\nTesting point query performance...");
    let iterations = 100;
    let mut total_point_time = 0.0;

    for i in 0..iterations {
        let key = (i * 500) as i64; // Query different keys
        let start = Instant::now();
        let result = storage.point_query(key).unwrap();
        total_point_time += start.elapsed().as_secs_f64();

        // Verify correctness
        if i == 0 {
            assert!(result.is_some(), "Key {} should exist", key);
        }
    }

    let avg_point_time_ms = (total_point_time / iterations as f64) * 1000.0;

    // Test full scan performance
    println!("Testing full scan performance...");
    let start = Instant::now();
    let all_rows = storage.scan_all().unwrap();
    let full_scan_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    assert_eq!(all_rows.len(), 50_000, "Should have 50K rows");

    // Calculate speedup
    let speedup = full_scan_time_ms / avg_point_time_ms;

    println!("\n=====================================");
    println!("RESULTS:");
    println!("  Dataset size: 50,000 rows");
    println!("  Point query (avg): {:.3}ms", avg_point_time_ms);
    println!("  Full scan: {:.3}ms ({:.1}ms per 1K rows)",
             full_scan_time_ms, full_scan_time_ms / 50.0);
    println!("  Speedup: {:.1}x", speedup);
    println!("=====================================");

    // Assert performance targets
    if speedup >= 100.0 {
        println!("✅ EXCELLENT: {:.1}x speedup!", speedup);
    } else if speedup >= 50.0 {
        println!("✅ VERY GOOD: {:.1}x speedup", speedup);
    } else if speedup >= 10.0 {
        println!("✓ GOOD: {:.1}x speedup", speedup);
    } else if speedup >= 5.0 {
        println!("✓ ACCEPTABLE: {:.1}x speedup", speedup);
    } else {
        println!("⚠ MARGINAL: {:.1}x speedup (expected >5x on 50K rows)", speedup);
    }

    // Minimum acceptable speedup for 50K rows
    assert!(
        speedup >= 5.0,
        "Learned index should provide >=5x speedup on 50K rows, got {:.1}x",
        speedup
    );
}

#[test]
fn test_100k_rows_direct_redbstorage() {
    println!("\n=== DIRECT RedbStorage Test: 100K Rows ===");
    println!("(Testing learned index at scale)");

    let dir = tempdir().unwrap();
    let db_path = dir.path().join("direct_100k.redb");
    let mut storage = RedbStorage::new(&db_path).unwrap();

    // Insert 100K rows
    println!("\nInserting 100K rows...");
    let start = Instant::now();

    let batch_size = 10_000;
    for batch_start in (0..100_000).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(100_000);

        let entries: Vec<(i64, Vec<u8>)> = (batch_start..batch_end)
            .map(|i| (i as i64, format!("value_{}", i).into_bytes()))
            .collect();

        storage.insert_batch(entries).unwrap();

        if batch_start > 0 && batch_start % 20_000 == 0 {
            let elapsed = start.elapsed().as_secs_f64();
            let rate = batch_start as f64 / elapsed;
            println!("  {} / 100000 rows ({:.0} rows/sec)", batch_start, rate);
        }
    }

    let insert_time = start.elapsed();
    println!("✓ Inserted 100,000 rows in {:.2}s ({:.0} rows/sec)",
             insert_time.as_secs_f64(),
             100_000.0 / insert_time.as_secs_f64());

    // Warm up
    let _ = storage.point_query(50000).unwrap();

    // Test point query performance
    println!("\nTesting point query performance...");
    let iterations = 100;
    let mut total_point_time = 0.0;

    for i in 0..iterations {
        let key = (i * 1000) as i64;
        let start = Instant::now();
        let _ = storage.point_query(key).unwrap();
        total_point_time += start.elapsed().as_secs_f64();
    }

    let avg_point_time_ms = (total_point_time / iterations as f64) * 1000.0;

    // Test full scan performance
    println!("Testing full scan performance...");
    let start = Instant::now();
    let all_rows = storage.scan_all().unwrap();
    let full_scan_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    assert_eq!(all_rows.len(), 100_000, "Should have 100K rows");

    let speedup = full_scan_time_ms / avg_point_time_ms;

    println!("\n=====================================");
    println!("RESULTS:");
    println!("  Dataset size: 100,000 rows");
    println!("  Point query (avg): {:.3}ms", avg_point_time_ms);
    println!("  Full scan: {:.3}ms ({:.1}ms per 1K rows)",
             full_scan_time_ms, full_scan_time_ms / 100.0);
    println!("  Speedup: {:.1}x", speedup);
    println!("=====================================");

    if speedup >= 500.0 {
        println!("✅ EXCELLENT: {:.1}x speedup!", speedup);
    } else if speedup >= 100.0 {
        println!("✅ VERY GOOD: {:.1}x speedup", speedup);
    } else if speedup >= 50.0 {
        println!("✓ GOOD: {:.1}x speedup", speedup);
    } else if speedup >= 10.0 {
        println!("✓ ACCEPTABLE: {:.1}x speedup", speedup);
    } else {
        println!("⚠ MARGINAL: {:.1}x speedup (expected >10x on 100K rows)", speedup);
    }

    assert!(
        speedup >= 10.0,
        "Learned index should provide >=10x speedup on 100K rows, got {:.1}x",
        speedup
    );
}
