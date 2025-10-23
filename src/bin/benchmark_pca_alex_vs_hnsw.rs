//! PCA-ALEX vs HNSW Benchmark
//!
//! Compares two vector search approaches:
//! - PCA-ALEX: 1536D → 64D PCA + ALEX learned index
//! - HNSW: Industry-standard graph-based search
//!
//! Metrics:
//! - Recall@10 (vs brute-force ground truth)
//! - Query latency (p50, p95, p99)
//! - Memory usage
//!
//! Goal: Validate if PCA-ALEX can achieve >90% recall

use omendb::pca::VectorPCA;
use omendb::vector::{PCAAlexIndex, Vector, VectorStore};
use rand::Rng;
use std::collections::HashSet;
use std::time::Instant;

/// Generate structured vectors that mimic real embeddings
fn generate_structured_vector(dim: usize, basis_dim: usize, basis: &[Vec<f32>], seed: usize) -> Vector {
    let mut rng = rand::thread_rng();
    let weights: Vec<f32> = (0..basis_dim)
        .map(|i| ((seed + i) as f32 * 0.1 + rng.gen_range(-0.2..0.2)).sin())
        .collect();

    let mut data = vec![0.0; dim];
    for i in 0..dim {
        for j in 0..basis_dim {
            data[i] += weights[j] * basis[j][i];
        }
        data[i] += rng.gen_range(-0.1..0.1);
    }

    Vector::new(data)
}

fn generate_basis_vectors(dim: usize, basis_dim: usize) -> Vec<Vec<f32>> {
    let mut rng = rand::thread_rng();
    (0..basis_dim)
        .map(|j| {
            (0..dim)
                .map(|i| {
                    ((i as f32 * 0.01).sin() * (j as f32 + i as f32 * 0.1).cos())
                        + rng.gen_range(-0.2..0.2)
                })
                .collect()
        })
        .collect()
}

/// Compute recall@k: what fraction of true top-k neighbors are in the results?
fn compute_recall(ground_truth: &[(usize, f32)], results: &[(usize, f32)], k: usize) -> f32 {
    let truth_ids: HashSet<usize> = ground_truth.iter().take(k).map(|(id, _)| *id).collect();
    let result_ids: HashSet<usize> = results.iter().take(k).map(|(id, _)| *id).collect();

    let intersection = truth_ids.intersection(&result_ids).count();
    intersection as f32 / k as f32
}

