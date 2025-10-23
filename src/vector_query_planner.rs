//! Vector query planning and optimization
//!
//! Detects vector similarity search patterns and chooses optimal execution strategy:
//! - HNSW+BQ index scan for large tables
//! - Sequential scan for small tables or when no index exists

use crate::vector::VectorValue;
use crate::vector_operators::VectorOperator;
use anyhow::{anyhow, Result};
use sqlparser::ast::{Expr, OrderByExpr};

/// Vector query pattern detection result
#[derive(Debug, Clone, PartialEq)]
pub struct VectorQueryPattern {
    /// Column name containing vectors
    pub column_name: String,

    /// Query vector to search for
    pub query_vector: VectorValue,

    /// Distance operator (<->, <#>, <=>)
    pub operator: VectorOperator,

    /// Number of nearest neighbors to return (from LIMIT clause)
    pub k: usize,

    /// Whether to use ascending or descending order
    pub ascending: bool,
}

/// Hybrid query pattern combining vector search with SQL predicates
#[derive(Debug, Clone, PartialEq)]
pub struct HybridQueryPattern {
    /// Vector similarity search pattern
    pub vector_pattern: VectorQueryPattern,

    /// SQL filter predicates from WHERE clause
    pub sql_predicates: Expr,

    /// Table name
    pub table_name: String,
}

/// Hybrid query execution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HybridQueryStrategy {
    /// Execute SQL predicates first, then vector search on filtered set
    /// Best for highly selective filters (< 10% of rows)
    FilterFirst,

    /// Execute vector search first, then apply SQL predicates
    /// Best for non-selective filters (> 50% of rows)
    VectorFirst {
        /// Over-fetch factor (fetch k * expansion candidates)
        expansion_factor: usize,
    },

    /// Execute both in parallel and merge results
    /// Best for medium selectivity (10-50% of rows)
    DualScan,
}

impl HybridQueryPattern {
    /// Detect hybrid query pattern from SQL query
    ///
    /// # Arguments
    /// * `table_name` - Name of the table being queried
    /// * `where_clause` - Optional WHERE clause
    /// * `order_by` - ORDER BY expressions
    /// * `limit` - LIMIT value
    ///
    /// # Returns
    /// HybridQueryPattern if both vector ORDER BY and WHERE clause exist, None otherwise
    pub fn detect(
        table_name: String,
        where_clause: Option<&Expr>,
        order_by: &[OrderByExpr],
        limit: Option<usize>,
    ) -> Result<Option<Self>> {
        // First, try to detect vector query pattern
        let vector_pattern = match VectorQueryPattern::detect(order_by, limit)? {
            Some(pattern) => pattern,
            None => return Ok(None), // Not a vector query
        };

        // Check if WHERE clause exists (required for hybrid)
        let sql_predicates = match where_clause {
            Some(expr) => expr.clone(),
            None => return Ok(None), // No WHERE clause = pure vector query
        };

        // Both vector pattern and WHERE clause exist = hybrid query
        Ok(Some(HybridQueryPattern {
            vector_pattern,
            sql_predicates,
            table_name,
        }))
    }

    /// Validate that this hybrid pattern can be executed
    pub fn validate(&self) -> Result<()> {
        self.vector_pattern.validate()
    }
}

