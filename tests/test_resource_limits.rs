//! Resource Exhaustion & Limit Testing
//!
//! Validates graceful handling of resource limits:
//! - Large batch operations
//! - Memory pressure scenarios
//! - Boundary conditions at scale
//! - Graceful degradation under stress

use omen::vector::types::Vector;
use omen::vector::store::VectorStore;

/// Test inserting very large batches
#[test]
fn test_large_batch_insert() {
    let dimensions = 128;
    let mut store = VectorStore::new(dimensions);

    // Insert a large batch (10K vectors)
    let batch_size = 10_000;
    let large_batch: Vec<Vector> = (0..batch_size)
        .map(|i| Vector::new(vec![i as f32; dimensions]))
        .collect();

    let result = store.batch_insert(large_batch);
    assert!(result.is_ok(), "Large batch insert should succeed");

    let ids = result.unwrap();
    assert_eq!(ids.len(), batch_size, "Should insert all vectors");
    assert_eq!(store.len(), batch_size, "Store should contain all vectors");

    eprintln!("✓ Large batch insert succeeded: {} vectors", batch_size);
}

/// Test many small inserts (stress test insert path)
#[test]
fn test_many_small_inserts() {
    let dimensions = 128;
    let mut store = VectorStore::new(dimensions);

    let num_inserts = 5000;
    for i in 0..num_inserts {
        let vec = Vector::new(vec![i as f32; dimensions]);
        let result = store.insert(vec);
        assert!(result.is_ok(), "Insert {} should succeed", i);
    }

    assert_eq!(store.len(), num_inserts);
    eprintln!("✓ {} small inserts completed successfully", num_inserts);
}

/// Test search on large dataset
#[test]
fn test_search_on_large_dataset() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let dimensions = 128;
    let mut store = VectorStore::new(dimensions);

    // Build large dataset
    let dataset_size = 20_000;
    let vectors: Vec<Vector> = (0..dataset_size)
        .map(|_| {
            let data: Vec<f32> = (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            Vector::new(data)
        })
        .collect();

    store.batch_insert(vectors).unwrap();

    // Test search
    let query_data: Vec<f32> = (0..dimensions)
        .map(|_| rng.gen_range(-1.0..1.0))
        .collect();
    let query = Vector::new(query_data);

    let result = store.knn_search(&query, 100);
    assert!(result.is_ok(), "Search on large dataset should succeed");

    let results = result.unwrap();
    assert!(!results.is_empty(), "Should return results");
    assert!(results.len() <= 100, "Should respect k limit");

    eprintln!(
        "✓ Search on {} vectors succeeded, returned {} results",
        dataset_size,
        results.len()
    );
}

/// Test high-dimensional vectors (stress dimension handling)
#[test]
fn test_very_high_dimensions() {
    let dimensions = 4096; // Very high dimensional
    let mut store = VectorStore::new(dimensions);

    // Insert a few high-dimensional vectors
    for i in 0..100 {
        let vec = Vector::new(vec![i as f32; dimensions]);
        let result = store.insert(vec);
        assert!(result.is_ok(), "High-dimensional insert should succeed");
    }

    assert_eq!(store.len(), 100);

    // Test search
    let query = Vector::new(vec![50.0; dimensions]);
    let result = store.knn_search(&query, 10);
    assert!(result.is_ok(), "High-dimensional search should succeed");

    eprintln!("✓ High-dimensional vectors ({}D) handled successfully", dimensions);
}

/// Test empty batch operations
#[test]
fn test_empty_operations() {
    let dimensions = 128;
    let mut store = VectorStore::new(dimensions);

    // Empty batch insert
    let empty_batch: Vec<Vector> = vec![];
    let result = store.batch_insert(empty_batch);
    assert!(result.is_ok(), "Empty batch should succeed");
    assert!(result.unwrap().is_empty(), "Should return empty ID list");

    // Search on empty store
    let query = Vector::new(vec![1.0; dimensions]);
    let results = store.knn_search(&query, 10).unwrap();
    assert!(results.is_empty(), "Search on empty store should return empty");

    eprintln!("✓ Empty operations handled gracefully");
}

/// Test dimension boundaries
#[test]
fn test_dimension_boundaries() {
    // Very small dimension
    let small_dim = 2;
    let mut small_store = VectorStore::new(small_dim);
    let small_vec = Vector::new(vec![1.0, 2.0]);
    assert!(small_store.insert(small_vec).is_ok());

    // Reasonable dimension
    let normal_dim = 512;
    let mut normal_store = VectorStore::new(normal_dim);
    let normal_vec = Vector::new(vec![1.0; normal_dim]);
    assert!(normal_store.insert(normal_vec).is_ok());

    // Large dimension
    let large_dim = 2048;
    let mut large_store = VectorStore::new(large_dim);
    let large_vec = Vector::new(vec![1.0; large_dim]);
    assert!(large_store.insert(large_vec).is_ok());

    eprintln!("✓ Dimension boundaries handled: {}D, {}D, {}D", small_dim, normal_dim, large_dim);
}

/// Test k parameter boundaries in search
#[test]
fn test_k_boundaries() {
    let dimensions = 128;
    let mut store = VectorStore::new(dimensions);

    // Insert test data
    for i in 0..100 {
        let vec = Vector::new(vec![i as f32; dimensions]);
        store.insert(vec).unwrap();
    }

    let query = Vector::new(vec![50.0; dimensions]);

    // k = 0 (edge case)
    let results_0 = store.knn_search(&query, 0).unwrap();
    assert!(results_0.is_empty(), "k=0 should return empty results");

    // k = 1 (minimum useful)
    let results_1 = store.knn_search(&query, 1).unwrap();
    assert_eq!(results_1.len(), 1, "k=1 should return 1 result");

    // k = dataset size (exact match)
    let results_100 = store.knn_search(&query, 100).unwrap();
    assert_eq!(results_100.len(), 100, "k=100 should return all 100 vectors");

    // k > dataset size (should return all available)
    let results_large = store.knn_search(&query, 1000).unwrap();
    assert_eq!(results_large.len(), 100, "k>size should return all available");

    eprintln!("✓ k parameter boundaries handled correctly");
}