fn main() {
    println!("==============================================");
    println!("PCA-ALEX vs HNSW Benchmark");
    println!("==============================================\n");

    let dimensions = 1536;
    let pca_dims = 64;
    let basis_dim = 80;
    let num_vectors = 10_000;
    let num_queries = 100;
    let k = 10;

    println!("Configuration:");
    println!("  Vectors: {}", num_vectors);
    println!("  Dimensions: {}D", dimensions);
    println!("  PCA reduction: {}D → {}D", dimensions, pca_dims);
    println!("  Intrinsic basis: {}D", basis_dim);
    println!("  Queries: {}", num_queries);
    println!("  k: {}\n", k);

    // Generate basis and vectors
    println!("Generating structured test data...");
    let basis = generate_basis_vectors(dimensions, basis_dim);
    let vectors: Vec<Vector> = (0..num_vectors)
        .map(|i| generate_structured_vector(dimensions, basis_dim, &basis, i))
        .collect();

    // ========================================
    // Brute-force baseline (ground truth)
    // ========================================
    println!("\n--- Building Brute-Force Baseline ---");
    let mut brute_force_store = VectorStore::new(dimensions);
    for vec in &vectors {
        brute_force_store.insert(vec.clone()).unwrap();
    }
    println!("✅ Brute-force index ready");

    // ========================================
    // HNSW Index
    // ========================================
    println!("\n--- Building HNSW Index ---");
    let hnsw_start = Instant::now();
    let mut hnsw_store = VectorStore::new(dimensions);

    for vec in &vectors {
        hnsw_store.insert(vec.clone()).unwrap();
    }
    let hnsw_build_time = hnsw_start.elapsed();
    println!("Build time: {:?}", hnsw_build_time);

    // ========================================
    // PCA-ALEX Index
    // ========================================
    println!("\n--- Building PCA-ALEX Index ---");
    let pca_alex_start = Instant::now();

    let mut pca_alex = PCAAlexIndex::new(dimensions, pca_dims);

    // Train PCA on all vectors
    let training_data: Vec<Vec<f32>> = vectors.iter().map(|v| v.data.clone()).collect();
    let variance = pca_alex.train_pca(&training_data).unwrap();
    println!("PCA explained variance: {:.2}%", variance * 100.0);

    // Insert vectors
    for vec in &vectors {
        pca_alex.insert(vec.clone()).unwrap();
    }
    let pca_alex_build_time = pca_alex_start.elapsed();
    println!("Build time: {:?}", pca_alex_build_time);

    // ========================================
    // Query Benchmarks
    // ========================================
    println!("\n==============================================");
    println!("Query Performance");
    println!("==============================================\n");

    let query_vectors: Vec<Vector> = (num_vectors..num_vectors + num_queries)
        .map(|i| generate_structured_vector(dimensions, basis_dim, &basis, i))
        .collect();

    // HNSW queries
    println!("--- HNSW Queries ---");
    let mut hnsw_recalls = Vec::new();
    let mut hnsw_times = Vec::new();

    for query in &query_vectors {
        // Ground truth
        let truth = brute_force_store
            .knn_search_brute_force(query, k)
            .unwrap();

        // HNSW query
        let start = Instant::now();
        let hnsw_results = hnsw_store.knn_search(query, k).unwrap();
        let duration = start.elapsed();

        hnsw_times.push(duration.as_secs_f64() * 1000.0);
        hnsw_recalls.push(compute_recall(&truth, &hnsw_results, k));
    }

    hnsw_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let hnsw_p50 = hnsw_times[num_queries / 2];
    let hnsw_p95 = hnsw_times[(num_queries as f32 * 0.95) as usize];
    let hnsw_p99 = hnsw_times[(num_queries as f32 * 0.99) as usize];
    let hnsw_avg_recall = hnsw_recalls.iter().sum::<f32>() / hnsw_recalls.len() as f32;

    println!("Recall@{}: {:.2}%", k, hnsw_avg_recall * 100.0);
    println!("Latency p50: {:.2}ms", hnsw_p50);
    println!("Latency p95: {:.2}ms", hnsw_p95);
    println!("Latency p99: {:.2}ms", hnsw_p99);

    // PCA-ALEX queries
    println!("\n--- PCA-ALEX Queries ---");
    let mut pca_alex_recalls = Vec::new();
    let mut pca_alex_times = Vec::new();

    for query in &query_vectors {
        // Ground truth
        let truth = brute_force_store
            .knn_search_brute_force(query, k)
            .unwrap();

        // PCA-ALEX query
        let start = Instant::now();
        let pca_alex_results = pca_alex.knn_search(query, k).unwrap();
        let duration = start.elapsed();

        pca_alex_times.push(duration.as_secs_f64() * 1000.0);
        pca_alex_recalls.push(compute_recall(&truth, &pca_alex_results, k));
    }

    pca_alex_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let pca_alex_p50 = pca_alex_times[num_queries / 2];
    let pca_alex_p95 = pca_alex_times[(num_queries as f32 * 0.95) as usize];
    let pca_alex_p99 = pca_alex_times[(num_queries as f32 * 0.99) as usize];
    let pca_alex_avg_recall = pca_alex_recalls.iter().sum::<f32>() / pca_alex_recalls.len() as f32;

    println!("Recall@{}: {:.2}%", k, pca_alex_avg_recall * 100.0);
    println!("Latency p50: {:.2}ms", pca_alex_p50);
    println!("Latency p95: {:.2}ms", pca_alex_p95);
    println!("Latency p99: {:.2}ms", pca_alex_p99);

    // ========================================
    // Summary Comparison
    // ========================================
    println!("\n==============================================");
    println!("Final Comparison");
    println!("==============================================\n");

    println!("| Metric             | HNSW        | PCA-ALEX    | Winner     |");
    println!("|--------------------+-------------+-------------+------------|");

    // Recall
    let recall_winner = if hnsw_avg_recall > pca_alex_avg_recall {
        "HNSW"
    } else {
        "PCA-ALEX"
    };
    println!(
        "| Recall@{}          | {:.2}%     | {:.2}%     | {}      |",
        k,
        hnsw_avg_recall * 100.0,
        pca_alex_avg_recall * 100.0,
        recall_winner
    );

    // Latency
    let latency_winner = if hnsw_p95 < pca_alex_p95 {
        "HNSW"
    } else {
        "PCA-ALEX"
    };
    println!(
        "| Latency p95        | {:.2}ms    | {:.2}ms    | {}      |",
        hnsw_p95, pca_alex_p95, latency_winner
    );

    // Build time
    let build_winner = if hnsw_build_time < pca_alex_build_time {
        "HNSW"
    } else {
        "PCA-ALEX"
    };
    println!(
        "| Build time         | {:?}  | {:?}  | {}      |",
        hnsw_build_time, pca_alex_build_time, build_winner
    );

    println!("\n==============================================");
    println!("Verdict");
    println!("==============================================\n");

    if pca_alex_avg_recall >= 0.90 {
        println!("✅ PCA-ALEX achieves >90% recall target!");
        println!("   This is a HUGE improvement over Week 1's 5% recall.");
        println!("   PCA-ALEX is viable for production!");
    } else {
        println!("⚠️  PCA-ALEX recall: {:.2}% (target: 90%)", pca_alex_avg_recall * 100.0);
        println!("   HNSW is safer choice for production (recall: {:.2}%)", hnsw_avg_recall * 100.0);
    }

    println!("\n--- Context ---");
    println!("Week 1 ALEX (1D projection): 5% recall ❌");
    println!("Week 2 PCA-ALEX (64D): {:.2}% recall", pca_alex_avg_recall * 100.0);
    println!("Week 2 HNSW (baseline): {:.2}% recall", hnsw_avg_recall * 100.0);
}
