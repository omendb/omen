//! Input Validation & Edge Case Tests
//!
//! Validates that the system handles invalid input gracefully:
//! - Malformed vectors (wrong dimensions)
//! - NaN and Inf values
//! - Empty datasets
//! - Boundary conditions

use omendb::vector::types::Vector;
use omendb::vector::store::VectorStore;

/// Test dimension mismatch detection
#[test]
fn test_dimension_mismatch_insert() {
    let mut store = VectorStore::new(128);

    // Correct dimension should work
    let valid = Vector::new(vec![1.0; 128]);
    assert!(store.insert(valid).is_ok());

    // Wrong dimension should fail
    let too_small = Vector::new(vec![1.0; 64]);
    let result = store.insert(too_small);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("dimension mismatch"));

    let too_large = Vector::new(vec![1.0; 256]);
    let result = store.insert(too_large);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("dimension mismatch"));
}

/// Test dimension mismatch in knn_search
#[test]
fn test_dimension_mismatch_search() {
    let mut store = VectorStore::new(128);

    // Insert some vectors
    for _ in 0..10 {
        store.insert(Vector::new(vec![1.0; 128])).unwrap();
    }

    // Correct dimension search should work
    let valid_query = Vector::new(vec![1.0; 128]);
    assert!(store.knn_search(&valid_query, 5).is_ok());

    // Wrong dimension search should fail
    let wrong_query = Vector::new(vec![1.0; 64]);
    let result = store.knn_search(&wrong_query, 5);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("dimension mismatch"));
}

/// Test NaN handling in vector operations
#[test]
fn test_nan_values() {
    // NaN in vector data
    let v1 = Vector::new(vec![1.0, 2.0, f32::NAN, 4.0]);
    let v2 = Vector::new(vec![1.0, 2.0, 3.0, 4.0]);

    // Distance calculations should handle NaN
    let dist = v1.l2_distance(&v2);
    assert!(dist.is_ok());
    assert!(dist.unwrap().is_nan(), "Distance with NaN should be NaN");

    // Cosine distance with NaN
    let cos_dist = v1.cosine_distance(&v2);
    assert!(cos_dist.is_ok());
    // Cosine distance with NaN will produce NaN
    assert!(cos_dist.unwrap().is_nan());
}

/// Test Inf handling
#[test]
fn test_inf_values() {
    let v1 = Vector::new(vec![1.0, 2.0, f32::INFINITY, 4.0]);
    let v2 = Vector::new(vec![1.0, 2.0, 3.0, 4.0]);

    // L2 distance with Inf
    let dist = v1.l2_distance(&v2);
    assert!(dist.is_ok());
    assert!(dist.unwrap().is_infinite());

    // Negative infinity
    let v3 = Vector::new(vec![1.0, f32::NEG_INFINITY, 3.0, 4.0]);
    let dist2 = v3.l2_distance(&v2);
    assert!(dist2.is_ok());
    assert!(dist2.unwrap().is_infinite());
}

/// Test zero vector handling
#[test]
fn test_zero_vector() {
    let zero = Vector::new(vec![0.0; 128]);
    let non_zero = Vector::new(vec![1.0; 128]);

    // L2 distance should work
    let dist = zero.l2_distance(&non_zero).unwrap();
    assert!(dist > 0.0);

    // Cosine distance with zero vector should fail or return special value
    let cos_dist = zero.cosine_distance(&non_zero);
    // Either error or NaN/Inf is acceptable
    if let Ok(d) = cos_dist {
        // If it succeeds, should be NaN or Inf (undefined for zero vector)
        assert!(d.is_nan() || d.is_infinite());
    }

    // Normalize zero vector should fail
    let normalized = zero.normalize();
    assert!(normalized.is_err());
    assert!(normalized.unwrap_err().to_string().contains("zero vector"));
}

