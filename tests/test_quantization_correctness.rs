//! Binary Quantization Correctness Tests
//!
//! Validates Binary Quantization (RaBitQ-style) implementation:
//! 1. Hamming distance correlates with L2 distance
//! 2. Accuracy degradation is acceptable
//! 3. Quantization training is stable
//! 4. Serialization preserves data

use omen::quantization::{QuantizationModel, QuantizedVector};

/// Compute Pearson correlation coefficient between two vectors
fn pearson_correlation(x: &[f32], y: &[f32]) -> f32 {
    assert_eq!(x.len(), y.len(), "Vectors must have same length");

    let n = x.len() as f32;
    let mean_x: f32 = x.iter().sum::<f32>() / n;
    let mean_y: f32 = y.iter().sum::<f32>() / n;

    let mut numerator = 0.0f32;
    let mut sum_sq_x = 0.0f32;
    let mut sum_sq_y = 0.0f32;

    for (&xi, &yi) in x.iter().zip(y.iter()) {
        let dx = xi - mean_x;
        let dy = yi - mean_y;
        numerator += dx * dy;
        sum_sq_x += dx * dx;
        sum_sq_y += dy * dy;
    }

    if sum_sq_x == 0.0 || sum_sq_y == 0.0 {
        return 0.0;
    }

    numerator / (sum_sq_x.sqrt() * sum_sq_y.sqrt())
}

/// Test that Hamming distance correlates with L2 distance
#[test]
fn test_hamming_l2_correlation() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let dimensions = 128;
    let num_vectors = 1000;

    // Generate random vectors
    let vectors: Vec<Vec<f32>> = (0..num_vectors)
        .map(|_| {
            (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect()
        })
        .collect();

    // Train quantization model
    let model = QuantizationModel::train(&vectors, false).unwrap();

    // Quantize all vectors
    let quantized: Vec<QuantizedVector> = vectors
        .iter()
        .map(|v| model.quantize(v).unwrap())
        .collect();

    // Pick a query vector and compute distances
    let query_idx = 0;
    let query = &vectors[query_idx];
    let query_quantized = &quantized[query_idx];

    let mut l2_distances = Vec::new();
    let mut hamming_distances = Vec::new();

    for (i, vector) in vectors.iter().enumerate() {
        if i == query_idx {
            continue;
        }

        // L2 distance
        let l2: f32 = query
            .iter()
            .zip(vector.iter())
            .map(|(a, b)| {
                let diff = a - b;
                diff * diff
            })
            .sum::<f32>()
            .sqrt();

        // Hamming distance
        let hamming = query_quantized.hamming_distance(&quantized[i]) as f32;

        l2_distances.push(l2);
        hamming_distances.push(hamming);
    }

    // Compute correlation
    let correlation = pearson_correlation(&l2_distances, &hamming_distances);

    eprintln!("Hamming vs L2 correlation: {:.4}", correlation);

    // Correlation should be strong (>0.6 for binary quantization)
    assert!(
        correlation >= 0.6,
        "Hamming distance should correlate with L2: correlation = {:.4}",
        correlation
    );
}

