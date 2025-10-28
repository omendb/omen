//! Distance Calculation Correctness Tests
//!
//! Validates that our distance implementations match reference implementations.
//! These tests are critical for an AI-generated codebase.

use omen::vector::types::Vector;

/// Test L2 (Euclidean) distance against known values
#[test]
fn test_l2_distance_known_values() {
    // Test case 1: Identical vectors (distance should be 0)
    let v1 = Vector::new(vec![1.0, 2.0, 3.0]);
    let v2 = Vector::new(vec![1.0, 2.0, 3.0]);
    let dist = v1.l2_distance(&v2).unwrap();
    assert!((dist - 0.0).abs() < 1e-6, "Identical vectors should have distance 0, got {}", dist);

    // Test case 2: Orthogonal unit vectors
    let v1 = Vector::new(vec![1.0, 0.0, 0.0]);
    let v2 = Vector::new(vec![0.0, 1.0, 0.0]);
    let dist = v1.l2_distance(&v2).unwrap();
    let expected = (2.0_f32).sqrt(); // sqrt(1^2 + 1^2) = sqrt(2)
    assert!((dist - expected).abs() < 1e-6, "Expected {}, got {}", expected, dist);

    // Test case 3: Known distance
    let v1 = Vector::new(vec![1.0, 2.0, 3.0]);
    let v2 = Vector::new(vec![4.0, 6.0, 8.0]);
    // Distance = sqrt((4-1)^2 + (6-2)^2 + (8-3)^2) = sqrt(9 + 16 + 25) = sqrt(50)
    let expected = 50.0_f32.sqrt();
    let dist = v1.l2_distance(&v2).unwrap();
    assert!((dist - expected).abs() < 1e-5, "Expected {}, got {}", expected, dist);

    // Test case 4: Negative coordinates
    let v1 = Vector::new(vec![-1.0, -2.0, -3.0]);
    let v2 = Vector::new(vec![1.0, 2.0, 3.0]);
    // Distance = sqrt(4 + 16 + 36) = sqrt(56)
    let expected = 56.0_f32.sqrt();
    let dist = v1.l2_distance(&v2).unwrap();
    assert!((dist - expected).abs() < 1e-5, "Expected {}, got {}", expected, dist);
}

/// Test L2 distance edge cases
#[test]
fn test_l2_distance_edge_cases() {
    // Zero vector
    let zero = Vector::new(vec![0.0, 0.0, 0.0]);
    let v = Vector::new(vec![3.0, 4.0, 0.0]);
    let dist = zero.l2_distance(&v).unwrap();
    assert!((dist - 5.0).abs() < 1e-6, "Distance from zero should be 5.0, got {}", dist);

    // Very small values (numerical stability)
    let v1 = Vector::new(vec![1e-10, 2e-10, 3e-10]);
    let v2 = Vector::new(vec![1e-10, 2e-10, 3e-10]);
    let dist = v1.l2_distance(&v2).unwrap();
    assert!(dist < 1e-9, "Small identical vectors should have near-zero distance, got {}", dist);

    // Large values (no overflow)
    let v1 = Vector::new(vec![1e6, 2e6, 3e6]);
    let v2 = Vector::new(vec![1e6, 2e6, 3e6]);
    let dist = v1.l2_distance(&v2).unwrap();
    assert!((dist - 0.0).abs() < 1.0, "Large identical vectors should have distance 0, got {}", dist);

    // Dimension mismatch should error
    let v1 = Vector::new(vec![1.0, 2.0]);
    let v2 = Vector::new(vec![1.0, 2.0, 3.0]);
    assert!(v1.l2_distance(&v2).is_err(), "Mismatched dimensions should error");
}

/// Test cosine distance against known values
#[test]
fn test_cosine_distance_known_values() {
    // Test case 1: Identical vectors (distance should be 0)
    let v1 = Vector::new(vec![1.0, 2.0, 3.0]);
    let v2 = Vector::new(vec![1.0, 2.0, 3.0]);
    let dist = v1.cosine_distance(&v2).unwrap();
    assert!(dist < 1e-6, "Identical vectors should have cosine distance ~0, got {}", dist);

    // Test case 2: Opposite vectors (distance should be 2.0)
    let v1 = Vector::new(vec![1.0, 2.0, 3.0]);
    let v2 = Vector::new(vec![-1.0, -2.0, -3.0]);
    let dist = v1.cosine_distance(&v2).unwrap();
    assert!((dist - 2.0).abs() < 1e-5, "Opposite vectors should have cosine distance 2.0, got {}", dist);

    // Test case 3: Orthogonal vectors (distance should be 1.0)
    let v1 = Vector::new(vec![1.0, 0.0, 0.0]);
    let v2 = Vector::new(vec![0.0, 1.0, 0.0]);
    let dist = v1.cosine_distance(&v2).unwrap();
    assert!((dist - 1.0).abs() < 1e-5, "Orthogonal vectors should have cosine distance 1.0, got {}", dist);

    // Test case 4: Scaled vectors (should have same distance as originals)
    let v1 = Vector::new(vec![1.0, 2.0, 3.0]);
    let v2_1 = Vector::new(vec![2.0, 4.0, 6.0]);
    let v2_2 = Vector::new(vec![10.0, 20.0, 30.0]);
    let dist1 = v1.cosine_distance(&v2_1).unwrap();
    let dist2 = v1.cosine_distance(&v2_2).unwrap();
    assert!((dist1 - dist2).abs() < 1e-5, "Scaled vectors should have same cosine distance");
}

