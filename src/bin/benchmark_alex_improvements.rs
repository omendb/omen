//! Benchmark Phase 8 ALEX improvements
//!
//! Validates optimizations from SOTA improvements (2025):
//! - Phase 8.1: std::simd SIMD search (2-4x query speedup)
//! - Phase 8.2: Exponential search (already implemented)
//! - Phase 8.4: CDFShop sampling (10-100x index building speedup)
//!
//! Compares:
//! 1. Index building time (full vs sampled training)
//! 2. Query performance (scalar vs SIMD search)
//!
//! Run with: cargo run --release --bin benchmark_alex_improvements

use omendb::alex::linear_model::LinearModel;
use rand::{Rng, SeedableRng};
use std::hint::black_box;
use std::time::Instant;

fn main() {
    println!("=== ALEX Phase 8 Improvements Benchmark ===\n");

    benchmark_index_building();
    println!();
    benchmark_query_performance();
}

fn benchmark_index_building() {
    println!("1. Index Building (CDFShop Sampling)\n");

    for scale in [10_000, 100_000, 1_000_000] {
        println!("  Scale: {} keys", format_number(scale));

        // Generate sorted data (typical ALEX use case)
        let data: Vec<(i64, usize)> = (0..scale).map(|i| (i as i64, i as usize)).collect();

        // Run multiple iterations for stable measurements
        let iterations = if scale >= 1_000_000 { 10 } else if scale >= 100_000 { 100 } else { 1000 };

        // Benchmark full training
        let start = Instant::now();
        for _ in 0..iterations {
            let mut full_model = LinearModel::new();
            full_model.train_full(&data);
            black_box(&full_model); // Prevent optimization
        }
        let full_time = start.elapsed();

        // Benchmark sampled training (CDFShop)
        let sample_size = (scale as f64).sqrt() as usize;
        let start = Instant::now();
        for _ in 0..iterations {
            let mut sampled_model = LinearModel::new();
            sampled_model.train_sampled(&data, sample_size);
            black_box(&sampled_model); // Prevent optimization
        }
        let sampled_time = start.elapsed();

        // Validation (single run)
        let mut sampled_model = LinearModel::new();
        sampled_model.train_sampled(&data, sample_size);

        // Validate accuracy
        let max_error = (0..scale)
            .step_by(scale / 100)
            .map(|i| {
                let predicted = sampled_model.predict(i as i64);
                (predicted as i64 - i as i64).abs()
            })
            .max()
            .unwrap_or(0);

        let speedup = full_time.as_secs_f64() / sampled_time.as_secs_f64();
        let full_per_iter = full_time.as_nanos() / iterations as u128;
        let sampled_per_iter = sampled_time.as_nanos() / iterations as u128;

        println!("    Full training:    {:>8}ns/iter ({} iters)", full_per_iter, iterations);
        println!("    Sampled training: {:>8}ns/iter ({} samples)",
                 sampled_per_iter, format_number(sample_size));
        println!("    Speedup:          {:>8.2}x", speedup);
        println!("    Max error:        {} positions", max_error);
        println!();
    }
}

fn benchmark_query_performance() {
    println!("2. Query Performance (SIMD vs Scalar)\n");

    #[cfg(not(feature = "simd"))]
    println!("  NOTE: SIMD feature not enabled - both using scalar implementation");
    println!("        Enable with: cargo run --release --features simd --bin benchmark_alex_improvements\n");

    use omendb::alex::simd_search;

    for scale in [1_000, 10_000, 100_000] {
        println!("  Array size: {} keys", format_number(scale));

        // Generate gapped array (ALEX data structure)
        let keys: Vec<Option<i64>> = (0..scale)
            .map(|i| {
                if i % 2 == 0 {
                    Some(i as i64)
                } else {
                    None // Gap
                }
            })
            .collect();

        let num_searches = 10_000; // More searches for stable measurements
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let search_keys: Vec<i64> = (0..num_searches)
            .map(|_| (rng.gen::<usize>() % (scale / 2)) as i64 * 2) // Search for actual keys
            .collect();

        // Warm up
        for &key in &search_keys[..100] {
            let _ = simd_search::scalar_search_exact(&keys, key);
            let _ = simd_search::simd_search_exact(&keys, key);
        }

        // Benchmark scalar search
        let start = Instant::now();
        for &key in &search_keys {
            let result = simd_search::scalar_search_exact(&keys, key);
            black_box(result); // Prevent optimization
        }
        let scalar_time = start.elapsed();

        // Benchmark SIMD search
        let start = Instant::now();
        for &key in &search_keys {
            let result = simd_search::simd_search_exact(&keys, key);
            black_box(result); // Prevent optimization
        }
        let simd_time = start.elapsed();

        let speedup = scalar_time.as_secs_f64() / simd_time.as_secs_f64();
        let scalar_latency = scalar_time.as_nanos() / num_searches;
        let simd_latency = simd_time.as_nanos() / num_searches;

        println!("    Scalar search:  {:>6}ns/query ({} queries)", scalar_latency, num_searches);
        println!("    SIMD search:    {:>6}ns/query", simd_latency);
        println!("    Speedup:        {:>6.2}x", speedup);
        println!();
    }
}

fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{}M", n / 1_000_000)
    } else if n >= 1_000 {
        format!("{}K", n / 1_000)
    } else {
        n.to_string()
    }
}
