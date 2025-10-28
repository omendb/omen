//! Standalone scale testing executable for OmenDB
//! Run with: cargo run --release --bin scale_test

use omen::scale_tests::{run_concurrent_stress_test, run_scale_test, ScaleTestConfig};
use std::env;
use std::time::Instant;

fn print_banner() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    OMENDB SCALE TESTING                     ║");
    println!("║                  Production Readiness Validation            ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
}

fn print_config(config: &ScaleTestConfig) {
    println!("📋 TEST CONFIGURATION");
    println!("===================");
    println!("Target Records: {}", format_number(config.target_records));
    println!("Batch Size: {}", format_number(config.batch_size));
    println!("Concurrent Threads: {}", config.concurrent_threads);
    println!("Test Duration Limit: {}s", config.test_duration_secs);
    println!("Memory Limit: {} MB", config.memory_limit_mb);
    println!();
}

/// Format large numbers with thousands separators
fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();

    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }

    result.chars().rev().collect()
}

fn run_production_scale_test() {
    println!("🎯 PRODUCTION SCALE TEST (10M records)");
    println!("=======================================");

    let config = ScaleTestConfig {
        target_records: 10_000_000,
        batch_size: 50_000,
        concurrent_threads: 1,    // Single threaded first
        test_duration_secs: 1800, // 30 minutes max
        memory_limit_mb: 4096,    // 4GB limit
    };

    print_config(&config);

    let start = Instant::now();
    let results = run_scale_test(config);
    let total_time = start.elapsed();

    results.print_summary();
    println!("\n⏱️  Total Test Time: {:.2}s", total_time.as_secs_f64());

    if results.is_production_ready() {
        println!("🎉 READY FOR ENTERPRISE DEPLOYMENT!");
    } else {
        println!("⚠️  NEEDS OPTIMIZATION BEFORE PRODUCTION");
    }
}

fn run_concurrent_scale_test() {
    println!("\n🔥 CONCURRENT STRESS TEST");
    println!("========================");

    let config = ScaleTestConfig {
        target_records: 5_000_000,
        batch_size: 10_000,
        concurrent_threads: 8,
        test_duration_secs: 900, // 15 minutes max
        memory_limit_mb: 2048,
    };

    print_config(&config);

    let start = Instant::now();
    let results = run_concurrent_stress_test(config);
    let total_time = start.elapsed();

    results.print_summary();
    println!("\n⏱️  Total Test Time: {:.2}s", total_time.as_secs_f64());
}

fn run_quick_validation() {
    println!("⚡ QUICK VALIDATION TEST");
    println!("======================");

    let config = ScaleTestConfig {
        target_records: 100_000,
        batch_size: 5_000,
        concurrent_threads: 2,
        test_duration_secs: 60, // 1 minute max
        memory_limit_mb: 512,
    };

    print_config(&config);

    let start = Instant::now();
    let results = run_scale_test(config);
    let total_time = start.elapsed();

    results.print_summary();
    println!("\n⏱️  Total Test Time: {:.2}s", total_time.as_secs_f64());
}

fn print_usage() {
    println!("Usage:");
    println!("  cargo run --release --bin scale_test [TEST_TYPE]");
    println!();
    println!("TEST_TYPES:");
    println!("  quick      - Fast validation (100K records, ~1 min)");
    println!("  production - Full scale test (10M records, ~30 min)");
    println!("  concurrent - Multi-threaded stress test (5M records, ~15 min)");
    println!("  all        - Run all tests in sequence");
    println!();
    println!("Example:");
    println!("  cargo run --release --bin scale_test quick");
    println!("  cargo run --release --bin scale_test production");
}

fn main() {
    print_banner();

    let args: Vec<String> = env::args().collect();

    let test_type = if args.len() > 1 {
        args[1].as_str()
    } else {
        "quick"
    };

    match test_type {
        "quick" => {
            run_quick_validation();
        }
        "production" => {
            run_production_scale_test();
        }
        "concurrent" => {
            run_concurrent_scale_test();
        }
        "all" => {
            run_quick_validation();
            run_concurrent_scale_test();
            run_production_scale_test();
        }
        "help" | "--help" | "-h" => {
            print_usage();
        }
        _ => {
            println!("❌ Unknown test type: {}", test_type);
            println!();
            print_usage();
            std::process::exit(1);
        }
    }

    println!("\n✅ Scale testing complete!");
}
