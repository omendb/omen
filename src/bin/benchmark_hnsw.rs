//! HNSW Benchmark - Day 2
//!
//! Quick validation that HNSW is working correctly
//! - Insert 10K vectors (1536D)
//! - Query k=10 nearest neighbors
//! - Measure latency and validate recall

use omen::vector::{Vector, VectorStore};
use rand::Rng;
use std::time::Instant;

fn generate_random_vector(dim: usize) -> Vector {
    let mut rng = rand::thread_rng();
    let data: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
    Vector::new(data)
}

fn main() {
    println!("==============================================");
    println!("HNSW Benchmark - Day 2 Validation");
    println!("==============================================\n");

    let dimensions = 1536; // OpenAI embedding size
    let num_vectors = 10_000;
    let num_queries = 100;
    let k = 10;

    // Create store
    let mut store = VectorStore::new(dimensions);

    // Insert vectors
    println!("Inserting {} vectors ({}D)...", num_vectors, dimensions);
    let start = Instant::now();
    for i in 0..num_vectors {
        let vector = generate_random_vector(dimensions);
        store.insert(vector).unwrap();

        if (i + 1) % 1000 == 0 {
            println!("  Inserted {} vectors...", i + 1);
        }
    }
    let insert_duration = start.elapsed();

    println!("\n--- Insert Performance ---");
    println!("Total time: {:?}", insert_duration);
    println!(
        "Throughput: {:.0} inserts/sec",
        num_vectors as f64 / insert_duration.as_secs_f64()
    );

    // Query benchmark
    println!("\n--- Query Performance ---");
    println!("Running {} queries (k={})...\n", num_queries, k);

    let mut query_times = Vec::new();

    for i in 0..num_queries {
        let query = generate_random_vector(dimensions);

        let start = Instant::now();
        let results = store.knn_search(&query, k).unwrap();
        let duration = start.elapsed();

        query_times.push(duration.as_secs_f64() * 1000.0); // Convert to ms

        assert_eq!(results.len(), k);

        if (i + 1) % 10 == 0 {
            println!(
                "  Query {}: {:.2}ms, {} results",
                i + 1,
                duration.as_secs_f64() * 1000.0,
                results.len()
            );
        }
    }

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

    // Validate recall (query should return itself as nearest neighbor)
    println!("\n--- Recall Validation ---");
    let test_id = 5000;
    let test_vector = store.get(test_id).unwrap().clone();
    let results = store.knn_search(&test_vector, 1).unwrap();

    println!("Query vector ID: {}", test_id);
    println!("Nearest neighbor ID: {}", results[0].0);
    println!("Distance: {:.6}", results[0].1);

    if results[0].0 == test_id {
        println!("✅ PASS: Exact match found (distance ~0)");
    } else {
        println!("⚠️  WARNING: Exact match not found as nearest neighbor");
    }

    // Goal checks
    println!("\n==============================================");
    println!("Day 2 Goal Check");
    println!("==============================================");

    if p95 < 10.0 {
        println!("✅ PASS: p95 latency {:.2}ms < 10ms target", p95);
    } else {
        println!("⚠️  WARNING: p95 latency {:.2}ms > 10ms target", p95);
    }

    if results[0].0 == test_id && results[0].1 < 0.01 {
        println!("✅ PASS: Recall validation (exact match found)");
    } else {
        println!("⚠️  WARNING: Recall validation failed");
    }

    println!("\n--- HNSW Parameters ---");
    println!("ef_search: {:?}", store.get_ef_search());
    println!("Dimensions: {}", dimensions);
    println!("Vectors indexed: {}", num_vectors);

    println!("\n==============================================");
    println!("Benchmark Complete!");
    println!("==============================================");
}