/// Test cosine distance edge cases
#[test]
fn test_cosine_distance_edge_cases() {
    // Zero vector should error or return special value
    let zero = Vector::new(vec![0.0, 0.0, 0.0]);
    let v = Vector::new(vec![1.0, 2.0, 3.0]);
    let result = zero.cosine_distance(&v);
    // Should either error or handle gracefully (document the behavior)
    match result {
        Ok(dist) => {
            // If it returns a value, document what it is
            println!("Cosine distance with zero vector: {}", dist);
        }
        Err(_) => {
            // Expected behavior - zero vector has no direction
        }
    }

    // Unit vectors
    let v1 = Vector::new(vec![1.0, 0.0, 0.0]);
    let v2 = Vector::new(vec![0.0, 1.0, 0.0]);
    let dist = v1.cosine_distance(&v2).unwrap();
    assert!((dist - 1.0).abs() < 1e-6, "Orthogonal unit vectors should have distance 1.0");

    // Numerical stability with very small values
    let v1 = Vector::new(vec![1e-20, 2e-20, 3e-20]);
    let v2 = Vector::new(vec![1e-20, 2e-20, 3e-20]);
    let result = v1.cosine_distance(&v2);
    // Should handle numerical stability gracefully
    if let Ok(dist) = result {
        assert!(dist < 0.1, "Small identical vectors should have near-zero cosine distance");
    }
}

/// Test dot product correctness
#[test]
fn test_dot_product_known_values() {
    // Test case 1: Orthogonal vectors (dot product = 0)
    let v1 = Vector::new(vec![1.0, 0.0, 0.0]);
    let v2 = Vector::new(vec![0.0, 1.0, 0.0]);
    let dot = v1.dot_product(&v2).unwrap();
    assert!((dot - 0.0).abs() < 1e-6, "Orthogonal vectors should have dot product 0, got {}", dot);

    // Test case 2: Parallel vectors
    let v1 = Vector::new(vec![1.0, 2.0, 3.0]);
    let v2 = Vector::new(vec![2.0, 4.0, 6.0]);
    let dot = v1.dot_product(&v2).unwrap();
    let expected = 1.0*2.0 + 2.0*4.0 + 3.0*6.0; // 2 + 8 + 18 = 28
    assert!((dot - expected).abs() < 1e-5, "Expected {}, got {}", expected, dot);

    // Test case 3: Known values
    let v1 = Vector::new(vec![1.0, 2.0, 3.0]);
    let v2 = Vector::new(vec![4.0, 5.0, 6.0]);
    let dot = v1.dot_product(&v2).unwrap();
    let expected = 1.0*4.0 + 2.0*5.0 + 3.0*6.0; // 4 + 10 + 18 = 32
    assert!((dot - expected).abs() < 1e-5, "Expected {}, got {}", expected, dot);

    // Test case 4: Negative values
    let v1 = Vector::new(vec![1.0, -2.0, 3.0]);
    let v2 = Vector::new(vec![-1.0, 2.0, -3.0]);
    let dot = v1.dot_product(&v2).unwrap();
    let expected = 1.0*(-1.0) + (-2.0)*2.0 + 3.0*(-3.0); // -1 - 4 - 9 = -14
    assert!((dot - expected).abs() < 1e-5, "Expected {}, got {}", expected, dot);
}

/// Test vector normalization
#[test]
fn test_normalize() {
    // Test case 1: Non-unit vector
    let v = Vector::new(vec![3.0, 4.0, 0.0]);
    let normalized = v.normalize().unwrap();

    // Check magnitude is 1.0
    let mag = (normalized.data[0].powi(2) + normalized.data[1].powi(2) + normalized.data[2].powi(2)).sqrt();
    assert!((mag - 1.0).abs() < 1e-6, "Normalized vector should have magnitude 1.0, got {}", mag);

    // Check direction is preserved (proportional to original)
    let ratio = normalized.data[0] / v.data[0];
    assert!((normalized.data[1] / v.data[1] - ratio).abs() < 1e-6, "Direction should be preserved");

    // Test case 2: Already normalized
    let v = Vector::new(vec![1.0, 0.0, 0.0]);
    let normalized = v.normalize().unwrap();
    assert!((normalized.data[0] - 1.0).abs() < 1e-6, "Already normalized vector should remain unchanged");

    // Test case 3: Zero vector should error
    let zero = Vector::new(vec![0.0, 0.0, 0.0]);
    assert!(zero.normalize().is_err(), "Cannot normalize zero vector");
}

