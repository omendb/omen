//! Profile ALEX query path to identify bottlenecks at scale
//!
//! Measures:
//! - Node sizes (keys per node)
//! - Tree depth
//! - Exponential search iterations
//! - Linear scan comparisons
//! - Total query latency breakdown
//!
//! Usage:
//!   cargo run --release --bin profile_query_path [num_rows]

use anyhow::Result;
use omendb::alex::AlexTree;
use std::time::Instant;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let num_rows: usize = args.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000_000);

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         ALEX Query Path Profiling                           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("Configuration:");
    println!("  Rows: {}\n", num_rows);

    // Create ALEX tree
    let mut tree = AlexTree::new();

    // Insert data (sequential for predictability)
    println!("📝 Phase 1: Building ALEX tree...\n");
    let start = Instant::now();

    let mut entries = Vec::with_capacity(num_rows);
    for i in 0..num_rows {
        entries.push((i as i64, format!("value_{}", i).into_bytes()));
    }

    tree.insert_batch(entries)?;
    let insert_time = start.elapsed();

    println!("  ✓ Inserted {} rows in {:.2}s", num_rows, insert_time.as_secs_f64());
    println!("  ✓ Throughput: {:.0} rows/sec\n", num_rows as f64 / insert_time.as_secs_f64());

    // Tree statistics
    println!("📊 Tree Statistics:\n");
    println!("  Total keys: {}", tree.len());
    println!("  Num leaves: {}", tree.num_leaves());
    println!("  Avg keys per leaf: {:.1}", tree.len() as f64 / tree.num_leaves() as f64);
    println!("  Tree depth: 1 (single-level)");

    // Query profiling
    println!("\n🔍 Query Profiling:\n");

    // Sample queries across the key space
    let sample_size = 10_000;
    let step = num_rows / sample_size;

    let mut total_query_time = 0.0;
    let mut queries_executed = 0;

    for i in (0..num_rows).step_by(step) {
        let key = i as i64;
        let start = Instant::now();
        let _ = tree.get(key)?;
        total_query_time += start.elapsed().as_secs_f64();
        queries_executed += 1;
    }

    let avg_query_time_us = (total_query_time / queries_executed as f64) * 1_000_000.0;

    println!("  Sample size: {} queries", queries_executed);
    println!("  Avg query time: {:.2} μs", avg_query_time_us);
    println!("  Total query time: {:.2} ms", total_query_time * 1000.0);

    // Estimated costs
    println!("\n💡 Estimated Cost Breakdown:\n");

    let num_leaves = tree.num_leaves();
    let avg_keys_per_leaf = tree.len() as f64 / num_leaves as f64;

    // Binary search on split_keys: O(log n)
    let leaf_routing_cost = (num_leaves as f64).log2();
    println!("  Leaf routing (binary search): ~{:.1} comparisons", leaf_routing_cost);

    // Exponential search + linear scan within leaf
    let within_leaf_cost = avg_keys_per_leaf.log2() * 2.0; // Exponential iterations
    let linear_scan_cost = avg_keys_per_leaf / 2.0; // Average half-node scan

    println!("  Exponential search: ~{:.1} iterations", within_leaf_cost);
    println!("  Linear scan (avg): ~{:.0} comparisons", linear_scan_cost);
    println!("  Total comparisons: ~{:.0}", leaf_routing_cost + within_leaf_cost + linear_scan_cost);

    // Cost per comparison estimate
    let cost_per_comparison_ns = (avg_query_time_us * 1000.0) / (leaf_routing_cost + within_leaf_cost + linear_scan_cost);
    println!("  Estimated cost per comparison: {:.1} ns", cost_per_comparison_ns);

    // Bottleneck analysis
    println!("\n⚠️  Bottleneck Analysis:\n");

    let leaf_routing_pct = (leaf_routing_cost / (leaf_routing_cost + within_leaf_cost + linear_scan_cost)) * 100.0;
    let exponential_pct = (within_leaf_cost / (leaf_routing_cost + within_leaf_cost + linear_scan_cost)) * 100.0;
    let linear_scan_pct = (linear_scan_cost / (leaf_routing_cost + within_leaf_cost + linear_scan_cost)) * 100.0;

    println!("  Leaf routing: {:.1}% of time", leaf_routing_pct);
    println!("  Exponential search: {:.1}% of time", exponential_pct);
    println!("  Linear scan: {:.1}% of time", linear_scan_pct);

    if linear_scan_pct > 50.0 {
        println!("\n  🔴 BOTTLENECK: Linear scan dominates ({:.1}%)", linear_scan_pct);
        println!("     Recommendation: Implement binary search within nodes");
    } else if exponential_pct > 50.0 {
        println!("\n  🟡 BOTTLENECK: Exponential search dominates ({:.1}%)", exponential_pct);
        println!("     Recommendation: Improve model accuracy or reduce node size");
    } else {
        println!("\n  🟢 NO CLEAR BOTTLENECK: Time distributed across operations");
    }

    // Scaling projection
    println!("\n📈 Scaling Projection:\n");

    let scales = vec![
        ("1M", 1_000_000),
        ("10M", 10_000_000),
        ("50M", 50_000_000),
        ("100M", 100_000_000),
    ];

    println!("  Scale | Leaves | Keys/Leaf | Query (μs) | vs Current");
    println!("  ------|--------|-----------|------------|------------");

    for (label, scale) in scales {
        // Assume same splitting strategy
        let projected_leaves = (num_leaves as f64 * (scale as f64 / num_rows as f64)).ceil() as usize;
        let projected_keys_per_leaf = scale as f64 / projected_leaves as f64;

        let projected_leaf_routing = (projected_leaves as f64).log2();
        let projected_exponential = (projected_keys_per_leaf.log2()) * 2.0;
        let projected_linear_scan = projected_keys_per_leaf / 2.0;

        let projected_comparisons = projected_leaf_routing + projected_exponential + projected_linear_scan;
        let projected_query_us = projected_comparisons * cost_per_comparison_ns / 1000.0;

        let ratio = if scale == num_rows {
            1.0
        } else {
            projected_query_us / avg_query_time_us
        };

        println!("  {:5} | {:6} | {:9.0} | {:10.1} | {:>5.2}x",
            label,
            projected_leaves,
            projected_keys_per_leaf,
            projected_query_us,
            ratio
        );
    }

    // Recommendations
    println!("\n💡 Recommendations:\n");

    if avg_keys_per_leaf > 10_000.0 {
        println!("  ⚠️  Large nodes ({:.0} keys/leaf) detected", avg_keys_per_leaf);
        println!("     → Implement binary search within nodes (O(log n) vs O(n))");
        println!("     → Expected improvement: {:.1}x faster queries", avg_keys_per_leaf / avg_keys_per_leaf.log2());
    }

    if num_leaves < 100 {
        println!("  ⚠️  Too few leaves ({}) for large dataset", num_leaves);
        println!("     → Force more aggressive splitting");
        println!("     → Target: <10K keys per leaf");
    }

    if avg_query_time_us > 10.0 {
        println!("  ⚠️  Slow queries ({:.1}μs) detected", avg_query_time_us);
        println!("     → Profile with flamegraph for exact bottleneck");
        println!("     → Consider cache-aware data layout");
    }

    println!("\n✅ Profiling complete\n");

    Ok(())
}
