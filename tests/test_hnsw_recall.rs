//! HNSW Recall Validation Tests
//!
//! Validates that HNSW approximate nearest neighbor search
//! achieves acceptable recall compared to brute-force search.

use omendb::vector::types::Vector;
use omendb::vector::store::VectorStore;

/// Brute-force k-nearest neighbor search for ground truth
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

    // Sort by distance (ascending)
    distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    // Return top-k
    distances.into_iter().take(k).collect()
}

/// Compute recall: what fraction of ground truth results are in HNSW results?
fn compute_recall(
    ground_truth: &[(usize, f32)],
    hnsw_results: &[(usize, f32)],
) -> f32 {
    let ground_truth_ids: std::collections::HashSet<usize> =
        ground_truth.iter().map(|(id, _)| *id).collect();
    let hnsw_ids: std::collections::HashSet<usize> =
        hnsw_results.iter().map(|(id, _)| *id).collect();

    let intersection = ground_truth_ids.intersection(&hnsw_ids).count();
    intersection as f32 / ground_truth.len() as f32
}

/// Test HNSW recall on small dataset (1000 vectors)
#[test]
fn test_hnsw_recall_1000_vectors() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let dimensions = 128;
    let num_vectors = 1000;
    let num_queries = 100;
    let k = 10; // Find 10 nearest neighbors

    // Generate random vectors
    let vectors: Vec<Vector> = (0..num_vectors)
        .map(|_| {
            let data: Vec<f32> = (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            Vector::new(data)
        })
        .collect();

    // Insert into HNSW
    let mut store = VectorStore::new(dimensions);
    let _ids = store.batch_insert(vectors.clone()).unwrap();

    // Generate queries
    let queries: Vec<Vector> = (0..num_queries)
        .map(|_| {
            let data: Vec<f32> = (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            Vector::new(data)
        })
        .collect();

    // Test recall for each query
    let mut total_recall = 0.0;

    for query in &queries {
        // Ground truth (brute-force)
        let ground_truth = brute_force_knn(query, &vectors, k);

        // HNSW search
        let hnsw_results = store.knn_search(query, k).unwrap();

        // Compute recall
        let recall = compute_recall(&ground_truth, &hnsw_results);
        total_recall += recall;
    }

    let avg_recall = total_recall / num_queries as f32;

    eprintln!("HNSW Recall (1000 vectors, k=10): {:.2}%", avg_recall * 100.0);

    // Assert recall is acceptable (>90%)
    assert!(
        avg_recall >= 0.90,
        "HNSW recall too low: {:.2}% (expected >= 90%)",
        avg_recall * 100.0
    );
}

/// Test HNSW recall on medium dataset (10K vectors)
#[test]
fn test_hnsw_recall_10k_vectors() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let dimensions = 128;
    let num_vectors = 10_000;
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

    // Insert into HNSW
    let mut store = VectorStore::new(dimensions);
    let _ids = store.batch_insert(vectors.clone()).unwrap();

    // Generate queries
    let queries: Vec<Vector> = (0..num_queries)
        .map(|_| {
            let data: Vec<f32> = (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            Vector::new(data)
        })
        .collect();

    // Test recall for each query
    let mut total_recall = 0.0;

    for query in &queries {
        // Ground truth (brute-force)
        let ground_truth = brute_force_knn(query, &vectors, k);

        // HNSW search
        let hnsw_results = store.knn_search(query, k).unwrap();

        // Compute recall
        let recall = compute_recall(&ground_truth, &hnsw_results);
        total_recall += recall;
    }

    let avg_recall = total_recall / num_queries as f32;

    eprintln!("HNSW Recall (10K vectors, k=10): {:.2}%", avg_recall * 100.0);

    // Assert recall is acceptable (>90%)
    assert!(
        avg_recall >= 0.90,
        "HNSW recall too low: {:.2}% (expected >= 90%)",
        avg_recall * 100.0
    );
}

/// Test HNSW recall with different k values
#[test]
fn test_hnsw_recall_varying_k() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let dimensions = 128;
    let num_vectors = 5000;
    let num_queries = 50;

    // Generate random vectors
    let vectors: Vec<Vector> = (0..num_vectors)
        .map(|_| {
            let data: Vec<f32> = (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            Vector::new(data)
        })
        .collect();

    // Insert into HNSW
    let mut store = VectorStore::new(dimensions);
    let _ids = store.batch_insert(vectors.clone()).unwrap();

    // Generate queries
    let queries: Vec<Vector> = (0..num_queries)
        .map(|_| {
            let data: Vec<f32> = (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            Vector::new(data)
        })
        .collect();

    // Test different k values
    for k in [5, 10, 20, 50] {
        let mut total_recall = 0.0;

        for query in &queries {
            let ground_truth = brute_force_knn(query, &vectors, k);
            let hnsw_results = store.knn_search(query, k).unwrap();
            let recall = compute_recall(&ground_truth, &hnsw_results);
            total_recall += recall;
        }

        let avg_recall = total_recall / num_queries as f32;
        eprintln!("HNSW Recall (k={}): {:.2}%", k, avg_recall * 100.0);

        // Assert recall is acceptable
        assert!(
            avg_recall >= 0.85,
            "HNSW recall too low for k={}: {:.2}% (expected >= 85%)",
            k,
            avg_recall * 100.0
        );
    }
}

/// Test HNSW recall with high-dimensional vectors (1536D, OpenAI embedding size)
#[test]
fn test_hnsw_recall_high_dimensional() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let dimensions = 1536; // OpenAI embedding size
    let num_vectors = 1000;
    let num_queries = 50;
    let k = 10;

    // Generate random normalized vectors (typical for embeddings)
    let vectors: Vec<Vector> = (0..num_vectors)
        .map(|_| {
            let data: Vec<f32> = (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            let vec = Vector::new(data);
            vec.normalize().unwrap() // Normalized like real embeddings
        })
        .collect();

    // Insert into HNSW
    let mut store = VectorStore::new(dimensions);
    let _ids = store.batch_insert(vectors.clone()).unwrap();

    // Generate queries
    let queries: Vec<Vector> = (0..num_queries)
        .map(|_| {
            let data: Vec<f32> = (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            let vec = Vector::new(data);
            vec.normalize().unwrap()
        })
        .collect();

    // Test recall
    let mut total_recall = 0.0;

    for query in &queries {
        let ground_truth = brute_force_knn(query, &vectors, k);
        let hnsw_results = store.knn_search(query, k).unwrap();
        let recall = compute_recall(&ground_truth, &hnsw_results);
        total_recall += recall;
    }

    let avg_recall = total_recall / num_queries as f32;

    eprintln!(
        "HNSW Recall (1536D vectors, k=10): {:.2}%",
        avg_recall * 100.0
    );

    // High-dimensional vectors can be harder, so allow slightly lower recall
    assert!(
        avg_recall >= 0.85,
        "HNSW recall too low: {:.2}% (expected >= 85%)",
        avg_recall * 100.0
    );
}

/// Test HNSW graph structure properties
#[test]
fn test_hnsw_graph_properties() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let dimensions = 128;
    let num_vectors = 1000;

    // Generate random vectors
    let vectors: Vec<Vector> = (0..num_vectors)
        .map(|_| {
            let data: Vec<f32> = (0..dimensions)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            Vector::new(data)
        })
        .collect();

    // Insert into HNSW
    let mut store = VectorStore::new(dimensions);
    let _ids = store.batch_insert(vectors.clone()).unwrap();

    // Verify HNSW index was created
    assert!(
        store.hnsw_index.is_some(),
        "HNSW index should be initialized after insert"
    );

    // Query should not panic
    let query = Vector::new(vec![0.0; dimensions]);
    let results = store.knn_search(&query, 10).unwrap();

    // Should return requested number of results (or less if not enough vectors)
    assert!(
        results.len() <= 10,
        "Should return at most k results"
    );
    assert!(
        !results.is_empty(),
        "Should return at least one result"
    );

    // Results should be sorted by distance (ascending)
    for i in 1..results.len() {
        assert!(
            results[i].1 >= results[i - 1].1,
            "Results should be sorted by distance"
        );
    }
}
