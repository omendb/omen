//! Query router for intelligent HTAP query execution
//!
//! Routes queries to optimal execution path:
//! - Point queries → ALEX learned index (389ns)
//! - Small ranges → ALEX learned index
//! - Large ranges → DataFusion vectorized scan
//! - Aggregates → DataFusion vectorized execution
//!
//! Tracks metrics for routing decisions and performance analysis.

use crate::cost_estimator::{CostEstimator, ExecutionPath};
use crate::query_classifier::{QueryClassifier, QueryType};
use datafusion::logical_expr::Expr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Routing decision with metadata
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    /// Query classification
    pub query_type: QueryType,

    /// Chosen execution path
    pub execution_path: ExecutionPath,

    /// Estimated cost in nanoseconds
    pub estimated_cost_ns: u64,

    /// Time spent making routing decision
    pub decision_time_ns: u64,
}

/// Query router metrics
#[derive(Debug, Default)]
pub struct RouterMetrics {
    /// Total queries routed
    pub total_queries: AtomicU64,

    /// Queries routed to ALEX index
    pub alex_routed: AtomicU64,

    /// Queries routed to DataFusion
    pub datafusion_routed: AtomicU64,

    /// Total decision time (nanoseconds)
    pub total_decision_time_ns: AtomicU64,

    /// Point queries
    pub point_queries: AtomicU64,

    /// Range queries
    pub range_queries: AtomicU64,

    /// Aggregate queries
    pub aggregate_queries: AtomicU64,

    /// Full scans
    pub full_scans: AtomicU64,
}

impl RouterMetrics {
    /// Get average decision time in nanoseconds
    pub fn avg_decision_time_ns(&self) -> u64 {
        let total = self.total_queries.load(Ordering::Relaxed);
        if total == 0 {
            return 0;
        }
        self.total_decision_time_ns.load(Ordering::Relaxed) / total
    }

    /// Get routing ratio (ALEX vs DataFusion)
    pub fn routing_ratio(&self) -> (f64, f64) {
        let total = self.total_queries.load(Ordering::Relaxed) as f64;
        if total == 0.0 {
            return (0.0, 0.0);
        }
        let alex = self.alex_routed.load(Ordering::Relaxed) as f64 / total;
        let df = self.datafusion_routed.load(Ordering::Relaxed) as f64 / total;
        (alex, df)
    }
}

/// Query router for HTAP workloads
pub struct QueryRouter {
    /// Query classifier
    classifier: QueryClassifier,

    /// Cost estimator
    estimator: CostEstimator,

    /// Routing metrics
    metrics: Arc<RouterMetrics>,
}

impl QueryRouter {
    /// Create new query router
    pub fn new(pk_column: String, table_size: usize) -> Self {
        Self {
            classifier: QueryClassifier::new(pk_column),
            estimator: CostEstimator::new(table_size),
            metrics: Arc::new(RouterMetrics::default()),
        }
    }

    /// Create with custom range threshold
    pub fn with_threshold(pk_column: String, table_size: usize, range_threshold: usize) -> Self {
        Self {
            classifier: QueryClassifier::new(pk_column),
            estimator: CostEstimator::with_threshold(table_size, range_threshold),
            metrics: Arc::new(RouterMetrics::default()),
        }
    }

    /// Route query based on filters
    pub fn route(&self, filters: &[Expr]) -> RoutingDecision {
        let start = Instant::now();

        // Classify query
        let query_type = self.classifier.classify_filters(filters);

        // Estimate best execution path
        let execution_path = self.estimator.estimate(&query_type);

        // Estimate cost
        let estimated_cost_ns = self.estimator.estimate_cost_ns(execution_path, &query_type);

        // Record decision time
        let decision_time_ns = start.elapsed().as_nanos() as u64;

        // Update metrics
        self.update_metrics(&query_type, execution_path, decision_time_ns);

        RoutingDecision {
            query_type,
            execution_path,
            estimated_cost_ns,
            decision_time_ns,
        }
    }

    /// Update routing metrics
    fn update_metrics(
        &self,
        query_type: &QueryType,
        execution_path: ExecutionPath,
        decision_time_ns: u64,
    ) {
        self.metrics.total_queries.fetch_add(1, Ordering::Relaxed);
        self.metrics
            .total_decision_time_ns
            .fetch_add(decision_time_ns, Ordering::Relaxed);

        // Update path counters
        match execution_path {
            ExecutionPath::AlexIndex => {
                self.metrics.alex_routed.fetch_add(1, Ordering::Relaxed);
            }
            ExecutionPath::DataFusion => {
                self.metrics
                    .datafusion_routed
                    .fetch_add(1, Ordering::Relaxed);
            }
        }

        // Update query type counters
        match query_type {
            QueryType::PointQuery { .. } => {
                self.metrics.point_queries.fetch_add(1, Ordering::Relaxed);
            }
            QueryType::RangeQuery { .. } => {
                self.metrics.range_queries.fetch_add(1, Ordering::Relaxed);
            }
            QueryType::AggregateQuery => {
                self.metrics
                    .aggregate_queries
                    .fetch_add(1, Ordering::Relaxed);
            }
            QueryType::FullScan => {
                self.metrics.full_scans.fetch_add(1, Ordering::Relaxed);
            }
            QueryType::Complex => {
                // Not tracked separately
            }
        }
    }

    /// Get routing metrics
    pub fn metrics(&self) -> Arc<RouterMetrics> {
        self.metrics.clone()
    }

