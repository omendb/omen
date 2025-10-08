//! Benchmark multi-level ALEX vs single-level at various scales
//!
//! Tests whether the multi-level architecture fixes the cache locality
//! bottleneck at 50M+ rows.

use anyhow::Result;
use omendb::alex::{AlexTree, MultiLevelAlexTree};
use rand::prelude::*;
use std::time::Instant;

fn main() -> Result<()> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     Multi-Level ALEX Performance Comparison                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Test at various scales
    let scales = vec![
        ("1M", 1_000_000),
        ("5M", 5_000_000),
        ("10M", 10_000_000),
        ("50M", 50_000_000),
    ];

    for (label, size) in scales {
        println!("\n📊 Testing at {} scale ({} rows)", label, size);
        println!("═══════════════════════════════════════════════════════════════\n");

        benchmark_scale(size)?;
    }

    Ok(())
}

fn benchmark_scale(size: usize) -> Result<()> {
    // Generate test data
    println!("🔄 Generating {} test keys...", size);
    let mut data = Vec::with_capacity(size);
    let mut rng = thread_rng();

    for _ in 0..size {
        data.push(rng.gen::<i64>());
    }

    // Sort for bulk loading
    data.sort();

    // Prepare data with values for multi-level
    let data_with_values: Vec<(i64, Vec<u8>)> = data
        .iter()
        .map(|&k| (k, vec![0u8; 8])) // Small value
        .collect();

    // Build single-level ALEX
    println!("\n📦 Building single-level ALEX tree...");
    let start = Instant::now();
    let mut single_tree = AlexTree::new();

    // Batch insert for efficiency
    let batch_size = 10_000;
    for chunk in data.chunks(batch_size) {
        let batch: Vec<_> = chunk.iter().map(|&k| (k, vec![0u8; 8])).collect();
        single_tree.insert_batch(batch)?;
    }

    let single_build_time = start.elapsed();
    println!("  Build time: {:.2}s", single_build_time.as_secs_f64());
    println!("  Leaves: {}", single_tree.num_leaves());

    // Build multi-level ALEX
    println!("\n📦 Building multi-level ALEX tree...");
    let start = Instant::now();
    let multi_tree = MultiLevelAlexTree::bulk_build(data_with_values.clone())?;
    let multi_build_time = start.elapsed();
    println!("  Build time: {:.2}s", multi_build_time.as_secs_f64());
    println!("  Height: {}", multi_tree.height());
    println!("  Leaves: {}", multi_tree.num_leaves());

    // Generate query keys (sample 10K random keys)
    let query_keys: Vec<i64> = data.choose_multiple(&mut rng, 10_000.min(size))
        .copied()
        .collect();

    // Benchmark single-level queries
    println!("\n🔍 Testing single-level queries...");
    let start = Instant::now();
    let mut single_found = 0;
    for &key in &query_keys {
        if single_tree.get(key)?.is_some() {
            single_found += 1;
        }
    }
    let single_query_time = start.elapsed();
    let single_query_avg = single_query_time.as_nanos() as f64 / query_keys.len() as f64;

    println!("  Total time: {:.2}ms", single_query_time.as_millis());
    println!("  Avg per query: {:.1}ns", single_query_avg);
    println!("  Found: {}/{}", single_found, query_keys.len());

    // Benchmark multi-level queries
    println!("\n🔍 Testing multi-level queries...");
    let start = Instant::now();
    let mut multi_found = 0;
    for &key in &query_keys {
        if multi_tree.get(key)?.is_some() {
            multi_found += 1;
        }
    }
    let multi_query_time = start.elapsed();
    let multi_query_avg = multi_query_time.as_nanos() as f64 / query_keys.len() as f64;

    println!("  Total time: {:.2}ms", multi_query_time.as_millis());
    println!("  Avg per query: {:.1}ns", multi_query_avg);
    println!("  Found: {}/{}", multi_found, query_keys.len());

    // Compare results
    println!("\n📈 Performance Comparison:");
    println!("  Build speedup: {:.2}x",
             single_build_time.as_secs_f64() / multi_build_time.as_secs_f64());
    println!("  Query speedup: {:.2}x",
             single_query_avg / multi_query_avg);

    if multi_query_avg < single_query_avg {
        println!("  ✅ Multi-level is FASTER by {:.1}%",
                 ((single_query_avg - multi_query_avg) / single_query_avg) * 100.0);
    } else {
        println!("  ⚠️ Single-level is faster by {:.1}%",
                 ((multi_query_avg - single_query_avg) / multi_query_avg) * 100.0);
    }

    // Test random inserts (small sample)
    println!("\n📝 Testing inserts...");
    let insert_keys: Vec<i64> = (0..1000).map(|_| rng.gen()).collect();

    // Single-level inserts
    let start = Instant::now();
    for &key in &insert_keys {
        single_tree.insert(key, vec![0u8; 8])?;
    }
    let single_insert_time = start.elapsed();

    // Multi-level inserts
    let mut multi_tree_mut = MultiLevelAlexTree::bulk_build(data_with_values)?;
    let start = Instant::now();
    for &key in &insert_keys {
        multi_tree_mut.insert(key, vec![0u8; 8])?;
    }
    let multi_insert_time = start.elapsed();

    println!("  Single-level: {:.2}μs avg",
             single_insert_time.as_nanos() as f64 / insert_keys.len() as f64 / 1000.0);
    println!("  Multi-level: {:.2}μs avg",
             multi_insert_time.as_nanos() as f64 / insert_keys.len() as f64 / 1000.0);
    println!("  Speedup: {:.2}x",
             single_insert_time.as_secs_f64() / multi_insert_time.as_secs_f64());

    Ok(())
}