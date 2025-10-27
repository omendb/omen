//! Resource Exhaustion Test Binary
//!
//! Tests OmenDB behavior under extreme resource constraints:
//! - Memory limits (OOM scenarios)
//! - CPU constraints
//! - File descriptor limits
//! - Combined constraints

use omendb::vector::types::Vector;
use omendb::vector::store::VectorStore;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <test_type> [args]", args[0]);
        eprintln!("Test types: memory, cpu, fdlimit, combined");
        process::exit(1);
    }

    let test_type = &args[1];

    let result = match test_type.as_str() {
        "memory" => {
            let limit_mb = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(512);
            test_memory_limit(limit_mb)
        },
        "cpu" => test_cpu_limit(),
        "fdlimit" => test_file_descriptor_limit(),
        "combined" => test_combined_constraints(),
        _ => {
            eprintln!("Unknown test type: {}", test_type);
            process::exit(1);
        }
    };

    match result {
        Ok(()) => {
            println!("✓ Test completed successfully");
            process::exit(0);
        },
        Err(e) => {
            eprintln!("✗ Test failed: {}", e);
            process::exit(1);
        }
    }
}

fn test_memory_limit(limit_mb: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing memory limit: {}MB", limit_mb);

    let dimensions = 128;
    let mut store = VectorStore::new(dimensions);

    // Calculate how many vectors we can safely insert
    // Each vector is roughly: dimensions * 4 bytes + overhead
    let bytes_per_vector = (dimensions * 4) + 64; // +64 for overhead
    let target_mb = (limit_mb as f32 * 0.6) as usize; // Use 60% of limit
    let num_vectors = (target_mb * 1024 * 1024) / bytes_per_vector;

    println!("Target: {} vectors ({}MB / {}MB limit)", num_vectors, target_mb, limit_mb);

    // Insert vectors in batches
    let batch_size = 1000;
    let mut inserted = 0;

    for batch_num in 0..(num_vectors / batch_size) {
        let batch: Vec<Vector> = (0..batch_size)
            .map(|i| {
                let val = ((batch_num * batch_size + i) as f32) % 100.0;
                Vector::new(vec![val; dimensions])
            })
            .collect();

        match store.batch_insert(batch) {
            Ok(_) => {
                inserted += batch_size;
                if batch_num % 10 == 0 {
                    println!("Inserted {} / {} vectors", inserted, num_vectors);
                }
            },
            Err(e) => {
                println!("Insertion failed after {} vectors: {}", inserted, e);
                break;
            }
        }
    }

    println!("Successfully inserted {} vectors under {}MB limit", inserted, limit_mb);

    // Test search
    let query = Vector::new(vec![50.0; dimensions]);
    match store.knn_search(&query, 10) {
        Ok(results) => {
            println!("✓ Search succeeded, returned {} results", results.len());
        },
        Err(e) => {
            println!("✗ Search failed: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

fn test_cpu_limit() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing CPU limit (0.5 cores)");

    let dimensions = 128;
    let mut store = VectorStore::new(dimensions);

    // Insert a moderate dataset
    let num_vectors = 5000;
    let batch: Vec<Vector> = (0..num_vectors)
        .map(|i| Vector::new(vec![(i % 100) as f32; dimensions]))
        .collect();

    println!("Inserting {} vectors...", num_vectors);
    store.batch_insert(batch)?;

    // Perform multiple searches (CPU-intensive)
    println!("Performing searches under CPU constraint...");
    for i in 0..50 {
        let query = Vector::new(vec![(i % 100) as f32; dimensions]);
        store.knn_search(&query, 10)?;
        if i % 10 == 0 {
            println!("Completed {} / 50 searches", i);
        }
    }

    println!("✓ All searches completed under CPU constraint");
    Ok(())
}

fn test_file_descriptor_limit() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing file descriptor limit (100 open files)");

    let dimensions = 128;
    let mut store = VectorStore::new(dimensions);

    // Insert moderate dataset
    let num_vectors = 1000;
    let batch: Vec<Vector> = (0..num_vectors)
        .map(|i| Vector::new(vec![(i % 100) as f32; dimensions]))
        .collect();

    println!("Inserting {} vectors with FD limit...", num_vectors);
    store.batch_insert(batch)?;

    // Test search
    let query = Vector::new(vec![50.0; dimensions]);
    let results = store.knn_search(&query, 10)?;

    println!("✓ Operations completed under FD limit, {} results returned", results.len());
    Ok(())
}

fn test_combined_constraints() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing combined constraints: 256MB RAM + 0.5 CPU + 100 FD");

    let dimensions = 128;
    let mut store = VectorStore::new(dimensions);

    // Conservative dataset for combined constraints
    let num_vectors = 2000;
    let batch_size = 500;

    println!("Inserting {} vectors in batches of {}...", num_vectors, batch_size);

    for batch_num in 0..(num_vectors / batch_size) {
        let batch: Vec<Vector> = (0..batch_size)
            .map(|i| {
                let val = ((batch_num * batch_size + i) % 100) as f32;
                Vector::new(vec![val; dimensions])
            })
            .collect();

        store.batch_insert(batch)?;
        println!("Batch {} / {} inserted", batch_num + 1, num_vectors / batch_size);
    }

    // Test multiple operations
    println!("Performing operations under combined constraints...");
    for i in 0..20 {
        let query = Vector::new(vec![(i % 100) as f32; dimensions]);
        let results = store.knn_search(&query, 5)?;
        if results.is_empty() {
            return Err("Search returned no results".into());
        }
    }

    println!("✓ All operations completed under combined constraints");
    Ok(())
}
