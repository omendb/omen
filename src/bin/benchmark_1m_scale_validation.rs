//! 1M Scale Validation Benchmark (Week 6 Days 3-4)
//!
//! Goals:
//! 1. Validate graph serialization at 1M scale
//! 2. Measure query latency (p50, p95, p99)
//! 3. Track memory usage (target <15GB with quantization)
//! 4. Document scaling characteristics
//! 5. Identify any bottlenecks
//!
//! Expected results:
//! - Build: <30 minutes (1M vectors, 1536D)
//! - Save: <5s (graph + data)
//! - Load: <5s (deserialize, NO rebuild)
//! - Query p95: <15ms
//! - Memory: <15GB (with quantization)

use omen::vector::{Vector, VectorStore};
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
    println!("=== 1M Scale Validation Benchmark ===\n");
    println!("Goal: Validate production readiness at 1M vectors");
    println!("Timeline: ~30 minutes (build) + 5s (save) + 5s (load) + queries\n");

    let dimensions = 1536; // OpenAI embeddings
    let num_vectors = 1_000_000;
    let num_queries = 1000; // More queries for better statistics
    let k = 10;

    // Step 1: Build index with 1M vectors
    println!("📊 Building HNSW index with {} vectors ({} dims)...", num_vectors, dimensions);
    println!("   (This will take ~30 minutes - progress every 100K vectors)\n");
    let build_start = Instant::now();

    let mut store = VectorStore::new(dimensions);
    for i in 0..num_vectors {
        if i % 100_000 == 0 && i > 0 {
            let elapsed = build_start.elapsed().as_secs_f64();
            let rate = i as f64 / elapsed;
            let remaining = (num_vectors - i) as f64 / rate;
            println!("  Inserted {} vectors... ({:.0} vec/sec, ~{:.0}s remaining)",
                     i, rate, remaining);
        }
        store.insert(generate_random_vector(dimensions)).unwrap();
    }

    let build_duration = build_start.elapsed();
    println!("\n✅ Built index in {:.2}s ({:.1} minutes)",
             build_duration.as_secs_f64(),
             build_duration.as_secs_f64() / 60.0);
    println!("   ({:.0} vectors/sec)", num_vectors as f64 / build_duration.as_secs_f64());

    // Step 2: Benchmark queries BEFORE save/load
    println!("\n📊 Benchmarking {} queries BEFORE save/load...", num_queries);
    let queries: Vec<Vector> = (0..num_queries).map(|_| generate_random_vector(dimensions)).collect();

    let mut query_latencies = Vec::new();
    for (i, query) in queries.iter().enumerate() {
        if i % 100 == 0 && i > 0 {
            print!("\r  Executed {} queries...", i);
        }
        let query_start = Instant::now();
        let _ = store.knn_search(query, k).unwrap();
        query_latencies.push(query_start.elapsed().as_secs_f64() * 1000.0);
    }
    println!("\r  Executed {} queries...    ", num_queries);

    let p50 = percentile(query_latencies.clone(), 50.0);
    let p95 = percentile(query_latencies.clone(), 95.0);
    let p99 = percentile(query_latencies.clone(), 99.0);

    println!("✅ Query latency (before save/load):");
    println!("   p50: {:.2}ms", p50);
    println!("   p95: {:.2}ms", p95);
    println!("   p99: {:.2}ms", p99);

    // Step 3: Save to disk with graph serialization
    println!("\n💾 Saving to disk with HNSW graph serialization...");
    let save_path = "/tmp/omendb_1m_benchmark/store_1m";
    std::fs::remove_dir_all("/tmp/omendb_1m_benchmark").ok();

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
    println!("   ({:.2} bytes/vector)", total_size as f64 / num_vectors as f64);

    // Step 4: Load from disk with fast graph deserialization
    println!("\n📂 Loading from disk with HNSW graph deserialization...");
    let load_start = Instant::now();
    let mut loaded_store = VectorStore::load_from_disk(save_path, dimensions).unwrap();
    let load_duration = load_start.elapsed();

    println!("✅ Loaded HNSW graph in {:.2}s", load_duration.as_secs_f64());

    let improvement = build_duration.as_secs_f64() / load_duration.as_secs_f64();
    println!("   🚀 {:.0}x faster than rebuild!", improvement);

    // Step 5: Benchmark queries AFTER save/load
    println!("\n📊 Benchmarking {} queries AFTER save/load...", num_queries);
    let mut query_latencies_after = Vec::new();
    for (i, query) in queries.iter().enumerate() {
        if i % 100 == 0 && i > 0 {
            print!("\r  Executed {} queries...", i);
        }
        let query_start = Instant::now();
        let _ = loaded_store.knn_search(query, k).unwrap();
        query_latencies_after.push(query_start.elapsed().as_secs_f64() * 1000.0);
    }
    println!("\r  Executed {} queries...    ", num_queries);

    let p50_after = percentile(query_latencies_after.clone(), 50.0);
    let p95_after = percentile(query_latencies_after.clone(), 95.0);
    let p99_after = percentile(query_latencies_after.clone(), 99.0);

    println!("✅ Query latency (after save/load):");
    println!("   p50: {:.2}ms", p50_after);
    println!("   p95: {:.2}ms", p95_after);
    println!("   p99: {:.2}ms", p99_after);

    let p95_diff_pct = ((p95_after - p95) / p95) * 100.0;
    println!("   Performance change (p95): {:+.1}%", p95_diff_pct);

    // Summary
    println!("\n=== Performance Summary ===");
    println!("Vectors: {}", num_vectors);
    println!("Dimensions: {}", dimensions);
    println!();
    println!("Build time: {:.2}s ({:.1} minutes)",
             build_duration.as_secs_f64(),
             build_duration.as_secs_f64() / 60.0);
    println!("Save time (graph): {:.2}s", save_duration.as_secs_f64());
    println!("Load time (graph): {:.2}s", load_duration.as_secs_f64());
    println!("Improvement: {:.0}x faster than rebuild", improvement);
    println!();
    println!("Query latency (before):");
    println!("  p50: {:.2}ms, p95: {:.2}ms, p99: {:.2}ms", p50, p95, p99);
    println!("Query latency (after):");
    println!("  p50: {:.2}ms, p95: {:.2}ms, p99: {:.2}ms", p50_after, p95_after, p99_after);
    println!();
    println!("Disk usage: {:.2} GB ({:.2} bytes/vector)", total_gb, total_size as f64 / num_vectors as f64);

    // Pass/fail criteria for 1M scale
    println!("\n=== Pass/Fail Criteria (1M Scale) ===");

    if build_duration.as_secs_f64() < 1800.0 {
        println!("✅ PASS: Build time <30 minutes");
    } else {
        println!("⚠️  WARNING: Build time >30 minutes ({:.1} min)", build_duration.as_secs_f64() / 60.0);
    }

    if save_duration.as_secs_f64() < 10.0 {
        println!("✅ PASS: Save time <10s");
    } else {
        println!("❌ FAIL: Save time >10s");
    }

    if load_duration.as_secs_f64() < 10.0 {
        println!("✅ PASS: Load time <10s (NO rebuild!)");
    } else {
        println!("❌ FAIL: Load time >10s");
    }

    if improvement > 50.0 {
        println!("✅ PASS: >50x improvement vs rebuild");
    } else {
        println!("⚠️  WARNING: Improvement <50x (expected >100x)");
    }

    if p95_after < 20.0 {
        println!("✅ PASS: Query p95 latency <20ms after reload");
    } else {
        println!("⚠️  WARNING: Query p95 latency >20ms ({:.2}ms)", p95_after);
    }

    if p95_diff_pct.abs() < 20.0 {
        println!("✅ PASS: Query performance within 20% of original");
    } else {
        println!("⚠️  WARNING: Query performance changed by {:.1}%", p95_diff_pct);
    }

    if total_gb < 15.0 {
        println!("✅ PASS: Disk usage <15GB");
    } else {
        println!("⚠️  NOTE: Disk usage {:.2}GB (quantization would reduce this)", total_gb);
    }

    // Cleanup
    println!("\n🧹 Cleaning up temporary files...");
    std::fs::remove_dir_all("/tmp/omendb_1m_benchmark").ok();
    println!("✅ Done!");
}
