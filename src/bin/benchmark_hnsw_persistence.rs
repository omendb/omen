//! Benchmark HNSW persistence (save/load)
//!
//! Tests:
//! 1. Build HNSW index for 100K vectors (1536D)
//! 2. Save to disk (measure time)
//! 3. Load from disk (measure rebuild time)
//! 4. Query performance before/after reload
//!
//! Expected results:
//! - Save: <5 seconds
//! - Load + rebuild: 10-15 seconds
//! - Query p95: <10ms (both before and after reload)

use omen::vector::{Vector, VectorStore};
use std::time::Instant;

fn generate_random_vector(dim: usize) -> Vector {
    let data: Vec<f32> = (0..dim).map(|_| rand::random::<f32>()).collect();
    Vector::new(data)
}

fn main() {
    println!("=== HNSW Persistence Benchmark ===\n");

    let dimensions = 1536; // OpenAI embeddings
    let num_vectors = 100_000;
    let num_queries = 100;
    let k = 10;

    // Step 1: Build index with 100K vectors
    println!("📊 Building HNSW index with {} vectors ({} dims)...", num_vectors, dimensions);
    let build_start = Instant::now();

    let mut store = VectorStore::new(dimensions);
    for i in 0..num_vectors {
        if i % 10_000 == 0 && i > 0 {
            println!("  Inserted {} vectors...", i);
        }
        store.insert(generate_random_vector(dimensions)).unwrap();
    }

    let build_duration = build_start.elapsed();
    println!("✅ Built index in {:.2}s", build_duration.as_secs_f64());
    println!("   ({:.0} vectors/sec)", num_vectors as f64 / build_duration.as_secs_f64());

    // Step 2: Benchmark queries BEFORE save/load
    println!("\n📊 Benchmarking queries BEFORE save/load...");
    let queries: Vec<Vector> = (0..num_queries).map(|_| generate_random_vector(dimensions)).collect();

    let query_start = Instant::now();
    for query in &queries {
        let _ = store.knn_search(query, k).unwrap();
    }
    let query_duration = query_start.elapsed();
    let avg_latency_ms = query_duration.as_secs_f64() * 1000.0 / num_queries as f64;

    println!("✅ Query latency: {:.2}ms avg", avg_latency_ms);
    println!("   ({:.0} QPS)", num_queries as f64 / query_duration.as_secs_f64());

    // Step 3: Save to disk
    println!("\n💾 Saving to disk...");
    let save_path = "/tmp/omendb_hnsw_benchmark/store_100k";
    std::fs::remove_dir_all("/tmp/omendb_hnsw_benchmark").ok();

    let save_start = Instant::now();
    store.save_to_disk(save_path).unwrap();
    let save_duration = save_start.elapsed();

    println!("✅ Saved in {:.2}s", save_duration.as_secs_f64());

    // Check file size
    let vectors_path = format!("{}.vectors.bin", save_path);
    let file_size = std::fs::metadata(&vectors_path).unwrap().len();
    let mb = file_size as f64 / 1_000_000.0;
    println!("   File size: {:.2} MB", mb);
    println!("   ({:.2} bytes/vector)", file_size as f64 / num_vectors as f64);

    // Step 4: Load from disk
    println!("\n📂 Loading from disk...");
    let load_start = Instant::now();
    let mut loaded_store = VectorStore::load_from_disk(save_path, dimensions).unwrap();
    let load_duration = load_start.elapsed();

    println!("✅ Loaded + rebuilt HNSW in {:.2}s", load_duration.as_secs_f64());

    // Step 5: Benchmark queries AFTER save/load
    println!("\n📊 Benchmarking queries AFTER save/load...");
    let query_start = Instant::now();
    for query in &queries {
        let _ = loaded_store.knn_search(query, k).unwrap();
    }
    let query_duration = query_start.elapsed();
    let avg_latency_ms_after = query_duration.as_secs_f64() * 1000.0 / num_queries as f64;

    println!("✅ Query latency: {:.2}ms avg", avg_latency_ms_after);
    println!("   ({:.0} QPS)", num_queries as f64 / query_duration.as_secs_f64());

    // Summary
    println!("\n=== Summary ===");
    println!("Build time: {:.2}s", build_duration.as_secs_f64());
    println!("Save time: {:.2}s", save_duration.as_secs_f64());
    println!("Load + rebuild time: {:.2}s", load_duration.as_secs_f64());
    println!("Query latency (before): {:.2}ms", avg_latency_ms);
    println!("Query latency (after): {:.2}ms", avg_latency_ms_after);

    // Cleanup
    std::fs::remove_dir_all("/tmp/omendb_hnsw_benchmark").ok();

    // Pass/fail criteria
    println!("\n=== Pass/Fail Criteria ===");

    if save_duration.as_secs_f64() < 5.0 {
        println!("✅ PASS: Save time <5s");
    } else {
        println!("❌ FAIL: Save time >5s");
    }

    if load_duration.as_secs_f64() < 20.0 {
        println!("✅ PASS: Load + rebuild time <20s");
    } else {
        println!("❌ FAIL: Load + rebuild time >20s");
    }

    if avg_latency_ms_after < 10.0 {
        println!("✅ PASS: Query latency <10ms after reload");
    } else {
        println!("❌ FAIL: Query latency >10ms after reload");
    }
}
