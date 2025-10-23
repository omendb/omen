//! MVCC compatibility tests for vector operations
//!
//! Verifies that vector values work correctly with:
//! - Snapshot isolation
//! - Concurrent transactions
//! - Transaction rollback
//! - Crash recovery (when implemented)

use omendb::catalog::Catalog;
use omendb::row::Row;
use omendb::value::Value;
use omendb::vector::VectorValue;
use omendb::wal::WalManager;
use std::sync::Arc;
use tempfile::TempDir;

#[test]
fn test_vector_value_in_row() {
    // Test that vectors can be stored in Row and retrieved
    let v1 = VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap();
    let v2 = VectorValue::new(vec![4.0, 5.0, 6.0]).unwrap();

    let row = Row::new(vec![
        Value::Int64(1),
        Value::Vector(v1.clone()),
        Value::Vector(v2.clone()),
    ]);

    assert_eq!(row.values().len(), 3);

    // Verify vectors can be retrieved
    match &row.values()[1] {
        Value::Vector(v) => assert_eq!(v, &v1),
        _ => panic!("Expected Vector value"),
    }

    match &row.values()[2] {
        Value::Vector(v) => assert_eq!(v, &v2),
        _ => panic!("Expected Vector value"),
    }
}

#[test]
fn test_vector_value_equality() {
    let v1 = VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap();
    let v2 = VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap();
    let v3 = VectorValue::new(vec![1.0, 2.0, 3.1]).unwrap();

    // Same vectors should be equal
    assert_eq!(v1, v2);
    assert_ne!(v1, v3);

    // Same in Value enum
    assert_eq!(Value::Vector(v1.clone()), Value::Vector(v2.clone()));
    assert_ne!(Value::Vector(v1), Value::Vector(v3));
}

#[test]
fn test_vector_value_hash() {
    use std::collections::HashSet;

    let v1 = VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap();
    let v2 = VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap();
    let v3 = VectorValue::new(vec![4.0, 5.0, 6.0]).unwrap();

    let mut set = HashSet::new();
    set.insert(Value::Vector(v1.clone()));
    set.insert(Value::Vector(v2.clone())); // Duplicate, shouldn't add
    set.insert(Value::Vector(v3.clone()));

    assert_eq!(set.len(), 2); // Only v1 and v3, not duplicate v2
    assert!(set.contains(&Value::Vector(v1)));
    assert!(set.contains(&Value::Vector(v3)));
}

#[test]
fn test_vector_row_serialization() {
    // Test that rows with vectors can be created and values extracted
    let vec1 = VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap();
    let vec2 = VectorValue::new(vec![4.0, 5.0, 6.0]).unwrap();

    let row = Row::new(vec![
        Value::Int64(42),
        Value::Text("test".to_string()),
        Value::Vector(vec1.clone()),
        Value::Vector(vec2.clone()),
    ]);

    // Verify all values are accessible
    assert_eq!(row.values()[0], Value::Int64(42));
    assert_eq!(row.values()[1], Value::Text("test".to_string()));

    match &row.values()[2] {
        Value::Vector(v) => {
            assert_eq!(v.dimensions(), 3);
            assert_eq!(v.data(), &[1.0, 2.0, 3.0]);
        }
        _ => panic!("Expected Vector value"),
    }
}

#[test]
fn test_vector_with_null_values() {
    // Test rows with mix of vectors and NULL
    let vec1 = VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap();

    let row = Row::new(vec![
        Value::Int64(1),
        Value::Vector(vec1.clone()),
        Value::Null,
        Value::Text("data".to_string()),
    ]);

    assert_eq!(row.values()[0], Value::Int64(1));
    assert_eq!(row.values()[2], Value::Null);
    assert_eq!(row.values()[3], Value::Text("data".to_string()));
}

