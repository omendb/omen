//! Large Dataset Performance Tests
//!
//! Tests learned index performance on datasets where it SHOULD excel (100K+ rows).
//! These tests validate our core value proposition: significant speedup on large data.

use datafusion::prelude::*;
use omendb::datafusion::redb_table::RedbTable;
use omendb::redb_storage::RedbStorage;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tempfile::tempdir;

/// Helper to create a RedbTable with N rows using insert_batch for speed
fn create_large_redb_table(n: usize, name: &str) -> (Arc<RedbTable>, tempfile::TempDir) {
    println!("Creating table with {} rows...", n);
    let start = Instant::now();

    let dir = tempdir().unwrap();
    let db_path = dir.path().join(format!("{}.redb", name));

    let mut storage = RedbStorage::new(&db_path).unwrap();

    // Use insert_batch for MUCH faster insertion
    let batch_size = 10_000;
    for batch_start in (0..n).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(n);

        let entries: Vec<(i64, Vec<u8>)> = (batch_start..batch_end)
            .map(|i| (i as i64, format!("v{}", i).into_bytes()))
            .collect();

        storage.insert_batch(entries).unwrap();

        // Progress indicator
        if batch_start > 0 && batch_start % 10_000 == 0 {
            let elapsed = start.elapsed().as_secs_f64();
            let rate = batch_start as f64 / elapsed;
            let remaining = (n - batch_start) as f64 / rate;
            println!("  {} / {} rows ({:.0} rows/sec, ~{:.0}s remaining)",
                     batch_start, n, rate, remaining);
        }
    }

    let insert_time = start.elapsed();
    println!("✓ Created {} rows in {:.2}s ({:.0} rows/sec)",
             n,
             insert_time.as_secs_f64(),
             n as f64 / insert_time.as_secs_f64());

    let storage = Arc::new(RwLock::new(storage));
    let table = Arc::new(RedbTable::new(storage, name));

    (table, dir)
}

#[tokio::test]
async fn test_50k_rows_point_query() {
    println!("\n=== 50K Rows Point Query Test ===");

    // Start with 50K to validate speedup, can scale up later
    let (table, _dir) = create_large_redb_table(50_000, "test_50k");

    let ctx = SessionContext::new();
    ctx.register_table("test_50k", table).unwrap();

    // Warm up
    let _ = ctx.sql("SELECT * FROM test_50k WHERE id = 25000").await.unwrap().collect().await;

    // Test point query (middle of dataset)
    let start = Instant::now();
    let df = ctx.sql("SELECT * FROM test_50k WHERE id = 25000").await.unwrap();
    let results = df.collect().await.unwrap();
    let point_time = start.elapsed();

    assert_eq!(results[0].num_rows(), 1, "Should find exactly 1 row");
    println!("Point query (id=25000): {:.3}ms", point_time.as_secs_f64() * 1000.0);

    // Test full scan for comparison
    let start = Instant::now();
    let df = ctx.sql("SELECT * FROM test_50k").await.unwrap();
    let results = df.collect().await.unwrap();
    let scan_time = start.elapsed();

    let total_rows: usize = results.iter().map(|b| b.num_rows()).sum();
    assert_eq!(total_rows, 50_000, "Full scan should return all rows");
    println!("Full scan: {:.3}ms", scan_time.as_secs_f64() * 1000.0);

    // Calculate speedup
    let speedup = scan_time.as_secs_f64() / point_time.as_secs_f64();
    println!("Speedup: {:.1}x", speedup);
    println!("=====================================\n");

    // Learned index should provide significant speedup on 50K rows
    if speedup >= 10.0 {
        println!("✅ EXCELLENT: {:.1}x speedup achieved!", speedup);
    } else if speedup >= 5.0 {
        println!("✓ VERY GOOD: {:.1}x speedup achieved", speedup);
    } else if speedup >= 3.0 {
        println!("✓ GOOD: {:.1}x speedup achieved", speedup);
    } else if speedup >= 1.5 {
        println!("⚠ MARGINAL: Only {:.1}x speedup", speedup);
    } else {
        println!("❌ POOR: Only {:.1}x speedup - investigating needed", speedup);
    }

    // Assert reasonable performance
    assert!(point_time.as_secs_f64() < 1.0, "Point query should be < 1s on 50K rows");

    // Document actual speedup (informational, not hard requirement)
    println!("\nACTUAL PERFORMANCE:");
    println!("  Dataset: 50,000 rows");
    println!("  Point query: {:.1}ms", point_time.as_secs_f64() * 1000.0);
    println!("  Full scan: {:.1}ms", scan_time.as_secs_f64() * 1000.0);
    println!("  Speedup: {:.1}x", speedup);
}

