//! Benchmark for temperature tracking performance
//!
//! Validates:
//! - Temperature calculation overhead
//! - Access pattern tracking efficiency
//! - Temperature-aware routing impact

use datafusion::logical_expr::{col, lit, BinaryExpr, Between, Operator};
use omendb::cost_estimator::ExecutionPath;
use omendb::query_router::QueryRouter;
use omendb::temperature::TemperatureModel;
use omendb::value::Value;
use std::time::Instant;

fn main() {
    println!("=== Temperature Tracking Performance Benchmark ===\n");

    let temp_model = TemperatureModel::with_params(
        300,  // 5 minute window
        1000, // frequency threshold
        0.6,  // alpha
        0.4,  // beta
        0.8,  // hot threshold
        0.3,  // cold threshold
    );

    // Benchmark 1: Access recording
    println!("1. Access Recording Performance");
    let iterations = 100_000;
    let start = Instant::now();
    for i in 0..iterations {
        temp_model.record_access(&Value::Int64(i % 1000));
    }
    let elapsed = start.elapsed();
    let avg_ns = elapsed.as_nanos() / iterations as u128;

    println!("   Iterations: {}", iterations);
    println!("   Total time: {:?}", elapsed);
    println!("   Avg access record time: {} ns", avg_ns);
    println!("   Target: <100 ns");
    println!(
        "   Status: {}",
        if avg_ns < 100 { "✅ PASS" } else { "⚠️  SLOW" }
    );
    println!();

    // Benchmark 2: Temperature calculation
    println!("2. Temperature Calculation Performance");

    // Create hot data
    for _ in 0..150 {
        temp_model.record_access(&Value::Int64(42));
    }

    let iterations = 100_000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _temp = temp_model.get_temperature(&Value::Int64(42));
    }
    let elapsed = start.elapsed();
    let avg_ns = elapsed.as_nanos() / iterations as u128;

    println!("   Iterations: {}", iterations);
    println!("   Total time: {:?}", elapsed);
    println!("   Avg temperature calc time: {} ns", avg_ns);
    println!("   Target: <200 ns");
    println!(
        "   Status: {}",
        if avg_ns < 200 { "✅ PASS" } else { "⚠️  SLOW" }
    );
    println!();

    // Benchmark 3: Temperature-aware routing overhead
    println!("3. Temperature-Aware Routing Overhead");

    let router = QueryRouter::new("id".to_string(), 1_000_000);

    // Create temperature model with some hot data
    let temp_model = TemperatureModel::new();
    for _ in 0..150 {
        temp_model.record_access(&Value::Int64(42));
    }

    let filter = datafusion::logical_expr::Expr::BinaryExpr(BinaryExpr {
        left: Box::new(col("id")),
        op: Operator::Eq,
        right: Box::new(lit(42i64)),
    });

    // Baseline routing (no temperature)
    let iterations = 100_000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _decision = router.route(&[filter.clone()]);
    }
    let baseline_ns = start.elapsed().as_nanos() / iterations as u128;

    // Temperature-aware routing
    let start = Instant::now();
    for _ in 0..iterations {
        let _decision = router.route_with_temperature(&[filter.clone()], &temp_model);
    }
    let temp_aware_ns = start.elapsed().as_nanos() / iterations as u128;

    let overhead_ns = temp_aware_ns.saturating_sub(baseline_ns);
    let overhead_pct = (overhead_ns as f64 / baseline_ns as f64) * 100.0;

    println!("   Baseline routing: {} ns", baseline_ns);
    println!("   Temperature-aware routing: {} ns", temp_aware_ns);
    println!("   Temperature overhead: {} ns ({:.1}%)", overhead_ns, overhead_pct);
    println!("   Target: <100 ns overhead");
    println!(
        "   Status: {}",
        if overhead_ns < 100 { "✅ PASS" } else { "⚠️  HIGH" }
    );
    println!();

    // Benchmark 4: Hot vs Cold routing decisions
    println!("4. Hot vs Cold Data Routing");

    let router = QueryRouter::new("id".to_string(), 1_000_000);
    let temp_model = TemperatureModel::with_params(60, 100, 0.6, 0.4, 0.8, 0.3);

    // Make key 1 hot
    for _ in 0..150 {
        temp_model.record_access(&Value::Int64(1));
    }

    // Key 999 is cold (never accessed)

    let hot_filter = datafusion::logical_expr::Expr::BinaryExpr(BinaryExpr {
        left: Box::new(col("id")),
        op: Operator::Eq,
        right: Box::new(lit(1i64)),
    });

    let cold_filter = datafusion::logical_expr::Expr::BinaryExpr(BinaryExpr {
        left: Box::new(col("id")),
        op: Operator::Eq,
        right: Box::new(lit(999i64)),
    });

    let hot_decision = router.route_with_temperature(&[hot_filter], &temp_model);
    let cold_decision = router.route_with_temperature(&[cold_filter], &temp_model);

    println!("   Hot data (key=1, 150 accesses):");
    println!("     Temperature: {:.2}", temp_model.get_temperature(&Value::Int64(1)));
    println!("     Routed to: {:?}", hot_decision.execution_path);
    println!("     Expected: AlexIndex");
    println!(
        "     Status: {}",
        if hot_decision.execution_path == ExecutionPath::AlexIndex {
            "✅ PASS"
        } else {
            "❌ FAIL"
        }
    );
    println!();

    println!("   Cold data (key=999, 0 accesses):");
    println!("     Temperature: {:.2}", temp_model.get_temperature(&Value::Int64(999)));
    println!("     Routed to: {:?}", cold_decision.execution_path);
    println!("     Expected: DataFusion");
    println!(
        "     Status: {}",
        if cold_decision.execution_path == ExecutionPath::DataFusion {
            "✅ PASS"
        } else {
            "❌ FAIL"
        }
    );
    println!();

    // Benchmark 5: Hot range override
    println!("5. Hot Range Override (Large Range → ALEX)");

    let router = QueryRouter::new("id".to_string(), 1_000_000);
    let temp_model = TemperatureModel::with_params(60, 100, 0.6, 0.4, 0.8, 0.3);

    // Make range 100-1100 hot (1000 rows, normally would go to DataFusion)
    for i in 100..1100 {
        for _ in 0..150 {
            temp_model.record_access(&Value::Int64(i));
        }
    }

    let hot_range_filter = datafusion::logical_expr::Expr::Between(Between {
        expr: Box::new(col("id")),
        negated: false,
        low: Box::new(lit(100i64)),
        high: Box::new(lit(1100i64)),
    });

    // Without temperature (size-based routing)
    let base_decision = router.route(&[hot_range_filter.clone()]);

    // With temperature (hot range override)
    let temp_decision = router.route_with_temperature(&[hot_range_filter], &temp_model);

    println!("   Range: 100-1100 (1000 rows, hot)");
    println!("   Base routing (size): {:?}", base_decision.execution_path);
    println!("   Temperature routing: {:?}", temp_decision.execution_path);
    println!("   Expected: AlexIndex (hot override)");
    println!(
        "   Status: {}",
        if temp_decision.execution_path == ExecutionPath::AlexIndex {
            "✅ PASS"
        } else {
            "❌ FAIL"
        }
    );
    println!();

    // Statistics
    println!("=== Temperature Model Statistics ===\n");
    let stats = temp_model.get_stats();
    println!("Tracked ranges: {}", stats.tracked_ranges);
    println!("Hot ranges: {}", stats.hot_count);
    println!("Warm ranges: {}", stats.warm_count);
    println!("Cold ranges: {}", stats.cold_count);
    println!("Total accesses: {}", stats.total_accesses);
    println!();

    // Metrics
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
    let (alex_ratio, df_ratio) = metrics.routing_ratio();
    println!(
        "Routing ratio: {:.1}% ALEX, {:.1}% DataFusion",
        alex_ratio * 100.0,
        df_ratio * 100.0
    );
    println!();

    println!("=== Benchmark Complete ===");
}