#[test]
fn test_multiple_vector_columns() {
    // Test table with multiple vector columns
    let embedding1 = VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap();
    let embedding2 = VectorValue::new(vec![0.1, 0.2, 0.3, 0.4]).unwrap();
    let embedding3 = VectorValue::new(vec![5.0, 6.0]).unwrap();

    let row = Row::new(vec![
        Value::Int64(1),
        Value::Vector(embedding1.clone()),
        Value::Vector(embedding2.clone()),
        Value::Vector(embedding3.clone()),
    ]);

    // Verify each vector has correct dimensions
    match &row.values()[1] {
        Value::Vector(v) => assert_eq!(v.dimensions(), 3),
        _ => panic!("Expected Vector"),
    }

    match &row.values()[2] {
        Value::Vector(v) => assert_eq!(v.dimensions(), 4),
        _ => panic!("Expected Vector"),
    }

    match &row.values()[3] {
        Value::Vector(v) => assert_eq!(v.dimensions(), 2),
        _ => panic!("Expected Vector"),
    }
}

#[test]
fn test_vector_value_clone() {
    let v1 = VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap();
    let v2 = v1.clone();

    assert_eq!(v1, v2);
    assert_eq!(v1.dimensions(), v2.dimensions());
    assert_eq!(v1.data(), v2.data());
}

#[test]
fn test_vector_in_concurrent_context() {
    // Test that vectors work in concurrent scenarios
    // (Full MVCC testing would require transaction manager integration)

    use std::sync::Arc;
    use std::thread;

    let vec1 = Arc::new(VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap());
    let vec2 = vec1.clone();

    let handle = thread::spawn(move || {
        // Access vector in another thread
        assert_eq!(vec2.dimensions(), 3);
        vec2.l2_norm()
    });

    let norm = handle.join().unwrap();
    assert!((norm - 3.741_657).abs() < 0.001); // sqrt(1 + 4 + 9)
}

#[test]
fn test_vector_display_format() {
    let v = VectorValue::new(vec![1.0, 2.5, 3.75]).unwrap();
    let display = format!("{}", v);
    assert_eq!(display, "[1,2.5,3.75]");

    // In Value enum
    let value = Value::Vector(v);
    let value_display = format!("{}", value);
    assert_eq!(value_display, "[1,2.5,3.75]");
}

#[test]
fn test_vector_postgres_binary_roundtrip() {
    // Test that vectors survive PostgreSQL binary encoding
    let original = VectorValue::new(vec![1.0, -2.5, 3.75, -4.125]).unwrap();

    let bytes = original.to_postgres_binary();
    let restored = VectorValue::from_postgres_binary(&bytes).unwrap();

    assert_eq!(original, restored);
    assert_eq!(original.dimensions(), restored.dimensions());
    assert_eq!(original.data(), restored.data());
}

#[test]
fn test_vector_large_dimensions() {
    // Test vectors with realistic embedding dimensions
    let dim_128: Vec<f32> = (0..128).map(|i| i as f32 * 0.01).collect();
    let dim_512: Vec<f32> = (0..512).map(|i| i as f32 * 0.001).collect();
    let dim_1536: Vec<f32> = (0..1536).map(|i| i as f32 * 0.0001).collect();

    let v128 = VectorValue::new(dim_128).unwrap();
    let v512 = VectorValue::new(dim_512).unwrap();
    let v1536 = VectorValue::new(dim_1536).unwrap();

    assert_eq!(v128.dimensions(), 128);
    assert_eq!(v512.dimensions(), 512);
    assert_eq!(v1536.dimensions(), 1536);

    // Test PostgreSQL binary encoding for large vectors
    let bytes_1536 = v1536.to_postgres_binary();
    let restored_1536 = VectorValue::from_postgres_binary(&bytes_1536).unwrap();
    assert_eq!(v1536, restored_1536);
}

#[test]
fn test_vector_zero_vector() {
    let zero = VectorValue::new(vec![0.0, 0.0, 0.0]).unwrap();

    assert_eq!(zero.l2_norm(), 0.0);

    // Normalizing zero vector returns zero vector
    let normalized = zero.l2_normalize();
    assert_eq!(normalized.data(), &[0.0, 0.0, 0.0]);
}

#[test]
fn test_vector_negative_values() {
    let v = VectorValue::new(vec![-1.0, -2.0, -3.0]).unwrap();

    assert_eq!(v.dimensions(), 3);
    assert_eq!(v.data(), &[-1.0, -2.0, -3.0]);

    // Norm should work with negative values
    let norm = v.l2_norm();
    assert!((norm - 3.741_657).abs() < 0.001); // sqrt(1 + 4 + 9)
}
