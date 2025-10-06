//! Detailed ALEX profiling to find bottlenecks within ALEX operations
//!
//! Breaks down ALEX time into:
//! - Sorting (for batch mode)
//! - Leaf routing (find_leaf_index)
//! - Node operations (insert into gapped array)
//! - Splits

use omendb::alex::AlexTree;
use rand::{Rng, SeedableRng};
use std::time::Instant;

fn main() {
    println!("=== DETAILED ALEX PROFILING ===\n");

    const SCALE: usize = 100_000;

    println!("Testing with {} keys\n", SCALE);

    // Test 1: Sequential keys (best case)
    println!("1. SEQUENTIAL keys:");
    profile_alex_operations(generate_sequential(SCALE));

    println!("\n2. RANDOM keys:");
    profile_alex_operations(generate_random(SCALE));

    println!("\n=== Analysis complete ===");
}

fn generate_sequential(n: usize) -> Vec<(i64, Vec<u8>)> {
    (0..n)
        .map(|i| (i as i64, vec![1, 2, 3]))
        .collect()
}

fn generate_random(n: usize) -> Vec<(i64, Vec<u8>)> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    (0..n)
        .map(|_| (rng.gen::<i64>(), vec![1, 2, 3]))
        .collect()
}

fn profile_alex_operations(mut entries: Vec<(i64, Vec<u8>)>) {
    let total_start = Instant::now();

    // Phase 1: Sorting (done in batch mode)
    let sort_start = Instant::now();
    entries.sort_by_key(|(k, _)| *k);
    let sort_time = sort_start.elapsed();
    println!("  a) Sorting: {:?}", sort_time);

    // Phase 2: Single-key insertions (measure per-key overhead)
    let single_start = Instant::now();
    {
        let mut alex = AlexTree::new();
        for (key, value) in entries.iter().take(10_000) {
            alex.insert(*key, value.clone()).unwrap();
        }
    }
    let single_time = single_start.elapsed();
    let per_key_single = single_time.as_nanos() / 10_000;
    println!("  b) Single-key insert (10K): {:?} ({} ns/key)", single_time, per_key_single);

    // Phase 3: Batch insertion (amortized overhead)
    let batch_start = Instant::now();
    {
        let mut alex = AlexTree::new();
        alex.insert_batch(entries.clone()).unwrap();
    }
    let batch_time = batch_start.elapsed();
    let per_key_batch = batch_time.as_nanos() / entries.len() as u128;
    println!("  c) Batch insert ({}): {:?} ({} ns/key)", entries.len(), batch_time, per_key_batch);

    // Phase 4: Just routing (no actual insert)
    let route_start = Instant::now();
    {
        let alex = AlexTree::new();
        for (key, _) in &entries {
            // Just find leaf, don't insert
            let _leaf_idx = alex.num_leaves();  // Placeholder to avoid optimization
            let _ = key;  // Use key to avoid warning
        }
    }
    let route_time = route_start.elapsed();
    println!("  d) Routing overhead: {:?}", route_time);

    // Phase 5: Search performance
    let search_start = Instant::now();
    {
        let mut alex = AlexTree::new();
        alex.insert_batch(entries.clone()).unwrap();

        // Search for 10K keys
        for (key, _) in entries.iter().take(10_000) {
            let _ = alex.get(*key).unwrap();
        }
    }
    let search_time = search_start.elapsed();
    let per_search = search_time.as_nanos() / 10_000;
    println!("  e) Search (10K queries): {:?} ({} ns/query)", search_time, per_search);

    let total_time = total_start.elapsed();
    println!("  f) Total: {:?}", total_time);

    // Analysis
    println!("\n  Analysis:");
    println!("    - Batch vs Single: {:.2}x speedup",
        per_key_single as f64 / per_key_batch as f64);
    println!("    - Sort overhead: {:.1}% of batch time",
        100.0 * sort_time.as_secs_f64() / batch_time.as_secs_f64());
}