/// Test that quantization recall is acceptable
#[test]
fn test_quantization_recall() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let dimensions = 128;
    let num_vectors = 1000;
    let k = 10; // Find 10 nearest neighbors

    // Generate random vectors
    let vectors: Vec<Vec<f32>> = (0..num_vectors)
        .map(|_| {
            (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect()
        })
        .collect();

    // Train quantization model
    let model = QuantizationModel::train(&vectors, false).unwrap();

    // Quantize all vectors
    let quantized: Vec<QuantizedVector> = vectors
        .iter()
        .map(|v| model.quantize(v).unwrap())
        .collect();

    // Test on 100 queries
    let num_queries = 100;
    let mut total_recall = 0.0f32;

    for _ in 0..num_queries {
        let query_idx = rng.gen_range(0..num_vectors);
        let query = &vectors[query_idx];
        let query_quantized = &quantized[query_idx];

        // Ground truth: L2 distance top-k
        let mut l2_results: Vec<(usize, f32)> = vectors
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let dist: f32 = query
                    .iter()
                    .zip(v.iter())
                    .map(|(a, b)| {
                        let diff = a - b;
                        diff * diff
                    })
                    .sum::<f32>()
                    .sqrt();
                (i, dist)
            })
            .collect();
        l2_results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let ground_truth: std::collections::HashSet<usize> =
            l2_results.iter().take(k).map(|(id, _)| *id).collect();

        // Hamming distance top-k
        let mut hamming_results: Vec<(usize, u32)> = quantized
            .iter()
            .enumerate()
            .map(|(i, qv)| (i, query_quantized.hamming_distance(qv)))
            .collect();
        hamming_results.sort_by(|a, b| a.1.cmp(&b.1));
        let hamming_top_k: std::collections::HashSet<usize> =
            hamming_results.iter().take(k).map(|(id, _)| *id).collect();

        // Compute recall
        let intersection = ground_truth.intersection(&hamming_top_k).count();
        let recall = intersection as f32 / k as f32;
        total_recall += recall;
    }

    let avg_recall = total_recall / num_queries as f32;

    eprintln!(
        "Binary Quantization recall@{}: {:.2}%",
        k,
        avg_recall * 100.0
    );

    // Binary quantization achieves 30-40% recall (expected for 1-bit quantization)
    // This is why reranking is critical for production use
    assert!(
        avg_recall >= 0.25,
        "Quantization recall too low: {:.2}% (expected >= 25%)",
        avg_recall * 100.0
    );

    // Document actual performance
    eprintln!("✓ Binary quantization baseline recall: {:.2}%", avg_recall * 100.0);
}

/// Test that reranking improves accuracy
#[test]
fn test_quantization_reranking() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let dimensions = 128;
    let num_vectors = 500;
    let k = 10;
    let rerank_k = 50; // Retrieve 50, rerank to top-10

    // Generate random vectors
    let vectors: Vec<Vec<f32>> = (0..num_vectors)
        .map(|_| {
            (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect()
        })
        .collect();

    // Train quantization model
    let model = QuantizationModel::train(&vectors, false).unwrap();

    // Quantize all vectors
    let quantized: Vec<QuantizedVector> = vectors
        .iter()
        .map(|v| model.quantize(v).unwrap())
        .collect();

    // Test on 50 queries
    let num_queries = 50;
    let mut recall_without_rerank = 0.0f32;
    let mut recall_with_rerank = 0.0f32;

    for _ in 0..num_queries {
        let query_idx = rng.gen_range(0..num_vectors);
        let query = &vectors[query_idx];
        let query_quantized = &quantized[query_idx];

        // Ground truth: L2 distance top-k
        let mut l2_results: Vec<(usize, f32)> = vectors
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let dist: f32 = query
                    .iter()
                    .zip(v.iter())
                    .map(|(a, b)| {
                        let diff = a - b;
                        diff * diff
                    })
                    .sum::<f32>()
                    .sqrt();
                (i, dist)
            })
            .collect();
        l2_results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let ground_truth: std::collections::HashSet<usize> =
            l2_results.iter().take(k).map(|(id, _)| *id).collect();

        // 1. Hamming distance top-k (no reranking)
        let mut hamming_results: Vec<(usize, u32)> = quantized
            .iter()
            .enumerate()
            .map(|(i, qv)| (i, query_quantized.hamming_distance(qv)))
            .collect();
        hamming_results.sort_by(|a, b| a.1.cmp(&b.1));
        let hamming_top_k: std::collections::HashSet<usize> =
            hamming_results.iter().take(k).map(|(id, _)| *id).collect();

        let recall_no_rerank = ground_truth.intersection(&hamming_top_k).count() as f32 / k as f32;

        // 2. Hamming top-50, then rerank with L2 for top-10
        let candidates: Vec<usize> = hamming_results
            .iter()
            .take(rerank_k)
            .map(|(id, _)| *id)
            .collect();

        let mut reranked: Vec<(usize, f32)> = candidates
            .iter()
            .map(|&i| {
                let dist: f32 = query
                    .iter()
                    .zip(vectors[i].iter())
                    .map(|(a, b)| {
                        let diff = a - b;
                        diff * diff
                    })
                    .sum::<f32>()
                    .sqrt();
                (i, dist)
            })
            .collect();
        reranked.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let reranked_top_k: std::collections::HashSet<usize> =
            reranked.iter().take(k).map(|(id, _)| *id).collect();

        let recall_rerank = ground_truth.intersection(&reranked_top_k).count() as f32 / k as f32;

        recall_without_rerank += recall_no_rerank;
        recall_with_rerank += recall_rerank;
    }

    let avg_recall_no_rerank = recall_without_rerank / num_queries as f32;
    let avg_recall_rerank = recall_with_rerank / num_queries as f32;

    eprintln!("Recall without reranking: {:.2}%", avg_recall_no_rerank * 100.0);
    eprintln!("Recall with reranking: {:.2}%", avg_recall_rerank * 100.0);

    // Reranking should improve recall significantly
    assert!(
        avg_recall_rerank >= avg_recall_no_rerank,
        "Reranking should improve or maintain recall: {:.2}% vs {:.2}%",
        avg_recall_rerank * 100.0,
        avg_recall_no_rerank * 100.0
    );

    // Reranking should achieve >65% recall (realistic for top-50 → top-10 with BQ)
    assert!(
        avg_recall_rerank >= 0.65,
        "Reranked recall too low: {:.2}% (expected >= 65%)",
        avg_recall_rerank * 100.0
    );

    // Document the improvement
    let improvement = (avg_recall_rerank - avg_recall_no_rerank) * 100.0;
    eprintln!("✓ Reranking improvement: +{:.2} percentage points", improvement);
}

