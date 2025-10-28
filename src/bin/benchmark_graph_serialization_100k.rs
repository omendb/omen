//! Benchmark HNSW Graph Serialization Performance (100K vectors)
//!
//! Tests the NEW graph serialization (file_dump/load_hnsw) vs old rebuild approach.
//!
//! Tests:
//! 1. Build HNSW index for 100K vectors (1536D)
//! 2. Save to disk with graph serialization (measure time)
//! 3. Load from disk with fast deserialization (measure time)
//! 4. Query performance before/after reload
//!
//! Expected results (NEW):
//! - Save: ~0.5s (graph + data)
//! - Load: <1s (just deserialize, NO rebuild)
//! - Query p95: <15ms (both before and after reload)
//!
//! Improvement: 1800x faster load (1800s rebuild → <1s deserialize)

use omen::vector::{Vector, VectorStore};
use std::time::Instant;

fn generate_random_vector(dim: usize) -> Vector {
    let data: Vec<f32> = (0..dim).map(|_| rand::random::<f32>()).collect();
    Vector::new(data)
}

fn main() {
    println!("=== HNSW Graph Serialization Benchmark (100K vectors) ===\n");

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

    // Step 3: Save to disk with graph serialization
    println!("\n💾 Saving to disk with HNSW graph serialization...");
    let save_path = "/tmp/omendb_graph_benchmark/store_100k";
    std::fs::remove_dir_all("/tmp/omendb_graph_benchmark").ok();

    let save_start = Instant::now();
    store.save_to_disk(save_path).unwrap();
    let save_duration = save_start.elapsed();

    println!("✅ Saved in {:.3}s", save_duration.as_secs_f64());

    // Check file sizes (NEW format has .hnsw.graph and .hnsw.data)
    let graph_path = format!("{}.hnsw.graph", save_path);
    let data_path = format!("{}.hnsw.data", save_path);

    let mut total_size = 0u64;
    if let Ok(metadata) = std::fs::metadata(&graph_path) {
        total_size += metadata.len();
        let mb = metadata.len() as f64 / 1_000_000.0;
        println!("   Graph file: {:.2} MB", mb);
    }

    if let Ok(metadata) = std::fs::metadata(&data_path) {
        total_size += metadata.len();
        let mb = metadata.len() as f64 / 1_000_000.0;
        println!("   Data file: {:.2} MB", mb);
    }

    let total_mb = total_size as f64 / 1_000_000.0;
    println!("   Total size: {:.2} MB", total_mb);
    println!("   ({:.2} bytes/vector)", total_size as f64 / num_vectors as f64);

    // Step 4: Load from disk with fast graph deserialization
    println!("\n📂 Loading from disk with HNSW graph deserialization...");
    let load_start = Instant::now();
    let mut loaded_store = VectorStore::load_from_disk(save_path, dimensions).unwrap();
    let load_duration = load_start.elapsed();

    println!("✅ Loaded HNSW graph in {:.3}s", load_duration.as_secs_f64());

    let improvement = build_duration.as_secs_f64() / load_duration.as_secs_f64();
    println!("   🚀 {:.0}x faster than rebuild!", improvement);

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

    let query_diff_pct = ((avg_latency_ms_after - avg_latency_ms) / avg_latency_ms) * 100.0;
    println!("   Performance change: {:+.1}%", query_diff_pct);

    // Summary
    println!("\n=== Performance Summary ===");
    println!("Build time: {:.2}s", build_duration.as_secs_f64());
    println!("Save time (graph): {:.3}s", save_duration.as_secs_f64());
    println!("Load time (graph): {:.3}s", load_duration.as_secs_f64());
    println!("Improvement: {:.0}x faster", improvement);
    println!("Query latency (before): {:.2}ms", avg_latency_ms);
    println!("Query latency (after): {:.2}ms", avg_latency_ms_after);

    // Pass/fail criteria for NEW graph serialization
    println!("\n=== Pass/Fail Criteria (Graph Serialization) ===");

    if save_duration.as_secs_f64() < 2.0 {
        println!("✅ PASS: Save time <2s (graph serialization)");
    } else {
        println!("❌ FAIL: Save time >2s");
    }

    if load_duration.as_secs_f64() < 5.0 {
        println!("✅ PASS: Load time <5s (NO rebuild!)");
    } else {
        println!("❌ FAIL: Load time >5s");
    }

    if improvement > 100.0 {
        println!("✅ PASS: >100x improvement vs rebuild");
    } else {
        println!("⚠️  WARNING: Improvement <100x (expected >1000x)");
    }

    if avg_latency_ms_after < 20.0 {
        println!("✅ PASS: Query latency <20ms after reload");
    } else {
        println!("❌ FAIL: Query latency >20ms after reload");
    }

    if query_diff_pct.abs() < 20.0 {
        println!("✅ PASS: Query performance within 20% of original");
    } else {
        println!("⚠️  WARNING: Query performance changed by {:.1}%", query_diff_pct);
    }

    // Cleanup
    std::fs::remove_dir_all("/tmp/omendb_graph_benchmark").ok();
}
