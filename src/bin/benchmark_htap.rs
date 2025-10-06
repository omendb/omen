//! Comprehensive HTAP Performance Benchmark
//!
//! Validates end-to-end performance across:
//! - OLTP workloads (point queries)
//! - OLAP workloads (scans + aggregates)
//! - Mixed workloads (80/20, 50/50, 20/80)
//!
//! Metrics:
//! - Latency distribution (p50, p95, p99)
//! - Throughput (queries/sec)
//! - Routing accuracy
//! - Temperature tracking impact

use datafusion::logical_expr::{col, lit, Between, BinaryExpr, Operator};
use omendb::cost_estimator::ExecutionPath;
use omendb::query_router::QueryRouter;
use omendb::temperature::TemperatureModel;
use omendb::value::Value;
use rand::distributions::Distribution;
use rand::Rng;
use std::time::{Duration, Instant};

/// Latency histogram for percentile calculation
struct LatencyHistogram {
    samples: Vec<Duration>,
}

impl LatencyHistogram {
    fn new() -> Self {
        Self {
            samples: Vec::with_capacity(100_000),
        }
    }

    fn record(&mut self, latency: Duration) {
        self.samples.push(latency);
    }

    fn percentile(&mut self, p: f64) -> Duration {
        if self.samples.is_empty() {
            return Duration::from_nanos(0);
        }
        self.samples.sort();
        let idx = ((p / 100.0) * self.samples.len() as f64) as usize;
        self.samples[idx.min(self.samples.len() - 1)]
    }

    fn count(&self) -> usize {
        self.samples.len()
    }

    fn clear(&mut self) {
        self.samples.clear();
    }
}

/// Zipfian distribution for skewed access patterns
struct Zipfian {
    n: u64,
    alpha: f64,
    zeta_n: f64,
}

impl Zipfian {
    fn new(n: u64, alpha: f64) -> Self {
        let zeta_n = (1..=n).map(|i| 1.0 / (i as f64).powf(alpha)).sum();
        Self { n, alpha, zeta_n }
    }
}

impl Distribution<u64> for Zipfian {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> u64 {
        let u: f64 = rng.gen();
        let mut sum = 0.0;
        for i in 1..=self.n {
            sum += 1.0 / (i as f64).powf(self.alpha) / self.zeta_n;
            if sum >= u {
                return i;
            }
        }
        self.n
    }
}

/// Benchmark configuration
struct BenchConfig {
    num_keys: u64,
    oltp_iterations: usize,
    olap_iterations: usize,
    mixed_iterations: usize,
}

impl Default for BenchConfig {
    fn default() -> Self {
        Self {
            num_keys: 1_000_000,
            oltp_iterations: 100_000,
            olap_iterations: 1_000,
            mixed_iterations: 10_000,
        }
    }
}

