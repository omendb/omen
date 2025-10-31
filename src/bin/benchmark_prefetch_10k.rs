/// Quick benchmark for testing prefetching optimization
///
/// Uses 10K vectors for fast iteration during optimization work.
/// Measures QPS improvement from cache optimizations.

use omen::vector::custom_hnsw::{DistanceFunction, HNSWIndex, HNSWParams};
use std::time::Instant;

fn generate_random_vector(dimensions: usize, seed: u64) -> Vec<f32> {
    // Simple LCG for deterministic random numbers
    let mut rng = seed;
    (0..dimensions)
        .map(|_| {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            ((rng >> 32) as f32) / (u32::MAX as f32)
        })
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("==============================================");
    println!("Prefetch Optimization Benchmark - 10K vectors");
    println!("==============================================\n");

    let dimensions = 1536;
    let num_vectors = 10_000;
    let num_queries = 100;

    let mut params = HNSWParams::default();
    params.m = 16; // Same as 100K benchmark
    params.ef_construction = 64;

    println!("Configuration:");
    println!("  Dimensions: {}", dimensions);
    println!("  Vectors: {}", num_vectors);
    println!("  M: {}", params.m);
    println!("  ef_construction: {}", params.ef_construction);
    println!("  Queries: {}", num_queries);
    println!();

    // Create index
    let mut index = HNSWIndex::new(dimensions, params, DistanceFunction::L2, false)?;

    // Insert vectors
    println!("Inserting {} vectors ({}D)...", num_vectors, dimensions);
    let insert_start = Instant::now();

    for i in 0..num_vectors {
        let vector = generate_random_vector(dimensions, i as u64);
        index.insert(vector)?;

        if (i + 1) % 2000 == 0 {
            let elapsed = insert_start.elapsed().as_secs_f64();
            let rate = (i + 1) as f64 / elapsed;
            println!("  Inserted {} vectors... ({:.0} vec/sec)", i + 1, rate);
        }
    }

    let insert_duration = insert_start.elapsed();
    println!("Insert complete: {:.2}s ({:.0} vec/sec)\n",
        insert_duration.as_secs_f64(),
        num_vectors as f64 / insert_duration.as_secs_f64()
    );

    // Optimize cache locality
    println!("Optimizing cache locality (BFS reordering)...");
    let opt_start = Instant::now();
    let num_reordered = index.optimize_cache_locality()?;
    let opt_duration = opt_start.elapsed();
    println!("Reordered {} nodes in {:.3}s\n", num_reordered, opt_duration.as_secs_f64());

    // Get index stats
    let stats = index.stats();
    println!("Index Statistics:");
    println!("  Total vectors: {}", stats.num_vectors);
    println!("  Max level: {}", stats.max_level);
    println!("  Avg neighbors (L0): {:.1}", stats.avg_neighbors_l0);
    println!("  Memory usage: {:.2} MB\n", stats.memory_bytes as f64 / (1024.0 * 1024.0));

    // Run queries
    println!("Running {} queries...", num_queries);
    let query_start = Instant::now();
    let mut total_results = 0;
    let mut latencies = Vec::with_capacity(num_queries);

    for i in 0..num_queries {
        let query = generate_random_vector(dimensions, (num_vectors + i) as u64);

        let q_start = Instant::now();
        let results = index.search(&query, 10, 100)?;
        let q_duration = q_start.elapsed();

        latencies.push(q_duration.as_secs_f64() * 1000.0); // Convert to ms
        total_results += results.len();
    }

    let query_duration = query_start.elapsed();
    let qps = num_queries as f64 / query_duration.as_secs_f64();

    // Calculate latency percentiles
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = latencies[num_queries / 2];
    let p95 = latencies[(num_queries * 95) / 100];
    let p99 = latencies[(num_queries * 99) / 100];
    let avg = latencies.iter().sum::<f64>() / latencies.len() as f64;

    println!("\nQuery Performance:");
    println!("  Total queries: {}", num_queries);
    println!("  Total results: {}", total_results);
    println!("  Avg latency: {:.2} ms", avg);
    println!("  p50 latency: {:.2} ms", p50);
    println!("  p95 latency: {:.2} ms", p95);
    println!("  p99 latency: {:.2} ms", p99);
    println!("  QPS: {:.0}", qps);
    println!();

    println!("==============================================");
    println!("Benchmark Complete!");
    println!("==============================================");

    Ok(())
}
