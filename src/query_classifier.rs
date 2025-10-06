//! Query classification for intelligent routing
//!
//! Analyzes DataFusion expressions to classify query types:
//! - Point queries (WHERE pk = value) → ALEX learned index
//! - Range queries (WHERE pk BETWEEN x AND y) → ALEX or DataFusion
//! - Aggregates (COUNT, SUM, AVG) → DataFusion vectorized
//! - Full scans → DataFusion

use crate::value::Value;
use datafusion::logical_expr::{Expr, Operator};
use datafusion::scalar::ScalarValue;

/// Query classification result
#[derive(Debug, Clone, PartialEq)]
pub enum QueryType {
    /// Point query on primary key (WHERE pk = value)
    PointQuery { pk_value: Value },

    /// Range query on primary key (WHERE pk BETWEEN start AND end)
    RangeQuery { start: Value, end: Value },

    /// Aggregate query (COUNT, SUM, AVG, etc.)
    AggregateQuery,

    /// Full table scan
    FullScan,

    /// Complex query (joins, subqueries, etc.)
    Complex,
}

impl QueryType {
    /// Check if this is a point query
    pub fn is_point_query(&self) -> bool {
        matches!(self, QueryType::PointQuery { .. })
    }

    /// Check if this is a range query
    pub fn is_range_query(&self) -> bool {
        matches!(self, QueryType::RangeQuery { .. })
    }

    /// Check if this is an aggregate query
    pub fn is_aggregate(&self) -> bool {
        matches!(self, QueryType::AggregateQuery)
    }
}

/// Query classifier for routing decisions
pub struct QueryClassifier {
    /// Primary key column name
    pk_column: String,
}

impl QueryClassifier {
    /// Create new query classifier for a table
    pub fn new(pk_column: String) -> Self {
        Self { pk_column }
    }

    /// Classify query based on filters
    pub fn classify_filters(&self, filters: &[Expr]) -> QueryType {
        // Check for point query (pk = value)
        if let Some(pk_value) = self.detect_point_query(filters) {
            return QueryType::PointQuery { pk_value };
        }

        // Check for range query (pk BETWEEN start AND end)
        if let Some((start, end)) = self.detect_range_query(filters) {
            return QueryType::RangeQuery { start, end };
        }

        // Check for aggregates (will be detected at higher level)
        // For now, assume full scan if not point/range
        QueryType::FullScan
    }

    /// Detect point query (WHERE pk = value)
    fn detect_point_query(&self, filters: &[Expr]) -> Option<Value> {
        for expr in filters {
            if let Expr::BinaryExpr(binary) = expr {
                // Check for: pk = value
                if binary.op == Operator::Eq {
                    if let (Expr::Column(col), Expr::Literal(scalar)) =
                        (&*binary.left, &*binary.right)
                    {
                        if col.name == self.pk_column {
                            return Self::scalar_to_value(scalar);
                        }
                    }
                    // Also check reversed: value = pk
                    if let (Expr::Literal(scalar), Expr::Column(col)) =
                        (&*binary.left, &*binary.right)
                    {
                        if col.name == self.pk_column {
                            return Self::scalar_to_value(scalar);
                        }
                    }
                }
            }
        }
        None
    }

