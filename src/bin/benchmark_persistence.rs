///! Custom HNSW Persistence Testing - Week 11 Day 3
///!
///! CRITICAL validation of save/load functionality:
///! - Test at 100K and 1M scale
///! - Verify graph structure preservation
///! - Verify query results match before/after
///! - Measure save/load performance
///! - Validate NO data corruption

use omen::vector::custom_hnsw::{DistanceFunction, HNSWIndex, HNSWParams};
use std::path::Path;
use std::time::Instant;

fn generate_random_vector(dim: usize, seed: u64) -> Vec<f32> {
    // Deterministic RNG for reproducibility
    let mut rng = seed;
    (0..dim)
        .map(|_| {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            ((rng >> 32) as f32) / (u32::MAX as f32) * 2.0 - 1.0
        })
        .collect()
}

fn test_persistence_at_scale(
    name: &str,
    dimensions: usize,
    num_vectors: usize,
    num_queries: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n==============================================");
    println!("Persistence Test: {}", name);
    println!("==============================================");
    println!("Dimensions: {}, Vectors: {}", dimensions, num_vectors);
    println!();

    let k = 10;
    let params = HNSWParams {
        m: 16,
        ef_construction: 64,
        ml: 1.0 / 16f32.ln(),
        seed: 42,
        max_level: 8,
    };

    // Step 1: Build index
    println!("Step 1: Building index with {} vectors...", num_vectors);
    let build_start = Instant::now();
    let mut index = HNSWIndex::new(dimensions, params, DistanceFunction::L2, false)?;

    for i in 0..num_vectors {
        let vector = generate_random_vector(dimensions, i as u64);
        index.insert(vector)?;

        if (i + 1) % 100_000 == 0 {
            let elapsed = build_start.elapsed().as_secs_f64();
            let rate = (i + 1) as f64 / elapsed;
            println!("  {} vectors inserted ({:.0} vec/sec)", i + 1, rate);
        }
    }

    let build_duration = build_start.elapsed();
    println!(
        "✅ Build complete: {:.2}s ({:.0} vec/sec)\n",
        build_duration.as_secs_f64(),
        num_vectors as f64 / build_duration.as_secs_f64()
    );

    // Step 2: Get baseline stats
    let stats_before = index.stats();
    println!("Stats BEFORE save/load:");
    println!("  Vectors: {}", stats_before.num_vectors);
    println!("  Dimensions: {}", stats_before.dimensions);
    println!("  Max level: {}", stats_before.max_level);
    println!("  Avg neighbors (L0): {:.2}", stats_before.avg_neighbors_l0);
    println!(
        "  Memory: {:.2} MB",
        stats_before.memory_bytes as f64 / (1024.0 * 1024.0)
    );
    println!();

    // Step 3: Run queries BEFORE save/load
    println!("Step 2: Running {} queries BEFORE save/load...", num_queries);
    let query_start = Instant::now();
    let mut results_before = Vec::new();

    for i in 0..num_queries {
        let query = generate_random_vector(dimensions, (num_vectors + i) as u64);
        let results = index.search(&query, k, 100)?;
        results_before.push(results);
    }

    let query_duration = query_start.elapsed();
    let qps_before = num_queries as f64 / query_duration.as_secs_f64();
    let avg_latency_before = query_duration.as_secs_f64() * 1000.0 / num_queries as f64;

    println!(
        "✅ Queries complete: {:.2}ms avg, {:.0} QPS\n",
        avg_latency_before, qps_before
    );

    // Step 4: Save to disk
    let save_path = format!("/tmp/omendb_persistence_test_{}.hnsw", name);
    if Path::new(&save_path).exists() {
        std::fs::remove_file(&save_path)?;
    }

    println!("Step 3: Saving index to disk...");
    let save_start = Instant::now();
    index.save(&save_path)?;
    let save_duration = save_start.elapsed();

    let file_size = std::fs::metadata(&save_path)?.len();
    let size_mb = file_size as f64 / (1024.0 * 1024.0);

    println!("✅ Save complete: {:.3}s", save_duration.as_secs_f64());
    println!("   File size: {:.2} MB", size_mb);
    println!(
        "   Bytes/vector: {:.0}\n",
        file_size as f64 / num_vectors as f64
    );

    // Step 5: Load from disk
    println!("Step 4: Loading index from disk...");
    let load_start = Instant::now();
    let loaded_index = HNSWIndex::load(&save_path)?;
    let load_duration = load_start.elapsed();

    println!("✅ Load complete: {:.3}s\n", load_duration.as_secs_f64());

    // Calculate speedup
    let speedup = build_duration.as_secs_f64() / load_duration.as_secs_f64();
    println!(
        "⚡ Speedup: {:.0}x faster than rebuild\n",
        speedup
    );

    // Step 6: Verify stats match
    let stats_after = loaded_index.stats();
    println!("Stats AFTER save/load:");
    println!("  Vectors: {}", stats_after.num_vectors);
    println!("  Dimensions: {}", stats_after.dimensions);
    println!("  Max level: {}", stats_after.max_level);
    println!("  Avg neighbors (L0): {:.2}", stats_after.avg_neighbors_l0);
    println!(
        "  Memory: {:.2} MB",
        stats_after.memory_bytes as f64 / (1024.0 * 1024.0)
    );
    println!();

    // Verify stats match
    let mut validation_passed = true;

    if stats_before.num_vectors != stats_after.num_vectors {
        println!(
            "❌ FAIL: Vector count mismatch ({} vs {})",
            stats_before.num_vectors, stats_after.num_vectors
        );
        validation_passed = false;
    } else {
        println!("✅ PASS: Vector count preserved");
    }

    if stats_before.dimensions != stats_after.dimensions {
        println!(
            "❌ FAIL: Dimensions mismatch ({} vs {})",
            stats_before.dimensions, stats_after.dimensions
        );
        validation_passed = false;
    } else {
        println!("✅ PASS: Dimensions preserved");
    }

    if stats_before.max_level != stats_after.max_level {
        println!(
            "❌ FAIL: Max level mismatch ({} vs {})",
            stats_before.max_level, stats_after.max_level
        );
        validation_passed = false;
    } else {
        println!("✅ PASS: Max level preserved");
    }

    // Allow small floating point differences in avg neighbors
    let neighbor_diff = (stats_before.avg_neighbors_l0 - stats_after.avg_neighbors_l0).abs();
    if neighbor_diff > 0.1 {
        println!(
            "❌ FAIL: Avg neighbors mismatch ({:.2} vs {:.2})",
            stats_before.avg_neighbors_l0, stats_after.avg_neighbors_l0
        );
        validation_passed = false;
    } else {
        println!("✅ PASS: Avg neighbors preserved");
    }

    // Step 7: Run same queries AFTER save/load
    println!("\nStep 5: Running {} queries AFTER save/load...", num_queries);
    let query_start = Instant::now();
    let mut results_after = Vec::new();

    for i in 0..num_queries {
        let query = generate_random_vector(dimensions, (num_vectors + i) as u64);
        let results = loaded_index.search(&query, k, 100)?;
        results_after.push(results);
    }

    let query_duration = query_start.elapsed();
    let qps_after = num_queries as f64 / query_duration.as_secs_f64();
    let avg_latency_after = query_duration.as_secs_f64() * 1000.0 / num_queries as f64;

    println!(
        "✅ Queries complete: {:.2}ms avg, {:.0} QPS\n",
        avg_latency_after, qps_after
    );

    // Step 8: Verify query results match
    println!("Step 6: Verifying query results match...");
    let mut query_matches = 0;
    let mut total_overlap = 0;

    for (before, after) in results_before.iter().zip(results_after.iter()) {
        // Check if top result is the same
        if before[0].id == after[0].id {
            query_matches += 1;
        }

        // Check overlap in top-k results
        let before_ids: std::collections::HashSet<_> = before.iter().map(|r| r.id).collect();
        let after_ids: std::collections::HashSet<_> = after.iter().map(|r| r.id).collect();
        total_overlap += before_ids.intersection(&after_ids).count();
    }

    let top_match_rate = query_matches as f64 / num_queries as f64;
    let avg_overlap = total_overlap as f64 / (num_queries * k) as f64;

    println!("  Top-1 match rate: {:.1}%", top_match_rate * 100.0);
    println!("  Top-{} overlap: {:.1}%", k, avg_overlap * 100.0);

    if top_match_rate < 0.95 {
        println!(
            "⚠️  WARNING: Top-1 match rate below 95% ({:.1}%)",
            top_match_rate * 100.0
        );
        validation_passed = false;
    } else {
        println!("✅ PASS: Query results match (>95% top-1)");
    }

    if avg_overlap < 0.90 {
        println!(
            "⚠️  WARNING: Top-{} overlap below 90% ({:.1}%)",
            k,
            avg_overlap * 100.0
        );
        validation_passed = false;
    } else {
        println!("✅ PASS: Query overlap high (>90% top-{})", k);
    }

    // Cleanup
    std::fs::remove_file(&save_path)?;

    // Summary
    println!("\n==============================================");
    println!("Summary: {}", name);
    println!("==============================================");
    println!("Build time: {:.2}s", build_duration.as_secs_f64());
    println!("Save time: {:.3}s", save_duration.as_secs_f64());
    println!("Load time: {:.3}s", load_duration.as_secs_f64());
    println!("Speedup: {:.0}x vs rebuild", speedup);
    println!("File size: {:.2} MB", size_mb);
    println!("QPS (before): {:.0}", qps_before);
    println!("QPS (after): {:.0}", qps_after);
    println!("Top-1 match: {:.1}%", top_match_rate * 100.0);
    println!("Top-{} overlap: {:.1}%", k, avg_overlap * 100.0);

    if validation_passed {
        println!("\n✅ ALL VALIDATIONS PASSED");
    } else {
        println!("\n❌ SOME VALIDATIONS FAILED");
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("==============================================");
    println!("Custom HNSW Persistence Testing");
    println!("Week 11 Day 3 - CRITICAL Validation");
    println!("==============================================");

    // Test 1: 100K vectors @ 1536D (OpenAI embeddings)
    test_persistence_at_scale("100K_1536D", 1536, 100_000, 100)?;

    // Test 2: 1M vectors @ 128D (faster iteration)
    test_persistence_at_scale("1M_128D", 128, 1_000_000, 100)?;

    println!("\n==============================================");
    println!("All Persistence Tests Complete!");
    println!("==============================================");

    Ok(())
}