    /// Reset metrics
    pub fn reset_metrics(&self) {
        self.metrics.total_queries.store(0, Ordering::Relaxed);
        self.metrics.alex_routed.store(0, Ordering::Relaxed);
        self.metrics
            .datafusion_routed
            .store(0, Ordering::Relaxed);
        self.metrics
            .total_decision_time_ns
            .store(0, Ordering::Relaxed);
        self.metrics.point_queries.store(0, Ordering::Relaxed);
        self.metrics.range_queries.store(0, Ordering::Relaxed);
        self.metrics
            .aggregate_queries
            .store(0, Ordering::Relaxed);
        self.metrics.full_scans.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;
    use datafusion::logical_expr::{col, lit, BinaryExpr, Between, Operator};

    #[test]
    fn test_route_point_query() {
        let router = QueryRouter::new("id".to_string(), 1_000_000);

        let filter = Expr::BinaryExpr(BinaryExpr {
            left: Box::new(col("id")),
            op: Operator::Eq,
            right: Box::new(lit(42i64)),
        });

        let decision = router.route(&[filter]);

        assert_eq!(
            decision.query_type,
            QueryType::PointQuery {
                pk_value: Value::Int64(42)
            }
        );
        assert_eq!(decision.execution_path, ExecutionPath::AlexIndex);
        assert_eq!(decision.estimated_cost_ns, 389); // ALEX point query cost
    }

    #[test]
    fn test_route_small_range_query() {
        let router = QueryRouter::new("id".to_string(), 1_000_000);

        // Range of 50 rows (< 100 threshold)
        let filter = Expr::Between(Between {
            expr: Box::new(col("id")),
            negated: false,
            low: Box::new(lit(100i64)),
            high: Box::new(lit(150i64)),
        });

        let decision = router.route(&[filter]);

        assert!(matches!(decision.query_type, QueryType::RangeQuery { .. }));
        assert_eq!(decision.execution_path, ExecutionPath::AlexIndex);
    }

    #[test]
    fn test_route_large_range_query() {
        let router = QueryRouter::new("id".to_string(), 1_000_000);

        // Range of 1000 rows (> 100 threshold)
        let filter = Expr::Between(Between {
            expr: Box::new(col("id")),
            negated: false,
            low: Box::new(lit(100i64)),
            high: Box::new(lit(1100i64)),
        });

        let decision = router.route(&[filter]);

        assert!(matches!(decision.query_type, QueryType::RangeQuery { .. }));
        assert_eq!(decision.execution_path, ExecutionPath::DataFusion);
    }

    #[test]
    fn test_route_full_scan() {
        let router = QueryRouter::new("id".to_string(), 1_000_000);

        // Filter on non-PK column
        let filter = Expr::BinaryExpr(BinaryExpr {
            left: Box::new(col("name")),
            op: Operator::Eq,
            right: Box::new(lit("Alice")),
        });

        let decision = router.route(&[filter]);

        assert_eq!(decision.query_type, QueryType::FullScan);
        assert_eq!(decision.execution_path, ExecutionPath::DataFusion);
    }

    #[test]
    fn test_routing_metrics() {
        let router = QueryRouter::new("id".to_string(), 1_000_000);

        // Route point query
        let filter1 = Expr::BinaryExpr(BinaryExpr {
            left: Box::new(col("id")),
            op: Operator::Eq,
            right: Box::new(lit(42i64)),
        });
        router.route(&[filter1]);

        // Route range query
        let filter2 = Expr::Between(Between {
            expr: Box::new(col("id")),
            negated: false,
            low: Box::new(lit(100i64)),
            high: Box::new(lit(150i64)),
        });
        router.route(&[filter2]);

        let metrics = router.metrics();

        assert_eq!(metrics.total_queries.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.alex_routed.load(Ordering::Relaxed), 2); // Both small enough for ALEX
        assert_eq!(metrics.point_queries.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.range_queries.load(Ordering::Relaxed), 1);

        // Check routing ratio
        let (alex_ratio, df_ratio) = metrics.routing_ratio();
        assert_eq!(alex_ratio, 1.0);
        assert_eq!(df_ratio, 0.0);
    }

    #[test]
    fn test_decision_time_tracking() {
        let router = QueryRouter::new("id".to_string(), 1_000_000);

        let filter = Expr::BinaryExpr(BinaryExpr {
            left: Box::new(col("id")),
            op: Operator::Eq,
            right: Box::new(lit(42i64)),
        });

        let decision = router.route(&[filter]);

        // Decision time should be very small (<1µs typically)
        assert!(decision.decision_time_ns < 1_000_000); // <1ms

        let avg_time = router.metrics().avg_decision_time_ns();
        assert!(avg_time > 0);
        assert!(avg_time < 1_000_000);
    }

    #[test]
    fn test_custom_threshold() {
        let router = QueryRouter::with_threshold("id".to_string(), 1_000_000, 500);

        // Range of 200 rows (< 500 threshold)
        let filter = Expr::Between(Between {
            expr: Box::new(col("id")),
            negated: false,
            low: Box::new(lit(100i64)),
            high: Box::new(lit(300i64)),
        });

        let decision = router.route(&[filter]);

        assert_eq!(decision.execution_path, ExecutionPath::AlexIndex);
    }

    #[test]
    fn test_metrics_reset() {
        let router = QueryRouter::new("id".to_string(), 1_000_000);

        let filter = Expr::BinaryExpr(BinaryExpr {
            left: Box::new(col("id")),
            op: Operator::Eq,
            right: Box::new(lit(42i64)),
        });

        router.route(&[filter]);
        assert_eq!(router.metrics().total_queries.load(Ordering::Relaxed), 1);

        router.reset_metrics();
        assert_eq!(router.metrics().total_queries.load(Ordering::Relaxed), 0);
    }
}
