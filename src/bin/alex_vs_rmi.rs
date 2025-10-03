//! Benchmark: ALEX vs RMI learned indexes
//!
//! Compares performance of ALEX (adaptive) vs RMI (static) at scale.
//!
//! Expected results (from research):
//! - RMI: Fast at small scale, O(n) rebuilds cause 10M degradation
//! - ALEX: Consistent performance, local splits avoid global rebuilds

use omendb::alex::AlexTree;
use omendb::index::RecursiveModelIndex;
use std::time::Instant;

fn main() {
    println!("=== ALEX vs RMI Benchmark ===\n");

    // Test at increasing scales
    for scale in [1_000, 10_000, 100_000, 1_000_000] {
        println!("--- Scale: {} keys ---", scale);
        benchmark_alex(scale);
        benchmark_rmi(scale);
        println!();
    }
}

fn benchmark_alex(n: usize) {
    let mut tree = AlexTree::new();
    let mut durations = Vec::new();

    // Insert phase
    let start = Instant::now();
    for i in 0..n {
        tree.insert(i as i64, vec![(i % 256) as u8]).unwrap();
    }
    let insert_time = start.elapsed();

    // Query phase - sample 1000 queries
    let queries = 1000.min(n);
    for i in 0..queries {
        let key = (i * n / queries) as i64;
        let start = Instant::now();
        let _ = tree.get(key).unwrap();
        durations.push(start.elapsed().as_nanos() as f64 / 1000.0); // microseconds
    }

    let avg_query = durations.iter().sum::<f64>() / durations.len() as f64;
    let p50 = percentile(&mut durations, 0.5);
    let p99 = percentile(&mut durations, 0.99);

    println!(
        "  ALEX: insert={:.2}s, query_avg={:.2}μs, p50={:.2}μs, p99={:.2}μs, leaves={}",
        insert_time.as_secs_f64(),
        avg_query,
        p50,
        p99,
        tree.num_leaves()
    );
}

fn benchmark_rmi(n: usize) {
    let mut index = RecursiveModelIndex::new(n);
    let mut durations = Vec::new();

    // Insert phase
    let start = Instant::now();
    for i in 0..n {
        index.add_key(i as i64);
    }
    let insert_time = start.elapsed();

    // Query phase - sample 1000 queries
    let queries = 1000.min(n);
    for i in 0..queries {
        let key = (i * n / queries) as i64;
        let start = Instant::now();
        let _ = index.search(key);
        durations.push(start.elapsed().as_nanos() as f64 / 1000.0); // microseconds
    }

    let avg_query = durations.iter().sum::<f64>() / durations.len() as f64;
    let p50 = percentile(&mut durations, 0.5);
    let p99 = percentile(&mut durations, 0.99);

    println!(
        "  RMI:  insert={:.2}s, query_avg={:.2}μs, p50={:.2}μs, p99={:.2}μs",
        insert_time.as_secs_f64(),
        avg_query,
        p50,
        p99
    );
}

fn percentile(data: &mut [f64], p: f64) -> f64 {
    data.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((data.len() as f64 - 1.0) * p) as usize;
    data[idx]
}
