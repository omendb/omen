//! OmenDB Integration Test Runner
//! Run comprehensive integration tests for production validation

use omendb::integration_tests::*;
use std::env;
use std::process;

#[tokio::main]
async fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                  OMENDB INTEGRATION TESTING                 ║");
    println!("║                 Production Readiness Validation             ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let test_name = &args[1];
        run_single_test(test_name).await;
    } else {
        run_all_tests().await;
    }
}

async fn run_all_tests() {
    println!("\n🎯 RUNNING ALL INTEGRATION TESTS");
    println!("================================");

    let results = run_all_integration_tests().await;

    let total = results.len();
    let passed = results.iter().filter(|r| r.success).count();
    let failed = total - passed;

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("📊 FINAL RESULTS");
    println!("═══════════════════════════════════════════════════════════════");

    if passed == total {
        println!("✅ ALL {} INTEGRATION TESTS PASSED!", total);
        println!("🎉 OmenDB is production ready for integration workflows!");
    } else {
        println!("❌ {}/{} TESTS FAILED", failed, total);
        println!("🔧 Integration issues need attention before production deployment");
    }

    // Print detailed results
    println!("\nDetailed Results:");
    for result in &results {
        let status = if result.success { "✅ PASS" } else { "❌ FAIL" };
        println!("  {} {}: {:.2}s, {} ops, {} errors",
                status,
                result.test_name,
                result.duration.as_secs_f64(),
                result.operations_completed,
                result.errors_encountered);
    }

    // Performance summary
    let total_ops = results.iter().map(|r| r.operations_completed).sum::<usize>();
    let total_duration = results.iter().map(|r| r.duration.as_secs_f64()).sum::<f64>();
    let overall_throughput = total_ops as f64 / total_duration;

    println!("\n📈 Performance Summary:");
    println!("  Total Operations: {}", total_ops);
    println!("  Total Duration: {:.2}s", total_duration);
    println!("  Overall Throughput: {:.0} ops/sec", overall_throughput);

    // Calculate average performance metrics
    let avg_insert_rate = results.iter()
        .map(|r| r.performance_metrics.insert_rate)
        .filter(|&rate| rate > 0.0)
        .sum::<f64>() / results.iter().filter(|r| r.performance_metrics.insert_rate > 0.0).count() as f64;

    let avg_query_latency = results.iter()
        .map(|r| r.performance_metrics.query_latency_avg)
        .filter(|&lat| lat > 0.0)
        .sum::<f64>() / results.iter().filter(|r| r.performance_metrics.query_latency_avg > 0.0).count() as f64;

    if !avg_insert_rate.is_nan() {
        println!("  Average Insert Rate: {:.0} records/sec", avg_insert_rate);
    }
    if !avg_query_latency.is_nan() {
        println!("  Average Query Latency: {:.2}ms", avg_query_latency);
    }

    // Exit with error code if any tests failed
    if failed > 0 {
        process::exit(1);
    }
}

async fn run_single_test(test_name: &str) {
    println!("\n🎯 RUNNING SINGLE INTEGRATION TEST: {}", test_name.to_uppercase());
    println!("═══════════════════════════════════════════════════════════════");

    let result = match test_name {
        "lifecycle" | "database_lifecycle" => {
            println!("🔄 Testing database lifecycle...");
            test_database_lifecycle().await
        }
        "metrics" | "metrics_integration" => {
            println!("🔄 Testing metrics integration...");
            test_metrics_integration().await
        }
        "wal" | "wal_persistence" => {
            println!("🔄 Testing WAL persistence...");
            test_wal_persistence_integration().await
        }
        "concurrent" | "concurrent_operations" => {
            println!("🔄 Testing concurrent operations...");
            test_concurrent_operations_integration().await
        }
        "http" | "http_server" => {
            println!("🔄 Testing HTTP server integration...");
            test_http_server_integration().await
        }
        "e2e" | "end_to_end" | "complete" => {
            println!("🔄 Testing complete end-to-end...");
            test_complete_end_to_end().await
        }
        _ => {
            println!("❌ Unknown test: {}", test_name);
            println!("\nAvailable tests:");
            println!("  lifecycle    - Database lifecycle test");
            println!("  metrics      - Metrics integration test");
            println!("  wal          - WAL persistence test");
            println!("  concurrent   - Concurrent operations test");
            println!("  http         - HTTP server integration test");
            println!("  e2e          - Complete end-to-end test");
            process::exit(1);
        }
    };

    println!("\n📊 TEST RESULT: {}", test_name.to_uppercase());
    println!("═══════════════════════════════════════════════════════════════");

    if result.success {
        println!("✅ PASSED!");
        println!("⏱️  Duration: {:.2}s", result.duration.as_secs_f64());
        println!("🔢 Operations: {}", result.operations_completed);
        println!("❌ Errors: {}", result.errors_encountered);

        // Performance details
        println!("\n📈 Performance Metrics:");
        if result.performance_metrics.insert_rate > 0.0 {
            println!("  Insert Rate: {:.0} records/sec", result.performance_metrics.insert_rate);
        }
        if result.performance_metrics.query_latency_avg > 0.0 {
            println!("  Query Latency (avg): {:.2}ms", result.performance_metrics.query_latency_avg);
            println!("  Query Latency (P95): {:.2}ms", result.performance_metrics.query_latency_p95);
        }
        println!("  Memory Usage: {:.1}MB", result.performance_metrics.memory_usage_mb);

    } else {
        println!("❌ FAILED!");
        println!("⏱️  Duration: {:.2}s", result.duration.as_secs_f64());
        println!("🔢 Operations: {}", result.operations_completed);
        println!("❌ Errors: {} (CRITICAL)", result.errors_encountered);

        println!("\n🔧 This test must pass before production deployment!");
        process::exit(1);
    }
}