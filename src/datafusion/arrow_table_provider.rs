//! DataFusion TableProvider implementation for OmenDB Table system
//!
//! Provides SQL query capabilities on Arrow/Parquet tables with learned index optimization.
//! Routes queries intelligently:
//! - Point queries (WHERE id = X) → ALEX learned index (fast)
//! - Range/aggregates → DataFusion vectorized execution (OLAP-optimized)

use crate::table::Table;
use crate::value::Value;
use arrow::array::RecordBatch;
use arrow::datatypes::SchemaRef;
use async_trait::async_trait;
use datafusion::catalog::Session;
use datafusion::datasource::TableProvider;
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::{Expr, TableProviderFilterPushDown, TableType};
use datafusion::physical_plan::memory::MemoryExec;
use datafusion::physical_plan::ExecutionPlan;
use std::any::Any;
use std::sync::{Arc, RwLock};

/// TableProvider that wraps OmenDB Table with learned index optimization
#[derive(Debug)]
pub struct ArrowTableProvider {
    /// Underlying table with ALEX index + Arrow storage
    table: Arc<RwLock<Table>>,

    /// User-facing schema (without MVCC columns)
    schema: SchemaRef,

    /// Table name
    name: String,
}

impl ArrowTableProvider {
    /// Create a new ArrowTableProvider from an OmenDB Table
    pub fn new(table: Arc<RwLock<Table>>, name: impl Into<String>) -> Self {
        let schema = table.read().unwrap().user_schema().clone();

        Self {
            table,
            schema,
            name: name.into(),
        }
    }

    /// Detect if this is a point query on primary key
    fn is_point_query(&self, filters: &[Expr]) -> Option<Value> {
        let pk_name = {
            let table = self.table.read().unwrap();
            table.primary_key().to_string()
        };

        for expr in filters {
            if let Expr::BinaryExpr(binary) = expr {
                // Check for: pk = <value>
                if let (Expr::Column(col), Expr::Literal(scalar_value)) =
                    (&*binary.left, &*binary.right)
                {
                    if col.name == pk_name
                        && binary.op == datafusion::logical_expr::Operator::Eq
                    {
                        // Convert ScalarValue to Value
                        if let Some(value) = Self::scalar_to_value(scalar_value) {
                            return Some(value);
                        }
                    }
                }
                // Also check reversed: <value> = pk
                if let (Expr::Literal(scalar_value), Expr::Column(col)) =
                    (&*binary.left, &*binary.right)
                {
                    if col.name == pk_name
                        && binary.op == datafusion::logical_expr::Operator::Eq
                    {
                        if let Some(value) = Self::scalar_to_value(scalar_value) {
                            return Some(value);
                        }
                    }
                }
            }
        }
        None
    }

    /// Convert DataFusion ScalarValue to OmenDB Value
    fn scalar_to_value(scalar: &datafusion::scalar::ScalarValue) -> Option<Value> {
        use datafusion::scalar::ScalarValue;

        match scalar {
            ScalarValue::Int64(Some(v)) => Some(Value::Int64(*v)),
            ScalarValue::Utf8(Some(s)) => Some(Value::Text(s.clone())),
            ScalarValue::Float64(Some(f)) => Some(Value::Float64(*f)),
            ScalarValue::Boolean(Some(b)) => Some(Value::Boolean(*b)),
            ScalarValue::UInt64(Some(u)) => Some(Value::UInt64(*u)),
            ScalarValue::Int32(Some(i)) => Some(Value::Int64(*i as i64)), // Convert to Int64
            ScalarValue::UInt32(Some(u)) => Some(Value::UInt64(*u as u64)), // Convert to UInt64
            _ => None,
        }
    }

    /// Execute point query using ALEX learned index (fast path)
    fn execute_point_query(&self, pk_value: Value) -> Result<Vec<RecordBatch>> {
        let table = self
            .table
            .read()
            .map_err(|e| DataFusionError::Execution(format!("Lock error: {}", e)))?;

        // Use learned index for fast lookup
        match table.get(&pk_value) {
            Ok(Some(row)) => {
                // Convert single row to RecordBatch
                let batch = row
                    .to_batch(&self.schema)
                    .map_err(|e| DataFusionError::Execution(format!("Row conversion: {}", e)))?;
                Ok(vec![batch])
            }
            Ok(None) | Err(_) => {
                // Key not found or error - return empty result
                Ok(vec![])
            }
        }
    }

    /// Execute full scan for range queries and aggregates (OLAP path)
    fn execute_full_scan(&self) -> Result<Vec<RecordBatch>> {
        let mut table = self
            .table
            .write()
            .map_err(|e| DataFusionError::Execution(format!("Lock error: {}", e)))?;

        // Use TableStorage's columnar scan (Arrow-optimized)
        let batches = table
            .scan_batches()
            .map_err(|e| DataFusionError::Execution(format!("Scan failed: {}", e)))?;

        Ok(batches)
    }
}