/// Test memory usage reporting
#[test]
fn test_memory_reporting() {
    let dimensions = 128;
    let mut store = VectorStore::new(dimensions);

    let initial_memory = store.memory_usage();
    eprintln!("Initial memory usage: {} bytes", initial_memory);

    // Insert vectors
    for i in 0..1000 {
        let vec = Vector::new(vec![i as f32; dimensions]);
        store.insert(vec).unwrap();
    }

    let after_insert = store.memory_usage();
    eprintln!("Memory after 1000 inserts: {} bytes", after_insert);

    // Memory should increase
    assert!(
        after_insert > initial_memory,
        "Memory should increase after inserts"
    );

    // Check bytes per vector calculation
    let bytes_per_vector = store.bytes_per_vector();
    eprintln!("Bytes per vector: {:.2}", bytes_per_vector);

    // Should be reasonable (dimensions * 4 bytes per f32 + overhead)
    let expected_min = (dimensions * 4) as f32;
    assert!(
        bytes_per_vector >= expected_min,
        "Bytes per vector should be at least dimension * 4"
    );

    eprintln!("✓ Memory reporting working correctly");
}

/// Test graceful handling of repeated identical vectors
#[test]
fn test_duplicate_vectors() {
    let dimensions = 128;
    let mut store = VectorStore::new(dimensions);

    // Insert same vector 100 times
    let duplicate_vec = Vector::new(vec![42.0; dimensions]);
    for _ in 0..100 {
        let result = store.insert(duplicate_vec.clone());
        assert!(result.is_ok(), "Duplicate insert should succeed");
    }

    assert_eq!(store.len(), 100, "Should store all duplicates");

    // Search should still work
    let query = Vector::new(vec![42.0; dimensions]);
    let results = store.knn_search(&query, 10).unwrap();
    assert_eq!(results.len(), 10, "Should return requested k results");

    // All results should have distance 0 (exact matches)
    for (_, dist) in &results {
        assert_eq!(*dist, 0.0, "Duplicate vectors should have distance 0");
    }

    eprintln!("✓ Duplicate vectors handled correctly");
}

/// Test batch operations with mixed sizes
#[test]
fn test_mixed_batch_sizes() {
    let dimensions = 128;
    let mut store = VectorStore::new(dimensions);

    // Small batch
    let small_batch: Vec<Vector> = (0..10)
        .map(|i| Vector::new(vec![i as f32; dimensions]))
        .collect();
    store.batch_insert(small_batch).unwrap();

    // Medium batch
    let medium_batch: Vec<Vector> = (10..100)
        .map(|i| Vector::new(vec![i as f32; dimensions]))
        .collect();
    store.batch_insert(medium_batch).unwrap();

    // Large batch
    let large_batch: Vec<Vector> = (100..1000)
        .map(|i| Vector::new(vec![i as f32; dimensions]))
        .collect();
    store.batch_insert(large_batch).unwrap();

    assert_eq!(store.len(), 1000, "Should insert all batches");

    eprintln!("✓ Mixed batch sizes handled successfully");
}

/// Test ef_search parameter boundaries
#[test]
fn test_ef_search_boundaries() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let dimensions = 128;
    let mut store = VectorStore::new(dimensions);

    // Build index with enough vectors
    let vectors: Vec<Vector> = (0..5000)
        .map(|_| {
            let data: Vec<f32> = (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            Vector::new(data)
        })
        .collect();

    store.batch_insert(vectors).unwrap();

    let query_data: Vec<f32> = (0..dimensions)
        .map(|_| rng.gen_range(-1.0..1.0))
        .collect();
    let query = Vector::new(query_data);

    // Test different ef_search values
    for ef in [10, 50, 100, 200] {
        store.set_ef_search(ef);
        let result = store.knn_search(&query, 10);
        assert!(
            result.is_ok(),
            "Search with ef_search={} should succeed",
            ef
        );
    }

    eprintln!("✓ ef_search parameter boundaries handled");
}

/// Test store operations after HNSW index is built
#[test]
fn test_operations_after_index_build() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let dimensions = 128;
    let mut store = VectorStore::new(dimensions);

    // Build HNSW index (needs >1000 vectors)
    let initial_vectors: Vec<Vector> = (0..2000)
        .map(|_| {
            let data: Vec<f32> = (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            Vector::new(data)
        })
        .collect();

    store.batch_insert(initial_vectors).unwrap();

    // Add more vectors after index is built
    for _ in 0..100 {
        let data: Vec<f32> = (0..dimensions)
            .map(|_| rng.gen_range(-1.0..1.0))
            .collect();
        let vec = Vector::new(data);
        let result = store.insert(vec);
        assert!(result.is_ok(), "Insert after index build should succeed");
    }

    assert_eq!(store.len(), 2100, "Should have all vectors");

    // Search should still work
    let query_data: Vec<f32> = (0..dimensions)
        .map(|_| rng.gen_range(-1.0..1.0))
        .collect();
    let query = Vector::new(query_data);

    let results = store.knn_search(&query, 10);
    assert!(results.is_ok(), "Search after more inserts should work");

    eprintln!("✓ Operations after HNSW index build work correctly");
}
