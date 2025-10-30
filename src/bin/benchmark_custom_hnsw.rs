//! Custom HNSW Benchmark - Week 9 Day 5
//!
//! Validate custom HNSW implementation baseline performance
//! - Insert 10K vectors (128D for quick testing)
//! - Query k=10 nearest neighbors
//! - Measure latency and QPS
//! - Compare against hnsw_rs baseline

use omen::vector::custom_hnsw::{DistanceFunction, HNSWIndex, HNSWParams};
use rand::Rng;
use std::time::Instant;

fn generate_random_vector(dim: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

fn main() {
    println!("==============================================");
    println!("Custom HNSW Benchmark - Week 9 Day 5");
    println!("==============================================\n");

    let dimensions = 128; // Start with 128D for fast iteration
    let num_vectors = 10_000;
    let num_queries = 1000;
    let k = 10;

    // Create custom HNSW index with default params
    let params = HNSWParams::default();
    println!("Parameters:");
    println!("  M: {}", params.m);
    println!("  ef_construction: {}", params.ef_construction);
    println!("  max_level: {}", params.max_level);
    println!();

    let mut index = HNSWIndex::new(dimensions, params, DistanceFunction::L2, false).unwrap();

    // Insert vectors
    println!("Inserting {} vectors ({}D)...", num_vectors, dimensions);
    let start = Instant::now();
    for i in 0..num_vectors {
        let vector = generate_random_vector(dimensions);
        index.insert(vector).unwrap();

        if (i + 1) % 2000 == 0 {
            println!("  Inserted {} vectors...", i + 1);
        }
    }
    let insert_duration = start.elapsed();

    println!("\n--- Insert Performance ---");
    println!("Total time: {:?}", insert_duration);
    println!(
        "Throughput: {:.0} vec/sec",
        num_vectors as f64 / insert_duration.as_secs_f64()
    );

    // Graph statistics
    println!("\n--- Graph Statistics ---");
    let total_neighbors: usize = (0..num_vectors as u32)
        .map(|id| index.neighbor_count(id, 0))
        .sum();
    let avg_neighbors = total_neighbors as f64 / num_vectors as f64;
    println!("Average neighbors at level 0: {:.1}", avg_neighbors);

    // Check entry point
    if let Some(ep) = index.entry_point() {
        println!("Entry point ID: {}", ep);
        if let Some(level) = index.node_level(ep) {
            println!("Entry point level: {}", level);
        }
    }

    // Query benchmark
    println!("\n--- Query Performance ---");
    println!("Running {} queries (k={})...\n", num_queries, k);

    let mut query_times = Vec::new();
    let ef_search = 100; // Standard ef for search

    for i in 0..num_queries {
        let query = generate_random_vector(dimensions);

        let start = Instant::now();
        let results = index.search(&query, k, ef_search).unwrap();
        let duration = start.elapsed();

        query_times.push(duration.as_secs_f64() * 1000.0); // Convert to ms

        // Only assert for first few queries to see what's happening
        if i < 5 && results.len() != k {
            println!("  WARNING: Query {} returned {} results instead of {}", i, results.len(), k);
        }

        if (i + 1) % 100 == 0 {
            println!(
                "  Query {}: {:.2}ms, {} results",
                i + 1,
                duration.as_secs_f64() * 1000.0,
                results.len()
            );
        }
    }

    println!("\nCompleted {} queries", num_queries);

    // Calculate statistics
    query_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = query_times[num_queries / 2];
    let p95 = query_times[(num_queries as f32 * 0.95) as usize];
    let p99 = query_times[(num_queries as f32 * 0.99) as usize];
    let avg = query_times.iter().sum::<f64>() / query_times.len() as f64;

    println!("\n--- Latency Statistics ---");
    println!("Average: {:.2}ms", avg);
    println!("p50: {:.2}ms", p50);
    println!("p95: {:.2}ms", p95);
    println!("p99: {:.2}ms", p99);

    // Calculate QPS
    let total_query_time: f64 = query_times.iter().sum();
    let qps = (num_queries as f64) / (total_query_time / 1000.0);
    println!("\n--- Throughput ---");
    println!("Queries per second (QPS): {:.0}", qps);

    // Validate recall (query an existing vector)
    println!("\n--- Recall Validation ---");
    let test_vector = generate_random_vector(dimensions);
    let test_id = index.insert(test_vector.clone()).unwrap();
    let results = index.search(&test_vector, 1, ef_search).unwrap();

    println!("Query vector ID: {}", test_id);
    println!("Nearest neighbor ID: {}", results[0].id);
    println!("Distance: {:.6}", results[0].distance);

    if results[0].id == test_id {
        println!("✅ PASS: Exact match found (distance ~0)");
    } else {
        println!("⚠️  WARNING: Exact match not found as nearest neighbor");
    }

    // Goal checks
    println!("\n==============================================");
    println!("Week 9 Day 5 Goal Check");
    println!("==============================================");

    println!("\nBaseline Target: 500-600 QPS");
    if qps >= 500.0 {
        println!("✅ PASS: QPS {:.0} >= 500 target", qps);
    } else {
        println!("⚠️  WARNING: QPS {:.0} < 500 target", qps);
    }

    println!("\nLatency Target: p95 < 10ms");
    if p95 < 10.0 {
        println!("✅ PASS: p95 latency {:.2}ms < 10ms target", p95);
    } else {
        println!("⚠️  WARNING: p95 latency {:.2}ms > 10ms target", p95);
    }

    println!("\nRecall Target: Exact match found");
    if results[0].id == test_id && results[0].distance < 0.01 {
        println!("✅ PASS: Recall validation (exact match found)");
    } else {
        println!("⚠️  WARNING: Recall validation failed");
    }

    // Memory usage
    let memory_mb = index.memory_usage() as f64 / (1024.0 * 1024.0);
    println!("\n--- Memory Usage ---");
    println!("Index size: {:.2} MB", memory_mb);
    println!("Bytes per vector: {:.0}", index.memory_usage() as f64 / num_vectors as f64);

    println!("\n==============================================");
    println!("Benchmark Complete!");
    println!("==============================================");
}
