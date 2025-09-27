//! Week 3: Arrow Integration and Range Query Benchmarks
//! Testing our learned index with columnar storage

use omendb::{OmenDB, index::RecursiveModelIndex};
use std::time::Instant;
use std::collections::BTreeMap;

fn main() {
    println!("🚀 OmenDB Week 3: Arrow Storage + Range Queries");
    println!("=" .repeat(60));

    // Test at different scales
    for num_keys in [10_000, 100_000, 1_000_000] {
        println!("\n📊 Testing with {} time-series points:", num_keys);
        benchmark_range_queries(num_keys);
    }
}

fn benchmark_range_queries(num_keys: usize) {
    // Create OmenDB instance
    let mut db = OmenDB::new("timeseries");

    // Create comparison B-tree
    let mut btree = BTreeMap::new();

    // Generate time-series data
    let base_timestamp = 1_600_000_000_000_000i64;
    let mut data = Vec::new();

    for i in 0..num_keys {
        let timestamp = base_timestamp + (i as i64 * 1000); // 1 second intervals
        let value = (i as f64).sin() * 100.0 + 50.0; // Simulated sensor data
        data.push((timestamp, value));
    }

    // Insert into OmenDB with timing
    let insert_start = Instant::now();
    for (ts, val) in &data {
        db.insert(*ts, *val, 1).unwrap();
    }
    let omendb_insert_time = insert_start.elapsed();

    // Insert into B-tree
    let insert_start = Instant::now();
    for (ts, val) in &data {
        btree.insert(*ts, *val);
    }
    let btree_insert_time = insert_start.elapsed();

    // Train the learned index
    let train_start = Instant::now();
    let training_data: Vec<(i64, usize)> = data.iter()
        .enumerate()
        .map(|(i, (ts, _))| (*ts, i))
        .collect();
    db.index.train(training_data);
    let train_time = train_start.elapsed();

    println!("\n📈 Performance Metrics:");
    println!("  Insert time:");
    println!("    OmenDB:  {:?}", omendb_insert_time);
    println!("    B-tree:  {:?}", btree_insert_time);
    println!("  Index training: {:?}", train_time);

    // Benchmark range queries
    let num_queries = 1000;
    let range_size = num_keys / 100; // Query 1% of data

    // OmenDB range queries
    let start = Instant::now();
    for i in 0..num_queries {
        let start_ts = base_timestamp + (i * range_size) as i64 * 1000;
        let end_ts = start_ts + (range_size as i64 * 1000);
        let _results = db.index.range_search(start_ts, end_ts);
    }
    let omendb_range_time = start.elapsed();

    // B-tree range queries
    let start = Instant::now();
    for i in 0..num_queries {
        let start_ts = base_timestamp + (i * range_size) as i64 * 1000;
        let end_ts = start_ts + (range_size as i64 * 1000);
        let _: Vec<_> = btree.range(start_ts..=end_ts).collect();
    }
    let btree_range_time = start.elapsed();

    // Calculate metrics
    let omendb_ns = omendb_range_time.as_nanos() as f64 / num_queries as f64;
    let btree_ns = btree_range_time.as_nanos() as f64 / num_queries as f64;
    let speedup = btree_ns / omendb_ns;

    println!("\n⚡ Range Query Performance:");
    println!("  OmenDB:  {:.0} ns/query", omendb_ns);
    println!("  B-tree:  {:.0} ns/query", btree_ns);
    println!("  Speedup: {:.2}x {}",
             speedup,
             if speedup > 2.0 { "✅" } else { "⚠️" });

    // Test aggregations
    let start = Instant::now();
    let sum = db.sum(base_timestamp, base_timestamp + (num_keys as i64 * 1000)).unwrap();
    let avg = db.avg(base_timestamp, base_timestamp + (num_keys as i64 * 1000)).unwrap();
    let agg_time = start.elapsed();

    println!("\n📊 Aggregations:");
    println!("  Sum: {:.2}", sum);
    println!("  Avg: {:.2}", avg);
    println!("  Time: {:?}", agg_time);

    if speedup > 3.0 {
        println!("\n🎯 Week 3 Goal Achieved: Range queries optimized!");
    }
}