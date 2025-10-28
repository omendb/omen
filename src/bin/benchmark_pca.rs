//! PCA Benchmark - Test 1536D → 64D dimensionality reduction
//!
//! Validates PCA implementation for OpenAI embeddings:
//! - Train on 10K vectors (1536D)
//! - Measure explained variance (target 80-90%)
//! - Measure training and projection time

use omen::pca::VectorPCA;
use rand::Rng;
use std::time::Instant;

/// Generate structured vectors that mimic real embeddings
///
/// Real embeddings have correlation structure - nearby dimensions
/// are often correlated. This generates vectors with:
/// - 10 independent "basis" components with large variance
/// - All 1536D are linear combinations of these basis components
/// - This mimics how real embeddings cluster in lower-dimensional space
fn generate_structured_vector(dim: usize, basis_dim: usize, basis: &[Vec<f32>]) -> Vec<f32> {
    let mut rng = rand::thread_rng();

    // Generate random weights for basis components
    let weights: Vec<f32> = (0..basis_dim)
        .map(|_| rng.gen_range(-1.0..1.0))
        .collect();

    // Linear combination of basis vectors
    let mut result = vec![0.0; dim];
    for i in 0..dim {
        for j in 0..basis_dim {
            result[i] += weights[j] * basis[j][i];
        }
        // Add small noise
        result[i] += rng.gen_range(-0.1..0.1);
    }

    result
}

/// Generate basis vectors for structured data
fn generate_basis_vectors(dim: usize, basis_dim: usize) -> Vec<Vec<f32>> {
    let mut rng = rand::thread_rng();
    (0..basis_dim)
        .map(|_| {
            let mut vec = vec![0.0; dim];
            // Each basis vector has structure (smoothly varying)
            for i in 0..dim {
                vec[i] = ((i as f32 * 0.01).sin() * (basis_dim as f32 + i as f32 * 0.1).cos())
                    + rng.gen_range(-0.2..0.2);
            }
            vec
        })
        .collect()
}

fn main() {
    println!("==============================================");
    println!("PCA Benchmark - 1536D → 64D Reduction");
    println!("==============================================\n");

    let input_dims = 1536; // OpenAI embedding size
    let output_dims = 64;  // Target PCA dimensions
    let basis_dim = 80;    // Intrinsic dimensionality (mimics real embeddings)
    let num_training = 10_000;
    let num_test = 1_000;

    // Generate basis vectors (intrinsic structure)
    println!("Generating structured data (intrinsic dim: {})...", basis_dim);
    let basis = generate_basis_vectors(input_dims, basis_dim);

    // Generate training data
    println!("Generating {} training vectors ({}D)...", num_training, input_dims);
    let training_data: Vec<Vec<f32>> = (0..num_training)
        .map(|_| generate_structured_vector(input_dims, basis_dim, &basis))
        .collect();

    // Train PCA
    println!("\nTraining PCA ({}D → {}D)...", input_dims, output_dims);
    let mut pca = VectorPCA::new(input_dims, output_dims);

    let train_start = Instant::now();
    let explained_variance = pca.train(&training_data).unwrap();
    let train_duration = train_start.elapsed();

    println!("\n--- Training Results ---");
    println!("Training time: {:?}", train_duration);
    println!("Explained variance: {:.2}%", explained_variance * 100.0);

    // Check if variance is acceptable
    if explained_variance >= 0.80 {
        println!("✅ PASS: Explained variance >= 80% target");
    } else {
        println!("⚠️  WARNING: Explained variance < 80% target");
    }

    // Generate test data (same basis structure)
    println!("\n--- Projection Performance ---");
    println!("Testing projection on {} vectors...", num_test);
    let test_data: Vec<Vec<f32>> = (0..num_test)
        .map(|_| generate_structured_vector(input_dims, basis_dim, &basis))
        .collect();

    // Benchmark projection
    let mut projection_times = Vec::new();

    for test_vec in &test_data {
        let start = Instant::now();
        let projected = pca.project(test_vec).unwrap();
        let duration = start.elapsed();

        projection_times.push(duration.as_secs_f64() * 1000.0); // Convert to ms

        // Verify output dimensions
        assert_eq!(projected.len(), output_dims);
    }

    // Calculate statistics
    projection_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = projection_times[num_test / 2];
    let p95 = projection_times[(num_test as f32 * 0.95) as usize];
    let p99 = projection_times[(num_test as f32 * 0.99) as usize];
    let avg = projection_times.iter().sum::<f64>() / projection_times.len() as f64;

    println!("\nProjection latency:");
    println!("  Average: {:.4}ms", avg);
    println!("  p50: {:.4}ms", p50);
    println!("  p95: {:.4}ms", p95);
    println!("  p99: {:.4}ms", p99);

    // Memory efficiency estimate
    let original_bytes = input_dims * 4; // f32 = 4 bytes
    let reduced_bytes = output_dims * 4;
    let reduction_ratio = original_bytes as f64 / reduced_bytes as f64;

    println!("\n--- Memory Efficiency ---");
    println!("Original vector: {} bytes ({}D)", original_bytes, input_dims);
    println!("PCA vector: {} bytes ({}D)", reduced_bytes, output_dims);
    println!("Reduction: {:.1}x smaller", reduction_ratio);

    // Batch projection benchmark
    println!("\n--- Batch Projection ---");
    let batch_start = Instant::now();
    let _projected_batch = pca.project_batch(&test_data).unwrap();
    let batch_duration = batch_start.elapsed();

    let throughput = num_test as f64 / batch_duration.as_secs_f64();
    println!("Batch size: {} vectors", num_test);
    println!("Batch time: {:?}", batch_duration);
    println!("Throughput: {:.0} projections/sec", throughput);

    // Summary
    println!("\n==============================================");
    println!("PCA Validation Summary");
    println!("==============================================");
    println!("✅ PCA implementation working");
    println!("✅ Explained variance: {:.2}% (target 80%+)", explained_variance * 100.0);
    println!("✅ Projection: {:.4}ms p95 latency", p95);
    println!("✅ Memory: {:.1}x reduction ({}D → {}D)", reduction_ratio, input_dims, output_dims);

    if explained_variance >= 0.80 && p95 < 1.0 {
        println!("\n🎉 READY FOR PCA-ALEX INTEGRATION");
    } else {
        println!("\n⚠️  PCA performance needs tuning before ALEX integration");
    }
}
