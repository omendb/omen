//! 1M Scale Validation with Parallel Building
//!
//! Same as benchmark_1m_scale_validation but uses batch_insert for parallel building

use omendb::vector::{Vector, VectorStore};
use std::time::Instant;

fn generate_random_vector(dim: usize) -> Vector {
    let data: Vec<f32> = (0..dim).map(|_| rand::random::<f32>()).collect();
    Vector::new(data)
}

fn percentile(mut values: Vec<f64>, p: f64) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((p / 100.0) * values.len() as f64) as usize;
    values[idx.min(values.len() - 1)]
}

fn main() {
    println!("=== 1M Scale Validation with Parallel Building ===\n");
    println!("Goal: Validate 4-5x faster build time vs sequential");
    println!("Expected: ~1.5-2 hours (vs 7 hours sequential)\\n");

    let dimensions = 1536;
    let num_vectors = 1_000_000;
    let num_queries = 1000;
    let k = 10;

    // Step 1: Build index with 1M vectors using PARALLEL insertion
    println!("📊 Building HNSW index with {} vectors ({} dims) - PARALLEL...", num_vectors, dimensions);
    println!("   (Chunked into 10K batches with progress reporting)\\n");
    let build_start = Instant::now();

    // Generate all vectors upfront
    println!("Generating {} vectors...", num_vectors);
    let gen_start = Instant::now();
    let vectors: Vec<Vector> = (0..num_vectors)
        .map(|_| generate_random_vector(dimensions))
        .collect();
    println!("✅ Generated in {:.2}s\n", gen_start.elapsed().as_secs_f64());

    // Batch insert with parallel building
    let mut store = VectorStore::new(dimensions);
    store.batch_insert(vectors).unwrap();

    let build_duration = build_start.elapsed();
    println!("\\n✅ Built index in {:.2}s ({:.1} minutes)",
             build_duration.as_secs_f64(),
             build_duration.as_secs_f64() / 60.0);
    println!("   ({:.0} vectors/sec)", num_vectors as f64 / build_duration.as_secs_f64());

    // Step 2: Benchmark queries BEFORE save/load
    println!("\\n📊 Benchmarking {} queries BEFORE save/load...", num_queries);
    let queries: Vec<Vector> = (0..num_queries).map(|_| generate_random_vector(dimensions)).collect();

    let mut query_latencies = Vec::new();
    for (i, query) in queries.iter().enumerate() {
        if i % 100 == 0 && i > 0 {
            print!("\\r  Executed {} queries...", i);
        }
        let query_start = Instant::now();
        let _ = store.knn_search(query, k).unwrap();
        query_latencies.push(query_start.elapsed().as_secs_f64() * 1000.0);
    }
    println!("\\r  Executed {} queries...    ", num_queries);

    let p50 = percentile(query_latencies.clone(), 50.0);
    let p95 = percentile(query_latencies.clone(), 95.0);
    let p99 = percentile(query_latencies.clone(), 99.0);

    println!("✅ Query latency (before save/load):");
    println!("   p50: {:.2}ms", p50);
    println!("   p95: {:.2}ms", p95);
    println!("   p99: {:.2}ms", p99);

    // Step 3: Save to disk with graph serialization
    println!("\\n💾 Saving to disk with HNSW graph serialization...");
    let save_path = "/tmp/omendb_1m_parallel/store_1m";
    std::fs::remove_dir_all("/tmp/omendb_1m_parallel").ok();

    let save_start = Instant::now();
    store.save_to_disk(save_path).unwrap();
    let save_duration = save_start.elapsed();

    println!("✅ Saved in {:.2}s", save_duration.as_secs_f64());

    // Check file sizes
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
    let total_gb = total_mb / 1000.0;
    println!("   Total size: {:.2} MB ({:.2} GB)", total_mb, total_gb);

    // Step 4: Load from disk
    println!("\\n📂 Loading from disk with HNSW graph deserialization...");
    let load_start = Instant::now();
    let mut loaded_store = VectorStore::load_from_disk(save_path, dimensions).unwrap();
    let load_duration = load_start.elapsed();

    println!("✅ Loaded HNSW graph in {:.2}s", load_duration.as_secs_f64());

    let improvement = build_duration.as_secs_f64() / load_duration.as_secs_f64();
    println!("   🚀 {:.0}x faster than rebuild!", improvement);

    // Step 5: Benchmark queries AFTER save/load
    println!("\\n📊 Benchmarking {} queries AFTER save/load...", num_queries);
    let mut query_latencies_after = Vec::new();
    for (i, query) in queries.iter().enumerate() {
        if i % 100 == 0 && i > 0 {
            print!("\\r  Executed {} queries...", i);
        }
        let query_start = Instant::now();
        let _ = loaded_store.knn_search(query, k).unwrap();
        query_latencies_after.push(query_start.elapsed().as_secs_f64() * 1000.0);
    }
    println!("\\r  Executed {} queries...    ", num_queries);

    let p50_after = percentile(query_latencies_after.clone(), 50.0);
    let p95_after = percentile(query_latencies_after.clone(), 95.0);
    let p99_after = percentile(query_latencies_after.clone(), 99.0);

    println!("✅ Query latency (after save/load):");
    println!("   p50: {:.2}ms", p50_after);
    println!("   p95: {:.2}ms", p95_after);
    println!("   p99: {:.2}ms", p99_after);

    let p95_diff_pct = ((p95_after - p95) / p95) * 100.0;
    println!("   Performance change (p95): {:+.1}%", p95_diff_pct);

    // Compare to sequential baseline (from previous run)
    let sequential_build_time = 25146.41; // From benchmark_1m_scale_validation
    let build_speedup = sequential_build_time / build_duration.as_secs_f64();

    // Summary
    println!("\\n=== Performance Summary ===");
    println!("Vectors: {}", num_vectors);
    println!("Dimensions: {}", dimensions);
    println!();
    println!("Build time (parallel): {:.2}s ({:.1} minutes)",
             build_duration.as_secs_f64(),
             build_duration.as_secs_f64() / 60.0);
    println!("Build time (sequential baseline): {:.2}s ({:.1} minutes)",
             sequential_build_time,
             sequential_build_time / 60.0);
    println!("Build speedup: {:.2}x faster than sequential", build_speedup);
    println!();
    println!("Save time (graph): {:.2}s", save_duration.as_secs_f64());
    println!("Load time (graph): {:.2}s", load_duration.as_secs_f64());
    println!("Improvement: {:.0}x faster than rebuild", improvement);
    println!();
    println!("Query latency (before):");
    println!("  p50: {:.2}ms, p95: {:.2}ms, p99: {:.2}ms", p50, p95, p99);
    println!("Query latency (after):");
    println!("  p50: {:.2}ms, p95: {:.2}ms, p99: {:.2}ms", p50_after, p95_after, p99_after);
    println!();
    println!("Disk usage: {:.2} GB", total_gb);

    // Pass/fail criteria
    println!("\\n=== Pass/Fail Criteria (Parallel Building) ===");

    if build_speedup >= 2.0 {
        println!("✅ PASS: Build speedup >= 2x (got {:.2}x)", build_speedup);
    } else {
        println!("⚠️  WARNING: Build speedup < 2x (got {:.2}x)", build_speedup);
    }

    if save_duration.as_secs_f64() < 10.0 {
        println!("✅ PASS: Save time <10s");
    } else {
        println!("❌ FAIL: Save time >10s");
    }

    if load_duration.as_secs_f64() < 10.0 {
        println!("✅ PASS: Load time <10s");
    } else {
        println!("❌ FAIL: Load time >10s");
    }

    if p95_after < 20.0 {
        println!("✅ PASS: Query p95 <20ms after reload");
    } else {
        println!("⚠️  WARNING: Query p95 >20ms ({:.2}ms)", p95_after);
    }

    if p95_diff_pct.abs() < 20.0 {
        println!("✅ PASS: Query performance within 20% of original");
    } else {
        println!("⚠️  WARNING: Query performance changed by {:.1}%", p95_diff_pct);
    }

    // Cleanup
    println!("\\n🧹 Cleaning up temporary files...");
    std::fs::remove_dir_all("/tmp/omendb_1m_parallel").ok();
    println!("✅ Done!");
}
