//! HNSW Graph Serialization Validation Tests
//!
//! Validates that HNSW graph serialization preserves:
//! - Graph structure (same query results after save/load)
//! - Search quality (recall unchanged)
//! - Data integrity (no corruption)

use omen::vector::types::Vector;
use omen::vector::store::VectorStore;
use tempfile::TempDir;

/// Test that graph serialization preserves query results
#[test]
fn test_graph_serialization_preserves_results() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let dimensions = 128;
    let num_vectors = 1000;
    let num_queries = 50;
    let k = 10;

    // Generate random vectors
    let vectors: Vec<Vector> = (0..num_vectors)
        .map(|_| {
            let data: Vec<f32> = (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            Vector::new(data)
        })
        .collect();

    // Create original index
    let mut store = VectorStore::new(dimensions);
    let _ids = store.batch_insert(vectors.clone()).unwrap();

    // Generate test queries
    let queries: Vec<Vector> = (0..num_queries)
        .map(|_| {
            let data: Vec<f32> = (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            Vector::new(data)
        })
        .collect();

    // Get results from original index
    let original_results: Vec<Vec<(usize, f32)>> = queries
        .iter()
        .map(|q| store.knn_search(q, k).unwrap())
        .collect();

    // Serialize to disk
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().join("test_graph");
    let graph_path = temp_dir.path().join("test_graph.hnsw.graph");
    let data_path = temp_dir.path().join("test_graph.hnsw.data");

    store.save_to_disk(base_path.to_str().unwrap()).unwrap();

    // Verify files exist
    assert!(graph_path.exists(), "Graph file should exist");
    assert!(data_path.exists(), "Data file should exist");

    // Load from disk into new store
    let mut loaded_store = VectorStore::load_from_disk(
        base_path.to_str().unwrap(),
        dimensions
    ).unwrap();

    // Note: Loaded store keeps vectors in HNSW, not in vectors array
    // So loaded_store.len() will be 0, but HNSW index has all the vectors
    // We verify this by checking that queries return results

    // Get results from loaded index
    let loaded_results: Vec<Vec<(usize, f32)>> = queries
        .iter()
        .map(|q| loaded_store.knn_search(q, k).unwrap())
        .collect();

    // Compare results
    let mut total_overlap = 0.0;
    let mut total_distance_diff = 0.0;

    for i in 0..num_queries {
        let orig = &original_results[i];
        let loaded = &loaded_results[i];

        // Check result count
        assert_eq!(
            orig.len(),
            loaded.len(),
            "Query {} should return same number of results",
            i
        );

        // Check ID overlap (should be 100% for deterministic HNSW)
        let orig_ids: std::collections::HashSet<usize> =
            orig.iter().map(|(id, _)| *id).collect();
        let loaded_ids: std::collections::HashSet<usize> =
            loaded.iter().map(|(id, _)| *id).collect();

        let overlap = orig_ids.intersection(&loaded_ids).count() as f32 / k as f32;
        total_overlap += overlap;

        // Check distance consistency
        for j in 0..k {
            let orig_dist = orig[j].1;
            let loaded_dist = loaded[j].1;
            total_distance_diff += (orig_dist - loaded_dist).abs();
        }
    }

    let avg_overlap = total_overlap / num_queries as f32;
    let avg_distance_diff = total_distance_diff / (num_queries * k) as f32;

    eprintln!("Graph Serialization Validation:");
    eprintln!("  Average ID overlap: {:.2}%", avg_overlap * 100.0);
    eprintln!("  Average distance diff: {:.6}", avg_distance_diff);

    // Expect near-perfect overlap (HNSW is deterministic after loading)
    assert!(
        avg_overlap >= 0.95,
        "ID overlap too low: {:.2}% (expected >= 95%)",
        avg_overlap * 100.0
    );

    // Distances should be very close (floating point tolerance)
    assert!(
        avg_distance_diff < 0.001,
        "Distance difference too high: {:.6} (expected < 0.001)",
        avg_distance_diff
    );
}

/// Test that serialization preserves recall quality
#[test]
fn test_serialization_preserves_recall() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let dimensions = 128;
    let num_vectors = 5000;
    let num_queries = 100;
    let k = 10;

    // Generate random vectors
    let vectors: Vec<Vector> = (0..num_vectors)
        .map(|_| {
            let data: Vec<f32> = (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            Vector::new(data)
        })
        .collect();

    // Create original index
    let mut store = VectorStore::new(dimensions);
    let _ids = store.batch_insert(vectors.clone()).unwrap();

    // Brute-force ground truth for queries
    fn brute_force_knn(
        query: &Vector,
        vectors: &[Vector],
        k: usize,
    ) -> Vec<(usize, f32)> {
        let mut distances: Vec<(usize, f32)> = vectors
            .iter()
            .enumerate()
            .map(|(id, vec)| {
                let dist = query.l2_distance(vec).unwrap();
                (id, dist)
            })
            .collect();
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        distances.into_iter().take(k).collect()
    }

    fn compute_recall(
        ground_truth: &[(usize, f32)],
        hnsw_results: &[(usize, f32)],
    ) -> f32 {
        let gt_ids: std::collections::HashSet<usize> =
            ground_truth.iter().map(|(id, _)| *id).collect();
        let hnsw_ids: std::collections::HashSet<usize> =
            hnsw_results.iter().map(|(id, _)| *id).collect();
        let intersection = gt_ids.intersection(&hnsw_ids).count();
        intersection as f32 / ground_truth.len() as f32
    }

    // Generate queries
    let queries: Vec<Vector> = (0..num_queries)
        .map(|_| {
            let data: Vec<f32> = (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            Vector::new(data)
        })
        .collect();

    // Measure original recall
    let mut original_recall = 0.0;
    for query in &queries {
        let ground_truth = brute_force_knn(query, &vectors, k);
        let hnsw_results = store.knn_search(query, k).unwrap();
        original_recall += compute_recall(&ground_truth, &hnsw_results);
    }
    original_recall /= num_queries as f32;

    // Serialize and load
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().join("recall_test");
    store.save_to_disk(base_path.to_str().unwrap()).unwrap();

    let mut loaded_store = VectorStore::load_from_disk(
        base_path.to_str().unwrap(),
        dimensions
    ).unwrap();

    // Measure loaded recall
    let mut loaded_recall = 0.0;
    for query in &queries {
        let ground_truth = brute_force_knn(query, &vectors, k);
        let hnsw_results = loaded_store.knn_search(query, k).unwrap();
        loaded_recall += compute_recall(&ground_truth, &hnsw_results);
    }
    loaded_recall /= num_queries as f32;

    eprintln!("Recall Preservation:");
    eprintln!("  Original recall: {:.2}%", original_recall * 100.0);
    eprintln!("  Loaded recall: {:.2}%", loaded_recall * 100.0);
    eprintln!("  Difference: {:.2}%", (original_recall - loaded_recall).abs() * 100.0);

    // Recall should be identical (deterministic HNSW)
    let recall_diff = (original_recall - loaded_recall).abs();
    assert!(
        recall_diff < 0.01,
        "Recall changed after serialization: {:.2}% difference (expected < 1%)",
        recall_diff * 100.0
    );

    // Both should have good recall
    assert!(
        loaded_recall >= 0.90,
        "Loaded recall too low: {:.2}% (expected >= 90%)",
        loaded_recall * 100.0
    );
}

/// Test serialization with high-dimensional vectors (1536D)
#[test]
fn test_serialization_high_dimensional() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let dimensions = 1536; // OpenAI embedding size
    let num_vectors = 1000;
    let k = 10;

    // Generate normalized vectors (typical for embeddings)
    let vectors: Vec<Vector> = (0..num_vectors)
        .map(|_| {
            let data: Vec<f32> = (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            let vec = Vector::new(data);
            vec.normalize().unwrap()
        })
        .collect();

    // Build index
    let mut store = VectorStore::new(dimensions);
    let _ids = store.batch_insert(vectors.clone()).unwrap();

    // Test query
    let query = Vector::new(vec![0.0; dimensions]);
    let original_results = store.knn_search(&query, k).unwrap();

    // Serialize
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().join("high_dim_test");
    store.save_to_disk(base_path.to_str().unwrap()).unwrap();

    // Load
    let mut loaded_store = VectorStore::load_from_disk(
        base_path.to_str().unwrap(),
        dimensions
    ).unwrap();

    let loaded_results = loaded_store.knn_search(&query, k).unwrap();

    // Compare
    assert_eq!(original_results.len(), loaded_results.len());

    let orig_ids: Vec<usize> = original_results.iter().map(|(id, _)| *id).collect();
    let loaded_ids: Vec<usize> = loaded_results.iter().map(|(id, _)| *id).collect();

    // IDs should match exactly (deterministic HNSW)
    assert_eq!(
        orig_ids, loaded_ids,
        "High-dimensional serialization should preserve query results"
    );

    eprintln!("✓ High-dimensional (1536D) serialization validated");
}

/// Test multiple save/load cycles
#[test]
fn test_multiple_serialization_cycles() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let dimensions = 128;
    let num_vectors = 500;
    let k = 5;

    let vectors: Vec<Vector> = (0..num_vectors)
        .map(|_| {
            let data: Vec<f32> = (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            Vector::new(data)
        })
        .collect();

    let mut store = VectorStore::new(dimensions);
    let _ids = store.batch_insert(vectors.clone()).unwrap();

    let query = Vector::new(vec![0.1; dimensions]);
    let original_results = store.knn_search(&query, k).unwrap();

    let temp_dir = TempDir::new().unwrap();

    // Cycle 1: Save and load
    let cycle1_path = temp_dir.path().join("cycle1");
    store.save_to_disk(cycle1_path.to_str().unwrap()).unwrap();

    let store1 = VectorStore::load_from_disk(
        cycle1_path.to_str().unwrap(),
        dimensions
    ).unwrap();

    // Cycle 2: Save loaded index and load again
    let cycle2_path = temp_dir.path().join("cycle2");
    store1.save_to_disk(cycle2_path.to_str().unwrap()).unwrap();

    let mut store2 = VectorStore::load_from_disk(
        cycle2_path.to_str().unwrap(),
        dimensions
    ).unwrap();

    // Query after 2 cycles
    let final_results = store2.knn_search(&query, k).unwrap();

    // Should still match original
    let orig_ids: Vec<usize> = original_results.iter().map(|(id, _)| *id).collect();
    let final_ids: Vec<usize> = final_results.iter().map(|(id, _)| *id).collect();

    assert_eq!(
        orig_ids, final_ids,
        "Multiple serialization cycles should preserve results"
    );

    eprintln!("✓ Multiple save/load cycles validated (2 cycles)");
}

/// Test that serialization handles empty index gracefully
#[test]
fn test_empty_index_serialization() {
    let dimensions = 128;
    let store = VectorStore::new(dimensions);

    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().join("empty_test");

    // Should not panic or error on empty index
    let result = store.save_to_disk(base_path.to_str().unwrap());

    // Empty index save should succeed (or return a clear error)
    match result {
        Ok(_) => {
            eprintln!("✓ Empty index save succeeded");
        }
        Err(e) => {
            eprintln!("✓ Empty index save returned expected error: {}", e);
            // This is acceptable behavior
        }
    }
}

/// Test file size and compression
#[test]
fn test_serialization_file_sizes() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let dimensions = 128;
    let num_vectors = 10000;

    let vectors: Vec<Vector> = (0..num_vectors)
        .map(|_| {
            let data: Vec<f32> = (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            Vector::new(data)
        })
        .collect();

    let mut store = VectorStore::new(dimensions);
    let _ids = store.batch_insert(vectors.clone()).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().join("size_test");
    store.save_to_disk(base_path.to_str().unwrap()).unwrap();

    // Check file sizes
    let graph_path = temp_dir.path().join("size_test.hnsw.graph");
    let data_path = temp_dir.path().join("size_test.hnsw.data");

    let graph_size = std::fs::metadata(&graph_path).unwrap().len();
    let data_size = std::fs::metadata(&data_path).unwrap().len();

    // Data file should be roughly: num_vectors * dimensions * sizeof(f32)
    let expected_data_size = (num_vectors * dimensions * 4) as u64;

    eprintln!("Serialization File Sizes:");
    eprintln!("  Graph file: {:.2} MB", graph_size as f64 / 1_000_000.0);
    eprintln!("  Data file: {:.2} MB", data_size as f64 / 1_000_000.0);
    eprintln!("  Expected data size: {:.2} MB", expected_data_size as f64 / 1_000_000.0);
    eprintln!("  Total: {:.2} MB", (graph_size + data_size) as f64 / 1_000_000.0);

    // Sanity checks
    assert!(graph_size > 0, "Graph file should not be empty");
    assert!(data_size > 0, "Data file should not be empty");

    // Data file should be close to expected size (within 20% for overhead)
    let data_size_ratio = data_size as f64 / expected_data_size as f64;
    assert!(
        data_size_ratio >= 0.8 && data_size_ratio <= 1.2,
        "Data file size unexpected: {} bytes (expected ~{} bytes, ratio: {:.2})",
        data_size,
        expected_data_size,
        data_size_ratio
    );
}
