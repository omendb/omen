/// Profile query workload for flamegraph analysis
///
/// Generates realistic query load for profiling with:
/// - cargo build --release --bin profile_queries
/// - sudo flamegraph -o queries.svg ./target/release/profile_queries
///
/// Creates 10K vectors, runs 1000 queries, suitable for profiling.

use omen::vector::{Vector, VectorStore};
use rand::prelude::*;

fn generate_realistic_embedding(rng: &mut StdRng, dim: usize) -> Vector {
    let data: Vec<f32> = (0..dim).map(|_| rng.gen_range(-0.3..0.3)).collect();
    Vector::new(data).normalize().unwrap()
}

fn main() {
    println!("=== Query Profiling Workload ===\n");

    let dimensions = 1536;
    let num_vectors = 10_000;
    let num_queries = 1000;
    let k = 10;

    println!("Building {} vectors...", num_vectors);
    let mut store = VectorStore::new(dimensions);
    let mut rng = StdRng::seed_from_u64(42);

    // Build index
    let mut batch = Vec::with_capacity(num_vectors);
    for _ in 0..num_vectors {
        batch.push(generate_realistic_embedding(&mut rng, dimensions));
    }
    store.batch_insert(batch).expect("Failed to insert batch");

    println!("Running {} queries (k={})...", num_queries, k);

    // Run queries (this is what we're profiling)
    let mut total_results = 0;
    for _ in 0..num_queries {
        let query = generate_realistic_embedding(&mut rng, dimensions);
        let results = store.knn_search(&query, k).expect("Query failed");
        total_results += results.len();
    }

    println!("Completed {} queries, {} total results", num_queries, total_results);
    println!("\nRun with: sudo flamegraph -o queries.svg ./target/release/profile_queries");
}