impl VectorQueryPattern {
    /// Detect vector query pattern from SQL ORDER BY clause
    ///
    /// Pattern: `ORDER BY column_name <-> '[...]' LIMIT k`
    ///
    /// # Arguments
    /// * `order_by` - ORDER BY expressions from query
    /// * `limit` - LIMIT value from query
    ///
    /// # Returns
    /// VectorQueryPattern if detected, None otherwise
    pub fn detect(order_by: &[OrderByExpr], limit: Option<usize>) -> Result<Option<Self>> {
        if order_by.is_empty() {
            return Ok(None);
        }

        // Check first ORDER BY expression for vector distance operator
        let first_order = &order_by[0];

        // Extract operator and operands from expression
        match &first_order.expr {
            Expr::BinaryOp { left, op, right } => {
                // Try to parse as vector operator
                let op_str = format!("{:?}", op); // Gets operator symbol
                let vector_op = match op_str.as_str() {
                    "Spaceship" => VectorOperator::CosineDistance, // <=>
                    "Custom(_)" => {
                        // Need to handle custom operators differently
                        // For now, return None
                        return Ok(None);
                    }
                    _ => return Ok(None),
                };

                // Extract column name from left operand
                let column_name = match left.as_ref() {
                    Expr::Identifier(ident) => ident.value.clone(),
                    _ => return Ok(None),
                };

                // Extract query vector from right operand
                let query_vector = match right.as_ref() {
                    Expr::Value(sqlparser::ast::Value::SingleQuotedString(s)) => {
                        // Parse '[1.0, 2.0, ...]' literal
                        VectorValue::from_literal(s)?
                    }
                    _ => return Ok(None),
                };

                // Get k from LIMIT clause (default to 10 if not specified)
                let k = limit.unwrap_or(10);

                // Check ordering (ASC for distances)
                let ascending = first_order.asc.unwrap_or(true);

                Ok(Some(VectorQueryPattern {
                    column_name,
                    query_vector,
                    operator: vector_op,
                    k,
                    ascending,
                }))
            }
            _ => Ok(None),
        }
    }

    /// Validate that this pattern can use a vector index
    pub fn validate(&self) -> Result<()> {
        if self.k == 0 {
            return Err(anyhow!("LIMIT must be greater than 0 for vector search"));
        }

        if self.k > 10000 {
            return Err(anyhow!("LIMIT too large for vector search (max: 10000)"));
        }

        // Vector distance operators should use ASC ordering (smaller = closer)
        if !self.ascending {
            return Err(anyhow!(
                "Vector distance queries should use ASC ordering (smaller distance = closer)"
            ));
        }

        Ok(())
    }
}

/// Query execution strategy for vector queries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorQueryStrategy {
    /// Use HNSW+BQ index for fast approximate search
    IndexScan {
        /// Expansion factor for binary quantization
        expansion: usize,
    },

    /// Use sequential scan with exact distances
    SequentialScan,
}

/// Vector query planner
pub struct VectorQueryPlanner {
    /// Minimum table size to use index (smaller tables use sequential scan)
    min_table_size_for_index: usize,

    /// Default expansion factor for BQ index scans
    default_expansion: usize,
}

impl Default for VectorQueryPlanner {
    fn default() -> Self {
        Self {
            min_table_size_for_index: 1000, // Use index for tables with 1000+ vectors
            default_expansion: 150,          // 92.7% recall @ 5.6ms
        }
    }
}

impl VectorQueryPlanner {
    /// Create new vector query planner with custom settings
    pub fn new(min_table_size_for_index: usize, default_expansion: usize) -> Self {
        Self {
            min_table_size_for_index,
            default_expansion,
        }
    }

    /// Estimate selectivity of SQL predicate (returns fraction of rows that match)
    ///
    /// # Arguments
    /// * `predicate` - SQL WHERE expression
    /// * `table_size` - Total number of rows in table
    /// * `primary_key` - Name of the primary key column
    ///
    /// # Returns
    /// Estimated selectivity (0.0 to 1.0)
    pub fn estimate_selectivity(
        &self,
        predicate: &Expr,
        table_size: usize,
        primary_key: &str,
    ) -> f64 {
        use sqlparser::ast::BinaryOperator;

        match predicate {
            // Primary key equality: 1 row (very selective)
            Expr::BinaryOp { left, op, right } if matches!(op, BinaryOperator::Eq) => {
                if let Expr::Identifier(col) = left.as_ref() {
                    if col.value == primary_key {
                        return 1.0 / table_size as f64;
                    }
                }
                // Non-PK equality: assume 1% selectivity
                0.01
            }

            // Primary key range (>, >=, <, <=): estimate 10% for single bound
            Expr::BinaryOp { left, op, .. }
                if matches!(
                    op,
                    BinaryOperator::Gt
                        | BinaryOperator::GtEq
                        | BinaryOperator::Lt
                        | BinaryOperator::LtEq
                ) =>
            {
                if let Expr::Identifier(col) = left.as_ref() {
                    if col.value == primary_key {
                        return 0.10; // Assume 10% of rows for single bound
                    }
                }
                // Non-PK range: assume 20% selectivity
                0.20
            }

            // AND: multiply selectivities (assumes independence)
            Expr::BinaryOp {
                left,
                op: BinaryOperator::And,
                right,
            } => {
                let left_sel = self.estimate_selectivity(left, table_size, primary_key);
                let right_sel = self.estimate_selectivity(right, table_size, primary_key);
                left_sel * right_sel
            }

            // OR: add selectivities (capped at 1.0)
            Expr::BinaryOp {
                left,
                op: BinaryOperator::Or,
                right,
            } => {
                let left_sel = self.estimate_selectivity(left, table_size, primary_key);
                let right_sel = self.estimate_selectivity(right, table_size, primary_key);
                (left_sel + right_sel).min(1.0)
            }

            // Default: assume 10% selectivity for unknown predicates
            _ => 0.10,
        }
    }

