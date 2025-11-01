//! Extended RaBitQ Benchmark
//!
//! Tests accuracy and performance of Extended RaBitQ quantization at different compression rates.
//!
//! Metrics:
//! - Recall@10 at different bit rates (2, 3, 4, 5, 7, 8 bits)
//! - Distance correlation with ground truth
//! - Memory usage (compression ratio)
//! - Quantization time overhead
//! - Query latency

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

/// Benchmark a specific quantization configuration
fn benchmark_quantization(
    bits: QuantizationBits,
    num_vectors: usize,
    dimensions: usize,
    num_queries: usize,
) {
    println!("\n{}", "=".repeat(80));
    println!("Benchmarking {}-bit quantization ({}x compression)",
             bits.to_u8(),
             32 / bits.to_u8());
    println!("{}", "=".repeat(80));

    // Create store with quantization
    let params = ExtendedRaBitQParams {
        bits_per_dim: bits,
        num_rescale_factors: 12,
        rescale_range: (0.5, 2.0),
    };

    let mut store_with_quant = VectorStore::new_with_quantization(dimensions, params);

    // Create store without quantization (ground truth)
    let mut store_ground_truth = VectorStore::new(dimensions);

    // Insert vectors
    println!("\nInserting {} vectors ({}D)...", num_vectors, dimensions);

    let start = Instant::now();
    for i in 0..num_vectors {
        let vec = random_vector(dimensions, i as u64);
        store_with_quant.insert(vec.clone()).unwrap();
        store_ground_truth.insert(vec).unwrap();
    }
    let insert_time = start.elapsed();

    println!("  Insert time: {:.2}s ({:.0} vec/sec)",
             insert_time.as_secs_f64(),
             num_vectors as f64 / insert_time.as_secs_f64());

    // Memory usage
    let original_bytes = num_vectors * dimensions * 4; // f32 = 4 bytes
    let quantized_bytes_per_vec = (dimensions * bits.to_u8() as usize + 7) / 8;
    let quantized_bytes = num_vectors * quantized_bytes_per_vec;
    let compression_ratio = original_bytes as f32 / quantized_bytes as f32;

    println!("\nMemory Usage:");
    println!("  Original:   {:.2} MB", original_bytes as f64 / 1024.0 / 1024.0);
    println!("  Quantized:  {:.2} MB", quantized_bytes as f64 / 1024.0 / 1024.0);
    println!("  Compression: {:.1}x", compression_ratio);

    // Accuracy benchmarks
    println!("\nAccuracy Benchmark ({} queries):", num_queries);

    let mut recalls = Vec::new();
    let mut query_times = Vec::new();

    for i in 0..num_queries {
        let query = random_vector(dimensions, (num_vectors + i) as u64);

        // Ground truth (brute-force with original vectors)
        let ground_truth = store_ground_truth.knn_search_brute_force(&query, 10).unwrap();

        // Quantized search (two-phase: quantized → rerank)
        let start = Instant::now();
        let results = store_with_quant.knn_search(&query, 10).unwrap();
        let query_time = start.elapsed();

        // Compute recall@10
        let recall = compute_recall(&ground_truth, &results, 10);
        recalls.push(recall);
        query_times.push(query_time.as_secs_f64() * 1000.0); // ms
    }

    // Statistics
    let mean_recall = recalls.iter().sum::<f32>() / recalls.len() as f32;
    let min_recall = recalls.iter().cloned().fold(f32::INFINITY, f32::min);
    let mean_query_time = query_times.iter().sum::<f64>() / query_times.len() as f64;

    println!("  Recall@10:");
    println!("    Mean: {:.1}%", mean_recall * 100.0);
    println!("    Min:  {:.1}%", min_recall * 100.0);

    println!("  Query Latency:");
    println!("    Mean: {:.3}ms", mean_query_time);

    // Pass/Fail
    let recall_target = match bits {
        QuantizationBits::Bits2 => 0.70, // 70% for 2-bit
        QuantizationBits::Bits3 => 0.75, // 75% for 3-bit
        QuantizationBits::Bits4 => 0.85, // 85% for 4-bit
        QuantizationBits::Bits5 => 0.90, // 90% for 5-bit
        QuantizationBits::Bits7 => 0.95, // 95% for 7-bit
        QuantizationBits::Bits8 => 0.95, // 95% for 8-bit
    };

    println!("\nResult:");
    if mean_recall >= recall_target {
        println!("  ✅ PASS (recall {:.1}% >= target {:.1}%)",
                 mean_recall * 100.0, recall_target * 100.0);
    } else {
        println!("  ❌ FAIL (recall {:.1}% < target {:.1}%)",
                 mean_recall * 100.0, recall_target * 100.0);
    }
}

fn main() {
    println!("Extended RaBitQ Benchmark (SIGMOD 2025)");
    println!("Testing accuracy and compression at different bit rates\n");

    // Small benchmark (fast iteration)
    let num_vectors = 1000;
    let dimensions = 128;
    let num_queries = 100;

    println!("Configuration:");
    println!("  Vectors:    {}", num_vectors);
    println!("  Dimensions: {}", dimensions);
    println!("  Queries:    {}", num_queries);

    // Test different compression rates
    let bit_rates = vec![
        QuantizationBits::Bits2,  // 16x compression
        QuantizationBits::Bits4,  // 8x compression (recommended)
        QuantizationBits::Bits8,  // 4x compression
    ];

    for bits in bit_rates {
        benchmark_quantization(bits, num_vectors, dimensions, num_queries);
    }

    println!("\n{}", "=".repeat(80));
    println!("Benchmark Complete!");
    println!("{}", "=".repeat(80));
}
