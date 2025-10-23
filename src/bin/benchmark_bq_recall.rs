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
    println!("=== Binary Quantization Recall Tuning ===\n");

    let dimensions = 1536;
    let num_vectors = 10_000;
    let num_queries = 100;
    let k = 10;

    println!("Dataset: {} vectors, {} dimensions", num_vectors, dimensions);
    println!("Queries: {} queries, k={}", num_queries, k);
    println!("Goal: Find minimum expansion for 95%+ recall @ <5ms p95\n");

    // Generate dataset
    println!("Generating dataset...");
    let dataset_start = Instant::now();
    let vectors: Vec<Vector> = (0..num_vectors)
        .map(|i| generate_random_vector(dimensions, i))
        .collect();
    println!("  Generated in {:.2}s\n", dataset_start.elapsed().as_secs_f64());

    // Build baseline HNSW for ground truth
    println!("Building baseline HNSW (ground truth)...");
    let mut hnsw_baseline = HNSWIndex::new(num_vectors, dimensions);
    for vector in &vectors {
        hnsw_baseline.insert(&vector.data).unwrap();
    }

    let mut baseline_results = Vec::new();
    for query_id in 0..num_queries {
        let query = &vectors[query_id * (num_vectors / num_queries)];
        let results = hnsw_baseline.search(&query.data, k).unwrap();
        baseline_results.push(results);
    }
    println!("  Ground truth computed\n");

    // Train quantization model
    println!("Training quantization model...");
    let training_vectors: Vec<Vec<f32>> = vectors.iter().take(1000).map(|v| v.data.clone()).collect();
    let model = QuantizationModel::train(&training_vectors, true).unwrap();

    // Build quantized store
    println!("Building quantized index...");
    let mut bq_store = QuantizedVectorStore::new(model, num_vectors, dimensions);
    for vector in &vectors {
        bq_store.insert(vector.clone()).unwrap();
    }
    println!("  Index built\n");

    // Test expansion factors from 10x to 500x
    println!("--- Recall vs Expansion Factor ---\n");
    println!("{:>6} | {:>10} | {:>10} | {:>10} | {:>10}",
             "Expand", "Recall@10", "p50 (ms)", "p95 (ms)", "p99 (ms)");
    println!("{:-<6}-+-{:-<10}-+-{:-<10}-+-{:-<10}-+-{:-<10}", "", "", "", "", "");

    let expansion_factors = vec![10, 20, 50, 75, 100, 150, 200, 300, 400, 500];

    let mut best_expansion = 0;
    let mut best_recall = 0.0;

    for &expansion in &expansion_factors {
        let candidates = k * expansion;

        let mut query_times = Vec::new();
        let mut recalls = Vec::new();

        for (query_idx, query_id) in (0..num_queries).enumerate() {
            let actual_query_id = query_id * (num_vectors / num_queries);
            let query = &vectors[actual_query_id];

            let query_start = Instant::now();
            let results = bq_store.knn_search(query, k, Some(candidates)).unwrap();
            query_times.push(query_start.elapsed().as_secs_f64() * 1000.0);

            let recall = compute_recall(&baseline_results[query_idx], &results, k);
            recalls.push(recall);
        }

        query_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p50 = query_times[query_times.len() / 2];
        let p95 = query_times[query_times.len() * 95 / 100];
        let p99 = query_times[query_times.len() * 99 / 100];

        let avg_recall = recalls.iter().sum::<f64>() / recalls.len() as f64;

        print!("{:>5}x | {:>9.1}% | {:>10.3} | {:>10.3} | {:>10.3}",
               expansion, avg_recall * 100.0, p50, p95, p99);

        // Check if meets criteria
        if avg_recall >= 0.95 && p95 < 5.0 {
            println!("  ✅ TARGET MET");
            if best_recall < 0.95 {
                best_expansion = expansion;
                best_recall = avg_recall;
            }
        } else if avg_recall >= 0.95 {
            println!("  ✅ Recall OK, ⚠️  Latency {:.1}ms > 5ms", p95);
        } else if p95 < 5.0 {
            println!("  ⚠️  Recall {:.1}% < 95%", avg_recall * 100.0);
        } else {
            println!("  ❌ Both criteria failed");
        }

        if avg_recall > best_recall && p95 < 5.0 {
            best_expansion = expansion;
            best_recall = avg_recall;
        }
    }

    println!("\n--- Summary ---");
    if best_recall >= 0.95 {
        println!("✅ SUCCESS: {}x expansion achieves {:.1}% recall @ <5ms p95",
                 best_expansion, best_recall * 100.0);
    } else {
        println!("⚠️  Best result: {}x expansion → {:.1}% recall @ <5ms p95",
                 best_expansion, best_recall * 100.0);
        println!("   Need higher expansion or algorithm tuning for 95% target");
    }

    // Memory analysis
    println!("\n--- Memory Analysis ---");
    let usage = bq_store.memory_usage();
    println!("  Quantized vectors: {:.2} MB ({:.1}%)",
             usage.quantized_vectors as f64 / 1e6,
             100.0 * usage.quantized_vectors as f64 / usage.total as f64);
    println!("  Original vectors: {:.2} MB ({:.1}%)",
             usage.original_vectors as f64 / 1e6,
             100.0 * usage.original_vectors as f64 / usage.total as f64);
    println!("  HNSW graph: {:.2} MB ({:.1}%)",
             usage.hnsw_graph as f64 / 1e6,
             100.0 * usage.hnsw_graph as f64 / usage.total as f64);
    println!("  Total: {:.2} MB", usage.total as f64 / 1e6);

    let quantized_only = usage.quantized_vectors + usage.hnsw_graph;
    let reduction = usage.original_vectors as f64 / quantized_only as f64;
    println!("\n  Quantized+graph only: {:.2} MB", quantized_only as f64 / 1e6);
    println!("  Potential reduction (without originals): {:.1}x", reduction);
    println!("  Note: Reranking requires originals in memory or on disk");

    println!("\n🎯 Recommendation: Use {}x expansion for optimal recall/latency trade-off", best_expansion);
}