    /// Choose optimal execution strategy for hybrid query
    ///
    /// # Arguments
    /// * `pattern` - Detected hybrid query pattern
    /// * `table_size` - Number of rows in table
    /// * `primary_key` - Name of the primary key column
    ///
    /// # Returns
    /// Recommended hybrid execution strategy
    pub fn plan_hybrid(
        &self,
        pattern: &HybridQueryPattern,
        table_size: usize,
        primary_key: &str,
    ) -> HybridQueryStrategy {
        // Estimate selectivity of SQL predicates
        let selectivity =
            self.estimate_selectivity(&pattern.sql_predicates, table_size, primary_key);

        // Choose strategy based on selectivity thresholds
        if selectivity < 0.10 {
            // Highly selective (< 10%) → Filter-First
            HybridQueryStrategy::FilterFirst
        } else if selectivity > 0.50 {
            // Low selectivity (> 50%) → Vector-First with over-fetch
            HybridQueryStrategy::VectorFirst {
                expansion_factor: 3, // Fetch 3x candidates to ensure k results after filtering
            }
        } else {
            // Medium selectivity (10-50%) → Dual-Scan
            // For now, fall back to Filter-First (Dual-Scan is Phase 2)
            HybridQueryStrategy::FilterFirst
        }
    }

    /// Choose optimal execution strategy for vector query
    ///
    /// # Arguments
    /// * `pattern` - Detected vector query pattern
    /// * `table_size` - Number of rows in table
    /// * `has_index` - Whether a vector index exists for this column
    ///
    /// # Returns
    /// Recommended execution strategy
    pub fn plan(
        &self,
        pattern: &VectorQueryPattern,
        table_size: usize,
        has_index: bool,
    ) -> VectorQueryStrategy {
        // Always use sequential scan if no index exists
        if !has_index {
            return VectorQueryStrategy::SequentialScan;
        }

        // Use sequential scan for small tables
        if table_size < self.min_table_size_for_index {
            return VectorQueryStrategy::SequentialScan;
        }

        // Adjust expansion based on k
        let expansion = if pattern.k <= 10 {
            self.default_expansion // 150x for top-10
        } else if pattern.k <= 100 {
            200 // 200x for top-100 (95.1% recall)
        } else {
            250 // 250x for larger k
        };

        VectorQueryStrategy::IndexScan { expansion }
    }