/// Test empty dataset operations
#[test]
fn test_empty_dataset() {
    let mut store = VectorStore::new(128);

    // Search on empty dataset should return empty results
    let query = Vector::new(vec![1.0; 128]);
    let results = store.knn_search(&query, 10).unwrap();
    assert!(results.is_empty(), "Empty dataset should return no results");

    // Length should be 0
    assert_eq!(store.len(), 0);
    assert!(store.is_empty());
}

/// Test k=0 in knn_search
#[test]
fn test_knn_search_k_zero() {
    let mut store = VectorStore::new(128);

    for i in 0..10 {
        store.insert(Vector::new(vec![i as f32; 128])).unwrap();
    }

    let query = Vector::new(vec![5.0; 128]);
    let results = store.knn_search(&query, 0).unwrap();

    // k=0 should return empty results (no neighbors requested)
    assert!(results.is_empty());
}

/// Test k > dataset size
#[test]
fn test_knn_search_k_exceeds_size() {
    let mut store = VectorStore::new(128);

    // Insert only 5 vectors
    for i in 0..5 {
        store.insert(Vector::new(vec![i as f32; 128])).unwrap();
    }

    let query = Vector::new(vec![2.5; 128]);
    let results = store.knn_search(&query, 100).unwrap();

    // Should return all available vectors (5), not fail
    assert_eq!(results.len(), 5);
}

/// Test extremely small values
#[test]
fn test_very_small_values() {
    let v1 = Vector::new(vec![1e-10, 1e-20, 1e-30]);
    let v2 = Vector::new(vec![2e-10, 2e-20, 2e-30]);

    // Should handle very small values without underflow
    let dist = v1.l2_distance(&v2).unwrap();
    assert!(dist > 0.0);
    assert!(dist.is_finite());
}

/// Test extremely large values
#[test]
fn test_very_large_values() {
    let v1 = Vector::new(vec![1e10, 1e20, 1e30]);
    let v2 = Vector::new(vec![2e10, 2e20, 2e30]);

    // Should handle large values without overflow (or overflow to Inf gracefully)
    let dist = v1.l2_distance(&v2);
    assert!(dist.is_ok());
    // Distance might be Inf due to overflow, which is acceptable
}

/// Test mixed positive and negative values
#[test]
fn test_mixed_signs() {
    let v1 = Vector::new(vec![-1.0, 2.0, -3.0, 4.0]);
    let v2 = Vector::new(vec![1.0, -2.0, 3.0, -4.0]);

    let dist = v1.l2_distance(&v2).unwrap();
    assert!(dist > 0.0);
    assert!(dist.is_finite());

    // Cosine distance should also work
    let cos_dist = v1.cosine_distance(&v2).unwrap();
    assert!(cos_dist.is_finite());
}

/// Test batch insert with some invalid vectors
#[test]
fn test_batch_insert_dimension_mismatch() {
    let mut store = VectorStore::new(128);

    let vectors = vec![
        Vector::new(vec![1.0; 128]), // Valid
        Vector::new(vec![2.0; 128]), // Valid
        Vector::new(vec![3.0; 64]),  // INVALID - wrong dimension
        Vector::new(vec![4.0; 128]), // Valid
    ];

    // Batch insert should fail if any vector has wrong dimension
    let result = store.batch_insert(vectors);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("dimension mismatch"));
}

/// Test empty batch insert
#[test]
fn test_batch_insert_empty() {
    let mut store = VectorStore::new(128);

    let empty_batch: Vec<Vector> = vec![];
    let result = store.batch_insert(empty_batch);

    // Empty batch should succeed and return empty ID list
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

/// Test single vector batch insert
#[test]
fn test_batch_insert_single() {
    let mut store = VectorStore::new(128);

    let single_batch = vec![Vector::new(vec![1.0; 128])];
    let result = store.batch_insert(single_batch);

    assert!(result.is_ok());
    let ids = result.unwrap();
    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0], 0);
}

/// Test all NaN vector
#[test]
fn test_all_nan_vector() {
    let all_nan = Vector::new(vec![f32::NAN; 128]);
    let normal = Vector::new(vec![1.0; 128]);

    let dist = all_nan.l2_distance(&normal);
    assert!(dist.is_ok());
    assert!(dist.unwrap().is_nan());
}

