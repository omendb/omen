//! Benchmark to verify learned index speedup with redb backend

use omendb::redb_storage::RedbStorage;
use std::collections::BTreeMap;
use std::time::Instant;
use tempfile::tempdir;

fn main() -> anyhow::Result<()> {
    println!("=== Learned Index + redb Benchmark ===\n");

    let num_keys = 100_000;
    println!("Dataset size: {} keys\n", num_keys);

    let dir = tempdir()?;
    let db_path = dir.path().join("benchmark.redb");

    println!("Phase 1: Inserting {} keys...", num_keys);
    let mut storage = RedbStorage::new(&db_path)?;

    let insert_start = Instant::now();

    let batch_size = 10_000;
    for batch_start in (0..num_keys).step_by(batch_size as usize) {
        let batch_end = (batch_start + batch_size).min(num_keys);
        let batch: Vec<(i64, Vec<u8>)> = (batch_start..batch_end)
            .map(|i| (i, format!("value_{}", i).into_bytes()))
            .collect();
        storage.insert_batch(batch)?;
    }

    let insert_duration = insert_start.elapsed();
    println!("  Insert time: {:?}", insert_duration);
    println!(
        "  Insert rate: {:.0} keys/sec\n",
        num_keys as f64 / insert_duration.as_secs_f64()
    );

    println!("Phase 2: Point query benchmark (learned index path)");
    let num_queries = 10_000;
    let mut total_duration = std::time::Duration::ZERO;

    for i in 0..num_queries {
        let key = (i * 10) % num_keys;
        let start = Instant::now();
        let result = storage.point_query(key)?;
        total_duration += start.elapsed();
        assert!(result.is_some(), "Expected to find key {}", key);
    }

    let avg_latency_us = total_duration.as_micros() as f64 / num_queries as f64;
    let qps = num_queries as f64 / total_duration.as_secs_f64();

    println!("  Total queries: {}", num_queries);
    println!("  Average latency: {:.2} µs", avg_latency_us);
    println!("  Queries/sec: {:.0}", qps);

    println!("\nPhase 3: Baseline comparison (BTreeMap in-memory)");
    let mut btree = BTreeMap::new();
    for i in 0..num_keys {
        btree.insert(i, format!("value_{}", i).into_bytes());
    }

    let mut btree_duration = std::time::Duration::ZERO;
    for i in 0..num_queries {
        let key = (i * 10) % num_keys;
        let start = Instant::now();
        let result = btree.get(&key);
        btree_duration += start.elapsed();
        assert!(result.is_some());
    }

    let btree_avg_us = btree_duration.as_micros() as f64 / num_queries as f64;
    let btree_qps = num_queries as f64 / btree_duration.as_secs_f64();

    println!("  BTreeMap average latency: {:.2} µs", btree_avg_us);
    println!("  BTreeMap queries/sec: {:.0}", btree_qps);

    println!("\nPhase 4: Range query benchmark");
    let range_start = Instant::now();
    let results = storage.range_query(1000, 2000)?;
    let range_duration = range_start.elapsed();

    println!(
        "  Range [1000, 2000]: {} results in {:?}",
        results.len(),
        range_duration
    );
    println!(
        "  Range query rate: {:.0} keys/sec",
        results.len() as f64 / range_duration.as_secs_f64()
    );

    println!("\n=== Performance Summary ===");
    println!(
        "  Point query speedup vs BTreeMap: {:.2}x",
        btree_avg_us / avg_latency_us
    );
    println!("  Total rows in database: {}", storage.count());

    if avg_latency_us < 100.0 {
        println!("\n✅ EXCELLENT: Sub-100µs point query latency!");
    } else if avg_latency_us < 500.0 {
        println!("\n✅ GOOD: Sub-500µs point query latency");
    } else {
        println!("\n⚠️  Point queries slower than expected");
    }

    Ok(())
}
