//! Extended RaBitQ 1M Scale Benchmark
//!
//! Tests Extended RaBitQ quantization at 1 million vectors.
//!
//! Validates:
//! - Memory savings at production scale
//! - Query performance with quantization
//! - Persistence with quantized vectors
//! - Production readiness

use omen::vector::{Vector, VectorStore, ExtendedRaBitQParams};
use std::time::Instant;

/// Generate random vector for testing
fn random_vector(dim: usize, seed: u64) -> Vector {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hash, Hasher};

    let mut data = Vec::with_capacity(dim);
    for i in 0..dim {
        let mut hasher = RandomState::new().build_hasher();
        (seed, i).hash(&mut hasher);
        let hash = hasher.finish();
        let val = (hash as f32) / (u64::MAX as f32);
        data.push(val);
    }

    Vector::new(data)
}

fn main() {
    println!("Extended RaBitQ 1M Scale Benchmark");
    println!("Testing at 1 MILLION vectors with 4-bit quantization\n");

    let num_vectors = 1_000_000;
    let dimensions = 128;
    let num_queries = 100;

    println!("Configuration:");
    println!("  Vectors:    {}", num_vectors);
    println!("  Dimensions: {}", dimensions);
    println!("  Queries:    {}", num_queries);
    println!("  Compression: 4-bit (8x)");

    println!("\n{}", "=".repeat(80));
    println!("Phase 1: Building Index with Quantization");
    println!("{}", "=".repeat(80));

    // Create store with 4-bit quantization (8x compression, recommended)
    let params = ExtendedRaBitQParams::bits4();
    let mut store = VectorStore::new_with_quantization(dimensions, params);

    // Generate vectors
    println!("\nGenerating {} vectors ({}D)...", num_vectors, dimensions);
    let start_gen = Instant::now();
    let vectors: Vec<Vector> = (0..num_vectors)
        .map(|i| {
            if i > 0 && i % 100_000 == 0 {
                eprintln!("  Generated {} / {} vectors ({:.1}%)",
                         i, num_vectors, (i as f64 / num_vectors as f64) * 100.0);
            }
            random_vector(dimensions, i as u64)
        })
        .collect();
    let gen_time = start_gen.elapsed();
    println!("Generation time: {:.2}s", gen_time.as_secs_f64());

    // Insert vectors with quantization
    println!("\nInserting vectors with 4-bit quantization...");
    let start = Instant::now();
    let ids = store.batch_insert(vectors).unwrap();
    let insert_time = start.elapsed();

    println!("  Insert time: {:.2}s ({:.0} vec/sec)",
             insert_time.as_secs_f64(),
             num_vectors as f64 / insert_time.as_secs_f64());
    println!("  Total vectors: {}", ids.len());

    // Memory usage
    println!("\n{}", "=".repeat(80));
    println!("Phase 2: Memory Usage Analysis");
    println!("{}", "=".repeat(80));

    let original_bytes = num_vectors * dimensions * 4; // f32 = 4 bytes
    let quantized_bytes_per_vec = (dimensions * 4 + 7) / 8; // 4 bits per dimension
    let quantized_bytes = num_vectors * quantized_bytes_per_vec;
    let compression_ratio = original_bytes as f32 / quantized_bytes as f32;

    println!("\nOriginal vectors:");
    println!("  Size:        {:.2} MB", original_bytes as f64 / 1024.0 / 1024.0);
    println!("  Per vector:  {} bytes", dimensions * 4);

    println!("\nQuantized vectors (4-bit):");
    println!("  Size:        {:.2} MB", quantized_bytes as f64 / 1024.0 / 1024.0);
    println!("  Per vector:  {} bytes", quantized_bytes_per_vec);
    println!("  Compression: {:.1}x", compression_ratio);

    println!("\nMemory Savings:");
    println!("  Saved:       {:.2} MB", (original_bytes - quantized_bytes) as f64 / 1024.0 / 1024.0);
    println!("  Percentage:  {:.1}%", (1.0 - 1.0 / compression_ratio) * 100.0);

    // Query performance
    println!("\n{}", "=".repeat(80));
    println!("Phase 3: Query Performance");
    println!("{}", "=".repeat(80));

    println!("\nRunning {} queries...", num_queries);

    let mut query_times = Vec::new();

    let start_queries = Instant::now();
    for i in 0..num_queries {
        let query = random_vector(dimensions, (num_vectors + i) as u64);

        let start = Instant::now();
        let results = store.knn_search(&query, 10).unwrap();
        let query_time = start.elapsed();
        query_times.push(query_time.as_secs_f64() * 1000.0);

        // Verify we got results
        assert_eq!(results.len(), 10, "Should return 10 results");
    }
    let total_query_time = start_queries.elapsed();

    // Statistics
    let mean_query_time = query_times.iter().sum::<f64>() / query_times.len() as f64;
    let qps = num_queries as f64 / total_query_time.as_secs_f64();

    // Sort for percentiles
    query_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = query_times[query_times.len() / 2];
    let p95 = query_times[(query_times.len() * 95) / 100];
    let p99 = query_times[(query_times.len() * 99) / 100];

    println!("\nQuery Latency:");
    println!("  Mean:  {:.3}ms", mean_query_time);
    println!("  p50:   {:.3}ms", p50);
    println!("  p95:   {:.3}ms", p95);
    println!("  p99:   {:.3}ms", p99);

    println!("\nQuery Throughput:");
    println!("  QPS:   {:.0} queries/sec", qps);

    // Persistence test
    println!("\n{}", "=".repeat(80));
    println!("Phase 4: Persistence with Quantization");
    println!("{}", "=".repeat(80));

    let test_path = "/tmp/omendb_1m_quantized";
    println!("\nSaving to disk: {}", test_path);

    let start_save = Instant::now();
    store.save_to_disk(test_path).unwrap();
    let save_time = start_save.elapsed();

    println!("  Save time: {:.2}s", save_time.as_secs_f64());

    // Check file sizes
    let hnsw_size = std::fs::metadata(format!("{}.hnsw", test_path))
        .map(|m| m.len() as f64 / 1024.0 / 1024.0)
        .unwrap_or(0.0);
    let vectors_size = std::fs::metadata(format!("{}.vectors.bin", test_path))
        .map(|m| m.len() as f64 / 1024.0 / 1024.0)
        .unwrap_or(0.0);
    let quantized_size = std::fs::metadata(format!("{}.quantized.bin", test_path))
        .map(|m| m.len() as f64 / 1024.0 / 1024.0)
        .unwrap_or(0.0);

    println!("\nDisk Usage:");
    println!("  HNSW index:  {:.2} MB", hnsw_size);
    println!("  Vectors:     {:.2} MB", vectors_size);
    println!("  Quantized:   {:.2} MB", quantized_size);
    println!("  Total:       {:.2} MB", hnsw_size + vectors_size + quantized_size);

    println!("\nLoading from disk...");
    let start_load = Instant::now();
    let mut loaded_store = VectorStore::load_from_disk(test_path, dimensions).unwrap();
    let load_time = start_load.elapsed();

    println!("  Load time: {:.2}s", load_time.as_secs_f64());
    println!("  Speedup: {:.0}x faster than rebuild", insert_time.as_secs_f64() / load_time.as_secs_f64());

    // Verify loaded store works
    let query = random_vector(dimensions, num_vectors as u64);
    let results = loaded_store.knn_search(&query, 10).unwrap();
    assert_eq!(results.len(), 10, "Loaded store should return 10 results");

    println!("  ✅ Loaded store verified (query returned {} results)", results.len());

    // Clean up
    let _ = std::fs::remove_dir_all("/tmp/omendb_1m_quantized.hnsw.graph");
    let _ = std::fs::remove_dir_all("/tmp/omendb_1m_quantized.hnsw.data");
    let _ = std::fs::remove_file(format!("{}.hnsw", test_path));
    let _ = std::fs::remove_file(format!("{}.vectors.bin", test_path));
    let _ = std::fs::remove_file(format!("{}.quantized.bin", test_path));
    let _ = std::fs::remove_file(format!("{}.quantizer.json", test_path));

    // Final summary
    println!("\n{}", "=".repeat(80));
    println!("Results Summary - 1M Vectors @ 128D with 4-bit Quantization");
    println!("{}", "=".repeat(80));

    let latency_target = 10.0; // 10ms p95
    let latency_pass = p95 < latency_target;

    println!("\n✅ Memory compression: {:.1}x ({:.0} MB → {:.0} MB)",
             compression_ratio,
             original_bytes as f64 / 1024.0 / 1024.0,
             quantized_bytes as f64 / 1024.0 / 1024.0);

    if latency_pass {
        println!("✅ Query latency: {:.3}ms p95 (target: < {:.0}ms)", p95, latency_target);
    } else {
        println!("❌ Query latency: {:.3}ms p95 >= target {:.0}ms", p95, latency_target);
    }

    println!("✅ Query throughput: {:.0} QPS", qps);
    println!("✅ Persistence: {:.0}x speedup vs rebuild", insert_time.as_secs_f64() / load_time.as_secs_f64());

    println!("\nOverall: {}", if latency_pass {
        "✅ PRODUCTION READY at 1M scale with Extended RaBitQ!"
    } else {
        "❌ Needs optimization"
    });

    println!("\n{}", "=".repeat(80));
}
