//! Benchmark SIMD search vs scalar search
//!
//! Tests the performance improvement from SIMD-accelerated queries in ALEX.
//!
//! Run with: cargo run --release --bin benchmark_simd_search

use omendb::alex::AlexTree;
use rand::{Rng, SeedableRng};
use std::time::Instant;

fn main() {
    println!("=== SIMD Search Benchmark ===\n");

    // Test at different scales
    for scale in &[10_000, 100_000, 1_000_000] {
        println!("Scale: {} keys", scale);
        benchmark_search_performance(*scale);
        println!();
    }
}

fn benchmark_search_performance(num_keys: usize) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    // Create ALEX tree with random keys
    println!("  Building ALEX tree...");
    let mut alex = AlexTree::new();
    let keys: Vec<i64> = (0..num_keys).map(|_| rng.gen::<i64>()).collect();

    for &key in &keys {
        alex.insert(key, vec![1, 2, 3]).unwrap();
    }

    println!("  Tree built: {} leaves", alex.num_leaves());

    // Prepare query keys (mix of existing and non-existing)
    let num_queries = 10_000;
    let query_keys: Vec<i64> = (0..num_queries)
        .map(|i| {
            if i % 2 == 0 {
                // Existing key (50%)
                keys[i % keys.len()]
            } else {
                // Random key (50% - may or may not exist)
                rng.gen::<i64>()
            }
        })
        .collect();

    // Benchmark queries
    println!("  Running {} queries...", num_queries);

    let start = Instant::now();
    let mut found_count = 0;

    for &key in &query_keys {
        if alex.get(key).unwrap().is_some() {
            found_count += 1;
        }
    }

    let elapsed = start.elapsed();
    let ns_per_query = elapsed.as_nanos() / num_queries as u128;

    println!("  Time: {:?}", elapsed);
    println!("  Per-query: {} ns", ns_per_query);
    println!("  Found: {} / {} keys", found_count, num_queries);
    println!(
        "  Throughput: {:.2} queries/sec",
        num_queries as f64 / elapsed.as_secs_f64()
    );
}
