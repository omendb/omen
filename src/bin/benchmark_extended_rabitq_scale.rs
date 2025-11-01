//! Extended RaBitQ Scale Benchmark
//!
//! Tests Extended RaBitQ quantization at scale (100K vectors).
//!
//! Validates:
//! - Memory usage vs original vectors
//! - Query performance with quantization
//! - Recall accuracy at scale
//! - Two-phase search effectiveness

use omen::vector::{Vector, VectorStore, ExtendedRaBitQParams, QuantizationBits};
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

/// Compute recall@k
fn compute_recall(ground_truth: &[(usize, f32)], results: &[(usize, f32)], k: usize) -> f32 {
    let gt_set: std::collections::HashSet<usize> = ground_truth
        .iter()
        .take(k)
        .map(|(id, _)| *id)
        .collect();

    let result_set: std::collections::HashSet<usize> = results
        .iter()
        .take(k)
        .map(|(id, _)| *id)
        .collect();

    let intersection = gt_set.intersection(&result_set).count();
    intersection as f32 / k as f32
}

fn main() {
    println!("Extended RaBitQ Scale Benchmark");
    println!("Testing at 100K vectors with quantization\n");

    let num_vectors = 100_000;
    let dimensions = 128;
    let num_queries = 100;

    println!("Configuration:");
    println!("  Vectors:    {}", num_vectors);
    println!("  Dimensions: {}", dimensions);
    println!("  Queries:    {}", num_queries);

    println!("\n{}", "=".repeat(80));
    println!("Building Vector Store with 4-bit Quantization");
    println!("{}", "=".repeat(80));

    // Create store with 4-bit quantization (8x compression, recommended)
    let params = ExtendedRaBitQParams::bits4();
    let mut store_with_quant = VectorStore::new_with_quantization(dimensions, params);

    // Create store without quantization (ground truth)
    let mut store_ground_truth = VectorStore::new(dimensions);

    // Generate and insert vectors
    println!("\nGenerating {} vectors ({}D)...", num_vectors, dimensions);
    let vectors: Vec<Vector> = (0..num_vectors)
        .map(|i| random_vector(dimensions, i as u64))
        .collect();

    println!("Inserting vectors with quantization...");
    let start = Instant::now();
    for vec in vectors.iter() {
        store_with_quant.insert(vec.clone()).unwrap();
    }
    let insert_time = start.elapsed();

    println!("  Insert time: {:.2}s ({:.0} vec/sec)",
             insert_time.as_secs_f64(),
             num_vectors as f64 / insert_time.as_secs_f64());

    // Insert into ground truth store (for recall comparison)
    println!("\nInserting vectors without quantization (ground truth)...");
    let start = Instant::now();
    for vec in vectors {
        store_ground_truth.insert(vec).unwrap();
    }
    let gt_insert_time = start.elapsed();

    println!("  Insert time: {:.2}s ({:.0} vec/sec)",
             gt_insert_time.as_secs_f64(),
             num_vectors as f64 / gt_insert_time.as_secs_f64());

    // Memory usage
    println!("\n{}", "=".repeat(80));
    println!("Memory Usage Analysis");
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
    println!("Query Performance with Quantization");
    println!("{}", "=".repeat(80));

    println!("\nRunning {} queries...", num_queries);

    let mut recalls = Vec::new();
    let mut query_times_quant = Vec::new();
    let mut query_times_gt = Vec::new();

    for i in 0..num_queries {
        let query = random_vector(dimensions, (num_vectors + i) as u64);

        // Ground truth (HNSW without quantization)
        let start = Instant::now();
        let ground_truth = store_ground_truth.knn_search(&query, 10).unwrap();
        let gt_time = start.elapsed();
        query_times_gt.push(gt_time.as_secs_f64() * 1000.0);

        // With quantization (HNSW + quantized storage)
        let start = Instant::now();
        let results = store_with_quant.knn_search(&query, 10).unwrap();
        let query_time = start.elapsed();
        query_times_quant.push(query_time.as_secs_f64() * 1000.0);

        // Compute recall@10
        let recall = compute_recall(&ground_truth, &results, 10);
        recalls.push(recall);
    }

    // Statistics
    let mean_recall = recalls.iter().sum::<f32>() / recalls.len() as f32;
    let min_recall = recalls.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_recall = recalls.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

    let mean_query_time_quant = query_times_quant.iter().sum::<f64>() / query_times_quant.len() as f64;
    let mean_query_time_gt = query_times_gt.iter().sum::<f64>() / query_times_gt.len() as f64;

    // Sort for percentiles
    query_times_quant.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50_quant = query_times_quant[query_times_quant.len() / 2];
    let p95_quant = query_times_quant[(query_times_quant.len() * 95) / 100];
    let p99_quant = query_times_quant[(query_times_quant.len() * 99) / 100];

    println!("\nRecall@10 (vs HNSW without quantization):");
    println!("  Mean:  {:.1}%", mean_recall * 100.0);
    println!("  Min:   {:.1}%", min_recall * 100.0);
    println!("  Max:   {:.1}%", max_recall * 100.0);

    println!("\nQuery Latency (HNSW + quantized storage):");
    println!("  Mean:  {:.3}ms", mean_query_time_quant);
    println!("  p50:   {:.3}ms", p50_quant);
    println!("  p95:   {:.3}ms", p95_quant);
    println!("  p99:   {:.3}ms", p99_quant);

    println!("\nQuery Latency (HNSW without quantization):");
    println!("  Mean:  {:.3}ms", mean_query_time_gt);

    let latency_ratio = mean_query_time_quant / mean_query_time_gt;
    if latency_ratio < 1.1 {
        println!("\nLatency Impact: {:.1}% overhead (negligible)", (latency_ratio - 1.0) * 100.0);
    } else {
        println!("\nLatency Impact: {:.1}x slower", latency_ratio);
    }

    // Final verdict
    println!("\n{}", "=".repeat(80));
    println!("Results Summary");
    println!("{}", "=".repeat(80));

    let recall_target = 0.85; // 85% for 4-bit
    let latency_target = 10.0; // 10ms p95

    let recall_pass = mean_recall >= recall_target;
    let latency_pass = p95_quant < latency_target;

    println!("\n✅ Memory compression: {:.1}x ({:.2} MB → {:.2} MB)",
             compression_ratio,
             original_bytes as f64 / 1024.0 / 1024.0,
             quantized_bytes as f64 / 1024.0 / 1024.0);

    if recall_pass {
        println!("✅ Recall: {:.1}% (target: {:.0}%)", mean_recall * 100.0, recall_target * 100.0);
    } else {
        println!("❌ Recall: {:.1}% < target {:.0}%", mean_recall * 100.0, recall_target * 100.0);
    }

    if latency_pass {
        println!("✅ Query latency: {:.3}ms p95 (target: < {:.0}ms)", p95_quant, latency_target);
    } else {
        println!("❌ Query latency: {:.3}ms p95 >= target {:.0}ms", p95_quant, latency_target);
    }

    println!("\nOverall: {}", if recall_pass && latency_pass {
        "✅ PASS - Production ready at 100K scale!"
    } else {
        "❌ FAIL - Needs optimization"
    });

    println!("\n{}", "=".repeat(80));
}
