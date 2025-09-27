//! Stress tests for production validation
//! Target: 50M+ keys, concurrent access, long-running stability

use crate::index::RecursiveModelIndex;
use crate::storage::ArrowStorage;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

#[test]
#[ignore] // Run with: cargo test --ignored
fn test_50m_keys_scale() {
    println!("🚀 Testing with 50M keys - this will take several minutes...");

    let mut index = RecursiveModelIndex::new(50_000_000);
    let mut data = Vec::with_capacity(50_000_000);

    // Generate 50M keys
    let base = 1_600_000_000_000_000i64;
    for i in 0..50_000_000 {
        data.push((base + i as i64 * 10, i));
    }

    // Train index
    let train_start = Instant::now();
    index.train(data.clone());
    let train_time = train_start.elapsed();

    println!("Training 50M keys took: {:?}", train_time);
    assert!(train_time < Duration::from_secs(60), "Training too slow");

    // Test lookup performance
    let mut total_lookup_time = Duration::ZERO;
    let sample_size = 10_000;

    for i in (0..50_000_000).step_by(5000) {
        let key = base + i as i64 * 10;
        let start = Instant::now();
        let result = index.search(key);
        total_lookup_time += start.elapsed();

        assert_eq!(result, Some(i), "Failed to find key at position {}", i);
    }

    let avg_lookup_ns = total_lookup_time.as_nanos() / sample_size;
    println!("Average lookup time at 50M scale: {} ns", avg_lookup_ns);

    // Should maintain <100ns even at 50M scale
    assert!(avg_lookup_ns < 100, "Lookup performance degraded at scale");
}

#[test]
#[ignore]
fn test_concurrent_reads() {
    let index = Arc::new(RwLock::new(RecursiveModelIndex::new(1_000_000)));

    // Train index
    let data: Vec<(i64, usize)> = (0..1_000_000)
        .map(|i| (i as i64 * 10, i))
        .collect();

    index.write().unwrap().train(data);

    // Spawn 100 concurrent readers
    let mut handles = vec![];

    for thread_id in 0..100 {
        let index_clone = Arc::clone(&index);

        let handle = thread::spawn(move || {
            let mut found = 0;

            // Each thread does 1000 lookups
            for i in 0..1000 {
                let key = (thread_id * 10000 + i) as i64 * 10;
                if let Ok(index) = index_clone.read() {
                    if index.search(key).is_some() {
                        found += 1;
                    }
                }
            }

            found
        });

        handles.push(handle);
    }

    // Wait for all threads
    let start = Instant::now();
    let mut total_found = 0;

    for handle in handles {
        total_found += handle.join().unwrap();
    }

    let elapsed = start.elapsed();

    println!("100 concurrent threads, 100K total lookups took: {:?}", elapsed);
    println!("Total successful lookups: {}", total_found);

    // Should complete within reasonable time
    assert!(elapsed < Duration::from_secs(5), "Concurrent reads too slow");
    assert!(total_found > 90_000, "Too many failed lookups under concurrency");
}

#[test]
#[ignore]
fn test_memory_usage() {
    // Monitor memory usage at different scales
    let scales = [100_000, 1_000_000, 10_000_000];

    for size in scales {
        let mut index = RecursiveModelIndex::new(size);
        let data: Vec<(i64, usize)> = (0..size)
            .map(|i| (i as i64, i))
            .collect();

        index.train(data);

        // In production, would use actual memory profiling
        // For now, just ensure it doesn't crash
        println!("Successfully handled {} keys", size);

        // Do some operations to ensure it's actually working
        for i in (0..size).step_by(size / 100) {
            assert!(index.search(i as i64).is_some());
        }
    }
}

#[test]
#[ignore]
fn test_sustained_load() {
    // Run for extended period to check for memory leaks or degradation
    let mut index = RecursiveModelIndex::new(1_000_000);
    let data: Vec<(i64, usize)> = (0..1_000_000)
        .map(|i| (i as i64, i))
        .collect();

    index.train(data);

    let start = Instant::now();
    let mut operations = 0;

    // Run for 30 seconds
    while start.elapsed() < Duration::from_secs(30) {
        // Mix of operations
        let key = (operations % 1_000_000) as i64;

        // Point query
        let _ = index.search(key);

        // Range query
        let _ = index.range_search(key, key + 1000);

        operations += 1;
    }

    let ops_per_sec = operations as f64 / 30.0;
    println!("Sustained load test: {} ops/sec over 30 seconds", ops_per_sec);

    assert!(ops_per_sec > 100_000.0, "Performance degraded during sustained load");
}

#[test]
fn test_worst_case_distribution() {
    // Test with adversarial data distribution
    let mut index = RecursiveModelIndex::new(10_000);
    let mut data = Vec::new();

    // Create clusters with gaps
    for cluster in 0..10usize {
        let base = cluster as i64 * 1_000_000;
        for i in 0..1000usize {
            data.push((base + i as i64, cluster * 1000 + i));
        }
    }

    index.train(data.clone());

    // Should still find keys even with bad distribution
    let mut found = 0;
    for (key, expected) in data.iter().take(100) {
        if index.search(*key) == Some(*expected) {
            found += 1;
        }
    }

    assert!(found > 90, "Poor performance on clustered data");
}

#[test]
#[ignore]
fn test_concurrent_write_safety() {
    let storage = Arc::new(RwLock::new(ArrowStorage::new()));
    let mut handles = vec![];

    // Spawn writers
    for thread_id in 0..10 {
        let storage_clone = Arc::clone(&storage);

        let handle = thread::spawn(move || {
            for i in 0..1000 {
                let timestamp = 1_600_000_000_000_000 + (thread_id * 1000 + i) as i64;
                if let Ok(mut storage) = storage_clone.write() {
                    let _ = storage.insert(timestamp, i as f64, thread_id as i64);
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all writers
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify data integrity
    let storage = storage.read().unwrap();
    let results = storage.range_query(1_600_000_000_000_000, 1_600_000_000_100_000);

    assert!(results.is_ok(), "Storage corrupted under concurrent writes");
}

#[test]
fn test_recovery_after_errors() {
    let mut index = RecursiveModelIndex::new(1000);

    // Train with valid data
    let data: Vec<(i64, usize)> = (0..100)
        .map(|i| (i as i64, i))
        .collect();
    index.train(data);

    // Try operations that might cause errors
    let _ = index.search(i64::MAX);
    let _ = index.search(i64::MIN);
    let _ = index.range_search(i64::MAX - 1, i64::MAX);

    // Index should still work after error conditions
    assert_eq!(index.search(50), Some(50));
    assert_eq!(index.range_search(0, 10).len(), 11);
}

#[test]
#[ignore]
fn test_performance_regression() {
    // Baseline performance test to catch regressions
    let mut index = RecursiveModelIndex::new(1_000_000);
    let data: Vec<(i64, usize)> = (0..1_000_000)
        .map(|i| (i as i64 * 10, i))
        .collect();

    // Training should be fast
    let train_start = Instant::now();
    index.train(data);
    let train_time = train_start.elapsed();

    assert!(train_time < Duration::from_millis(500),
        "Training regression: took {:?}", train_time);

    // Lookups should maintain speed
    let mut total_time = Duration::ZERO;
    for i in (0..1_000_000).step_by(1000) {
        let key = i as i64 * 10;
        let start = Instant::now();
        let _ = index.search(key);
        total_time += start.elapsed();
    }

    let avg_lookup_ns = total_time.as_nanos() / 1000;
    assert!(avg_lookup_ns < 100,
        "Lookup regression: {} ns average", avg_lookup_ns);
}