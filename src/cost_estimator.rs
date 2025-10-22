//! Cost estimation for query routing decisions
//!
//! Estimates execution cost for different paths:
//! - ALEX learned index: O(log n) for point queries, O(k log n) for ranges
//! - DataFusion: O(n) scan with vectorized operations
//!
//! Decision logic:
//! - Point query → Always ALEX (389ns vs >10µs)
//! - Small range (<100 rows) → ALEX (faster for small k)
//! - Large range (≥100 rows) → DataFusion (vectorized wins)
//! - Aggregates → Always DataFusion (vectorized)

use crate::query_classifier::QueryType;

/// Execution path for queries
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecutionPath {
    /// Use ALEX learned index (point/small range queries)
    AlexIndex,

    /// Use DataFusion vectorized execution (aggregates, large scans)
    DataFusion,
}

/// Cost estimation for routing decisions
pub struct CostEstimator {
    /// Total rows in table (for cost calculation)
    table_size: usize,

    /// Threshold for range query routing (rows)
    /// Below this: use ALEX, above: use DataFusion
    range_threshold: usize,
}

impl CostEstimator {
    /// Create new cost estimator
    pub fn new(table_size: usize) -> Self {
        Self {
            table_size,
            range_threshold: 100, // Empirically determined threshold
        }
    }

    /// Create with custom range threshold
    pub fn with_threshold(table_size: usize, range_threshold: usize) -> Self {
        Self {
            table_size,
            range_threshold,
        }
    }

    /// Estimate best execution path for query
    pub fn estimate(&self, query_type: &QueryType) -> ExecutionPath {
        match query_type {
            QueryType::PointQuery { .. } => {
                // Point queries always use ALEX (389ns vs >10µs for scan)
                ExecutionPath::AlexIndex
            }

            QueryType::RangeQuery { start, end } => {
                // Estimate range size
                let estimated_rows = self.estimate_range_size(start, end);

                // Small range: ALEX faster (k log n where k is small)
                // Large range: DataFusion faster (vectorized scan)
                if estimated_rows < self.range_threshold {
                    ExecutionPath::AlexIndex
                } else {
                    ExecutionPath::DataFusion
                }
            }

            QueryType::AggregateQuery => {
                // Aggregates always use DataFusion (vectorized operations)
                ExecutionPath::DataFusion
            }

            QueryType::FullScan => {
                // Full scans use DataFusion (optimized for bulk processing)
                ExecutionPath::DataFusion
            }

            QueryType::Complex => {
                // Complex queries use DataFusion (joins, subqueries, etc.)
                ExecutionPath::DataFusion
            }
        }
    }

    /// Estimate number of rows in range query
    ///
    /// Simple heuristic: If both bounds are Int64, calculate difference.
    /// Otherwise, assume 10% of table size.
    fn estimate_range_size(&self, start: &crate::value::Value, end: &crate::value::Value) -> usize {
        use crate::value::Value;

        match (start, end) {
            (Value::Int64(s), Value::Int64(e)) => {
                let range = (e - s).unsigned_abs() as usize;
                // Cap at table size
                range.min(self.table_size)
            }
            (Value::UInt64(s), Value::UInt64(e)) => {
                let range = e.saturating_sub(*s) as usize;
                range.min(self.table_size)
            }
            _ => {
                // Unknown type: assume 10% of table
                self.table_size / 10
            }
        }
    }