/// Test quantization training stability
#[test]
fn test_quantization_training_stability() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let dimensions = 128;
    let num_vectors = 500;

    // Generate training data
    let vectors: Vec<Vec<f32>> = (0..num_vectors)
        .map(|_| {
            (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect()
        })
        .collect();

    // Train model (non-randomized for reproducibility)
    let model1 = QuantizationModel::train(&vectors, false).unwrap();
    let model2 = QuantizationModel::train(&vectors, false).unwrap();

    // Quantize same vector with both models
    let test_vector = vec![0.5; dimensions];
    let q1 = model1.quantize(&test_vector).unwrap();
    let q2 = model2.quantize(&test_vector).unwrap();

    // Results should be identical (non-randomized training)
    assert_eq!(
        q1.hamming_distance(&q2),
        0,
        "Non-randomized training should be deterministic"
    );
}

/// Test quantization with edge case vectors
#[test]
fn test_quantization_edge_cases() {
    let dimensions = 64;

    // Training data with normal vectors
    let mut vectors: Vec<Vec<f32>> = vec![];
    for i in 0..100 {
        let v: Vec<f32> = (0..dimensions).map(|j| (i + j) as f32 * 0.01).collect();
        vectors.push(v);
    }

    let model = QuantizationModel::train(&vectors, false).unwrap();

    // Test 1: Zero vector
    let zero = vec![0.0; dimensions];
    let q_zero = model.quantize(&zero).unwrap();
    assert_eq!(q_zero.dimensions(), dimensions);

    // Test 2: All positive
    let positive = vec![1.0; dimensions];
    let q_pos = model.quantize(&positive).unwrap();
    assert_eq!(q_pos.dimensions(), dimensions);

    // Test 3: All negative
    let negative = vec![-1.0; dimensions];
    let q_neg = model.quantize(&negative).unwrap();
    assert_eq!(q_neg.dimensions(), dimensions);

    // Test 4: Hamming distance should be symmetric
    assert_eq!(
        q_pos.hamming_distance(&q_neg),
        q_neg.hamming_distance(&q_pos)
    );
}