    /// Estimate query cost in milliseconds
    ///
    /// # Arguments
    /// * `strategy` - Execution strategy
    /// * `table_size` - Number of rows in table
    /// * `k` - Number of results to return
    ///
    /// # Returns
    /// Estimated query time in milliseconds
    pub fn estimate_cost(
        &self,
        strategy: VectorQueryStrategy,
        table_size: usize,
        k: usize,
    ) -> f64 {
        match strategy {
            VectorQueryStrategy::IndexScan { expansion } => {
                // HNSW cost: log(N) graph traversal + expansion * hamming + k * L2
                let hnsw_traversal = (table_size as f64).log2() * 0.001; // ~1µs per hop
                let hamming_comparisons = expansion as f64 * 0.00001; // ~10ns per hamming
                let l2_reranking = k as f64 * 0.0001; // ~100µs per L2 distance
                hnsw_traversal + hamming_comparisons + l2_reranking
            }
            VectorQueryStrategy::SequentialScan => {
                // Sequential scan: N * L2 distance computations
                table_size as f64 * 0.0001 // ~100µs per L2 distance
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlparser::ast::{Ident, Value as SqlValue};

    #[test]
    fn test_vector_query_pattern_validate() {
        let pattern = VectorQueryPattern {
            column_name: "embedding".to_string(),
            query_vector: VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap(),
            operator: VectorOperator::L2Distance,
            k: 10,
            ascending: true,
        };

        assert!(pattern.validate().is_ok());
    }

    #[test]
    fn test_vector_query_pattern_validate_zero_k() {
        let pattern = VectorQueryPattern {
            column_name: "embedding".to_string(),
            query_vector: VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap(),
            operator: VectorOperator::L2Distance,
            k: 0,
            ascending: true,
        };

        assert!(pattern.validate().is_err());
    }

    #[test]
    fn test_vector_query_pattern_validate_descending() {
        let pattern = VectorQueryPattern {
            column_name: "embedding".to_string(),
            query_vector: VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap(),
            operator: VectorOperator::L2Distance,
            k: 10,
            ascending: false,
        };

        assert!(pattern.validate().is_err());
    }

    #[test]
    fn test_query_planner_sequential_scan_no_index() {
        let planner = VectorQueryPlanner::default();
        let pattern = VectorQueryPattern {
            column_name: "embedding".to_string(),
            query_vector: VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap(),
            operator: VectorOperator::L2Distance,
            k: 10,
            ascending: true,
        };

        let strategy = planner.plan(&pattern, 10000, false);
        assert_eq!(strategy, VectorQueryStrategy::SequentialScan);
    }

    #[test]
    fn test_query_planner_sequential_scan_small_table() {
        let planner = VectorQueryPlanner::default();
        let pattern = VectorQueryPattern {
            column_name: "embedding".to_string(),
            query_vector: VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap(),
            operator: VectorOperator::L2Distance,
            k: 10,
            ascending: true,
        };

        let strategy = planner.plan(&pattern, 500, true);
        assert_eq!(strategy, VectorQueryStrategy::SequentialScan);
    }

    #[test]
    fn test_query_planner_index_scan_large_table() {
        let planner = VectorQueryPlanner::default();
        let pattern = VectorQueryPattern {
            column_name: "embedding".to_string(),
            query_vector: VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap(),
            operator: VectorOperator::L2Distance,
            k: 10,
            ascending: true,
        };

        let strategy = planner.plan(&pattern, 10000, true);
        assert_eq!(
            strategy,
            VectorQueryStrategy::IndexScan { expansion: 150 }
        );
    }

    #[test]
    fn test_query_planner_expansion_for_large_k() {
        let planner = VectorQueryPlanner::default();
        let pattern = VectorQueryPattern {
            column_name: "embedding".to_string(),
            query_vector: VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap(),
            operator: VectorOperator::L2Distance,
            k: 150,
            ascending: true,
        };

        let strategy = planner.plan(&pattern, 10000, true);
        assert_eq!(
            strategy,
            VectorQueryStrategy::IndexScan { expansion: 250 }
        );
    }

    #[test]
    fn test_estimate_cost_index_scan() {
        let planner = VectorQueryPlanner::default();
        let cost = planner.estimate_cost(
            VectorQueryStrategy::IndexScan { expansion: 150 },
            10000,
            10,
        );

        // Should be much faster than sequential scan
        assert!(cost < 10.0); // Less than 10ms
    }

    #[test]
    fn test_estimate_cost_sequential_scan() {
        let planner = VectorQueryPlanner::default();
        let cost = planner.estimate_cost(VectorQueryStrategy::SequentialScan, 10000, 10);

        // 10K * 0.0001ms = 1ms
        assert!((cost - 1.0).abs() < 0.1);
    }
}
