//! Concurrency & Thread Safety Tests
//!
//! Validates that the system handles concurrent operations correctly:
//! - Parallel vector insertions
//! - Concurrent searches
//! - Mixed read/write workloads
//! - Thread safety of public APIs
//! - No data races or deadlocks

use omendb::vector::types::Vector;
use omendb::vector::store::VectorStore;
use std::sync::{Arc, Mutex};
use std::thread;

/// Test parallel vector insertions
#[test]
fn test_parallel_insertions() {
    let dimensions = 128;
    let store = Arc::new(Mutex::new(VectorStore::new(dimensions)));
    let num_threads = 8;
    let vectors_per_thread = 100;

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let store_clone = Arc::clone(&store);
            thread::spawn(move || {
                for i in 0..vectors_per_thread {
                    let value = (thread_id * vectors_per_thread + i) as f32;
                    let vec = Vector::new(vec![value; dimensions]);
                    let mut store = store_clone.lock().unwrap();
                    store.insert(vec).unwrap();
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all vectors were inserted
    let store = store.lock().unwrap();
    let expected_count = num_threads * vectors_per_thread;
    assert_eq!(
        store.len(),
        expected_count,
        "Should have inserted {} vectors",
        expected_count
    );
}

/// Test concurrent searches (read-only workload)
#[test]
fn test_concurrent_searches() {
    use rand::Rng;

    let dimensions = 128;
    let mut store = VectorStore::new(dimensions);

    // Insert test data
    let num_vectors = 1000;
    for i in 0..num_vectors {
        let vec = Vector::new(vec![i as f32; dimensions]);
        store.insert(vec).unwrap();
    }

    // Share store across threads (read-only)
    let store = Arc::new(Mutex::new(store));
    let num_threads = 8;
    let queries_per_thread = 50;

    let handles: Vec<_> = (0..num_threads)
        .map(|_| {
            let store_clone = Arc::clone(&store);
            thread::spawn(move || {
                let mut local_rng = rand::thread_rng();
                for _ in 0..queries_per_thread {
                    let query_val = local_rng.gen_range(0.0..num_vectors as f32);
                    let query = Vector::new(vec![query_val; dimensions]);
                    let mut store = store_clone.lock().unwrap();
                    let results = store.knn_search(&query, 10).unwrap();
                    assert!(!results.is_empty(), "Should return results");
                    assert!(results.len() <= 10, "Should return at most k results");
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    eprintln!(
        "✓ {} concurrent searches completed successfully",
        num_threads * queries_per_thread
    );
}

/// Test mixed read/write workload
#[test]
fn test_mixed_read_write() {
    let dimensions = 128;
    let store = Arc::new(Mutex::new(VectorStore::new(dimensions)));

    // Pre-populate with some data
    {
        let mut store = store.lock().unwrap();
        for i in 0..500 {
            let vec = Vector::new(vec![i as f32; dimensions]);
            store.insert(vec).unwrap();
        }
    }

    let num_threads = 4;
    let operations_per_thread = 100;

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let store_clone = Arc::clone(&store);
            thread::spawn(move || {
                let mut local_rng = rand::thread_rng();
                for i in 0..operations_per_thread {
                    let mut store = store_clone.lock().unwrap();

                    if i % 2 == 0 {
                        // Insert operation
                        let value = (thread_id * 1000 + i) as f32;
                        let vec = Vector::new(vec![value; dimensions]);
                        store.insert(vec).unwrap();
                    } else {
                        // Search operation
                        use rand::Rng;
                        let query_val = local_rng.gen_range(0.0..1000.0);
                        let query = Vector::new(vec![query_val; dimensions]);
                        let _results = store.knn_search(&query, 5).unwrap();
                    }
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify final state
    let store = store.lock().unwrap();
    let expected_min = 500 + (num_threads * operations_per_thread / 2);
    assert!(
        store.len() >= expected_min,
        "Should have at least {} vectors after mixed workload",
        expected_min
    );

    eprintln!("✓ Mixed read/write workload completed (final size: {})", store.len());
}

/// Test batch insert thread safety
#[test]
fn test_parallel_batch_inserts() {
    let dimensions = 128;
    let store = Arc::new(Mutex::new(VectorStore::new(dimensions)));
    let num_threads = 4;
    let batch_size = 100;

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let store_clone = Arc::clone(&store);
            thread::spawn(move || {
                let batch: Vec<Vector> = (0..batch_size)
                    .map(|i| {
                        let value = (thread_id * batch_size + i) as f32;
                        Vector::new(vec![value; dimensions])
                    })
                    .collect();

                let mut store = store_clone.lock().unwrap();
                let ids = store.batch_insert(batch).unwrap();
                assert_eq!(ids.len(), batch_size, "Should insert all vectors in batch");
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all batches were inserted
    let store = store.lock().unwrap();
    let expected_count = num_threads * batch_size;
    assert_eq!(
        store.len(),
        expected_count,
        "Should have inserted {} vectors",
        expected_count
    );
}

/// Test concurrent searches with HNSW index
#[test]
fn test_concurrent_hnsw_searches() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let dimensions = 128;
    let mut store = VectorStore::new(dimensions);

    // Build HNSW index with enough vectors
    let num_vectors = 5000;
    let vectors: Vec<Vector> = (0..num_vectors)
        .map(|_| {
            let data: Vec<f32> = (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            Vector::new(data)
        })
        .collect();

    store.batch_insert(vectors).unwrap();

    // Share store across threads for concurrent HNSW searches
    let store = Arc::new(Mutex::new(store));
    let num_threads = 8;
    let queries_per_thread = 50;

    let handles: Vec<_> = (0..num_threads)
        .map(|_| {
            let store_clone = Arc::clone(&store);
            thread::spawn(move || {
                let mut local_rng = rand::thread_rng();
                for _ in 0..queries_per_thread {
                    let query_data: Vec<f32> = (0..dimensions)
                        .map(|_| local_rng.gen_range(-1.0..1.0))
                        .collect();
                    let query = Vector::new(query_data);

                    let mut store = store_clone.lock().unwrap();
                    let results = store.knn_search(&query, 10).unwrap();

                    // Verify results are valid
                    assert!(!results.is_empty(), "HNSW should return results");
                    assert!(results.len() <= 10, "Should return at most k results");

                    // Verify results are sorted by distance
                    for i in 1..results.len() {
                        assert!(
                            results[i].1 >= results[i - 1].1,
                            "Results should be sorted by distance"
                        );
                    }
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    eprintln!(
        "✓ {} concurrent HNSW searches completed successfully",
        num_threads * queries_per_thread
    );
}

/// Test thread safety of vector operations
#[test]
fn test_vector_operations_thread_safety() {
    let dimensions = 128;
    let num_threads = 8;
    let operations_per_thread = 100;

    let handles: Vec<_> = (0..num_threads)
        .map(|_| {
            thread::spawn(move || {
                let mut local_rng = rand::thread_rng();
                for _ in 0..operations_per_thread {
                    use rand::Rng;
                    let data: Vec<f32> = (0..dimensions)
                        .map(|_| local_rng.gen_range(-1.0..1.0))
                        .collect();
                    let v1 = Vector::new(data.clone());

                    let data2: Vec<f32> = (0..dimensions)
                        .map(|_| local_rng.gen_range(-1.0..1.0))
                        .collect();
                    let v2 = Vector::new(data2);

                    // Perform various vector operations
                    let _dist = v1.l2_distance(&v2).unwrap();
                    let _cos = v1.cosine_distance(&v2).unwrap();
                    let _dot = v1.dot_product(&v2).unwrap();
                    let _norm = v1.l2_norm();

                    // Normalize if possible
                    if v1.l2_norm() > 0.0 {
                        let _normalized = v1.normalize().unwrap();
                    }
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    eprintln!(
        "✓ {} vector operations completed across {} threads",
        num_threads * operations_per_thread,
        num_threads
    );
}

/// Test no data corruption under concurrent access
#[test]
fn test_no_data_corruption() {
    let dimensions = 128;
    let store = Arc::new(Mutex::new(VectorStore::new(dimensions)));
    let num_threads = 4;
    let vectors_per_thread = 50;

    // Each thread inserts vectors with a unique pattern
    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let store_clone = Arc::clone(&store);
            thread::spawn(move || {
                for _ in 0..vectors_per_thread {
                    // Use thread_id as the value pattern
                    let vec = Vector::new(vec![thread_id as f32; dimensions]);
                    let mut store = store_clone.lock().unwrap();
                    let id = store.insert(vec).unwrap();
                    // Verify we can retrieve it immediately
                    let retrieved = store.get(id).unwrap();
                    assert_eq!(
                        retrieved.data[0], thread_id as f32,
                        "Retrieved vector should match inserted data"
                    );
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify no corruption: check all vectors
    let store = store.lock().unwrap();
    for i in 0..store.len() {
        let vec = store.get(i).unwrap();
        // Each vector should have all dimensions set to the same value
        let first_val = vec.data[0];
        assert!(
            vec.data.iter().all(|&x| x == first_val),
            "Vector {} should have consistent values (thread pattern)",
            i
        );
    }

    eprintln!("✓ No data corruption detected after concurrent insertions");
}

/// Test that concurrent operations don't cause panics
#[test]
fn test_no_panics_under_concurrency() {
    let dimensions = 128;
    let store = Arc::new(Mutex::new(VectorStore::new(dimensions)));

    // Pre-populate
    {
        let mut store = store.lock().unwrap();
        for i in 0..100 {
            let vec = Vector::new(vec![i as f32; dimensions]);
            store.insert(vec).unwrap();
        }
    }

    let num_threads = 16; // High contention
    let operations_per_thread = 50;

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let store_clone = Arc::clone(&store);
            thread::spawn(move || {
                let mut local_rng = rand::thread_rng();
                for i in 0..operations_per_thread {
                    let mut store = store_clone.lock().unwrap();

                    match i % 4 {
                        0 => {
                            // Insert
                            let vec = Vector::new(vec![thread_id as f32; dimensions]);
                            let _ = store.insert(vec);
                        }
                        1 => {
                            // Search
                            use rand::Rng;
                            let query = Vector::new(vec![local_rng.gen_range(0.0..100.0); dimensions]);
                            let _ = store.knn_search(&query, 5);
                        }
                        2 => {
                            // Get
                            use rand::Rng;
                            let id = local_rng.gen_range(0..100.min(store.len()));
                            let _ = store.get(id);
                        }
                        _ => {
                            // Len/is_empty
                            let _ = store.len();
                            let _ = store.is_empty();
                        }
                    }
                }
            })
        })
        .collect();

    // Wait for all threads - will panic if any thread panicked
    for handle in handles {
        handle.join().unwrap();
    }

    eprintln!("✓ No panics under high concurrency ({} threads)", num_threads);
}

/// Test that get() is thread-safe for concurrent reads
#[test]
fn test_concurrent_get_operations() {
    let dimensions = 128;
    let store = Arc::new(Mutex::new(VectorStore::new(dimensions)));

    // Pre-populate with data
    {
        let mut store = store.lock().unwrap();
        for i in 0..500 {
            let vec = Vector::new(vec![i as f32; dimensions]);
            store.insert(vec).unwrap();
        }
    }

    let num_threads = 8;
    let gets_per_thread = 100;

    let handles: Vec<_> = (0..num_threads)
        .map(|_| {
            let store_clone = Arc::clone(&store);
            thread::spawn(move || {
                let mut local_rng = rand::thread_rng();
                for _ in 0..gets_per_thread {
                    use rand::Rng;
                    let id = local_rng.gen_range(0..500);
                    let store = store_clone.lock().unwrap();
                    let vec = store.get(id);
                    assert!(vec.is_some(), "Should find vector with id {}", id);
                    // Verify data integrity
                    if let Some(v) = vec {
                        assert_eq!(v.data[0], id as f32, "Data should match id");
                    }
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    eprintln!(
        "✓ {} concurrent get() operations completed successfully",
        num_threads * gets_per_thread
    );
}
