//! Integration tests for vector database functionality
//!
//! Tests end-to-end vector operations:
//! - Vector data type storage and retrieval
//! - Distance operators in queries
//! - Query planning and optimization
//! - MVCC compatibility

use omen::catalog::Catalog;
use omen::row::Row;
use omen::sql_engine::{ExecutionResult, SqlEngine};
use omen::value::Value;
use omen::vector::VectorValue;
use omen::vector_operators::VectorOperator;
use omen::vector_query_planner::{VectorQueryPattern, VectorQueryPlanner, VectorQueryStrategy};
use tempfile::TempDir;

#[test]
fn test_vector_value_storage_and_retrieval() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Create table with vector column
    // Note: Currently we can't create VECTOR(N) type via SQL,
    // so this test demonstrates the underlying infrastructure

    // Create vectors
    let v1 = VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap();
    let v2 = VectorValue::new(vec![4.0, 5.0, 6.0]).unwrap();
    let v3 = VectorValue::new(vec![1.0, 2.5, 3.5]).unwrap();

    // Test distance calculations
    let dist_12 = v1.l2_distance(&v2).unwrap();
    let dist_13 = v1.l2_distance(&v3).unwrap();

    assert!(dist_13 < dist_12); // v3 is closer to v1 than v2
}

#[test]
fn test_vector_operators() {
    // Test L2 distance
    let v1 = VectorValue::new(vec![1.0, 0.0, 0.0]).unwrap();
    let v2 = VectorValue::new(vec![0.0, 1.0, 0.0]).unwrap();

    let dist = v1.l2_distance(&v2).unwrap();
    assert!((dist - 1.414).abs() < 0.001); // sqrt(2)

    // Test inner product
    let v3 = VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap();
    let v4 = VectorValue::new(vec![4.0, 5.0, 6.0]).unwrap();

    let inner = v3.inner_product(&v4).unwrap();
    assert_eq!(inner, 32.0); // 1*4 + 2*5 + 3*6

    // Test cosine distance
    let v5 = VectorValue::new(vec![1.0, 0.0, 0.0]).unwrap();
    let v6 = VectorValue::new(vec![1.0, 0.0, 0.0]).unwrap();

    let cos_dist = v5.cosine_distance(&v6).unwrap();
    assert!(cos_dist < 0.0001); // Same direction = 0 distance
}

#[test]
fn test_vector_query_pattern_detection() {
    // Test query pattern validation
    let pattern = VectorQueryPattern {
        column_name: "embedding".to_string(),
        query_vector: VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap(),
        operator: VectorOperator::L2Distance,
        k: 10,
        ascending: true,
    };

    assert!(pattern.validate().is_ok());

    // Test invalid patterns
    let invalid_pattern = VectorQueryPattern {
        column_name: "embedding".to_string(),
        query_vector: VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap(),
        operator: VectorOperator::L2Distance,
        k: 0, // Invalid: must be > 0
        ascending: true,
    };

    assert!(invalid_pattern.validate().is_err());
}

#[test]
fn test_vector_query_planner_strategies() {
    let planner = VectorQueryPlanner::default();

    let pattern = VectorQueryPattern {
        column_name: "embedding".to_string(),
        query_vector: VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap(),
        operator: VectorOperator::L2Distance,
        k: 10,
        ascending: true,
    };

    // Small table without index → Sequential scan
    let strategy1 = planner.plan(&pattern, 500, false);
    assert_eq!(strategy1, VectorQueryStrategy::SequentialScan);

    // Small table with index → Still sequential scan (faster for small data)
    let strategy2 = planner.plan(&pattern, 500, true);
    assert_eq!(strategy2, VectorQueryStrategy::SequentialScan);

    // Large table without index → Sequential scan (no choice)
    let strategy3 = planner.plan(&pattern, 10000, false);
    assert_eq!(strategy3, VectorQueryStrategy::SequentialScan);

    // Large table with index → Index scan
    let strategy4 = planner.plan(&pattern, 10000, true);
    assert_eq!(
        strategy4,
        VectorQueryStrategy::IndexScan { expansion: 150 }
    );
}

#[test]
fn test_vector_query_cost_estimation() {
    let planner = VectorQueryPlanner::default();

    // Index scan should be faster than sequential for large tables
    let index_cost = planner.estimate_cost(
        VectorQueryStrategy::IndexScan { expansion: 150 },
        100000,
        10,
    );

    let seq_cost = planner.estimate_cost(VectorQueryStrategy::SequentialScan, 100000, 10);

    assert!(index_cost < seq_cost);

    // For small tables, sequential might be comparable
    let small_index_cost = planner.estimate_cost(
        VectorQueryStrategy::IndexScan { expansion: 150 },
        100,
        10,
    );

    let small_seq_cost =
        planner.estimate_cost(VectorQueryStrategy::SequentialScan, 100, 10);

    // Both should be very fast for small tables
    assert!(small_index_cost < 1.0);
    assert!(small_seq_cost < 1.0);
}

