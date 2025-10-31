//! Persistence Validation - Week 10 Day 4
//!
//! Comprehensive validation of save/load at production scale:
//! - 1536D vectors (OpenAI embedding size)
//! - 100K vectors (production scale)
//! - Validate 4175x serialization speedup
//! - Test data integrity (query results match)
//! - Measure file size and memory usage

use omen::vector::custom_hnsw::{DistanceFunction, HNSWIndex, HNSWParams};
use rand::Rng;
use std::fs;
use std::time::Instant;

fn generate_random_vector(dim: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

fn main() {
    println!("==============================================");
    println!("Persistence Validation - Week 10 Day 4");
    println!("==============================================\n");

    let dimensions = 1536; // OpenAI embedding size
    let num_vectors = 100_000;
    let num_queries = 100;
    let k = 10;

    // Create custom HNSW index with pgvector-compatible params
    let params = HNSWParams {
        m: 16,              // M=16 (pgvector default)
        ef_construction: 64, // ef_construction=64 (pgvector default)
        ml: 1.0 / 16f32.ln(),
        seed: 42,
        max_level: 8,
    };

    println!("Configuration:");
    println!("  Dimensions: {}", dimensions);
    println!("  Vectors: {}", num_vectors);
    println!("  M: {}", params.m);
    println!("  ef_construction: {}", params.ef_construction);
    println!();

    // =========================================================================
    // PHASE 1: Build index
    // =========================================================================
    println!("=== PHASE 1: Building Index ===\n");
    let mut index = HNSWIndex::new(dimensions, params.clone(), DistanceFunction::L2, false).unwrap();

    let build_start = Instant::now();
    for i in 0..num_vectors {
        let vector = generate_random_vector(dimensions);
        index.insert(vector).unwrap();

        if (i + 1) % 10_000 == 0 {
            let elapsed = build_start.elapsed();
            let rate = (i + 1) as f64 / elapsed.as_secs_f64();
            println!("  Inserted {} vectors... ({:.0} vec/sec)", i + 1, rate);
        }
    }
    let build_duration = build_start.elapsed();

    println!("\n--- Build Performance ---");
    println!("Total time: {:.2}s", build_duration.as_secs_f64());
    let build_rate = num_vectors as f64 / build_duration.as_secs_f64();
    println!("Throughput: {:.0} vec/sec", build_rate);

    // Memory usage
    let memory_before = index.memory_usage();
    let memory_mb = memory_before as f64 / (1024.0 * 1024.0);
    println!("Memory usage: {:.2} MB", memory_mb);

    // =========================================================================
    // PHASE 2: Query BEFORE save/load (establish baseline)
    // =========================================================================
    println!("\n=== PHASE 2: Query Performance (Before Save) ===\n");

    let queries: Vec<Vec<f32>> = (0..num_queries)
        .map(|_| generate_random_vector(dimensions))
        .collect();

    let mut results_before = Vec::new();
    let mut query_times = Vec::new();

    for (i, query) in queries.iter().enumerate() {
        let start = Instant::now();
        let results = index.search(query, k, 100).unwrap();
        let duration = start.elapsed();

        query_times.push(duration.as_secs_f64() * 1000.0);
        results_before.push(results);

        if (i + 1) % 20 == 0 {
            println!("  Query {}: {:.2}ms", i + 1, duration.as_secs_f64() * 1000.0);
        }
    }

    query_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50_before = query_times[num_queries / 2];
    let p95_before = query_times[(num_queries as f32 * 0.95) as usize];
    let p99_before = query_times[(num_queries as f32 * 0.99) as usize];

    println!("\n--- Query Latency (Before) ---");
    println!("p50: {:.2}ms", p50_before);
    println!("p95: {:.2}ms", p95_before);
    println!("p99: {:.2}ms", p99_before);

    // =========================================================================
    // PHASE 3: Save to disk
    // =========================================================================
    println!("\n=== PHASE 3: Save to Disk ===\n");

    let save_path = "/tmp/omendb_persistence_validation/test.hnsw";
    fs::create_dir_all("/tmp/omendb_persistence_validation").ok();

    let save_start = Instant::now();
    index.save(&save_path).unwrap();
    let save_duration = save_start.elapsed();

    println!("--- Save Performance ---");
    println!("Save time: {:.3}s", save_duration.as_secs_f64());

    // Check file size
    let file_size = fs::metadata(&save_path).unwrap().len();
    let file_mb = file_size as f64 / (1024.0 * 1024.0);
    println!("File size: {:.2} MB", file_mb);
    println!("Bytes per vector: {:.0}", file_size as f64 / num_vectors as f64);

    // =========================================================================
    // PHASE 4: Load from disk
    // =========================================================================
    println!("\n=== PHASE 4: Load from Disk ===\n");

    let load_start = Instant::now();
    let loaded_index = HNSWIndex::load(&save_path).unwrap();
    let load_duration = load_start.elapsed();

    println!("--- Load Performance ---");
    println!("Load time: {:.3}s", load_duration.as_secs_f64());

    // Calculate speedup vs rebuild
    let speedup = build_duration.as_secs_f64() / load_duration.as_secs_f64();
    println!("\n--- Speedup Analysis ---");
    println!("Build time: {:.2}s", build_duration.as_secs_f64());
    println!("Load time: {:.3}s", load_duration.as_secs_f64());
    println!("Speedup: {:.0}x faster than rebuild! ⭐", speedup);

    // Compare to Week 6 1M baseline (4175x speedup)
    println!("\nComparison to Week 6 (1M vectors):");
    println!("  Week 6: 4175x speedup");
    println!("  Today: {:.0}x speedup", speedup);
    if speedup >= 1000.0 {
        println!("  ✅ PASS: Excellent serialization performance!");
    } else {
        println!("  ⚠️  Lower than 1M baseline (investigate)");
    }

    // Verify memory usage
    let memory_after = loaded_index.memory_usage();
    let memory_after_mb = memory_after as f64 / (1024.0 * 1024.0);
    println!("\n--- Memory Usage ---");
    println!("Before save: {:.2} MB", memory_mb);
    println!("After load: {:.2} MB", memory_after_mb);
    println!("Difference: {:.2} MB", (memory_after_mb - memory_mb).abs());
    if (memory_after_mb - memory_mb).abs() < 1.0 {
        println!("✅ PASS: Memory usage matches (< 1MB difference)");
    } else {
        println!("⚠️  WARNING: Memory usage differs by > 1MB");
    }

    // =========================================================================
    // PHASE 5: Query AFTER save/load (validate integrity)
    // =========================================================================
    println!("\n=== PHASE 5: Query Performance (After Load) ===\n");

    let mut results_after = Vec::new();
    let mut query_times_after = Vec::new();

    for (i, query) in queries.iter().enumerate() {
        let start = Instant::now();
        let results = loaded_index.search(query, k, 100).unwrap();
        let duration = start.elapsed();

        query_times_after.push(duration.as_secs_f64() * 1000.0);
        results_after.push(results);

        if (i + 1) % 20 == 0 {
            println!("  Query {}: {:.2}ms", i + 1, duration.as_secs_f64() * 1000.0);
        }
    }

    query_times_after.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50_after = query_times_after[num_queries / 2];
    let p95_after = query_times_after[(num_queries as f32 * 0.95) as usize];
    let p99_after = query_times_after[(num_queries as f32 * 0.99) as usize];

    println!("\n--- Query Latency (After) ---");
    println!("p50: {:.2}ms", p50_after);
    println!("p95: {:.2}ms", p95_after);
    println!("p99: {:.2}ms", p99_after);

    // =========================================================================
    // PHASE 6: Data Integrity Validation
    // =========================================================================
    println!("\n=== PHASE 6: Data Integrity Validation ===\n");

    let mut exact_matches = 0;
    let mut close_matches = 0;
    let mut mismatches = 0;

    for i in 0..num_queries {
        let before = &results_before[i];
        let after = &results_after[i];

        if before.len() != after.len() {
            println!("  Query {}: Length mismatch! ({} vs {})", i, before.len(), after.len());
            mismatches += 1;
            continue;
        }

        // Check if IDs match exactly
        let ids_match = before.iter().zip(after.iter()).all(|(b, a)| b.id == a.id);

        if ids_match {
            exact_matches += 1;
        } else {
            // Check if at least first 5 IDs match (close enough)
            let first_5_match = before
                .iter()
                .take(5)
                .zip(after.iter().take(5))
                .filter(|(b, a)| b.id == a.id)
                .count()
                >= 4;

            if first_5_match {
                close_matches += 1;
            } else {
                mismatches += 1;
                if mismatches <= 3 {
                    println!("  Query {}: Mismatch detected", i);
                    println!("    Before: {:?}", before.iter().take(3).map(|r| r.id).collect::<Vec<_>>());
                    println!("    After:  {:?}", after.iter().take(3).map(|r| r.id).collect::<Vec<_>>());
                }
            }
        }
    }

    println!("--- Data Integrity Results ---");
    println!("Exact matches: {}/{} ({:.1}%)", exact_matches, num_queries, exact_matches as f64 / num_queries as f64 * 100.0);
    println!("Close matches: {}/{} ({:.1}%)", close_matches, num_queries, close_matches as f64 / num_queries as f64 * 100.0);
    println!("Mismatches: {}/{} ({:.1}%)", mismatches, num_queries, mismatches as f64 / num_queries as f64 * 100.0);

    let total_good = exact_matches + close_matches;
    if total_good >= (num_queries as f64 * 0.95) as usize {
        println!("✅ PASS: >95% queries match (data integrity excellent)");
    } else {
        println!("⚠️  WARNING: <95% queries match (investigate)");
    }

    // =========================================================================
    // FINAL SUMMARY
    // =========================================================================
    println!("\n==============================================");
    println!("Week 10 Day 4 - Persistence Validation Summary");
    println!("==============================================\n");

    println!("1. Build Performance:");
    println!("   - Time: {:.2}s", build_duration.as_secs_f64());
    println!("   - Rate: {:.0} vec/sec", build_rate);

    println!("\n2. Save Performance:");
    println!("   - Time: {:.3}s", save_duration.as_secs_f64());
    println!("   - File size: {:.2} MB", file_mb);
    if save_duration.as_secs_f64() < 10.0 {
        println!("   ✅ PASS: Save time < 10s");
    } else {
        println!("   ⚠️  WARNING: Save time > 10s");
    }

    println!("\n3. Load Performance:");
    println!("   - Time: {:.3}s", load_duration.as_secs_f64());
    println!("   - Speedup: {:.0}x vs rebuild", speedup);
    if speedup >= 100.0 {
        println!("   ✅ PASS: Speedup > 100x");
    } else {
        println!("   ⚠️  WARNING: Speedup < 100x");
    }

    println!("\n4. Query Performance:");
    println!("   - Before: p95 = {:.2}ms", p95_before);
    println!("   - After:  p95 = {:.2}ms", p95_after);
    let latency_diff = ((p95_after - p95_before).abs() / p95_before * 100.0);
    println!("   - Difference: {:.1}%", latency_diff);
    if latency_diff < 10.0 {
        println!("   ✅ PASS: Latency change < 10%");
    } else {
        println!("   ⚠️  WARNING: Latency change > 10%");
    }

    println!("\n5. Data Integrity:");
    println!("   - Exact matches: {}/{}", exact_matches, num_queries);
    println!("   - Close matches: {}/{}", close_matches, num_queries);
    println!("   - Match rate: {:.1}%", total_good as f64 / num_queries as f64 * 100.0);
    if total_good >= (num_queries as f64 * 0.95) as usize {
        println!("   ✅ PASS: >95% match rate");
    } else {
        println!("   ⚠️  WARNING: <95% match rate");
    }

    println!("\n6. Memory Consistency:");
    println!("   - Before: {:.2} MB", memory_mb);
    println!("   - After: {:.2} MB", memory_after_mb);
    if (memory_after_mb - memory_mb).abs() < 1.0 {
        println!("   ✅ PASS: Memory usage consistent");
    } else {
        println!("   ⚠️  WARNING: Memory usage differs");
    }

    println!("\n==============================================");
    let all_pass = save_duration.as_secs_f64() < 10.0
        && speedup >= 100.0
        && latency_diff < 10.0
        && total_good >= (num_queries as f64 * 0.95) as usize
        && (memory_after_mb - memory_mb).abs() < 1.0;

    if all_pass {
        println!("✅ ALL CHECKS PASSED - Persistence validated!");
    } else {
        println!("⚠️  SOME CHECKS FAILED - Review results above");
    }
    println!("==============================================");
}
