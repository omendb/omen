//! Hybrid Search Recall Validation - Week 5 Day 3
//!
//! Validates recall accuracy of hybrid search (vector + SQL predicates)
//! Compares hybrid search results against ground truth (naive sequential scan + rerank)
//!
//! Tests:
//! - High selectivity (1-10%): Should match ground truth
//! - Medium selectivity (20-50%): Should maintain >90% recall
//! - Low selectivity (90%+): Should maintain >90% recall

use omen::catalog::Catalog;
use omen::sql_engine::SqlEngine;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::collections::HashSet;
use std::time::Instant;

fn generate_random_vector(dimensions: usize, seed: usize) -> Vec<f32> {
    let mut rng = StdRng::seed_from_u64(seed as u64);
    (0..dimensions).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

/// Compute recall@k: fraction of true neighbors found
fn compute_recall(true_ids: &[i64], retrieved_ids: &[i64], k: usize) -> f64 {
    let true_set: HashSet<i64> = true_ids.iter().take(k).copied().collect();
    let retrieved_set: HashSet<i64> = retrieved_ids.iter().take(k).copied().collect();
    let intersection_size = true_set.intersection(&retrieved_set).count();
    intersection_size as f64 / k.min(true_set.len()).min(retrieved_set.len()) as f64
}

/// Extract product IDs from query results
fn extract_ids_from_results(result_str: &str) -> Vec<i64> {
    // Parse the ExecutionResult output to extract IDs
    // This is a simple implementation - in production would use structured data
    let mut ids = Vec::new();

    // Look for id column values in the output
    // Format: "id: 123" or similar
    for line in result_str.lines() {
        if let Some(id_str) = line.split_whitespace().nth(0) {
            if let Ok(id) = id_str.parse::<i64>() {
                ids.push(id);
            }
        }
    }

    ids
}

fn main() {
    println!("==============================================");
    println!("Hybrid Search Recall Validation - Week 5 Day 3");
    println!("==============================================\n");

    let dimensions = 128;
    let num_products = 5_000; // Smaller dataset for ground truth computation
    let num_queries = 20;
    let k = 10;

    // Create temporary database
    let temp_dir = tempfile::tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    println!("Creating database in {:?}...", data_dir);

    // Create catalog and engine
    let catalog = Catalog::new(data_dir.clone()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Create products table
    println!("\nCreating products table...");
    let create_table_sql = format!(
        "CREATE TABLE products (
            id INT PRIMARY KEY,
            category TEXT,
            price FLOAT,
            embedding VECTOR({})
        )",
        dimensions
    );
    engine.execute(&create_table_sql).unwrap();

    // Insert products with known patterns
    println!("\nInserting {} products...", num_products);
    let categories = vec!["electronics", "clothing", "books", "home", "toys"];

    // Store vectors for ground truth computation
    let mut all_embeddings = Vec::new();

    for i in 0..num_products {
        let category = categories[i % categories.len()];
        let price = 10.0 + (i % 1000) as f64;

        let embedding = generate_random_vector(dimensions, i);
        all_embeddings.push(embedding.clone());

        let embedding_str = format!(
            "[{}]",
            embedding
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let insert_sql = format!(
            "INSERT INTO products (id, category, price, embedding) VALUES ({}, '{}', {}, '{}')",
            i, category, price, embedding_str
        );

        engine.execute(&insert_sql).unwrap();

        if (i + 1) % 1000 == 0 {
            println!("  Inserted {} products...", i + 1);
        }
    }

    println!("\nInsert complete. Reloading catalog...");
    drop(engine);
    let catalog = Catalog::new(data_dir.clone()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Run recall validation tests
    println!("\n==============================================");
    println!("Recall Validation Tests");
    println!("==============================================\n");

    // Test 1: High Selectivity (category filter ~20%)
    test_recall(
        &mut engine,
        "High Selectivity (20%)",
        "category = 'electronics'",
        &all_embeddings,
        num_products,
        dimensions,
        num_queries,
        k,
    );

    // Test 2: Medium Selectivity (price range ~50%)
    test_recall(
        &mut engine,
        "Medium Selectivity (50%)",
        "price >= 200.0 AND price <= 700.0",
        &all_embeddings,
        num_products,
        dimensions,
        num_queries,
        k,
    );

    // Test 3: Low Selectivity (price > 100, ~90%)
    test_recall(
        &mut engine,
        "Low Selectivity (90%)",
        "price > 100.0",
        &all_embeddings,
        num_products,
        dimensions,
        num_queries,
        k,
    );

    println!("\n==============================================");
    println!("Recall Validation Complete");
    println!("==============================================");
}

fn test_recall(
    engine: &mut SqlEngine,
    test_name: &str,
    where_clause: &str,
    all_embeddings: &[Vec<f32>],
    num_products: usize,
    dimensions: usize,
    num_queries: usize,
    k: usize,
) {
    println!("--- {} ---", test_name);
    println!("WHERE clause: {}\n", where_clause);

    let mut total_recall = 0.0;
    let mut successful_queries = 0;

    for query_idx in 0..num_queries {
        // Generate query vector
        let query_seed = query_idx * 1000 + 12345;
        let query_embedding = generate_random_vector(dimensions, query_seed);
        let query_str = format!(
            "[{}]",
            query_embedding
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );

        // Compute ground truth: naive sequential scan
        // This filters first, then computes exact distances
        let ground_truth_ids = compute_ground_truth(
            where_clause,
            &query_embedding,
            all_embeddings,
            num_products,
            k,
        );

        if ground_truth_ids.is_empty() {
            println!("  Query {}: Skipped (no matching rows)", query_idx + 1);
            continue;
        }

        // Run hybrid search
        let hybrid_sql = format!(
            "SELECT id FROM products WHERE {} ORDER BY embedding <=> '{}' LIMIT {}",
            where_clause, query_str, k
        );

        match engine.execute(&hybrid_sql) {
            Ok(result) => {
                // Extract IDs from result
                let hybrid_ids = extract_ids_from_result(&result);

                if hybrid_ids.is_empty() {
                    println!("  Query {}: No results returned", query_idx + 1);
                    continue;
                }

                // Compute recall
                let recall = compute_recall(&ground_truth_ids, &hybrid_ids, k);
                total_recall += recall;
                successful_queries += 1;

                if (query_idx + 1) % 5 == 0 {
                    println!(
                        "  Query {}: Recall = {:.1}% ({}/{} correct)",
                        query_idx + 1,
                        recall * 100.0,
                        (recall * k as f64) as usize,
                        k
                    );
                }
            }
            Err(e) => {
                println!("  Query {}: Error - {}", query_idx + 1, e);
            }
        }
    }

    if successful_queries > 0 {
        let avg_recall = total_recall / successful_queries as f64;
        println!("\n{} Results:", test_name);
        println!("  Successful queries: {}/{}", successful_queries, num_queries);
        println!("  Average recall@{}: {:.2}%", k, avg_recall * 100.0);

        if avg_recall >= 0.90 {
            println!("  ✅ PASS: Recall >= 90% target");
        } else {
            println!("  ⚠️  WARN: Recall < 90% target");
        }
    } else {
        println!("\n{}: No successful queries", test_name);
    }

    println!();
}

/// Compute ground truth by naive filtering + exact distance computation
fn compute_ground_truth(
    where_clause: &str,
    query: &[f32],
    all_embeddings: &[Vec<f32>],
    num_products: usize,
    k: usize,
) -> Vec<i64> {
    let mut candidates = Vec::new();

    // Simple filter evaluation (for benchmark purposes)
    // In production, would use proper SQL parser
    for id in 0..num_products {
        if matches_filter(id, where_clause, num_products) {
            let distance = l2_distance(query, &all_embeddings[id]);
            candidates.push((id as i64, distance));
        }
    }

    // Sort by distance
    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    // Return top-k IDs
    candidates.iter().take(k).map(|(id, _)| *id).collect()
}

/// Simple filter matching (for benchmark purposes)
fn matches_filter(id: usize, where_clause: &str, num_products: usize) -> bool {
    let categories = vec!["electronics", "clothing", "books", "home", "toys"];
    let category = categories[id % categories.len()];
    let price = 10.0 + (id % 1000) as f64;

    if where_clause.contains("category = 'electronics'") {
        category == "electronics"
    } else if where_clause.contains("price >= 200.0 AND price <= 700.0") {
        price >= 200.0 && price <= 700.0
    } else if where_clause.contains("price > 100.0") {
        price > 100.0
    } else {
        true
    }
}

/// Compute L2 distance between two vectors
fn l2_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

/// Extract IDs from ExecutionResult
fn extract_ids_from_result(result: &omen::sql_engine::ExecutionResult) -> Vec<i64> {
    use omen::sql_engine::ExecutionResult;
    use omen::value::Value;

    match result {
        ExecutionResult::Selected { data, columns, .. } => {
            // Find the "id" column index
            if let Some(id_idx) = columns.iter().position(|col| col == "id") {
                data.iter()
                    .filter_map(|row| {
                        if let Ok(Value::Int64(id)) = row.get(id_idx) {
                            Some(*id)
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    }
}