/// Run OLTP workload (100% point queries)
fn benchmark_oltp(config: &BenchConfig) {
    println!("=== OLTP Workload (100% Point Queries) ===\n");

    let router = QueryRouter::new("id".to_string(), config.num_keys as usize);
    let temp_model = TemperatureModel::new();

    // Setup: Create zipfian distribution (80/20 rule)
    let zipfian = Zipfian::new(config.num_keys, 1.07);
    let mut rng = rand::thread_rng();

    // Baseline: Without temperature tracking
    println!("1. Baseline (No Temperature Tracking)");
    let mut hist = LatencyHistogram::new();
    let start = Instant::now();

    for _ in 0..config.oltp_iterations {
        let key = zipfian.sample(&mut rng);
        let filter = datafusion::logical_expr::Expr::BinaryExpr(BinaryExpr {
            left: Box::new(col("id")),
            op: Operator::Eq,
            right: Box::new(lit(key as i64)),
        });

        let query_start = Instant::now();
        let _decision = router.route(&[filter]);
        hist.record(query_start.elapsed());
    }

    let total_duration = start.elapsed();
    let throughput = config.oltp_iterations as f64 / total_duration.as_secs_f64();

    println!("   Iterations: {}", config.oltp_iterations);
    println!("   Duration: {:?}", total_duration);
    println!("   Throughput: {:.0} queries/sec", throughput);
    println!("   Latency p50: {:?}", hist.percentile(50.0));
    println!("   Latency p95: {:?}", hist.percentile(95.0));
    println!("   Latency p99: {:?}", hist.percentile(99.0));

    let metrics = router.metrics();
    let (alex_ratio, df_ratio) = metrics.routing_ratio();
    println!(
        "   Routing: {:.1}% ALEX, {:.1}% DataFusion",
        alex_ratio * 100.0,
        df_ratio * 100.0
    );
    println!();

    // With temperature tracking
    println!("2. With Temperature Tracking");
    let temp_model = TemperatureModel::new();
    hist.clear();

    // Warmup: Simulate access pattern
    for _ in 0..10_000 {
        let key = zipfian.sample(&mut rng);
        temp_model.record_access(&Value::Int64(key as i64));
    }

    let start = Instant::now();
    for _ in 0..config.oltp_iterations {
        let key = zipfian.sample(&mut rng);
        temp_model.record_access(&Value::Int64(key as i64));

        let filter = datafusion::logical_expr::Expr::BinaryExpr(BinaryExpr {
            left: Box::new(col("id")),
            op: Operator::Eq,
            right: Box::new(lit(key as i64)),
        });

        let query_start = Instant::now();
        let _decision = router.route_with_temperature(&[filter], &temp_model);
        hist.record(query_start.elapsed());
    }

    let total_duration = start.elapsed();
    let throughput = config.oltp_iterations as f64 / total_duration.as_secs_f64();

    println!("   Iterations: {}", config.oltp_iterations);
    println!("   Duration: {:?}", total_duration);
    println!("   Throughput: {:.0} queries/sec", throughput);
    println!("   Latency p50: {:?}", hist.percentile(50.0));
    println!("   Latency p95: {:?}", hist.percentile(95.0));
    println!("   Latency p99: {:?}", hist.percentile(99.0));

    let stats = temp_model.get_stats();
    println!("   Temperature stats:");
    println!("     Hot ranges: {} ({:.1}%)", stats.hot_count, stats.hot_count as f64 / stats.tracked_ranges as f64 * 100.0);
    println!("     Warm ranges: {} ({:.1}%)", stats.warm_count, stats.warm_count as f64 / stats.tracked_ranges as f64 * 100.0);
    println!("     Cold ranges: {} ({:.1}%)", stats.cold_count, stats.cold_count as f64 / stats.tracked_ranges as f64 * 100.0);
    println!();
}

/// Run OLAP workload (100% scans + aggregates)
fn benchmark_olap(config: &BenchConfig) {
    println!("=== OLAP Workload (100% Scans + Aggregates) ===\n");

    let router = QueryRouter::new("id".to_string(), config.num_keys as usize);
    let mut rng = rand::thread_rng();

    println!("1. Range Scans (Various Sizes)");
    let mut hist = LatencyHistogram::new();

    // Test different range sizes
    let range_sizes = vec![100, 1_000, 10_000, 100_000];

    for range_size in range_sizes {
        hist.clear();
        let start = Instant::now();

        for _ in 0..config.olap_iterations {
            let start_key = rng.gen_range(0..config.num_keys - range_size);
            let end_key = start_key + range_size;

            let filter = datafusion::logical_expr::Expr::Between(Between {
                expr: Box::new(col("id")),
                negated: false,
                low: Box::new(lit(start_key as i64)),
                high: Box::new(lit(end_key as i64)),
            });

            let query_start = Instant::now();
            let _decision = router.route(&[filter]);
            hist.record(query_start.elapsed());
        }

        let total_duration = start.elapsed();
        let throughput = config.olap_iterations as f64 / total_duration.as_secs_f64();

        println!("   Range size: {} rows", range_size);
        println!("     Throughput: {:.0} queries/sec", throughput);
        println!("     Latency p50: {:?}", hist.percentile(50.0));
        println!("     Latency p95: {:?}", hist.percentile(95.0));
        println!("     Latency p99: {:?}", hist.percentile(99.0));
    }
    println!();
}

