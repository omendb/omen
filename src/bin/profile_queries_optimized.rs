///! Query Profiling Benchmark - Week 11 Day 2
///!
///! Focused benchmark for profiling query performance.
///! Designed for use with flamegraph/Instruments.

use omen::vector::custom_hnsw::{DistanceFunction, HNSWIndex, HNSWParams};
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dimensions = 128; // Fast iteration
    let num_vectors = 10_000;
    let num_queries = 5000; // More queries for better profiling signal

    let params = HNSWParams {
        m: 16,
        ef_construction: 64,
        ml: 1.0 / 16f32.ln(),
        seed: 42,
        max_level: 8,
    };

    println!("Building index: {} vectors @ {}D", num_vectors, dimensions);
    let mut index = HNSWIndex::new(dimensions, params, DistanceFunction::L2, false)?;

    // Insert vectors
    for i in 0..num_vectors {
        let vector = generate_random_vector(dimensions, i as u64);
        index.insert(vector)?;
    }

    // Optimize cache locality
    println!("Optimizing cache locality...");
    index.optimize_cache_locality()?;

    // Run queries (this is what we'll profile)
    println!("Running {} queries for profiling...", num_queries);
    let query_start = Instant::now();

    for i in 0..num_queries {
        let query = generate_random_vector(dimensions, (num_vectors + i) as u64);
        let _results = index.search(&query, 10, 100)?;
    }

    let query_duration = query_start.elapsed();
    let qps = num_queries as f64 / query_duration.as_secs_f64();

    println!("Complete: {} queries in {:.2}s ({:.0} QPS)",
             num_queries, query_duration.as_secs_f64(), qps);

    Ok(())
}