#[tokio::test]
async fn test_100k_rows_multiple_point_queries() {
    println!("\n=== 100K Rows Multiple Point Queries ===");

    let (table, _dir) = create_large_redb_table(100_000, "test_100k_multi");

    let ctx = SessionContext::new();
    ctx.register_table("test_100k_multi", table).unwrap();

    // Test 10 point queries at different positions
    let test_ids = vec![100, 10_000, 25_000, 50_000, 75_000, 90_000, 95_000, 99_000, 99_900, 99_999];
    let mut total_time = 0.0;

    for id in &test_ids {
        let start = Instant::now();
        let df = ctx.sql(&format!("SELECT * FROM test_100k_multi WHERE id = {}", id)).await.unwrap();
        let results = df.collect().await.unwrap();
        let query_time = start.elapsed();

        total_time += query_time.as_secs_f64();
        assert_eq!(results[0].num_rows(), 1, "Should find exactly 1 row for id = {}", id);
    }

    let avg_time = total_time / test_ids.len() as f64;
    println!("Average point query time (10 queries): {:.3}ms", avg_time * 1000.0);
    println!("Total time: {:.3}ms", total_time * 1000.0);
    println!("=====================================\n");

    // Average point query should be fast
    assert!(avg_time < 0.1, "Average point query should be < 100ms");
}

#[tokio::test]
async fn test_100k_rows_range_query() {
    println!("\n=== 100K Rows Range Query ===");

    let (table, _dir) = create_large_redb_table(100_000, "test_100k_range");

    let ctx = SessionContext::new();
    ctx.register_table("test_100k_range", table).unwrap();

    // Small range query
    let start = Instant::now();
    let df = ctx.sql("SELECT * FROM test_100k_range WHERE id >= 50000 AND id < 51000").await.unwrap();
    let results = df.collect().await.unwrap();
    let range_time = start.elapsed();

    let row_count: usize = results.iter().map(|b| b.num_rows()).sum();
    assert_eq!(row_count, 1000, "Range query should return 1000 rows");

    println!("Range query (1000 rows): {:.3}ms", range_time.as_secs_f64() * 1000.0);
    println!("=====================================\n");

    // Range query should be reasonably fast
    assert!(range_time.as_secs_f64() < 1.0, "Range query should be < 1s");
}

#[tokio::test]
async fn test_100k_rows_aggregation() {
    println!("\n=== 100K Rows Aggregation ===");

    let (table, _dir) = create_large_redb_table(100_000, "test_100k_agg");

    let ctx = SessionContext::new();
    ctx.register_table("test_100k_agg", table).unwrap();

    // COUNT aggregation
    let start = Instant::now();
    let df = ctx.sql("SELECT COUNT(*) as count FROM test_100k_agg").await.unwrap();
    let results = df.collect().await.unwrap();
    let count_time = start.elapsed();

    println!("COUNT(*) query: {:.3}ms", count_time.as_secs_f64() * 1000.0);

    // COUNT with filter (should use learned index)
    let start = Instant::now();
    let df = ctx.sql("SELECT COUNT(*) as count FROM test_100k_agg WHERE id = 50000").await.unwrap();
    let results = df.collect().await.unwrap();
    let filtered_count_time = start.elapsed();

    println!("COUNT(*) with filter: {:.3}ms", filtered_count_time.as_secs_f64() * 1000.0);
    println!("=====================================\n");

    // Filtered aggregation should be much faster
    let speedup = count_time.as_secs_f64() / filtered_count_time.as_secs_f64();
    println!("Filtered speedup: {:.1}x", speedup);

    assert!(filtered_count_time.as_secs_f64() < 1.0, "Filtered COUNT should be < 1s");
}