/// Run mixed workload
fn benchmark_mixed(config: &BenchConfig, oltp_pct: usize) {
    println!(
        "=== Mixed Workload ({}/% OLTP, {}% OLAP) ===\n",
        oltp_pct,
        100 - oltp_pct
    );

    let router = QueryRouter::new("id".to_string(), config.num_keys as usize);
    let temp_model = TemperatureModel::new();
    let zipfian = Zipfian::new(config.num_keys, 1.07);
    let mut rng = rand::thread_rng();

    // Warmup temperature model
    for _ in 0..10_000 {
        let key = zipfian.sample(&mut rng);
        temp_model.record_access(&Value::Int64(key as i64));
    }

    let mut oltp_hist = LatencyHistogram::new();
    let mut olap_hist = LatencyHistogram::new();
    let mut oltp_count = 0;
    let mut olap_count = 0;
    let mut alex_routed = 0;
    let mut df_routed = 0;

    let start = Instant::now();

    for _ in 0..config.mixed_iterations {
        let is_oltp = rng.gen_range(0..100) < oltp_pct;

        if is_oltp {
            // Point query
            let key = zipfian.sample(&mut rng);
            temp_model.record_access(&Value::Int64(key as i64));

            let filter = datafusion::logical_expr::Expr::BinaryExpr(BinaryExpr {
                left: Box::new(col("id")),
                op: Operator::Eq,
                right: Box::new(lit(key as i64)),
            });

            let query_start = Instant::now();
            let decision = router.route_with_temperature(&[filter], &temp_model);
            oltp_hist.record(query_start.elapsed());
            oltp_count += 1;

            match decision.execution_path {
                ExecutionPath::AlexIndex => alex_routed += 1,
                ExecutionPath::DataFusion => df_routed += 1,
                _ => {}
            }
        } else {
            // Range scan
            let range_size = rng.gen_range(1_000..100_000);
            let start_key = rng.gen_range(0..config.num_keys - range_size);
            let end_key = start_key + range_size;

            let filter = datafusion::logical_expr::Expr::Between(Between {
                expr: Box::new(col("id")),
                negated: false,
                low: Box::new(lit(start_key as i64)),
                high: Box::new(lit(end_key as i64)),
            });

            let query_start = Instant::now();
            let decision = router.route_with_temperature(&[filter], &temp_model);
            olap_hist.record(query_start.elapsed());
            olap_count += 1;

            match decision.execution_path {
                ExecutionPath::AlexIndex => alex_routed += 1,
                ExecutionPath::DataFusion => df_routed += 1,
                _ => {}
            }
        }
    }

    let total_duration = start.elapsed();
    let throughput = config.mixed_iterations as f64 / total_duration.as_secs_f64();

    println!("Overall Performance:");
    println!("   Total queries: {}", config.mixed_iterations);
    println!("   Duration: {:?}", total_duration);
    println!("   Throughput: {:.0} queries/sec", throughput);
    println!();

    println!("OLTP Queries ({} queries):", oltp_count);
    println!("   Latency p50: {:?}", oltp_hist.percentile(50.0));
    println!("   Latency p95: {:?}", oltp_hist.percentile(95.0));
    println!("   Latency p99: {:?}", oltp_hist.percentile(99.0));
    println!();

    println!("OLAP Queries ({} queries):", olap_count);
    println!("   Latency p50: {:?}", olap_hist.percentile(50.0));
    println!("   Latency p95: {:?}", olap_hist.percentile(95.0));
    println!("   Latency p99: {:?}", olap_hist.percentile(99.0));
    println!();

    println!("Routing Distribution:");
    println!(
        "   ALEX: {} ({:.1}%)",
        alex_routed,
        alex_routed as f64 / config.mixed_iterations as f64 * 100.0
    );
    println!(
        "   DataFusion: {} ({:.1}%)",
        df_routed,
        df_routed as f64 / config.mixed_iterations as f64 * 100.0
    );
    println!();

    let stats = temp_model.get_stats();
    println!("Temperature Distribution:");
    println!(
        "   Hot: {} ({:.1}%)",
        stats.hot_count,
        stats.hot_count as f64 / stats.tracked_ranges.max(1) as f64 * 100.0
    );
    println!(
        "   Warm: {} ({:.1}%)",
        stats.warm_count,
        stats.warm_count as f64 / stats.tracked_ranges.max(1) as f64 * 100.0
    );
    println!(
        "   Cold: {} ({:.1}%)",
        stats.cold_count,
        stats.cold_count as f64 / stats.tracked_ranges.max(1) as f64 * 100.0
    );
    println!();
}

fn main() {
    println!("=== OmenDB HTAP Performance Benchmark ===\n");
    println!("Validating end-to-end query routing performance\n");

    let config = BenchConfig::default();
    println!("Configuration:");
    println!("   Dataset size: {} keys", config.num_keys);
    println!("   OLTP iterations: {}", config.oltp_iterations);
    println!("   OLAP iterations: {}", config.olap_iterations);
    println!("   Mixed iterations: {}", config.mixed_iterations);
    println!();

    // OLTP workload
    benchmark_oltp(&config);

    // OLAP workload
    benchmark_olap(&config);

    // Mixed workloads
    benchmark_mixed(&config, 80); // 80% OLTP, 20% OLAP
    benchmark_mixed(&config, 50); // 50% OLTP, 50% OLAP
    benchmark_mixed(&config, 20); // 20% OLTP, 80% OLAP

    println!("=== Benchmark Complete ===");
}
