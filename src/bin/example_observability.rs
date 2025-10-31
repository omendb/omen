/// Example: Observability Features
///
/// Demonstrates the logging, metrics, and stats API added in Week 11 Day 2.
///
/// Features:
/// - Structured logging with tracing
/// - Performance metrics (insert/search latency)
/// - Index statistics API (memory usage, neighbors, levels)
/// - Error logging with context

use omen::vector::custom_hnsw::{DistanceFunction, HNSWIndex, HNSWParams};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure tracing subscriber for structured logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO) // Set to DEBUG for more detailed logs
        .with_target(false) // Don't show target module names
        .compact() // Use compact format
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    info!("Starting observability example");

    // Create HNSW index
    let params = HNSWParams::default();
    let mut index = HNSWIndex::new(128, params, DistanceFunction::L2, false)?;

    info!("Index created with 128 dimensions");

    // Insert vectors (logging happens automatically via #[instrument])
    info!("Inserting 1000 vectors...");
    let start = std::time::Instant::now();

    for i in 0..1000 {
        let vector: Vec<f32> = (0..128).map(|j| (i * j) as f32 / 100.0).collect();
        index.insert(vector)?;

        // Progress logging every 100 vectors
        if (i + 1) % 100 == 0 {
            info!(
                inserted = i + 1,
                elapsed_ms = start.elapsed().as_millis(),
                "Insert progress"
            );
        }
    }

    let insert_duration = start.elapsed();
    info!(
        total_inserts = 1000,
        duration_ms = insert_duration.as_millis(),
        rate_per_sec = 1000.0 / insert_duration.as_secs_f64(),
        "All vectors inserted"
    );

    // Get and display index statistics
    info!("Retrieving index statistics...");
    let stats = index.stats();

    println!("\n=== Index Statistics ===");
    println!("Total vectors: {}", stats.num_vectors);
    println!("Dimensions: {}", stats.dimensions);
    println!("Entry point: {:?}", stats.entry_point);
    println!("Max level: {}", stats.max_level);
    println!("Average neighbors (L0): {:.2}", stats.avg_neighbors_l0);
    println!("Max neighbors (L0): {}", stats.max_neighbors_l0);
    println!(
        "Memory usage: {:.2} MB",
        stats.memory_bytes as f64 / (1024.0 * 1024.0)
    );
    println!("Quantization: {}", stats.quantization_enabled);
    println!("Distance function: {:?}", stats.distance_function);

    println!("\n=== Level Distribution ===");
    for (level, count) in stats.level_distribution.iter().enumerate() {
        if *count > 0 {
            println!("Level {}: {} nodes", level, count);
        }
    }

    println!("\n=== HNSW Parameters ===");
    println!("M (connections per layer): {}", stats.params.m);
    println!("ef_construction: {}", stats.params.ef_construction);
    println!("max_level: {}", stats.params.max_level);

    // Perform searches (logging happens automatically)
    info!("Performing search operations...");
    let search_start = std::time::Instant::now();
    let num_searches = 100;

    let mut total_results = 0;
    for i in 0..num_searches {
        let query: Vec<f32> = (0..128).map(|j| ((i * 5 + j) as f32) / 100.0).collect();
        let results = index.search(&query, 10, 50)?;
        total_results += results.len();
    }

    let search_duration = search_start.elapsed();
    info!(
        num_searches,
        total_results,
        duration_ms = search_duration.as_millis(),
        avg_latency_ms = search_duration.as_millis() as f64 / num_searches as f64,
        qps = num_searches as f64 / search_duration.as_secs_f64(),
        "Search operations completed"
    );

    println!("\n=== Search Performance ===");
    println!("Total searches: {}", num_searches);
    println!("Total results: {}", total_results);
    println!("Duration: {} ms", search_duration.as_millis());
    println!(
        "Average latency: {:.2} ms",
        search_duration.as_millis() as f64 / num_searches as f64
    );
    println!(
        "Queries per second (QPS): {:.2}",
        num_searches as f64 / search_duration.as_secs_f64()
    );

    // Demonstrate save/load with logging
    info!("Testing persistence...");
    let temp_path = std::env::temp_dir().join("observability_example_index.bin");

    info!(path = ?temp_path, "Saving index to disk");
    index.save(&temp_path)?;

    info!(path = ?temp_path, "Loading index from disk");
    let loaded_index = HNSWIndex::load(&temp_path)?;

    // Verify loaded index
    let loaded_stats = loaded_index.stats();
    assert_eq!(loaded_stats.num_vectors, stats.num_vectors);
    info!("Index loaded and verified successfully");

    // Clean up
    std::fs::remove_file(temp_path)?;

    info!("Observability example completed successfully");

    Ok(())
}
