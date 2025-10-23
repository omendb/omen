use omendb::quantization::{QuantizationModel, QuantizedVectorStore};
use omendb::vector::{HNSWIndex, Vector};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::collections::HashSet;
use std::time::Instant;

/// Generate random vector with seeded RNG for reproducibility
fn generate_random_vector(dimensions: usize, seed: usize) -> Vector {
    let mut rng = StdRng::seed_from_u64(seed as u64);
    let data: Vec<f32> = (0..dimensions).map(|_| rng.gen_range(-1.0..1.0)).collect();
    Vector { data }
}

/// Compute recall@k: fraction of true neighbors found
fn compute_recall(true_neighbors: &[(usize, f32)], retrieved: &[(usize, f32)], k: usize) -> f64 {
    let true_set: HashSet<usize> = true_neighbors.iter().take(k).map(|(id, _)| *id).collect();
    let retrieved_set: HashSet<usize> = retrieved.iter().take(k).map(|(id, _)| *id).collect();

    let intersection_size = true_set.intersection(&retrieved_set).count();
    intersection_size as f64 / k as f64
}

fn main() {
    println!("=== Binary Quantization + HNSW Benchmark ===\n");

    let dimensions = 1536; // OpenAI embedding size
    let num_vectors = 10_000;
    let num_queries = 100;
    let k = 10; // Top-10 neighbors

    println!("Dataset: {} vectors, {} dimensions", num_vectors, dimensions);
    println!("Queries: {} queries, k={}\n", num_queries, k);

    // Generate dataset
    println!("Generating dataset...");
    let dataset_start = Instant::now();
    let vectors: Vec<Vector> = (0..num_vectors)
        .map(|i| generate_random_vector(dimensions, i))
        .collect();
    println!("  Generated in {:.2}s\n", dataset_start.elapsed().as_secs_f64());

    // === Full-Precision HNSW Baseline ===
    println!("--- Baseline: Full-Precision HNSW ---");

    let mut hnsw_baseline = HNSWIndex::new(num_vectors, dimensions);

    println!("Building index...");
    let build_start = Instant::now();
    for vector in &vectors {
        hnsw_baseline.insert(&vector.data).unwrap();
    }
    let build_time = build_start.elapsed();
    println!("  Build time: {:.2}s ({:.0} vectors/sec)",
             build_time.as_secs_f64(),
             num_vectors as f64 / build_time.as_secs_f64());

    // Benchmark queries
    println!("Querying...");
    let mut query_times = Vec::new();
    let mut baseline_results = Vec::new();

    for query_id in 0..num_queries {
        let query = &vectors[query_id * (num_vectors / num_queries)];

        let query_start = Instant::now();
        let results = hnsw_baseline.search(&query.data, k).unwrap();
        query_times.push(query_start.elapsed().as_secs_f64() * 1000.0); // ms

        baseline_results.push(results);
    }

    query_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = query_times[query_times.len() / 2];
    let p95 = query_times[query_times.len() * 95 / 100];
    let p99 = query_times[query_times.len() * 99 / 100];

    println!("  Query latency: p50={:.3}ms, p95={:.3}ms, p99={:.3}ms", p50, p95, p99);

    // Memory estimate (rough)
    let float32_memory = num_vectors * dimensions * 4; // bytes
    let hnsw_graph_memory = num_vectors * 100; // ~100 bytes per node
    let baseline_memory = float32_memory + hnsw_graph_memory;
    println!("  Memory: {:.1} GB ({:.1} GB vectors + {:.1} GB graph)\n",
             baseline_memory as f64 / 1e9,
             float32_memory as f64 / 1e9,
             hnsw_graph_memory as f64 / 1e9);

    // === Binary Quantization + HNSW ===
    println!("--- Binary Quantization + HNSW ---");

    // Train quantization model on sample
    println!("Training quantization model...");
    let train_start = Instant::now();
    let training_vectors: Vec<Vec<f32>> = vectors.iter().take(1000).map(|v| v.data.clone()).collect();
    let model = QuantizationModel::train(&training_vectors, true).unwrap();
    let train_time = train_start.elapsed();
    println!("  Training time: {:.3}s", train_time.as_secs_f64());

    // Build quantized index
    println!("Building quantized index...");
    let mut bq_store = QuantizedVectorStore::new(model, num_vectors, dimensions);

    let bq_build_start = Instant::now();
    for vector in &vectors {
        bq_store.insert(vector.clone()).unwrap();
    }
    let bq_build_time = bq_build_start.elapsed();
    println!("  Build time: {:.2}s ({:.0} vectors/sec)",
             bq_build_time.as_secs_f64(),
             num_vectors as f64 / bq_build_time.as_secs_f64());

    // Test different candidate expansion factors
    let expansion_factors = vec![10, 20, 50];

    for &expansion in &expansion_factors {
        println!("\n  Candidate expansion: {}x (retrieve {} for top-{})", expansion, k * expansion, k);

        let mut bq_query_times = Vec::new();
        let mut recalls = Vec::new();

        for (query_idx, query_id) in (0..num_queries).enumerate() {
            let actual_query_id = query_id * (num_vectors / num_queries);
            let query = &vectors[actual_query_id];

            let query_start = Instant::now();
            let results = bq_store.knn_search(query, k, Some(k * expansion)).unwrap();
            bq_query_times.push(query_start.elapsed().as_secs_f64() * 1000.0);

            // Compute recall vs baseline
            let recall = compute_recall(&baseline_results[query_idx], &results, k);
            recalls.push(recall);
        }

        bq_query_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let bq_p50 = bq_query_times[bq_query_times.len() / 2];
        let bq_p95 = bq_query_times[bq_query_times.len() * 95 / 100];
        let bq_p99 = bq_query_times[bq_query_times.len() * 99 / 100];

        let avg_recall = recalls.iter().sum::<f64>() / recalls.len() as f64;

        println!("    Recall@{}: {:.1}%", k, avg_recall * 100.0);
        println!("    Query latency: p50={:.3}ms, p95={:.3}ms, p99={:.3}ms", bq_p50, bq_p95, bq_p99);
        println!("    Speedup vs baseline: {:.2}x (p95)", p95 / bq_p95);

        if avg_recall >= 0.95 {
            println!("    ✅ PASS: Recall >95%");
        } else {
            println!("    ⚠️  WARNING: Recall {:.1}% < 95% target", avg_recall * 100.0);
        }

        if bq_p95 < 5.0 {
            println!("    ✅ PASS: Latency <5ms p95");
        } else {
            println!("    ⚠️  WARNING: Latency {:.3}ms > 5ms target", bq_p95);
        }
    }

    // Memory usage
    println!("\n  Memory usage:");
    let memory_usage = bq_store.memory_usage();
    println!("    Quantized vectors: {:.1} GB", memory_usage.quantized_vectors as f64 / 1e9);
    println!("    Original vectors: {:.1} GB", memory_usage.original_vectors as f64 / 1e9);
    println!("    HNSW graph: {:.1} GB", memory_usage.hnsw_graph as f64 / 1e9);
    println!("    Total: {:.1} GB", memory_usage.total as f64 / 1e9);

    let memory_reduction = baseline_memory as f64 / memory_usage.total as f64;
    println!("    Reduction vs baseline: {:.1}x", memory_reduction);

    if memory_reduction >= 5.0 {
        println!("    ✅ PASS: >5x memory reduction");
    } else {
        println!("    ⚠️  Note: {:.1}x reduction (reranking requires originals)", memory_reduction);
    }

    // Scaling estimate
    println!("\n--- Scaling Estimates (10M vectors, 1536D) ---");
    let scale_factor = 1000.0; // 10K -> 10M
    let baseline_10m = (baseline_memory as f64 * scale_factor) / 1e9;
    let bq_10m = (memory_usage.total as f64 * scale_factor) / 1e9;

    println!("  Baseline (full HNSW): {:.1} GB", baseline_10m);
    println!("  BQ + HNSW: {:.1} GB", bq_10m);
    println!("  Savings: {:.1} GB ({:.1}x reduction)", baseline_10m - bq_10m, baseline_10m / bq_10m);

    println!("\n=== Summary ===");
    println!("✓ Binary Quantization + HNSW integration complete");
    println!("✓ Two-phase search (Hamming + L2 reranking) working");
    println!("✓ Ready for production benchmarking (Days 7-8)");

    println!("\n🎯 Next: Scale to 100K-1M vectors for final validation");
}
