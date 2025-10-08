//! Quick YCSB test to validate benchmark implementation

use anyhow::Result;
use omendb::alex::MultiLevelAlexTree;
use rand::prelude::*;
use std::time::Instant;
use tracing::info;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("╔══════════════════════════════════════════════════════════════╗");
    info!("║              YCSB Quick Test - OmenDB                       ║");
    info!("╚══════════════════════════════════════════════════════════════╝");

    // Small dataset for quick validation
    let record_count = 10_000;
    let operation_count = 10_000;

    // Generate initial dataset
    info!("\n🔄 Generating {} records...", record_count);
    let mut data = Vec::with_capacity(record_count);
    for i in 0..record_count {
        let key = i as i64;
        let value = format!("value_{}", i).into_bytes();
        data.push((key, value));
    }

    // Build tree
    info!("📦 Building multi-level ALEX tree...");
    let start = Instant::now();
    let mut tree = MultiLevelAlexTree::bulk_build(data)?;
    let build_time = start.elapsed();

    info!("  ✅ Build time: {:.2}ms", build_time.as_millis());
    info!("  Height: {}", tree.height());
    info!("  Leaves: {}", tree.num_leaves());

    // Test Workload C (Read Only) - simplest to validate
    info!("\n🔍 Testing Workload C: Read Only (100% reads)");
    let mut rng = thread_rng();
    let mut latencies = Vec::new();

    let start = Instant::now();
    for _ in 0..operation_count {
        let key = rng.gen_range(0..record_count) as i64;
        let op_start = Instant::now();
        let _ = tree.get(key)?;
        latencies.push(op_start.elapsed().as_nanos());
    }
    let total_time = start.elapsed();

    // Calculate metrics
    latencies.sort_unstable();
    let count = latencies.len();
    let avg = latencies.iter().sum::<u128>() as f64 / count as f64;
    let p50 = latencies[count * 50 / 100];
    let p99 = latencies[count * 99 / 100];
    let throughput = operation_count as f64 / total_time.as_secs_f64();

    info!("\n📊 Results:");
    info!("  Operations: {}", operation_count);
    info!("  Total time: {:.2}ms", total_time.as_millis());
    info!("  Throughput: {:.0} ops/sec", throughput);
    info!("  Avg latency: {:.1} ns", avg);
    info!("  P50 latency: {} ns", p50);
    info!("  P99 latency: {} ns", p99);

    // Test mixed workload (Workload A simplified)
    info!("\n🔄 Testing Mixed Workload: 50% reads, 50% updates");
    let start = Instant::now();
    let mut read_count = 0;
    let mut update_count = 0;

    for _ in 0..operation_count {
        let key = rng.gen_range(0..record_count) as i64;

        if rng.gen::<f64>() < 0.5 {
            // Read
            let _ = tree.get(key)?;
            read_count += 1;
        } else {
            // Update (insert with existing key)
            let value = format!("updated_value_{}", key).into_bytes();
            tree.insert(key, value)?;
            update_count += 1;
        }
    }
    let mixed_time = start.elapsed();
    let mixed_throughput = operation_count as f64 / mixed_time.as_secs_f64();

    info!("  Read operations: {}", read_count);
    info!("  Update operations: {}", update_count);
    info!("  Total time: {:.2}ms", mixed_time.as_millis());
    info!("  Throughput: {:.0} ops/sec", mixed_throughput);

    // Comparison with typical performance
    info!("\n📈 Performance Assessment:");
    if throughput > 500_000.0 {
        info!("  🚀 EXCELLENT: >500K ops/sec (top tier)");
    } else if throughput > 200_000.0 {
        info!("  ✅ GOOD: >200K ops/sec (competitive)");
    } else if throughput > 50_000.0 {
        info!("  ⚠️  ACCEPTABLE: >50K ops/sec (baseline)");
    } else {
        info!("  ❌ POOR: <50K ops/sec (needs optimization)");
    }

    info!("\n✅ Quick YCSB test complete!");
    info!("Ready to run full benchmark suite with:");
    info!("  cargo run --release --bin ycsb_benchmark");

    Ok(())
}