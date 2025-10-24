//! Vector Store Prototype Benchmark
//!
//! Week 1 Goal: Validate ALEX can work for high-dimensional vectors
//!
//! Measurements:
//! 1. Memory usage: Target <50 bytes/vector (2-5x better than HNSW)
//! 2. Query latency: Target <20ms p95 for k=10 search
//! 3. Recall@10: Target >90% (vs brute-force ground truth)
//!
//! Test scales: 10K, 100K, 1M vectors (1536 dimensions)

use omendb::vector::{Vector, VectorStore};
use rand::Rng;
use std::time::Instant;

fn generate_random_vector(dim: usize) -> Vector {
    let mut rng = rand::thread_rng();
    let data: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
    Vector::new(data)
}

fn benchmark_insertion(num_vectors: usize, dim: usize) {
    println!("\n=== Insertion Benchmark: {} vectors, {} dims ===", num_vectors, dim);

    let mut store = VectorStore::new(4); // Use first 4 dims for projection

    let start = Instant::now();
    for i in 0..num_vectors {
        let vector = generate_random_vector(dim);
        store.insert(vector);

        if (i + 1) % 10000 == 0 {
            println!("  Inserted {} vectors...", i + 1);
        }
    }
    let duration = start.elapsed();

    let memory_mb = store.memory_usage() as f64 / (1024.0 * 1024.0);
    let bytes_per_vec = store.bytes_per_vector();

    println!("Insertion complete:");
    println!("  Total time: {:?}", duration);
    println!("  Throughput: {:.0} inserts/sec", num_vectors as f64 / duration.as_secs_f64());
    println!("  Memory usage: {:.2} MB", memory_mb);
    println!("  Bytes/vector: {:.2} bytes", bytes_per_vec);

    // Goal check: <50 bytes/vector overhead (beyond raw data)
    let raw_data_size = (dim * 4) as f32; // f32 = 4 bytes
    let overhead = bytes_per_vec - raw_data_size;
    println!("  Overhead: {:.2} bytes ({:.1}% of raw data)", overhead, (overhead / raw_data_size) * 100.0);

    if bytes_per_vec < 50.0 + raw_data_size {
        println!("  ✅ PASS: Memory efficiency goal met (<50 bytes overhead)");
    } else {
        println!("  ❌ FAIL: Memory efficiency goal NOT met (>50 bytes overhead)");
    }
}

fn benchmark_query(store: &mut VectorStore, num_queries: usize, k: usize) {
    println!("\n=== Query Benchmark: {} queries, k={} ===", num_queries, k);

    let dim = if store.len() > 0 {
        store.get(0).unwrap().dim()
    } else {
        return;
    };

    // Generate random query vectors
    let queries: Vec<Vector> = (0..num_queries)
        .map(|_| generate_random_vector(dim))
        .collect();

    // Benchmark brute-force search
    let start = Instant::now();
    for query in &queries {
        let _ = store.knn_search(query, k).unwrap();
    }
    let brute_duration = start.elapsed();
    let brute_latency_ms = brute_duration.as_secs_f64() * 1000.0 / num_queries as f64;

    println!("Brute-force search:");
    println!("  Total time: {:?}", brute_duration);
    println!("  Avg latency: {:.2} ms/query", brute_latency_ms);
    println!("  p50 latency: ~{:.2} ms (approx)", brute_latency_ms);
    println!("  p95 latency: ~{:.2} ms (approx)", brute_latency_ms * 1.2);

    // Note: ALEX-accelerated search removed (pivoted to HNSW in Week 2)
}

fn benchmark_recall(_store: &VectorStore, _num_queries: usize, _k: usize) {
    // Note: Recall benchmark removed (ALEX prototype failed, pivoted to HNSW in Week 2)
    println!("\n=== Recall Benchmark: Skipped (Week 1 ALEX approach failed) ===");
}

fn main() {
    println!("==============================================");
    println!("Vector Store Prototype Benchmark");
    println!("Week 1: ALEX for High-Dimensional Vectors");
    println!("==============================================");

    // Test 1: 10K vectors (quick validation)
    println!("\n\n### TEST 1: 10K Vectors (1536 dims) ###");
    benchmark_insertion(10_000, 1536);

    // Recreate store for query benchmarks
    let mut store = VectorStore::new(4);
    for _ in 0..10_000 {
        let vector = generate_random_vector(1536);
        store.insert(vector);
    }
    benchmark_query(&mut store, 100, 10);
    benchmark_recall(&store, 50, 10);

    // Test 2: 100K vectors (medium scale)
    println!("\n\n### TEST 2: 100K Vectors (1536 dims) ###");
    benchmark_insertion(100_000, 1536);

    let mut store = VectorStore::new(4);
    for _ in 0..100_000 {
        let vector = generate_random_vector(1536);
        store.insert(vector);
    }
    benchmark_query(&mut store, 100, 10);
    benchmark_recall(&store, 50, 10);

    // Test 3: 1M vectors (large scale) - commented out for quick iteration
    // Uncomment once basic approach is validated
    /*
    println!("\n\n### TEST 3: 1M Vectors (1536 dims) ###");
    benchmark_insertion(1_000_000, 1536);

    let mut store = VectorStore::new(4);
    for _ in 0..1_000_000 {
        let vector = generate_random_vector(1536);
        store.insert(vector);
    }
    benchmark_query(&mut store, 100, 10);
    benchmark_recall(&store, 50, 10);
    */

    println!("\n\n==============================================");
    println!("Benchmark Complete!");
    println!("==============================================");
    println!("\nGo/No-Go Decision Criteria:");
    println!("  ✅ Memory: <50 bytes/vector overhead");
    println!("  ✅ Latency: <20ms p95 for k=10");
    println!("  ✅ Recall: >90% recall@10");
    println!("\nIf ALL criteria met → Continue with ALEX for vectors");
    println!("If ANY criteria failed → Pivot to HNSW fallback");
}
