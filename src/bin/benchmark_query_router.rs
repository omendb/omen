//! Benchmark for query router performance
//!
//! Validates:
//! - Routing decision overhead (<50ns target)
//! - Correct routing for different query types
//! - Metrics accuracy

use datafusion::logical_expr::{col, lit, BinaryExpr, Between, Operator};
use omendb::cost_estimator::ExecutionPath;
use omendb::query_router::QueryRouter;
use std::time::Instant;

fn main() {
    println!("=== Query Router Performance Benchmark ===\n");

    let router = QueryRouter::new("id".to_string(), 1_000_000);

    // Benchmark 1: Point query routing
    println!("1. Point Query Routing");
    println!("   Query: WHERE id = 42");
    let point_filter = datafusion::logical_expr::Expr::BinaryExpr(BinaryExpr {
        left: Box::new(col("id")),
        op: Operator::Eq,
        right: Box::new(lit(42i64)),
    });

    let iterations = 100_000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _decision = router.route(&[point_filter.clone()]);
    }
    let elapsed = start.elapsed();
    let avg_ns = elapsed.as_nanos() / iterations;

    println!("   Iterations: {}", iterations);
    println!("   Total time: {:?}", elapsed);
    println!("   Avg routing time: {} ns", avg_ns);
    println!("   Target: <50 ns");
    println!(
        "   Status: {}",
        if avg_ns < 50 { "✅ PASS" } else { "❌ FAIL" }
    );
    println!();

    // Benchmark 2: Small range query routing
    println!("2. Small Range Query Routing");
    println!("   Query: WHERE id BETWEEN 100 AND 150 (50 rows)");
    let small_range_filter = datafusion::logical_expr::Expr::Between(Between {
        expr: Box::new(col("id")),
        negated: false,
        low: Box::new(lit(100i64)),
        high: Box::new(lit(150i64)),
    });

    let start = Instant::now();
    for _ in 0..iterations {
        let _decision = router.route(&[small_range_filter.clone()]);
    }
    let elapsed = start.elapsed();
    let avg_ns = elapsed.as_nanos() / iterations;

    println!("   Iterations: {}", iterations);
    println!("   Total time: {:?}", elapsed);
    println!("   Avg routing time: {} ns", avg_ns);
    println!(
        "   Routed to: {:?}",
        router.route(&[small_range_filter.clone()]).execution_path
    );
    println!();

    // Benchmark 3: Large range query routing
    println!("3. Large Range Query Routing");
    println!("   Query: WHERE id BETWEEN 100 AND 1100 (1000 rows)");
    let large_range_filter = datafusion::logical_expr::Expr::Between(Between {
        expr: Box::new(col("id")),
        negated: false,
        low: Box::new(lit(100i64)),
        high: Box::new(lit(1100i64)),
    });

    let start = Instant::now();
    for _ in 0..iterations {
        let _decision = router.route(&[large_range_filter.clone()]);
    }
    let elapsed = start.elapsed();
    let avg_ns = elapsed.as_nanos() / iterations;

    println!("   Iterations: {}", iterations);
    println!("   Total time: {:?}", elapsed);
    println!("   Avg routing time: {} ns", avg_ns);
    println!(
        "   Routed to: {:?}",
        router.route(&[large_range_filter.clone()]).execution_path
    );
    println!();

    // Benchmark 4: Full scan routing
    println!("4. Full Scan Routing");
    println!("   Query: WHERE name = 'Alice' (non-PK)");
    let full_scan_filter = datafusion::logical_expr::Expr::BinaryExpr(BinaryExpr {
        left: Box::new(col("name")),
        op: Operator::Eq,
        right: Box::new(lit("Alice")),
    });

    let start = Instant::now();
    for _ in 0..iterations {
        let _decision = router.route(&[full_scan_filter.clone()]);
    }
    let elapsed = start.elapsed();
    let avg_ns = elapsed.as_nanos() / iterations;

    println!("   Iterations: {}", iterations);
    println!("   Total time: {:?}", elapsed);
    println!("   Avg routing time: {} ns", avg_ns);
    println!(
        "   Routed to: {:?}",
        router.route(&[full_scan_filter.clone()]).execution_path
    );
    println!();

    // Report metrics
    println!("=== Routing Metrics ===\n");
    let metrics = router.metrics();
    println!(
        "Total queries routed: {}",
        metrics
            .total_queries
            .load(std::sync::atomic::Ordering::Relaxed)
    );
    println!(
        "Routed to ALEX: {}",
        metrics.alex_routed.load(std::sync::atomic::Ordering::Relaxed)
    );
    println!(
        "Routed to DataFusion: {}",
        metrics
            .datafusion_routed
            .load(std::sync::atomic::Ordering::Relaxed)
    );
    println!("Avg decision time: {} ns", metrics.avg_decision_time_ns());

    let (alex_ratio, df_ratio) = metrics.routing_ratio();
    println!(
        "Routing ratio: {:.1}% ALEX, {:.1}% DataFusion",
        alex_ratio * 100.0,
        df_ratio * 100.0
    );
    println!();

    // Verify routing decisions
    println!("=== Routing Decision Validation ===\n");

    let point_decision = router.route(&[point_filter]);
    println!("Point query (id = 42):");
    println!("  Path: {:?}", point_decision.execution_path);
    println!("  Expected: AlexIndex");
    println!(
        "  Status: {}",
        if point_decision.execution_path == ExecutionPath::AlexIndex {
            "✅ PASS"
        } else {
            "❌ FAIL"
        }
    );
    println!();

    let small_range_decision = router.route(&[small_range_filter]);
    println!("Small range (50 rows):");
    println!("  Path: {:?}", small_range_decision.execution_path);
    println!("  Expected: AlexIndex");
    println!(
        "  Status: {}",
        if small_range_decision.execution_path == ExecutionPath::AlexIndex {
            "✅ PASS"
        } else {
            "❌ FAIL"
        }
    );
    println!();

    let large_range_decision = router.route(&[large_range_filter]);
    println!("Large range (1000 rows):");
    println!("  Path: {:?}", large_range_decision.execution_path);
    println!("  Expected: DataFusion");
    println!(
        "  Status: {}",
        if large_range_decision.execution_path == ExecutionPath::DataFusion {
            "✅ PASS"
        } else {
            "❌ FAIL"
        }
    );
    println!();

    let full_scan_decision = router.route(&[full_scan_filter]);
    println!("Full scan (non-PK filter):");
    println!("  Path: {:?}", full_scan_decision.execution_path);
    println!("  Expected: DataFusion");
    println!(
        "  Status: {}",
        if full_scan_decision.execution_path == ExecutionPath::DataFusion {
            "✅ PASS"
        } else {
            "❌ FAIL"
        }
    );
    println!();

    println!("=== Benchmark Complete ===");
}
