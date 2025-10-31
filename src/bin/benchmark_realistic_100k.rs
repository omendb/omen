//! Realistic HNSW Benchmark - Week 10 Day 3
//!
//! Production-like validation with:
//! - 1536D vectors (OpenAI embedding size)
//! - 100K vectors (production scale)
//! - Measure insert throughput, query latency, QPS, memory usage
//! - Validate that performance scales with realistic data

use omen::vector::custom_hnsw::{DistanceFunction, HNSWIndex, HNSWParams};
use rand::Rng;
use std::time::Instant;

fn generate_random_vector(dim: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

fn main() {
    println!("==============================================");
    println!("Realistic HNSW Benchmark - Week 10 Day 3");
    println!("==============================================\n");

    let dimensions = 1536; // OpenAI embedding size
    let num_vectors = 100_000;
    let num_queries = 1000;
    let k = 10;

    // Create custom HNSW index with pgvector-compatible params
    let params = HNSWParams {
        m: 16,              // M=16 (pgvector default)
        ef_construction: 64, // ef_construction=64 (pgvector default)
        ml: 1.0 / 16f32.ln(),
        seed: 42,
        max_level: 8,
    };

    println!("Configuration:");
    println!("  Dimensions: {}", dimensions);
    println!("  Vectors: {}", num_vectors);
    println!("  M: {}", params.m);
    println!("  ef_construction: {}", params.ef_construction);
    println!("  ef_search: 100 (default)");
    println!();

    let mut index = HNSWIndex::new(dimensions, params, DistanceFunction::L2, false).unwrap();

    // Insert vectors
    println!("Inserting {} vectors ({}D)...", num_vectors, dimensions);
    let start = Instant::now();
    for i in 0..num_vectors {
        let vector = generate_random_vector(dimensions);
        index.insert(vector).unwrap();

        if (i + 1) % 10_000 == 0 {
            let elapsed = start.elapsed();
            let rate = (i + 1) as f64 / elapsed.as_secs_f64();
            println!("  Inserted {} vectors... ({:.0} vec/sec)", i + 1, rate);
        }
    }
    let insert_duration = start.elapsed();

    println!("\n--- Insert Performance ---");
    println!("Total time: {:?}", insert_duration);
    let insert_rate = num_vectors as f64 / insert_duration.as_secs_f64();
    println!("Throughput: {:.0} vec/sec", insert_rate);

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

        // Check first few queries
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

    // Memory usage
    let memory_mb = index.memory_usage() as f64 / (1024.0 * 1024.0);
    println!("\n--- Memory Usage ---");
    println!("Index size: {:.2} MB", memory_mb);
    println!("Bytes per vector: {:.0}", index.memory_usage() as f64 / num_vectors as f64);

    // Compare to raw vector storage (1536D * 4 bytes/float * 100K vectors)
    let raw_vector_size = dimensions * 4 * num_vectors;
    let raw_vector_mb = raw_vector_size as f64 / (1024.0 * 1024.0);
    println!("Raw vector storage: {:.2} MB", raw_vector_mb);
    println!("Overhead: {:.1}x", memory_mb / raw_vector_mb);

    // Goal checks
    println!("\n==============================================");
    println!("Week 10 Day 3 Goal Check");
    println!("==============================================");

    println!("\nRealistic Data Validation:");
    println!("  ✅ 1536D vectors (OpenAI embedding size)");
    println!("  ✅ 100K vectors (production scale)");

    println!("\nPerformance Targets:");

    // Insert target: should be reasonable (100+ vec/sec for sequential)
    println!("\n1. Insert throughput: {:.0} vec/sec", insert_rate);
    if insert_rate >= 100.0 {
        println!("   ✅ PASS: Acceptable insert rate for sequential");
    } else {
        println!("   ⚠️  WARNING: Insert rate lower than expected");
    }

    // Query latency target: <15ms p95 (pgvector-competitive)
    println!("\n2. Query latency p95: {:.2}ms", p95);
    if p95 < 15.0 {
        println!("   ✅ PASS: p95 < 15ms (pgvector-competitive)");
    } else {
        println!("   ⚠️  WARNING: p95 > 15ms target");
    }

    // QPS comparison to baseline (1,677 QPS at 128D/10K)
    println!("\n3. QPS: {:.0}", qps);
    println!("   Baseline (128D, 10K): 1,677 QPS");
    println!("   Current (1536D, 100K): {:.0} QPS", qps);
    println!("   Impact: {:.1}x", qps / 1677.0);

    // Memory efficiency target: <10x overhead
    let overhead = memory_mb / raw_vector_mb;
    println!("\n4. Memory overhead: {:.1}x", overhead);
    if overhead < 10.0 {
        println!("   ✅ PASS: <10x overhead (efficient)");
    } else {
        println!("   ⚠️  WARNING: >10x overhead (review)");
    }

    // Recall validation
    println!("\n5. Recall validation:");
    if results[0].id == test_id && results[0].distance < 0.01 {
        println!("   ✅ PASS: Exact match found");
    } else {
        println!("   ⚠️  WARNING: Recall validation failed");
    }

    println!("\n==============================================");
    println!("Benchmark Complete!");
    println!("==============================================");
}
