//! Quick test at 25M scale to validate before 100M

use anyhow::Result;
use omendb::alex::MultiLevelAlexTree;
use rand::prelude::*;
use std::time::Instant;

fn main() -> Result<()> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              25M Quick Test - Multi-Level ALEX              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let size = 25_000_000;

    // Generate test data
    println!("🔄 Generating {} test keys...", size);
    let start = Instant::now();

    // Use deterministic seed for reproducibility
    let mut rng = StdRng::seed_from_u64(12345);
    let mut data = Vec::with_capacity(size);

    for _ in 0..size {
        data.push(rng.gen::<i64>());
    }

    let gen_time = start.elapsed();
    println!("  Generation time: {:.2}s", gen_time.as_secs_f64());

    // Sort for bulk loading
    println!("  Sorting keys...");
    let start = Instant::now();
    data.sort_unstable();
    let sort_time = start.elapsed();
    println!("  Sort time: {:.2}s", sort_time.as_secs_f64());

    // Prepare data with values
    let data_with_values: Vec<(i64, Vec<u8>)> = data
        .iter()
        .map(|&k| (k, vec![0u8; 8]))
        .collect();

    // Build multi-level ALEX
    println!("\n📦 Building multi-level ALEX tree...");
    let start = Instant::now();
    let tree = MultiLevelAlexTree::bulk_build(data_with_values)?;
    let alex_build_time = start.elapsed();

    println!("  ✅ Build time: {:.2}s", alex_build_time.as_secs_f64());
    println!("  Height: {}", tree.height());
    println!("  Leaves: {}", tree.num_leaves());
    println!("  Keys/leaf: {:.1}", size as f64 / tree.num_leaves() as f64);

    // Memory estimation
    let leaf_memory = tree.num_leaves() * 88;
    let total_memory = leaf_memory + (tree.num_leaves() * 8);
    println!("  Memory estimate: {:.2} MB", total_memory as f64 / (1024.0 * 1024.0));

    // Test queries
    println!("\n🔍 Testing queries (1000 samples)...");
    let mut rng = thread_rng();
    let query_keys: Vec<i64> = data.choose_multiple(&mut rng, 1000)
        .copied()
        .collect();

    let start = Instant::now();
    let mut found = 0;
    for &key in &query_keys {
        if tree.get(key)?.is_some() {
            found += 1;
        }
    }
    let query_time = start.elapsed();
    let avg_query = query_time.as_nanos() as f64 / query_keys.len() as f64;

    println!("  Total time: {:.2}ms", query_time.as_millis());
    println!("  Avg per query: {:.1}ns", avg_query);
    println!("  Found: {}/{}", found, query_keys.len());

    // Summary
    println!("\n✅ 25M Summary:");
    println!("  Build performance: {:.1} M keys/sec",
             size as f64 / alex_build_time.as_secs_f64() / 1_000_000.0);
    println!("  Query throughput: {:.1} M queries/sec",
             1_000_000_000.0 / avg_query / 1_000_000.0);
    println!("  Space efficiency: {:.2} bytes/key",
             total_memory as f64 / size as f64);

    Ok(())
}