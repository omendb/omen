use omendb::quantization::{QuantizationModel, QuantizedVector};
use rand::Rng;
use std::time::Instant;

/// Generate random vector with normal distribution
fn generate_random_vector(dimensions: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    (0..dimensions).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

fn main() {
    println!("=== Binary Quantization Benchmark ===\n");

    let dimensions = 1536; // OpenAI embedding size
    let num_samples = 10_000;
    let num_training = 1_000;

    // Generate training data
    println!("Generating {} training vectors ({} dimensions)...", num_training, dimensions);
    let training_start = Instant::now();
    let training_vectors: Vec<Vec<f32>> = (0..num_training)
        .map(|_| generate_random_vector(dimensions))
        .collect();
    let training_gen_time = training_start.elapsed();
    println!("  Generated in {:.2}ms\n", training_gen_time.as_secs_f64() * 1000.0);

    // Train quantization model
    println!("Training quantization model...");
    let train_start = Instant::now();
    let model = QuantizationModel::train(&training_vectors, true).unwrap();
    let train_time = train_start.elapsed();
    println!("  Training complete in {:.2}ms", train_time.as_secs_f64() * 1000.0);
    println!("  Per-sample: {:.4}ms\n", train_time.as_secs_f64() * 1000.0 / num_training as f64);

    // Generate test vectors
    println!("Generating {} test vectors...", num_samples);
    let test_vectors: Vec<Vec<f32>> = (0..num_samples)
        .map(|_| generate_random_vector(dimensions))
        .collect();

    // Benchmark quantization
    println!("\n--- Quantization Performance ---");
    let quantize_start = Instant::now();
    let quantized_vectors: Vec<QuantizedVector> = test_vectors
        .iter()
        .map(|v| model.quantize(v).unwrap())
        .collect();
    let quantize_time = quantize_start.elapsed();

    let avg_quantize_time = quantize_time.as_secs_f64() * 1000.0 / num_samples as f64;
    let quantize_throughput = num_samples as f64 / quantize_time.as_secs_f64();

    println!("  Total time: {:.2}ms", quantize_time.as_secs_f64() * 1000.0);
    println!("  Average: {:.4}ms per vector", avg_quantize_time);
    println!("  Throughput: {:.0} vectors/sec", quantize_throughput);
    println!("  Target: <0.1ms per vector");

    if avg_quantize_time < 0.1 {
        println!("  ✅ PASS: {:.1}x faster than target", 0.1 / avg_quantize_time);
    } else {
        println!("  ❌ FAIL: {:.1}x slower than target", avg_quantize_time / 0.1);
    }

    // Benchmark Hamming distance
    println!("\n--- Hamming Distance Performance ---");
    let num_distance_pairs = 100_000;

    let hamming_start = Instant::now();
    let mut total_distance = 0u32;
    for i in 0..num_distance_pairs {
        let idx1 = i % num_samples;
        let idx2 = (i + 1) % num_samples;
        total_distance += quantized_vectors[idx1].hamming_distance(&quantized_vectors[idx2]);
    }
    let hamming_time = hamming_start.elapsed();

    let avg_hamming_time = hamming_time.as_secs_f64() * 1000.0 / num_distance_pairs as f64;
    let hamming_throughput = num_distance_pairs as f64 / hamming_time.as_secs_f64();

    println!("  Total time: {:.2}ms ({} pairs)", hamming_time.as_secs_f64() * 1000.0, num_distance_pairs);
    println!("  Average: {:.6}ms per pair", avg_hamming_time);
    println!("  Throughput: {:.0} distances/sec", hamming_throughput);
    println!("  Average distance: {:.1} bits", total_distance as f64 / num_distance_pairs as f64);
    println!("  Target: <0.01ms per pair");

    if avg_hamming_time < 0.01 {
        println!("  ✅ PASS: {:.1}x faster than target", 0.01 / avg_hamming_time);
    } else {
        println!("  ❌ FAIL: {:.1}x slower than target", avg_hamming_time / 0.01);
    }

    // Memory analysis
    println!("\n--- Memory Footprint ---");
    let float32_size = dimensions * 4; // 4 bytes per f32
    let quantized_size = quantized_vectors[0].memory_size();
    let reduction_ratio = float32_size as f64 / quantized_size as f64;

    println!("  Original (float32): {} bytes", float32_size);
    println!("  Quantized: {} bytes", quantized_size);
    println!("  Reduction: {:.1}x ({:.1}% memory savings)", reduction_ratio, (1.0 - 1.0/reduction_ratio) * 100.0);
    println!("  Target: ~32x reduction (192 bytes for 1536D)");

    if quantized_size <= 256 {
        println!("  ✅ PASS: Memory usage within target");
    } else {
        println!("  ⚠️  WARNING: Memory usage higher than target");
    }

    // Scaling estimate
    println!("\n--- Scaling Estimates ---");
    let vectors_10m = 10_000_000;
    let float32_10m_gb = (vectors_10m * float32_size) as f64 / 1_000_000_000.0;
    let quantized_10m_gb = (vectors_10m * quantized_size) as f64 / 1_000_000_000.0;

    println!("  10M vectors (1536D):");
    println!("    - float32: {:.1} GB", float32_10m_gb);
    println!("    - quantized: {:.1} GB", quantized_10m_gb);
    println!("    - Savings: {:.1} GB ({:.1}%)",
             float32_10m_gb - quantized_10m_gb,
             (1.0 - quantized_10m_gb / float32_10m_gb) * 100.0);

    println!("\n=== Summary ===");
    println!("✓ Quantization: {:.4}ms/vector ({:.0} vectors/sec)", avg_quantize_time, quantize_throughput);
    println!("✓ Hamming distance: {:.6}ms/pair ({:.0} distances/sec)", avg_hamming_time, hamming_throughput);
    println!("✓ Memory: {:.1}x reduction", reduction_ratio);
    println!("✓ 10M vectors: {:.1} GB quantized (vs {:.1} GB float32)", quantized_10m_gb, float32_10m_gb);

    println!("\n🎯 Ready for HNSW integration (Phase 2: Days 4-6)");
}
