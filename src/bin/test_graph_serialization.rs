//! Test HNSW graph serialization roundtrip
//!
//! Quick validation that graph serialization works correctly.

use omendb::vector::{Vector, VectorStore};
use std::time::Instant;

fn generate_random_vector(dim: usize) -> Vector {
    let data: Vec<f32> = (0..dim).map(|_| rand::random::<f32>()).collect();
    Vector::new(data)
}

fn main() {
    println!("=== HNSW Graph Serialization Test ===\n");

    let dimensions = 128; // Smaller for quick test
    let num_vectors = 1000; // 1K vectors
    let k = 10;

    // Step 1: Build index
    println!("📊 Building HNSW index with {} vectors ({} dims)...", num_vectors, dimensions);
    let build_start = Instant::now();

    let mut store = VectorStore::new(dimensions);
    for _ in 0..num_vectors {
        store.insert(generate_random_vector(dimensions)).unwrap();
    }

    let build_duration = build_start.elapsed();
    println!("✅ Built index in {:.2}s", build_duration.as_secs_f64());

    // Step 2: Test query before save
    let query = generate_random_vector(dimensions);
    let results_before = store.knn_search(&query, k).unwrap();
    println!("✅ Query returned {} results", results_before.len());

    // Step 3: Save with graph serialization
    println!("\n💾 Saving with graph serialization...");
    let save_path = "/tmp/omendb_graph_test/store";
    std::fs::remove_dir_all("/tmp/omendb_graph_test").ok();

    let save_start = Instant::now();
    store.save_to_disk(save_path).unwrap();
    let save_duration = save_start.elapsed();
    println!("✅ Saved in {:.3}s", save_duration.as_secs_f64());

    // Step 4: Verify dump files exist
    let graph_path = "/tmp/omendb_graph_test/store.hnsw.graph";
    let data_path = "/tmp/omendb_graph_test/store.hnsw.data";

    if std::path::Path::new(graph_path).exists() {
        println!("✅ Graph file created: {}", graph_path);
    } else {
        println!("❌ Graph file NOT found: {}", graph_path);
    }

    if std::path::Path::new(data_path).exists() {
        println!("✅ Data file created: {}", data_path);
    } else {
        println!("❌ Data file NOT found: {}", data_path);
    }

    // Step 5: Load with graph deserialization
    println!("\n📂 Loading with graph deserialization...");
    let load_start = Instant::now();
    let mut loaded_store = VectorStore::load_from_disk(save_path, dimensions).unwrap();
    let load_duration = load_start.elapsed();
    println!("✅ Loaded in {:.3}s", load_duration.as_secs_f64());

    // Step 6: Test query after load
    let results_after = loaded_store.knn_search(&query, k).unwrap();
    println!("✅ Query after load returned {} results", results_after.len());

    // Step 7: Compare results
    println!("\n=== Results Comparison ===");
    println!("Before save: {} results", results_before.len());
    println!("After load:  {} results", results_after.len());

    if results_before.len() == results_after.len() {
        println!("✅ Result count matches!");

        // Check if top results are similar
        let mut matching = 0;
        for i in 0..results_before.len().min(5) {
            if results_before[i].0 == results_after[i].0 {
                matching += 1;
            }
        }
        println!("✅ Top 5 results: {}/5 IDs match", matching);

        if matching >= 3 {
            println!("✅ PASS: Results are reasonably similar");
        } else {
            println!("⚠️  WARNING: Results differ significantly");
        }
    } else {
        println!("❌ FAIL: Result count mismatch");
    }

    // Step 8: Performance summary
    println!("\n=== Performance Summary ===");
    println!("Build time: {:.2}s", build_duration.as_secs_f64());
    println!("Save time:  {:.3}s", save_duration.as_secs_f64());
    println!("Load time:  {:.3}s", load_duration.as_secs_f64());

    let improvement = build_duration.as_secs_f64() / load_duration.as_secs_f64();
    println!("\n🚀 Load is {:.0}x faster than rebuild!", improvement);

    // Cleanup
    std::fs::remove_dir_all("/tmp/omendb_graph_test").ok();
}