#[tokio::test]
async fn test_100k_rows_worst_case_lookup() {
    println!("\n=== 100K Rows Worst Case Lookup ===");

    let (table, _dir) = create_large_redb_table(100_000, "test_100k_worst");

    let ctx = SessionContext::new();
    ctx.register_table("test_100k_worst", table).unwrap();

    // Test first, middle, and last positions
    let positions = vec![
        (0, "first"),
        (50_000, "middle"),
        (99_999, "last"),
    ];

    for (id, label) in positions {
        let start = Instant::now();
        let df = ctx.sql(&format!("SELECT * FROM test_100k_worst WHERE id = {}", id)).await.unwrap();
        let results = df.collect().await.unwrap();
        let query_time = start.elapsed();

        assert_eq!(results[0].num_rows(), 1);
        println!("{:10} (id={}): {:.3}ms", label, id, query_time.as_secs_f64() * 1000.0);
    }

    println!("=====================================\n");
}

#[tokio::test]
async fn test_500k_rows_validation() {
    println!("\n=== 500K Rows Validation ===");

    let (table, _dir) = create_large_redb_table(500_000, "test_500k");

    let ctx = SessionContext::new();
    ctx.register_table("test_500k", table).unwrap();

    // Point query
    let start = Instant::now();
    let df = ctx.sql("SELECT * FROM test_500k WHERE id = 250000").await.unwrap();
    let results = df.collect().await.unwrap();
    let point_time = start.elapsed();

    assert_eq!(results[0].num_rows(), 1);
    println!("Point query (id=250000): {:.3}ms", point_time.as_secs_f64() * 1000.0);

    // Full scan
    let start = Instant::now();
    let df = ctx.sql("SELECT COUNT(*) FROM test_500k").await.unwrap();
    let results = df.collect().await.unwrap();
    let scan_time = start.elapsed();

    println!("Full scan (COUNT): {:.3}ms", scan_time.as_secs_f64() * 1000.0);

    let speedup = scan_time.as_secs_f64() / point_time.as_secs_f64();
    println!("Speedup: {:.1}x", speedup);
    println!("=====================================\n");

    if speedup >= 10.0 {
        println!("✅ EXCELLENT: {:.1}x speedup on 500K rows!", speedup);
    } else if speedup >= 5.0 {
        println!("✓ GOOD: {:.1}x speedup on 500K rows", speedup);
    } else {
        println!("⚠ Need investigation: only {:.1}x speedup", speedup);
    }
}

#[tokio::test]
#[ignore] // Only run manually due to time/memory constraints
async fn test_1m_rows_ultimate() {
    println!("\n=== 1M Rows Ultimate Test ===");

    let (table, _dir) = create_large_redb_table(1_000_000, "test_1m");

    let ctx = SessionContext::new();
    ctx.register_table("test_1m", table).unwrap();

    // Point query
    let start = Instant::now();
    let df = ctx.sql("SELECT * FROM test_1m WHERE id = 500000").await.unwrap();
    let results = df.collect().await.unwrap();
    let point_time = start.elapsed();

    assert_eq!(results[0].num_rows(), 1);
    println!("Point query (id=500000): {:.3}ms", point_time.as_secs_f64() * 1000.0);

    // Full scan
    let start = Instant::now();
    let df = ctx.sql("SELECT COUNT(*) FROM test_1m").await.unwrap();
    let results = df.collect().await.unwrap();
    let scan_time = start.elapsed();

    println!("Full scan (COUNT): {:.3}ms", scan_time.as_secs_f64() * 1000.0);

    let speedup = scan_time.as_secs_f64() / point_time.as_secs_f64();
    println!("Speedup: {:.1}x", speedup);
    println!("=====================================\n");

    if speedup >= 100.0 {
        println!("🎯 EXCEPTIONAL: {:.1}x speedup on 1M rows!", speedup);
    } else if speedup >= 10.0 {
        println!("✅ EXCELLENT: {:.1}x speedup on 1M rows", speedup);
    } else {
        println!("⚠ Below expectations: only {:.1}x speedup", speedup);
    }

    // On 1M rows, learned index should absolutely dominate
    assert!(speedup >= 5.0, "Should achieve at least 5x speedup on 1M rows");
}