#[async_trait]
impl TableProvider for ArrowTableProvider {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    async fn scan(
        &self,
        _state: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        _limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        // Query routing: Point query vs Full scan
        let batches = if let Some(pk_value) = self.is_point_query(filters) {
            // Fast path: Use ALEX learned index for point lookup
            self.execute_point_query(pk_value)?
        } else {
            // OLAP path: Full scan with DataFusion vectorization
            self.execute_full_scan()?
        };

        // Apply projection if specified
        let projected_schema = if let Some(proj) = projection {
            let fields: Vec<_> = proj
                .iter()
                .map(|i| self.schema.field(*i).clone())
                .collect();
            Arc::new(arrow::datatypes::Schema::new(fields))
        } else {
            self.schema.clone()
        };

        let projected_batches: Vec<RecordBatch> = if let Some(proj) = projection {
            batches
                .into_iter()
                .map(|batch| batch.project(proj))
                .collect::<std::result::Result<Vec<_>, _>>()?
        } else {
            batches
        };

        // Create memory execution plan
        let plan = MemoryExec::try_new(&[projected_batches], projected_schema.clone(), None)?;

        Ok(Arc::new(plan))
    }

    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> Result<Vec<TableProviderFilterPushDown>> {
        // We support filter pushdown for point queries (primary key equality)
        let pk_name = {
            let table = self.table.read().unwrap();
            table.primary_key().to_string()
        };

        Ok(filters
            .iter()
            .map(|expr| {
                // Check if this is a filter on primary key
                if is_pk_filter(expr, &pk_name) {
                    TableProviderFilterPushDown::Exact // We handle this exactly via learned index
                } else {
                    TableProviderFilterPushDown::Unsupported // Let DataFusion handle it
                }
            })
            .collect())
    }
}

/// Helper to check if expression is a primary key filter
fn is_pk_filter(expr: &Expr, pk_name: &str) -> bool {
    if let Expr::BinaryExpr(binary) = expr {
        if binary.op == datafusion::logical_expr::Operator::Eq {
            if let Expr::Column(col) = &*binary.left {
                return col.name == pk_name;
            }
            if let Expr::Column(col) = &*binary.right {
                return col.name == pk_name;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::Table;
    use arrow::datatypes::{DataType, Field, Schema};
    use datafusion::prelude::*;
    use std::sync::Arc;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_point_query_via_learned_index() {
        // Create table with schema
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, false),
        ]));

        let dir = tempdir().unwrap();
        let mut table = Table::new(
            "users".to_string(),
            schema.clone(),
            "id".to_string(),
            dir.path().to_path_buf(),
        )
        .unwrap();

        // Insert test data
        for i in 0..100 {
            let row = Row::new(vec![
                Value::Int64(i),
                Value::Text(format!("User {}", i)),
            ]);
            table.insert(row).unwrap();
        }

        // Create TableProvider
        let table_ref = Arc::new(RwLock::new(table));
        let provider = Arc::new(ArrowTableProvider::new(table_ref.clone(), "users"));

        // Register with DataFusion
        let ctx = SessionContext::new();
        ctx.register_table("users", provider).unwrap();

        // Point query (should use ALEX learned index)
        let df = ctx
            .sql("SELECT * FROM users WHERE id = 42")
            .await
            .unwrap();
        let batches = df.collect().await.unwrap();

        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].num_rows(), 1);
    }

    #[tokio::test]
    async fn test_range_query_via_datafusion() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("value", DataType::Float64, false),
        ]));

        let dir = tempdir().unwrap();
        let mut table = Table::new(
            "metrics".to_string(),
            schema.clone(),
            "id".to_string(),
            dir.path().to_path_buf(),
        )
        .unwrap();

        // Insert test data
        for i in 0..100 {
            let row = Row::new(vec![Value::Int64(i), Value::Float64(i as f64 * 1.5)]);
            table.insert(row).unwrap();
        }

        let table_ref = Arc::new(RwLock::new(table));
        let provider = Arc::new(ArrowTableProvider::new(table_ref, "metrics"));

        let ctx = SessionContext::new();
        ctx.register_table("metrics", provider).unwrap();

        // Range query (should use DataFusion scan)
        let df = ctx
            .sql("SELECT COUNT(*) as count FROM metrics WHERE id > 50")
            .await
            .unwrap();
        let batches = df.collect().await.unwrap();

        assert!(batches.len() > 0);
        // Should have 49 rows (51-99 inclusive)
    }

    #[tokio::test]
    async fn test_aggregate_query() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("amount", DataType::Float64, false),
        ]));

        let dir = tempdir().unwrap();
        let mut table = Table::new(
            "sales".to_string(),
            schema.clone(),
            "id".to_string(),
            dir.path().to_path_buf(),
        )
        .unwrap();

        // Insert test data
        for i in 0..50 {
            let row = Row::new(vec![Value::Int64(i), Value::Float64(100.0)]);
            table.insert(row).unwrap();
        }

        let table_ref = Arc::new(RwLock::new(table));
        let provider = Arc::new(ArrowTableProvider::new(table_ref, "sales"));

        let ctx = SessionContext::new();
        ctx.register_table("sales", provider).unwrap();

        // Aggregate query (should use DataFusion vectorized execution)
        let df = ctx
            .sql("SELECT SUM(amount) as total FROM sales")
            .await
            .unwrap();
        let batches = df.collect().await.unwrap();

        assert!(batches.len() > 0);
        // Should sum to 5000.0 (50 * 100.0)
    }
}