    /// Detect range query (WHERE pk BETWEEN start AND end)
    fn detect_range_query(&self, filters: &[Expr]) -> Option<(Value, Value)> {
        // Check for BETWEEN expression
        for expr in filters {
            if let Expr::Between(between) = expr {
                if let Expr::Column(col) = &*between.expr {
                    if col.name == self.pk_column && !between.negated {
                        if let (Expr::Literal(low), Expr::Literal(high)) =
                            (&*between.low, &*between.high)
                        {
                            if let (Some(start), Some(end)) =
                                (Self::scalar_to_value(low), Self::scalar_to_value(high))
                            {
                                return Some((start, end));
                            }
                        }
                    }
                }
            }
        }

        // Check for range as AND of >= and <=
        let mut lower_bound: Option<Value> = None;
        let mut upper_bound: Option<Value> = None;

        for expr in filters {
            if let Expr::BinaryExpr(binary) = expr {
                if let Expr::Column(col) = &*binary.left {
                    if col.name == self.pk_column {
                        if let Expr::Literal(scalar) = &*binary.right {
                            match binary.op {
                                Operator::GtEq | Operator::Gt => {
                                    if let Some(val) = Self::scalar_to_value(scalar) {
                                        lower_bound = Some(val);
                                    }
                                }
                                Operator::LtEq | Operator::Lt => {
                                    if let Some(val) = Self::scalar_to_value(scalar) {
                                        upper_bound = Some(val);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        if let (Some(start), Some(end)) = (lower_bound, upper_bound) {
            return Some((start, end));
        }

        None
    }

    /// Convert ScalarValue to OmenDB Value
    fn scalar_to_value(scalar: &ScalarValue) -> Option<Value> {
        match scalar {
            ScalarValue::Int64(Some(v)) => Some(Value::Int64(*v)),
            ScalarValue::UInt64(Some(v)) => Some(Value::UInt64(*v)),
            ScalarValue::Float64(Some(v)) => Some(Value::Float64(*v)),
            ScalarValue::Utf8(Some(s)) => Some(Value::Text(s.clone())),
            ScalarValue::Boolean(Some(b)) => Some(Value::Boolean(*b)),
            ScalarValue::Int32(Some(i)) => Some(Value::Int64(*i as i64)),
            ScalarValue::UInt32(Some(u)) => Some(Value::UInt64(*u as u64)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::logical_expr::{col, lit, BinaryExpr, Between};
    use std::sync::Arc;

    #[test]
    fn test_classify_point_query() {
        let classifier = QueryClassifier::new("id".to_string());

        // WHERE id = 42
        let filter = Expr::BinaryExpr(BinaryExpr {
            left: Box::new(col("id")),
            op: Operator::Eq,
            right: Box::new(lit(42i64)),
        });

        let query_type = classifier.classify_filters(&[filter]);
        assert_eq!(
            query_type,
            QueryType::PointQuery {
                pk_value: Value::Int64(42)
            }
        );
    }

    #[test]
    fn test_classify_range_query_between() {
        let classifier = QueryClassifier::new("id".to_string());

        // WHERE id BETWEEN 10 AND 100
        let filter = Expr::Between(Between {
            expr: Box::new(col("id")),
            negated: false,
            low: Box::new(lit(10i64)),
            high: Box::new(lit(100i64)),
        });

        let query_type = classifier.classify_filters(&[filter]);
        assert_eq!(
            query_type,
            QueryType::RangeQuery {
                start: Value::Int64(10),
                end: Value::Int64(100)
            }
        );
    }

    #[test]
    fn test_classify_range_query_operators() {
        let classifier = QueryClassifier::new("id".to_string());

        // WHERE id >= 10 AND id <= 100
        let filter1 = Expr::BinaryExpr(BinaryExpr {
            left: Box::new(col("id")),
            op: Operator::GtEq,
            right: Box::new(lit(10i64)),
        });
        let filter2 = Expr::BinaryExpr(BinaryExpr {
            left: Box::new(col("id")),
            op: Operator::LtEq,
            right: Box::new(lit(100i64)),
        });

        let query_type = classifier.classify_filters(&[filter1, filter2]);
        assert_eq!(
            query_type,
            QueryType::RangeQuery {
                start: Value::Int64(10),
                end: Value::Int64(100)
            }
        );
    }

    #[test]
    fn test_classify_full_scan() {
        let classifier = QueryClassifier::new("id".to_string());

        // WHERE name = 'Alice' (not on primary key)
        let filter = Expr::BinaryExpr(BinaryExpr {
            left: Box::new(col("name")),
            op: Operator::Eq,
            right: Box::new(lit("Alice")),
        });

        let query_type = classifier.classify_filters(&[filter]);
        assert_eq!(query_type, QueryType::FullScan);
    }

    #[test]
    fn test_classify_no_filters() {
        let classifier = QueryClassifier::new("id".to_string());

        let query_type = classifier.classify_filters(&[]);
        assert_eq!(query_type, QueryType::FullScan);
    }
}
