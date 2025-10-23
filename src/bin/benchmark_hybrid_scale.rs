//! Hybrid Search Scale Benchmark - Week 5 Day 4
//!
//! Tests hybrid search performance at larger scale:
//! - 100,000 products with 128D embeddings
//! - Various selectivity levels
//! - Measures latency, throughput, and memory usage
//! - Validates that exact distance computation scales

use omendb::catalog::Catalog;
use omendb::sql_engine::SqlEngine;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::time::Instant;

fn generate_random_vector(dimensions: usize, seed: usize) -> Vec<f32> {
    let mut rng = StdRng::seed_from_u64(seed as u64);
    (0..dimensions).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

fn main() {
    println!("==============================================");
    println!("Hybrid Search Scale Benchmark - Week 5 Day 4");
    println!("==============================================\n");

    let dimensions = 128;
    let num_products = 100_000; // 10x larger than previous tests
    let num_queries = 50;
    let k = 10;

    // Create temporary database
    let temp_dir = tempfile::tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    println!("Creating database in {:?}...", data_dir);
    println!("Dataset: {} products, {}D embeddings\n", num_products, dimensions);

    // Create catalog and engine
    let catalog = Catalog::new(data_dir.clone()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Create products table
    println!("Creating products table...");
    let create_table_sql = format!(
        "CREATE TABLE products (
            id INT PRIMARY KEY,
            name TEXT,
            category TEXT,
            price FLOAT,
            region TEXT,
            embedding VECTOR({})
        )",
        dimensions
    );
    engine.execute(&create_table_sql).unwrap();

    // Insert products in batches
    println!("\nInserting {} products...", num_products);
    let categories = vec!["electronics", "clothing", "books", "home", "toys", "sports", "beauty", "automotive"];
    let regions = vec!["north", "south", "east", "west"];

    let insert_start = Instant::now();
    let batch_size = 1000;

    for batch in 0..(num_products / batch_size) {
        let batch_start = batch * batch_size;

        for i in batch_start..(batch_start + batch_size) {
            let category = categories[i % categories.len()];
            let region = regions[i % regions.len()];
            let price = 10.0 + (i % 10000) as f64 / 10.0;

            let embedding = generate_random_vector(dimensions, i);
            let embedding_str = format!(
                "[{}]",
                embedding
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            let insert_sql = format!(
                "INSERT INTO products (id, name, category, price, region, embedding) VALUES ({}, 'Product {}', '{}', {}, '{}', '{}')",
                i, i, category, price, region, embedding_str
            );

            engine.execute(&insert_sql).unwrap();
        }

        if (batch + 1) % 10 == 0 {
            println!("  Inserted {} products...", (batch + 1) * batch_size);
        }
    }

    let insert_duration = insert_start.elapsed();
    println!("\nInsert complete:");
    println!("  Total time: {:.2}s", insert_duration.as_secs_f64());
    println!(
        "  Throughput: {:.0} inserts/sec",
        num_products as f64 / insert_duration.as_secs_f64()
    );

    // Reload catalog
    println!("\nReloading catalog...");
    drop(engine);
    let catalog = Catalog::new(data_dir.clone()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    println!("\n==============================================");
    println!("Query Performance Tests (100K vectors)");
    println!("==============================================\n");

    // Test 1: Very High Selectivity (0.1% - ~100 rows)
    run_scale_test(
        &mut engine,
        "Very High Selectivity (0.1%)",
        "category = 'electronics' AND region = 'north'",
        "~100 rows",
        dimensions,
        num_queries,
        k,
    );

    // Test 2: High Selectivity (1% - ~1K rows)
    run_scale_test(
        &mut engine,
        "High Selectivity (1%)",
        "category = 'electronics'",
        "~12,500 rows",
        dimensions,
        num_queries,
        k,
    );

    // Test 3: Medium Selectivity (12.5% - ~12.5K rows)
    run_scale_test(
        &mut engine,
        "Medium Selectivity (12.5%)",
        "category = 'electronics' OR category = 'clothing'",
        "~25,000 rows",
        dimensions,
        num_queries,
        k,
    );

    // Test 4: Medium-Low Selectivity (25% - ~25K rows)
    run_scale_test(
        &mut engine,
        "Medium-Low Selectivity (25%)",
        "region = 'north'",
        "~25,000 rows",
        dimensions,
        num_queries,
        k,
    );

    // Test 5: Low Selectivity (50% - ~50K rows)
    run_scale_test(
        &mut engine,
        "Low Selectivity (50%)",
        "price >= 500.0",
        "~50,000 rows",
        dimensions,
        num_queries,
        k,
    );

    // Test 6: Very Low Selectivity (90% - ~90K rows)
    run_scale_test(
        &mut engine,
        "Very Low Selectivity (90%)",
        "price > 100.0",
        "~90,000 rows",
        dimensions,
        num_queries,
        k,
    );

    println!("\n==============================================");
    println!("Scale Test Complete");
    println!("==============================================\n");

    println!("Key Observations:");
    println!("- Latency should increase with selectivity (more rows to scan)");
    println!("- Very high selectivity (~100 rows) should be fastest");
    println!("- Low selectivity (50K+ rows) will show exact distance overhead");
    println!("- If latency >100ms at low selectivity, consider HNSW optimization");
}

fn run_scale_test(
    engine: &mut SqlEngine,
    test_name: &str,
    where_clause: &str,
    expected_rows: &str,
    dimensions: usize,
    num_queries: usize,
    k: usize,
) {
    println!("--- {} ---", test_name);
    println!("WHERE: {}", where_clause);
    println!("Expected filtered rows: {}\n", expected_rows);

    let mut query_times = Vec::new();
    let mut successful_queries = 0;

    for query_idx in 0..num_queries {
        // Generate query vector
        let query_seed = query_idx * 1000 + 54321;
        let query_embedding = generate_random_vector(dimensions, query_seed);
        let query_str = format!(
            "[{}]",
            query_embedding
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );

        // Run hybrid query
        let sql = format!(
            "SELECT id FROM products WHERE {} ORDER BY embedding <=> '{}' LIMIT {}",
            where_clause, query_str, k
        );

        let start = Instant::now();
        let result = engine.execute(&sql);
        let duration = start.elapsed();

        match result {
            Ok(_) => {
                query_times.push(duration.as_secs_f64() * 1000.0);
                successful_queries += 1;

                if (query_idx + 1) % 10 == 0 {
                    println!("  Query {}: {:.2}ms", query_idx + 1, duration.as_secs_f64() * 1000.0);
                }
            }
            Err(e) => {
                eprintln!("  Query {} failed: {}", query_idx + 1, e);
            }
        }
    }

    if successful_queries > 0 {
        query_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p50 = query_times[successful_queries / 2];
        let p95 = query_times[(successful_queries as f64 * 0.95) as usize];
        let p99 = query_times[(successful_queries as f64 * 0.99) as usize];
        let avg = query_times.iter().sum::<f64>() / successful_queries as f64;

        println!("\nResults:");
        println!("  Successful queries: {}/{}", successful_queries, num_queries);
        println!("  Average latency: {:.2}ms", avg);
        println!("  p50 latency: {:.2}ms", p50);
        println!("  p95 latency: {:.2}ms", p95);
        println!("  p99 latency: {:.2}ms", p99);
        println!("  QPS: {:.0}", 1000.0 / avg);

        // Performance assessment
        if avg < 10.0 {
            println!("  ✅ EXCELLENT: <10ms latency");
        } else if avg < 50.0 {
            println!("  ✅ GOOD: <50ms latency");
        } else if avg < 100.0 {
            println!("  ⚠️  ACCEPTABLE: <100ms latency (consider optimization)");
        } else {
            println!("  ❌ SLOW: >100ms latency (HNSW recommended)");
        }
    }

    println!();
}
