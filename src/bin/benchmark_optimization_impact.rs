///! Optimization Impact Benchmark - Week 11 Day 2
///!
///! A/B test to measure actual impact of optimizations:
///! - Run 100K @ 128D WITHOUT cache optimization
///! - Run 100K @ 128D WITH cache optimization
///! - Compare QPS, latency, memory

use omen::vector::custom_hnsw::{DistanceFunction, HNSWIndex, HNSWParams};
use rand::Rng;
use std::time::Instant;

fn generate_random_vector(dim: usize, seed: u64) -> Vec<f32> {
    let mut rng = seed;
    (0..dim)
        .map(|_| {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            ((rng >> 32) as f32) / (u32::MAX as f32) * 2.0 - 1.0
        })
        .collect()
}

fn run_benchmark(optimize: bool, num_vectors: usize, dimensions: usize) -> Result<(f64, f64, f64), Box<dyn std::error::Error>> {
    let params = HNSWParams {
        m: 16,
        ef_construction: 64,
        ml: 1.0 / 16f32.ln(),
        seed: 42,
        max_level: 8,
    };

    let mut index = HNSWIndex::new(dimensions, params, DistanceFunction::L2, false)?;

    // Insert vectors
    println!("  Inserting {} vectors...", num_vectors);
    let insert_start = Instant::now();
    for i in 0..num_vectors {
        let vector = generate_random_vector(dimensions, i as u64);
        index.insert(vector)?;

        if (i + 1) % 20_000 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }
    }
    println!();
    let insert_duration = insert_start.elapsed().as_secs_f64();

    // Optionally optimize cache locality
    if optimize {
        println!("  Optimizing cache locality...");
        let opt_start = Instant::now();
        index.optimize_cache_locality()?;
        let opt_duration = opt_start.elapsed();
        println!("  Optimization took: {:.3}s", opt_duration.as_secs_f64());
    }

    // Run queries
    let num_queries = 500;
    println!("  Running {} queries...", num_queries);
    let mut latencies = Vec::new();

    for i in 0..num_queries {
        let query = generate_random_vector(dimensions, (num_vectors + i) as u64);

        let q_start = Instant::now();
        let _results = index.search(&query, 10, 100)?;
        let q_duration = q_start.elapsed();

        latencies.push(q_duration.as_secs_f64() * 1000.0);
    }

    // Calculate metrics
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = latencies[num_queries / 2];
    let p95 = latencies[(num_queries * 95) / 100];
    let total_query_time: f64 = latencies.iter().sum();
    let qps = (num_queries as f64) / (total_query_time / 1000.0);

    Ok((qps, p50, p95))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("==============================================");
    println!("Optimization Impact Benchmark");
    println!("==============================================\n");

    let num_vectors = 100_000;

    // Test both 128D and 1536D to see cache impact
    for dimensions in [128, 1536] {
        println!("\n==============================================");
        println!("Testing {} dimensions", dimensions);
        println!("==============================================");
        println!("Configuration:");
        println!("  Vectors: {}", num_vectors);
        println!("  Dimensions: {}", dimensions);
        println!("  Queries: 500");
        println!();

        // Baseline (no optimization)
        println!("--- Test 1: WITHOUT Cache Optimization ---");
        let (qps_baseline, p50_baseline, p95_baseline) = run_benchmark(false, num_vectors, dimensions)?;
        println!("  QPS: {:.0}", qps_baseline);
        println!("  p50: {:.2}ms", p50_baseline);
        println!("  p95: {:.2}ms", p95_baseline);
        println!();

        // With optimization
        println!("--- Test 2: WITH Cache Optimization ---");
        let (qps_optimized, p50_optimized, p95_optimized) = run_benchmark(true, num_vectors, dimensions)?;
        println!("  QPS: {:.0}", qps_optimized);
        println!("  p50: {:.2}ms", p50_optimized);
        println!("  p95: {:.2}ms", p95_optimized);
        println!();

        // Show improvement
        println!("--- Optimization Impact ({}D) ---", dimensions);
        let qps_improvement = (qps_optimized / qps_baseline - 1.0) * 100.0;
        let p50_improvement = (1.0 - p50_optimized / p50_baseline) * 100.0;
        let p95_improvement = (1.0 - p95_optimized / p95_baseline) * 100.0;

        println!("  QPS improvement: {:.1}%", qps_improvement);
        println!("  p50 improvement: {:.1}%", p50_improvement);
        println!("  p95 improvement: {:.1}%", p95_improvement);
        println!();

        if qps_improvement > 5.0 {
            println!("✅ Cache optimization provides significant improvement!");
        } else if qps_improvement > 0.0 {
            println!("⚠️  Cache optimization provides minor improvement");
        } else {
            println!("❌ Cache optimization has no measurable benefit");
        }
    }

    Ok(())
}
