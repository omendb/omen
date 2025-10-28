//! Realistic benchmark: ALEX vs RMI with mixed workload
//!
//! Tests the real bottleneck: insert-query interleaving
//! RMI suffers from O(n) rebuilds, ALEX handles incrementally

use omen::alex::AlexTree;
use omen::index::RecursiveModelIndex;
use std::time::Instant;

fn main() {
    println!("=== ALEX vs RMI: Realistic Mixed Workload ===\n");
    println!("Workload: Bulk insert → Query 100x → Insert 1K more → Query 100x");
    println!("(Simulates production: writes invalidate RMI, forcing rebuilds)\n");

    for scale in [1_000_000, 10_000_000] {
        println!("--- Scale: {} initial keys ---", scale);
        benchmark_alex_mixed(scale);
        benchmark_rmi_mixed(scale);
        println!();
    }
}

fn benchmark_alex_mixed(n: usize) {
    let mut tree = AlexTree::new();

    // Phase 1: Bulk insert
    let start = Instant::now();
    for i in 0..n {
        tree.insert(i as i64, vec![(i % 256) as u8]).unwrap();
    }
    let bulk_insert = start.elapsed();

    // Phase 2: Queries (100 queries)
    let start = Instant::now();
    for i in 0..100 {
        let key = (i * n / 100) as i64;
        let _ = tree.get(key).unwrap();
    }
    let first_query_batch = start.elapsed();

    // Phase 3: More inserts (simulates ongoing writes)
    let start = Instant::now();
    for i in n..(n + 1000) {
        tree.insert(i as i64, vec![(i % 256) as u8]).unwrap();
    }
    let incremental_insert = start.elapsed();

    // Phase 4: Queries again (RMI would need rebuild here!)
    let start = Instant::now();
    for i in 0..100 {
        let key = (i * n / 100) as i64;
        let _ = tree.get(key).unwrap();
    }
    let second_query_batch = start.elapsed();

    let total = bulk_insert + first_query_batch + incremental_insert + second_query_batch;

    println!("  ALEX:");
    println!("    Bulk insert: {:.3}s", bulk_insert.as_secs_f64());
    println!("    Query batch 1 (100): {:.2}ms ({:.2}μs avg)",
        first_query_batch.as_secs_f64() * 1000.0,
        first_query_batch.as_micros() as f64 / 100.0);
    println!("    +1K inserts: {:.2}ms", incremental_insert.as_secs_f64() * 1000.0);
    println!("    Query batch 2 (100): {:.2}ms ({:.2}μs avg)",
        second_query_batch.as_secs_f64() * 1000.0,
        second_query_batch.as_micros() as f64 / 100.0);
    println!("    TOTAL: {:.3}s, leaves={}", total.as_secs_f64(), tree.num_leaves());
}

fn benchmark_rmi_mixed(n: usize) {
    let mut index = RecursiveModelIndex::new(n);

    // Phase 1: Bulk insert
    let start = Instant::now();
    for i in 0..n {
        index.add_key(i as i64);
    }
    let bulk_insert = start.elapsed();

    // Phase 2: Queries (100 queries) - triggers rebuild on first query!
    let start = Instant::now();
    for i in 0..100 {
        let key = (i * n / 100) as i64;
        let _ = index.search(key);
    }
    let first_query_batch = start.elapsed();

    // Phase 3: More inserts (marks dirty again)
    let start = Instant::now();
    for i in n..(n + 1000) {
        index.add_key(i as i64);
    }
    let incremental_insert = start.elapsed();

    // Phase 4: Queries again (triggers ANOTHER rebuild!)
    let start = Instant::now();
    for i in 0..100 {
        let key = (i * n / 100) as i64;
        let _ = index.search(key);
    }
    let second_query_batch = start.elapsed();

    let total = bulk_insert + first_query_batch + incremental_insert + second_query_batch;

    println!("  RMI:");
    println!("    Bulk insert: {:.3}s", bulk_insert.as_secs_f64());
    println!("    Query batch 1 (100): {:.2}ms ({:.2}μs avg) [includes O(n) rebuild]",
        first_query_batch.as_secs_f64() * 1000.0,
        first_query_batch.as_micros() as f64 / 100.0);
    println!("    +1K inserts: {:.2}ms [marks dirty]", incremental_insert.as_secs_f64() * 1000.0);
    println!("    Query batch 2 (100): {:.2}ms ({:.2}μs avg) [includes O(n) rebuild]",
        second_query_batch.as_secs_f64() * 1000.0,
        second_query_batch.as_micros() as f64 / 100.0);
    println!("    TOTAL: {:.3}s", total.as_secs_f64());
}
