///! Profiling benchmark for HNSW operations
///!
///! Focused workload for CPU profiling:
///! - 10K vectors @ 128D (fast, representative)
///! - Insert + search operations
///! - Designed for flamegraph analysis

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
    println!("=== HNSW Profiling Benchmark ===");
    println!("Workload: 10K vectors @ 128D");
    println!("Focus: Insert + Search operations\n");

    let dimensions = 128;
    let num_vectors = 10_000;
    let num_queries = 1000;
    let k = 10;

    let params = HNSWParams {
        m: 16,
        ef_construction: 64,
        ml: 1.0 / 16f32.ln(),
        seed: 42,
        max_level: 8,
    };

    // Phase 1: Insert (CPU intensive)
    println!("Phase 1: Inserting {} vectors...", num_vectors);
    let insert_start = Instant::now();
    let mut index = HNSWIndex::new(dimensions, params, DistanceFunction::L2, false)?;

    for i in 0..num_vectors {
        let vector = generate_random_vector(dimensions, i as u64);
        index.insert(vector)?;
    }

    let insert_duration = insert_start.elapsed();
    println!(
        "  Inserted in {:.2}s ({:.0} vec/sec)\n",
        insert_duration.as_secs_f64(),
        num_vectors as f64 / insert_duration.as_secs_f64()
    );

    // Phase 2: Search (CPU intensive)
    println!("Phase 2: Running {} queries...", num_queries);
    let search_start = Instant::now();

    for i in 0..num_queries {
        let query = generate_random_vector(dimensions, (num_vectors + i) as u64);
        let _ = index.search(&query, k, 100)?;
    }

    let search_duration = search_start.elapsed();
    let qps = num_queries as f64 / search_duration.as_secs_f64();
    println!(
        "  Searched in {:.2}s ({:.0} QPS)\n",
        search_duration.as_secs_f64(),
        qps
    );

    // Phase 3: Mixed workload (realistic usage)
    println!("Phase 3: Mixed workload (insert + search)...");
    let mixed_start = Instant::now();

    // Simulate real usage: 10% inserts, 90% searches
    for i in 0..1000 {
        if i % 10 == 0 {
            // Insert
            let vector = generate_random_vector(dimensions, (num_vectors + 1000 + i) as u64);
            index.insert(vector)?;
        } else {
            // Search
            let query = generate_random_vector(dimensions, (num_vectors + 2000 + i) as u64);
            let _ = index.search(&query, k, 100)?;
        }
    }

    let mixed_duration = mixed_start.elapsed();
    println!("  Mixed workload in {:.2}s\n", mixed_duration.as_secs_f64());

    println!("=== Profiling Complete ===");
    println!("Total time: {:.2}s", (insert_duration + search_duration + mixed_duration).as_secs_f64());

    Ok(())
}
