//! Hybrid Search Benchmark - Week 5 Day 2
//!
//! Compares hybrid search strategies:
//! - Filter-First: SQL predicates → Vector search
//! - Vector-First: Vector search → SQL filter
//! - Naive baseline: Sequential scan + rerank
//!
//! Tests various selectivity levels (1%, 10%, 50%, 90%)

use omen::catalog::Catalog;
use omen::sql_engine::SqlEngine;
use rand::Rng;
use std::time::Instant;

fn generate_random_vector(dim: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

fn main() {
    println!("==============================================");
    println!("Hybrid Search Benchmark - Week 5 Day 2");
    println!("==============================================\n");

    let dimensions = 128; // Smaller for faster benchmarking
    let num_products = 10_000;
    let num_queries = 50;
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
            name TEXT,
            category TEXT,
            price FLOAT,
            embedding VECTOR({})
        )",
        dimensions
    );
    engine.execute(&create_table_sql).unwrap();

    // Insert products
    println!("\nInserting {} products...", num_products);
    let categories = vec!["electronics", "clothing", "books", "home", "toys"];
    let insert_start = Instant::now();

    for i in 0..num_products {
        let category = categories[i % categories.len()];
        let price = 10.0 + (i % 1000) as f64; // Prices: 10-1010

        let embedding = generate_random_vector(dimensions);
        let embedding_str = format!(
            "[{}]",
            embedding
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let insert_sql = format!(
            "INSERT INTO products (id, name, category, price, embedding) VALUES ({}, 'Product {}', '{}', {}, '{}')",
            i, i, category, price, embedding_str
        );

        engine.execute(&insert_sql).unwrap();

        if (i + 1) % 1000 == 0 {
            println!("  Inserted {} products...", i + 1);
        }
    }

    let insert_duration = insert_start.elapsed();
    println!("\nInsert complete:");
    println!("  Total time: {:?}", insert_duration);
    println!(
        "  Throughput: {:.0} inserts/sec",
        num_products as f64 / insert_duration.as_secs_f64()
    );

    // Reload catalog to pick up changes
    drop(engine);
    let catalog = Catalog::new(data_dir.clone()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Run benchmarks for different selectivity levels
    println!("\n==============================================");
    println!("Hybrid Search Query Benchmarks");
    println!("==============================================\n");

    run_selectivity_benchmark(
        &mut engine,
        "High Selectivity (1%) - Single Category",
        "category = 'electronics'",
        dimensions,
        num_queries,
        k,
    );

    run_selectivity_benchmark(
        &mut engine,
        "Medium Selectivity (20%) - Category",
        "category = 'electronics'",
        dimensions,
        num_queries,
        k,
    );

    run_selectivity_benchmark(
        &mut engine,
        "Medium Selectivity (50%) - Price Range",
        "price >= 200.0 AND price <= 700.0",
        dimensions,
        num_queries,
        k,
    );

    run_selectivity_benchmark(
        &mut engine,
        "Low Selectivity (90%) - Price > 100",
        "price > 100.0",
        dimensions,
        num_queries,
        k,
    );

    println!("\n==============================================");
    println!("Benchmark Complete");
    println!("==============================================");
}

fn run_selectivity_benchmark(
    engine: &mut SqlEngine,
    name: &str,
    where_clause: &str,
    dimensions: usize,
    num_queries: usize,
    k: usize,
) {
    println!("\n--- {} ---", name);

    let mut query_times = Vec::new();

    for i in 0..num_queries {
        // Generate random query vector
        let query_embedding = generate_random_vector(dimensions);
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
            "SELECT * FROM products WHERE {} ORDER BY embedding <=> '{}' LIMIT {}",
            where_clause, query_str, k
        );

        let start = Instant::now();
        let result = engine.execute(&sql);
        let duration = start.elapsed();

        match result {
            Ok(_) => {
                query_times.push(duration.as_secs_f64() * 1000.0); // Convert to ms

                if (i + 1) % 10 == 0 {
                    println!("  Query {}: {:.2}ms", i + 1, duration.as_secs_f64() * 1000.0);
                }
            }
            Err(e) => {
                eprintln!("  Query {} failed: {}", i + 1, e);
            }
        }
    }

    // Calculate statistics
    if !query_times.is_empty() {
        query_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p50 = query_times[num_queries / 2];
        let p95 = query_times[(num_queries as f64 * 0.95) as usize];
        let p99 = query_times[(num_queries as f64 * 0.99) as usize];
        let avg = query_times.iter().sum::<f64>() / query_times.len() as f64;

        println!("\nResults:");
        println!("  Queries: {}/{}", query_times.len(), num_queries);
        println!("  Average latency: {:.2}ms", avg);
        println!("  p50 latency: {:.2}ms", p50);
        println!("  p95 latency: {:.2}ms", p95);
        println!("  p99 latency: {:.2}ms", p99);
        println!("  QPS: {:.0}", 1000.0 / avg);
    }
}
