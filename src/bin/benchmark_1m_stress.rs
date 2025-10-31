///! 1M Vector Stress Test - Week 11 Day 2
///!
///! Validates performance and stability at production scale:
///! - 1M vectors @ 128D (fast iteration, memory validation)
///! - Measure insert throughput, query latency, QPS
///! - Memory usage tracking
///! - Stress test with concurrent queries

use omen::vector::custom_hnsw::{DistanceFunction, HNSWIndex, HNSWParams};
use rand::Rng;
use std::time::Instant;

fn generate_random_vector(dim: usize, seed: u64) -> Vec<f32> {
    // Deterministic RNG for reproducibility
    let mut rng = seed;
    (0..dim)
        .map(|_| {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            ((rng >> 32) as f32) / (u32::MAX as f32) * 2.0 - 1.0
        })
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("==============================================");
    println!("1M Vector Stress Test - Week 11 Day 2");
    println!("==============================================\n");

    let dimensions = 128; // Use 128D for faster iteration
    let num_vectors = 1_000_000;
    let num_queries = 1000;
    let k = 10;

    // HNSW parameters (pgvector defaults)
    let params = HNSWParams {
        m: 16,
        ef_construction: 64,
        ml: 1.0 / 16f32.ln(),
        seed: 42,
        max_level: 8,
    };

    println!("Configuration:");
    println!("  Dimensions: {}", dimensions);
    println!("  Vectors: {}", num_vectors);
    println!("  M: {}", params.m);
    println!("  ef_construction: {}", params.ef_construction);
    println!("  Queries: {}", num_queries);
    println!();

    // Create index
    let mut index = HNSWIndex::new(dimensions, params, DistanceFunction::L2, false)?;

    // Insert 1M vectors
    println!("Inserting {} vectors ({}D)...", num_vectors, dimensions);
    let insert_start = Instant::now();

    for i in 0..num_vectors {
        let vector = generate_random_vector(dimensions, i as u64);
        index.insert(vector)?;

        if (i + 1) % 100_000 == 0 {
            let elapsed = insert_start.elapsed().as_secs_f64();
            let rate = (i + 1) as f64 / elapsed;
            let memory_mb = index.memory_usage() as f64 / (1024.0 * 1024.0);
            println!(
                "  Inserted {} vectors... ({:.0} vec/sec, {:.2} MB)",
                i + 1, rate, memory_mb
            );
        }
    }

    let insert_duration = insert_start.elapsed();
    println!(
        "\nInsert complete: {:.2}s ({:.0} vec/sec)",
        insert_duration.as_secs_f64(),
        num_vectors as f64 / insert_duration.as_secs_f64()
    );

    // Optimize cache locality
    println!("\nOptimizing cache locality (BFS reordering)...");
    let opt_start = Instant::now();
    let num_reordered = index.optimize_cache_locality()?;
    let opt_duration = opt_start.elapsed();
    println!(
        "Reordered {} nodes in {:.3}s",
        num_reordered,
        opt_duration.as_secs_f64()
    );

    // Get index stats
    let stats = index.stats();
    println!("\n--- Index Statistics ---");
    println!("  Total vectors: {}", stats.num_vectors);
    println!("  Max level: {}", stats.max_level);
    println!("  Avg neighbors (L0): {:.1}", stats.avg_neighbors_l0);
    println!(
        "  Memory usage: {:.2} MB",
        stats.memory_bytes as f64 / (1024.0 * 1024.0)
    );
    println!(
        "  Bytes per vector: {:.0}",
        stats.memory_bytes as f64 / num_vectors as f64
    );

    // Run query stress test
    println!("\n--- Query Stress Test ---");
    println!("Running {} queries...", num_queries);
    let query_start = Instant::now();
    let mut latencies = Vec::with_capacity(num_queries);

    for i in 0..num_queries {
        let query = generate_random_vector(dimensions, (num_vectors + i) as u64);

        let q_start = Instant::now();
        let results = index.search(&query, k, 100)?;
        let q_duration = q_start.elapsed();

        latencies.push(q_duration.as_secs_f64() * 1000.0); // Convert to ms

        if results.len() != k {
            eprintln!(
                "WARNING: Query {} returned {} results instead of {}",
                i,
                results.len(),
                k
            );
        }
    }

    let query_duration = query_start.elapsed();
    let qps = num_queries as f64 / query_duration.as_secs_f64();

    // Calculate latency percentiles
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = latencies[num_queries / 2];
    let p95 = latencies[(num_queries * 95) / 100];
    let p99 = latencies[(num_queries * 99) / 100];
    let avg = latencies.iter().sum::<f64>() / latencies.len() as f64;

    println!("\n--- Query Performance ---");
    println!("  Total queries: {}", num_queries);
    println!("  Avg latency: {:.2} ms", avg);
    println!("  p50 latency: {:.2} ms", p50);
    println!("  p95 latency: {:.2} ms", p95);
    println!("  p99 latency: {:.2} ms", p99);
    println!("  QPS: {:.0}", qps);

    // Recall validation (query an existing vector)
    println!("\n--- Recall Validation ---");
    let test_id = 500_000; // Middle of dataset
    let test_vector = generate_random_vector(dimensions, test_id);
    let results = index.search(&test_vector, 1, 100)?;

    println!("Query vector ID: {}", test_id);
    println!("Nearest neighbor ID: {}", results[0].id);
    println!("Distance: {:.6}", results[0].distance);

    if results[0].distance < 0.01 {
        println!("✅ PASS: Exact match found (distance ~0)");
    } else {
        println!("⚠️  WARNING: Distance > 0.01");
    }

    // Final stats
    println!("\n==============================================");
    println!("1M Stress Test Complete!");
    println!("==============================================");
    println!("\nPerformance Summary:");
    println!("  Insert: {:.0} vec/sec", num_vectors as f64 / insert_duration.as_secs_f64());
    println!("  Query QPS: {:.0}", qps);
    println!("  Query p95: {:.2} ms", p95);
    println!("  Memory: {:.2} MB", stats.memory_bytes as f64 / (1024.0 * 1024.0));
    println!("  Memory/vector: {:.0} bytes", stats.memory_bytes as f64 / num_vectors as f64);

    Ok(())
}