/// Test quantization serialization roundtrip
#[test]
fn test_quantization_serialization() {
    let dimensions = 128;
    let num_vectors = 100;

    // Generate training data
    let vectors: Vec<Vec<f32>> = (0..num_vectors)
        .map(|i| (0..dimensions).map(|j| (i + j) as f32 * 0.01).collect())
        .collect();

    // Train model
    let model = QuantizationModel::train(&vectors, false).unwrap();

    // Serialize model
    let model_bytes = bincode::serialize(&model).unwrap();

    // Deserialize model
    let restored_model: QuantizationModel = bincode::deserialize(&model_bytes).unwrap();

    // Quantize with both models
    let test_vector = vec![0.5; dimensions];
    let q1 = model.quantize(&test_vector).unwrap();
    let q2 = restored_model.quantize(&test_vector).unwrap();

    // Results should be identical
    assert_eq!(
        q1.hamming_distance(&q2),
        0,
        "Serialized model should produce identical results"
    );
}

/// Test high-dimensional quantization (1536D, OpenAI embedding size)
#[test]
fn test_quantization_high_dimensional() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let dimensions = 1536;
    let num_vectors = 500;
    let k = 10;

    // Generate random normalized vectors (typical for embeddings)
    let vectors: Vec<Vec<f32>> = (0..num_vectors)
        .map(|_| {
            let v: Vec<f32> = (0..dimensions).map(|_| rng.gen_range(-1.0..1.0)).collect();
            // Normalize
            let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
            v.iter().map(|x| x / norm).collect()
        })
        .collect();

    // Train quantization model
    let model = QuantizationModel::train(&vectors, false).unwrap();

    // Quantize all vectors
    let quantized: Vec<QuantizedVector> = vectors
        .iter()
        .map(|v| model.quantize(v).unwrap())
        .collect();

    // Test memory efficiency
    let original_memory = dimensions * 4; // 4 bytes per float32
    let quantized_memory = quantized[0].memory_size();
    let compression_ratio = original_memory as f32 / quantized_memory as f32;

    eprintln!("Original memory: {} bytes", original_memory);
    eprintln!("Quantized memory: {} bytes", quantized_memory);
    eprintln!("Compression ratio: {:.2}x", compression_ratio);

    // Should achieve ~30x compression for 1536D
    assert!(
        compression_ratio >= 25.0,
        "Compression ratio too low: {:.2}x (expected >= 25x)",
        compression_ratio
    );

    // Test recall on high-dimensional data
    let query_idx = 0;
    let query = &vectors[query_idx];
    let query_quantized = &quantized[query_idx];

    // Ground truth
    let mut l2_results: Vec<(usize, f32)> = vectors
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let dist: f32 = query
                .iter()
                .zip(v.iter())
                .map(|(a, b)| {
                    let diff = a - b;
                    diff * diff
                })
                .sum::<f32>()
                .sqrt();
            (i, dist)
        })
        .collect();
    l2_results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let ground_truth: std::collections::HashSet<usize> =
        l2_results.iter().take(k).map(|(id, _)| *id).collect();

    // Hamming distance
    let mut hamming_results: Vec<(usize, u32)> = quantized
        .iter()
        .enumerate()
        .map(|(i, qv)| (i, query_quantized.hamming_distance(qv)))
        .collect();
    hamming_results.sort_by(|a, b| a.1.cmp(&b.1));
    let hamming_top_k: std::collections::HashSet<usize> =
        hamming_results.iter().take(k).map(|(id, _)| *id).collect();

    let recall = ground_truth.intersection(&hamming_top_k).count() as f32 / k as f32;

    eprintln!("High-dimensional recall@{}: {:.2}%", k, recall * 100.0);

    // High-dimensional binary quantization achieves 40-50% recall
    // This is expected for 1-bit quantization on normalized embeddings
    assert!(
        recall >= 0.40,
        "High-dimensional recall too low: {:.2}% (expected >= 40%)",
        recall * 100.0
    );

    eprintln!("✓ High-dimensional BQ recall validated: {:.2}%", recall * 100.0);
}