/// Test subnormal numbers (very small denormalized floats)
#[test]
fn test_subnormal_numbers() {
    let v1 = Vector::new(vec![f32::MIN_POSITIVE / 2.0; 128]);
    let v2 = Vector::new(vec![f32::MIN_POSITIVE; 128]);

    // Should handle subnormal numbers
    let dist = v1.l2_distance(&v2);
    assert!(dist.is_ok());
    assert!(dist.unwrap().is_finite());
}

/// Test vector normalization edge cases
#[test]
fn test_normalize_edge_cases() {
    // Small but reasonable vector
    let small = Vector::new(vec![1e-10; 128]);
    let normalized = small.normalize();

    // Should succeed and produce unit vector
    assert!(normalized.is_ok());
    let norm = normalized.unwrap();
    let magnitude = norm.l2_norm();
    assert!((magnitude - 1.0).abs() < 1e-6, "Normalized vector should have unit length");

    // Already normalized vector
    let unit = Vector::new(vec![1.0, 0.0, 0.0]);
    let normalized2 = unit.normalize().unwrap();
    assert!((normalized2.l2_norm() - 1.0).abs() < 1e-6);

    // Vector with mixed magnitudes
    let mixed = Vector::new(vec![3.0, 4.0, 0.0]);
    let normalized3 = mixed.normalize().unwrap();
    assert!((normalized3.l2_norm() - 1.0).abs() < 1e-6);
    // Check normalized values: 3/5, 4/5, 0
    assert!((normalized3.data[0] - 0.6).abs() < 1e-6);
    assert!((normalized3.data[1] - 0.8).abs() < 1e-6);
    assert_eq!(normalized3.data[2], 0.0);
}

/// Test distance calculation with identical vectors
#[test]
fn test_distance_identical_vectors() {
    let v1 = Vector::new(vec![1.0, 2.0, 3.0, 4.0]);
    let v2 = Vector::new(vec![1.0, 2.0, 3.0, 4.0]);

    // Distance should be exactly 0
    let dist = v1.l2_distance(&v2).unwrap();
    assert_eq!(dist, 0.0);

    // Cosine distance should be 0 (or very close)
    let cos_dist = v1.cosine_distance(&v2).unwrap();
    assert!(cos_dist.abs() < 1e-6);
}

/// Test l2_norm with edge cases
#[test]
fn test_l2_norm_edge_cases() {
    // Zero vector
    let zero = Vector::new(vec![0.0; 128]);
    assert_eq!(zero.l2_norm(), 0.0);

    // Single dimension
    let single = Vector::new(vec![3.0]);
    assert_eq!(single.l2_norm(), 3.0);

    // Unit vector
    let unit = Vector::new(vec![1.0, 0.0, 0.0, 0.0]);
    assert_eq!(unit.l2_norm(), 1.0);

    // Pythagorean triple
    let triple = Vector::new(vec![3.0, 4.0]);
    assert_eq!(triple.l2_norm(), 5.0);
}

/// Test dot product edge cases
#[test]
fn test_dot_product_edge_cases() {
    // Orthogonal vectors
    let v1 = Vector::new(vec![1.0, 0.0, 0.0]);
    let v2 = Vector::new(vec![0.0, 1.0, 0.0]);
    assert_eq!(v1.dot_product(&v2).unwrap(), 0.0);

    // Parallel vectors
    let v3 = Vector::new(vec![2.0, 0.0, 0.0]);
    let v4 = Vector::new(vec![3.0, 0.0, 0.0]);
    assert_eq!(v3.dot_product(&v4).unwrap(), 6.0);

    // Negative dot product
    let v5 = Vector::new(vec![1.0, 0.0]);
    let v6 = Vector::new(vec![-1.0, 0.0]);
    assert_eq!(v5.dot_product(&v6).unwrap(), -1.0);
}