/// Test distance symmetry (d(a,b) == d(b,a))
#[test]
fn test_distance_symmetry() {
    let v1 = Vector::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    let v2 = Vector::new(vec![5.0, 4.0, 3.0, 2.0, 1.0]);

    // L2 distance should be symmetric
    let dist12 = v1.l2_distance(&v2).unwrap();
    let dist21 = v2.l2_distance(&v1).unwrap();
    assert!((dist12 - dist21).abs() < 1e-6, "L2 distance should be symmetric");

    // Cosine distance should be symmetric
    let cos12 = v1.cosine_distance(&v2).unwrap();
    let cos21 = v2.cosine_distance(&v1).unwrap();
    assert!((cos12 - cos21).abs() < 1e-6, "Cosine distance should be symmetric");

    // Dot product should be commutative
    let dot12 = v1.dot_product(&v2).unwrap();
    let dot21 = v2.dot_product(&v1).unwrap();
    assert!((dot12 - dot21).abs() < 1e-6, "Dot product should be commutative");
}

/// Test triangle inequality: d(a,c) <= d(a,b) + d(b,c)
#[test]
fn test_triangle_inequality() {
    let v1 = Vector::new(vec![0.0, 0.0, 0.0]);
    let v2 = Vector::new(vec![1.0, 0.0, 0.0]);
    let v3 = Vector::new(vec![1.0, 1.0, 0.0]);

    let d12 = v1.l2_distance(&v2).unwrap();
    let d23 = v2.l2_distance(&v3).unwrap();
    let d13 = v1.l2_distance(&v3).unwrap();

    assert!(d13 <= d12 + d23 + 1e-6, "Triangle inequality violated: {} > {} + {}", d13, d12, d23);
}

/// Test high-dimensional vectors (1536D like OpenAI embeddings)
#[test]
fn test_high_dimensional_distances() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Generate two random 1536-dimensional vectors
    let v1_data: Vec<f32> = (0..1536).map(|_| rng.gen_range(-1.0..1.0)).collect();
    let v2_data: Vec<f32> = (0..1536).map(|_| rng.gen_range(-1.0..1.0)).collect();

    let v1 = Vector::new(v1_data.clone());
    let v2 = Vector::new(v2_data.clone());

    // Compute distances - should not panic or overflow
    let l2 = v1.l2_distance(&v2).unwrap();
    let cosine = v1.cosine_distance(&v2).unwrap();
    let dot = v1.dot_product(&v2).unwrap();

    // Sanity checks
    assert!(l2 >= 0.0, "L2 distance should be non-negative");
    assert!(l2.is_finite(), "L2 distance should be finite");
    assert!(cosine >= 0.0 && cosine <= 2.0, "Cosine distance should be in [0, 2]");
    assert!(cosine.is_finite(), "Cosine distance should be finite");
    assert!(dot.is_finite(), "Dot product should be finite");

    // Reference implementation (manual calculation)
    let mut manual_l2_sq = 0.0f32;
    let mut manual_dot = 0.0f32;
    let mut v1_mag_sq = 0.0f32;
    let mut v2_mag_sq = 0.0f32;

    for i in 0..1536 {
        let diff = v1_data[i] - v2_data[i];
        manual_l2_sq += diff * diff;
        manual_dot += v1_data[i] * v2_data[i];
        v1_mag_sq += v1_data[i] * v1_data[i];
        v2_mag_sq += v2_data[i] * v2_data[i];
    }

    let manual_l2 = manual_l2_sq.sqrt();
    let manual_cosine_sim = manual_dot / (v1_mag_sq.sqrt() * v2_mag_sq.sqrt());
    let manual_cosine_dist = 1.0 - manual_cosine_sim;

    // Verify against manual calculation
    assert!((l2 - manual_l2).abs() < 1e-3, "L2 distance mismatch: {} vs {}", l2, manual_l2);
    assert!((dot - manual_dot).abs() < 1e-2, "Dot product mismatch: {} vs {}", dot, manual_dot);
    assert!((cosine - manual_cosine_dist).abs() < 1e-4, "Cosine distance mismatch: {} vs {}", cosine, manual_cosine_dist);
}

/// Test that NaN and Inf are handled properly
#[test]
fn test_nan_inf_handling() {
    // NaN in vector
    let v_nan = Vector::new(vec![1.0, f32::NAN, 3.0]);
    let v_normal = Vector::new(vec![1.0, 2.0, 3.0]);

    let result = v_nan.l2_distance(&v_normal);
    // Should either error or return NaN (document the behavior)
    match result {
        Ok(dist) => assert!(dist.is_nan() || dist.is_infinite(), "NaN should propagate"),
        Err(_) => (), // Error is also acceptable
    }

    // Inf in vector
    let v_inf = Vector::new(vec![1.0, f32::INFINITY, 3.0]);
    let result = v_inf.l2_distance(&v_normal);
    match result {
        Ok(dist) => assert!(dist.is_infinite(), "Inf should propagate"),
        Err(_) => (), // Error is also acceptable
    }
}
