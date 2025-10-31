///! Quick SIMD Performance Test - 128D
///!
///! Test SIMD performance on 128D vectors (fast iteration)

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
    let dimensions = 128;
    let num_vectors = 10_000;
    let num_queries = 500;

    let params = HNSWParams {
        m: 16,
        ef_construction: 64,
        ml: 1.0 / 16f32.ln(),
        seed: 42,
        max_level: 8,
    };

    println!("Building index: {} vectors @ {}D", num_vectors, dimensions);
    let mut index = HNSWIndex::new(dimensions, params, DistanceFunction::L2, false)?;

    let insert_start = Instant::now();
    for i in 0..num_vectors {
        let vector = generate_random_vector(dimensions, i as u64);
        index.insert(vector)?;
    }
    let insert_duration = insert_start.elapsed();

    println!("Optimizing cache locality...");
    index.optimize_cache_locality()?;

    println!("Running {} queries...", num_queries);
    let query_start = Instant::now();
    let mut latencies = Vec::new();

    for i in 0..num_queries {
        let query = generate_random_vector(dimensions, (num_vectors + i) as u64);
        let q_start = Instant::now();
        let _results = index.search(&query, 10, 100)?;
        latencies.push(q_start.elapsed().as_secs_f64() * 1000.0);
    }

    let query_duration = query_start.elapsed();
    let qps = num_queries as f64 / query_duration.as_secs_f64();

    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = latencies[num_queries / 2];
    let p95 = latencies[(num_queries * 95) / 100];

    println!("\n=== Results (128D) ===");
    println!("Insert: {:.0} vec/sec", num_vectors as f64 / insert_duration.as_secs_f64());
    println!("QPS: {:.0}", qps);
    println!("p50: {:.2}ms", p50);
    println!("p95: {:.2}ms", p95);

    Ok(())
}