    /// Estimate cost in nanoseconds (for benchmarking)
    pub fn estimate_cost_ns(&self, path: ExecutionPath, query_type: &QueryType) -> u64 {
        match path {
            ExecutionPath::AlexIndex => match query_type {
                QueryType::PointQuery { .. } => 389, // Measured ALEX point query time
                QueryType::RangeQuery { start, end } => {
                    let k = self.estimate_range_size(start, end);
                    // k * log(n) * 389ns per lookup
                    let log_n = (self.table_size as f64).log2() as u64;
                    k as u64 * log_n * 389
                }
                _ => u64::MAX, // Should not use ALEX for these
            },

            ExecutionPath::DataFusion => {
                // DataFusion vectorized scan: ~10ns per row
                match query_type {
                    QueryType::PointQuery { .. } => {
                        // Full scan for 1 row (inefficient, but DataFusion can handle it)
                        self.table_size as u64 * 10
                    }
                    QueryType::RangeQuery { start, end } => {
                        // Scan entire table, filter to range
                        self.table_size as u64 * 10
                    }
                    QueryType::AggregateQuery | QueryType::FullScan => {
                        // Vectorized scan: ~10ns per row
                        self.table_size as u64 * 10
                    }
                    QueryType::Complex => {
                        // Complex query overhead + scan
                        self.table_size as u64 * 20
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn test_point_query_routing() {
        let estimator = CostEstimator::new(1_000_000);

        let query = QueryType::PointQuery {
            pk_value: Value::Int64(42),
        };

        assert_eq!(estimator.estimate(&query), ExecutionPath::AlexIndex);
    }

    #[test]
    fn test_small_range_query_routing() {
        let estimator = CostEstimator::new(1_000_000);

        // Range of 50 rows (< 100 threshold)
        let query = QueryType::RangeQuery {
            start: Value::Int64(100),
            end: Value::Int64(150),
        };

        assert_eq!(estimator.estimate(&query), ExecutionPath::AlexIndex);
    }

    #[test]
    fn test_large_range_query_routing() {
        let estimator = CostEstimator::new(1_000_000);

        // Range of 1000 rows (> 100 threshold)
        let query = QueryType::RangeQuery {
            start: Value::Int64(100),
            end: Value::Int64(1100),
        };

        assert_eq!(estimator.estimate(&query), ExecutionPath::DataFusion);
    }

    #[test]
    fn test_aggregate_query_routing() {
        let estimator = CostEstimator::new(1_000_000);

        let query = QueryType::AggregateQuery;

        assert_eq!(estimator.estimate(&query), ExecutionPath::DataFusion);
    }

    #[test]
    fn test_full_scan_routing() {
        let estimator = CostEstimator::new(1_000_000);

        let query = QueryType::FullScan;

        assert_eq!(estimator.estimate(&query), ExecutionPath::DataFusion);
    }

    #[test]
    fn test_cost_estimation_point_query() {
        let estimator = CostEstimator::new(1_000_000);

        let query = QueryType::PointQuery {
            pk_value: Value::Int64(42),
        };

        // ALEX: 389ns
        let alex_cost = estimator.estimate_cost_ns(ExecutionPath::AlexIndex, &query);
        assert_eq!(alex_cost, 389);

        // DataFusion: 1M rows * 10ns = 10ms (much slower)
        let df_cost = estimator.estimate_cost_ns(ExecutionPath::DataFusion, &query);
        assert_eq!(df_cost, 10_000_000);

        assert!(alex_cost < df_cost);
    }

    #[test]
    fn test_cost_estimation_range_query() {
        let estimator = CostEstimator::new(1_000_000);

        // Small range: 50 rows
        let small_range = QueryType::RangeQuery {
            start: Value::Int64(100),
            end: Value::Int64(150),
        };

        let alex_cost = estimator.estimate_cost_ns(ExecutionPath::AlexIndex, &small_range);
        let df_cost = estimator.estimate_cost_ns(ExecutionPath::DataFusion, &small_range);

        // ALEX: 50 * log2(1M) * 389ns ≈ 389µs
        // DataFusion: 1M * 10ns = 10ms
        assert!(alex_cost < df_cost);

        // Large range: 10000 rows
        let large_range = QueryType::RangeQuery {
            start: Value::Int64(100),
            end: Value::Int64(10100),
        };

        let alex_cost = estimator.estimate_cost_ns(ExecutionPath::AlexIndex, &large_range);
        let df_cost = estimator.estimate_cost_ns(ExecutionPath::DataFusion, &large_range);

        // ALEX: 10000 * log2(1M) * 389ns ≈ 77ms
        // DataFusion: 1M * 10ns = 10ms
        // DataFusion is faster for large ranges
        assert!(df_cost < alex_cost);
    }

    #[test]
    fn test_custom_threshold() {
        let estimator = CostEstimator::with_threshold(1_000_000, 500);

        // Range of 200 rows (< 500 threshold)
        let query = QueryType::RangeQuery {
            start: Value::Int64(100),
            end: Value::Int64(300),
        };

        assert_eq!(estimator.estimate(&query), ExecutionPath::AlexIndex);
    }
}