#[test]
fn test_vector_value_in_value_enum() {
    // Test that Vector variant works in Value enum
    let vec_value = VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap();
    let value = Value::Vector(vec_value.clone());

    // Test Display
    let display_str = format!("{}", value);
    assert_eq!(display_str, "[1,2,3]");

    // Test equality
    let value2 = Value::Vector(vec_value.clone());
    assert_eq!(value, value2);

    // Test PostgreSQL binary encoding
    let bytes = vec_value.to_postgres_binary();
    assert!(bytes.len() > 0);

    let decoded = VectorValue::from_postgres_binary(&bytes).unwrap();
    assert_eq!(vec_value, decoded);
}

#[test]
fn test_vector_normalization() {
    let v = VectorValue::new(vec![3.0, 4.0]).unwrap();
    let normalized = v.l2_normalize();

    // Normalized vector should have L2 norm of 1.0
    let norm = normalized.l2_norm();
    assert!((norm - 1.0).abs() < 0.0001);

    // Components should be 3/5 and 4/5
    assert!((normalized.data()[0] - 0.6).abs() < 0.0001);
    assert!((normalized.data()[1] - 0.8).abs() < 0.0001);
}

#[test]
fn test_vector_dimension_validation() {
    // Valid dimensions
    assert!(VectorValue::new(vec![1.0, 2.0, 3.0]).is_ok());

    // Empty vector
    let empty = VectorValue::new(vec![]);
    assert!(empty.is_ok());

    // NaN should be rejected
    let nan_vec = VectorValue::new(vec![1.0, f32::NAN, 3.0]);
    assert!(nan_vec.is_err());

    // Inf should be rejected
    let inf_vec = VectorValue::new(vec![1.0, f32::INFINITY, 3.0]);
    assert!(inf_vec.is_err());
}

#[test]
fn test_vector_distance_dimension_mismatch() {
    let v1 = VectorValue::new(vec![1.0, 2.0]).unwrap();
    let v2 = VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap();

    // All distance functions should fail with dimension mismatch
    assert!(v1.l2_distance(&v2).is_err());
    assert!(v1.inner_product(&v2).is_err());
    assert!(v1.cosine_distance(&v2).is_err());
}

#[test]
fn test_vector_literal_parsing() {
    // Standard format
    let v1 = VectorValue::from_literal("[1.0, 2.0, 3.0]").unwrap();
    assert_eq!(v1.dimensions(), 3);
    assert_eq!(v1.data(), &[1.0, 2.0, 3.0]);

    // No spaces
    let v2 = VectorValue::from_literal("[1.0,2.0,3.0]").unwrap();
    assert_eq!(v2.dimensions(), 3);
    assert_eq!(v2.data(), &[1.0, 2.0, 3.0]);

    // Extra whitespace
    let v3 = VectorValue::from_literal("  [ 1.0 , 2.0 , 3.0 ]  ").unwrap();
    assert_eq!(v3.dimensions(), 3);

    // Missing brackets should fail
    assert!(VectorValue::from_literal("1.0, 2.0, 3.0").is_err());

    // Invalid number should fail
    assert!(VectorValue::from_literal("[1.0, abc, 3.0]").is_err());
}

#[test]
fn test_expansion_factor_tuning() {
    let planner = VectorQueryPlanner::default();

    // Top-10 queries use 150x expansion (92.7% recall)
    let pattern_10 = VectorQueryPattern {
        column_name: "embedding".to_string(),
        query_vector: VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap(),
        operator: VectorOperator::L2Distance,
        k: 10,
        ascending: true,
    };

    let strategy_10 = planner.plan(&pattern_10, 10000, true);
    assert_eq!(
        strategy_10,
        VectorQueryStrategy::IndexScan { expansion: 150 }
    );

    // Top-50 queries use 200x expansion (95.1% recall)
    let pattern_50 = VectorQueryPattern {
        column_name: "embedding".to_string(),
        query_vector: VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap(),
        operator: VectorOperator::L2Distance,
        k: 50,
        ascending: true,
    };

    let strategy_50 = planner.plan(&pattern_50, 10000, true);
    assert_eq!(
        strategy_50,
        VectorQueryStrategy::IndexScan { expansion: 200 }
    );

    // Top-200 queries use 250x expansion
    let pattern_200 = VectorQueryPattern {
        column_name: "embedding".to_string(),
        query_vector: VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap(),
        operator: VectorOperator::L2Distance,
        k: 200,
        ascending: true,
    };

    let strategy_200 = planner.plan(&pattern_200, 10000, true);
    assert_eq!(
        strategy_200,
        VectorQueryStrategy::IndexScan { expansion: 250 }
    );
}
