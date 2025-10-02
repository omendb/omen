//! DataFusion TableProvider implementation for redb storage with learned index optimization

use crate::redb_storage::RedbStorage;
use arrow::array::{ArrayRef, Int64Array, RecordBatch, StringArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use async_trait::async_trait;
use datafusion::catalog::Session;
use datafusion::datasource::TableProvider;
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::{Expr, TableProviderFilterPushDown, TableType};
use datafusion::physical_plan::memory::MemoryExec;
use datafusion::physical_plan::ExecutionPlan;
use std::any::Any;
use std::sync::{Arc, RwLock};

/// TableProvider that wraps redb storage with learned index optimization
#[derive(Debug)]
pub struct RedbTable {
    /// Underlying redb storage with learned index
    storage: Arc<RwLock<RedbStorage>>,

    /// Schema for this table
    schema: SchemaRef,

    /// Table name
    name: String,
}

impl RedbTable {
    /// Create a new RedbTable
    pub fn new(storage: Arc<RwLock<RedbStorage>>, name: impl Into<String>) -> Self {
        // Simple schema: (id: Int64, value: String)
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("value", DataType::Utf8, false),
        ]));

        Self {
            storage,
            schema,
            name: name.into(),
        }
    }

    /// Create with custom schema
    pub fn with_schema(
        storage: Arc<RwLock<RedbStorage>>,
        name: impl Into<String>,
        schema: SchemaRef,
    ) -> Self {
        Self {
            storage,
            schema,
            name: name.into(),
        }
    }

    /// Detect if this is a point query (WHERE id = <value>)
    fn is_point_query(filters: &[Expr]) -> Option<i64> {
        for expr in filters {
            if let Expr::BinaryExpr(binary) = expr {
                // Check for: id = <value>
                if let (Expr::Column(col), Expr::Literal(scalar_value)) =
                    (&*binary.left, &*binary.right)
                {
                    if col.name == "id" && binary.op == datafusion::logical_expr::Operator::Eq {
                        if let datafusion::scalar::ScalarValue::Int64(Some(value)) = scalar_value {
                            return Some(*value);
                        }
                    }
                }
                // Also check reversed: <value> = id
                if let (Expr::Literal(scalar_value), Expr::Column(col)) =
                    (&*binary.left, &*binary.right)
                {
                    if col.name == "id" && binary.op == datafusion::logical_expr::Operator::Eq {
                        if let datafusion::scalar::ScalarValue::Int64(Some(value)) = scalar_value {
                            return Some(*value);
                        }
                    }
                }
            }
        }
        None
    }

    /// Detect if this is a range query (WHERE id BETWEEN x AND y, or id >= x AND id <= y)
    /// Returns (start_key, end_key) if detected
    fn is_range_query(filters: &[Expr]) -> Option<(i64, i64)> {
        use datafusion::logical_expr::Operator;
        use datafusion::scalar::ScalarValue;

        // Check for BETWEEN expression: id BETWEEN low AND high
        for expr in filters {
            if let Expr::Between(between) = expr {
                if let Expr::Column(col) = &*between.expr {
                    if col.name == "id" && !between.negated {
                        // Extract low and high bounds
                        if let (Expr::Literal(ScalarValue::Int64(Some(low))), Expr::Literal(ScalarValue::Int64(Some(high)))) =
                            (&*between.low, &*between.high)
                        {
                            return Some((*low, *high));
                        }
                    }
                }
            }
        }

        // Check for range expressed as AND of two binary expressions
        // Pattern: id >= x AND id <= y
        let mut lower_bound: Option<i64> = None;
        let mut upper_bound: Option<i64> = None;

        for expr in filters {
            if let Expr::BinaryExpr(binary) = expr {
                // Check if this is a comparison on id column
                if let Expr::Column(col) = &*binary.left {
                    if col.name == "id" {
                        if let Expr::Literal(ScalarValue::Int64(Some(value))) = &*binary.right {
                            match binary.op {
                                Operator::GtEq | Operator::Gt => {
                                    let adjusted = if binary.op == Operator::Gt { value + 1 } else { *value };
                                    lower_bound = Some(lower_bound.map_or(adjusted, |lb| lb.max(adjusted)));
                                }
                                Operator::LtEq | Operator::Lt => {
                                    let adjusted = if binary.op == Operator::Lt { value - 1 } else { *value };
                                    upper_bound = Some(upper_bound.map_or(adjusted, |ub| ub.min(adjusted)));
                                }
                                _ => {}
                            }
                        }
                    }
                }
                // Also check reversed comparisons: value <= id, value >= id
                if let Expr::Column(col) = &*binary.right {
                    if col.name == "id" {
                        if let Expr::Literal(ScalarValue::Int64(Some(value))) = &*binary.left {
                            match binary.op {
                                Operator::LtEq | Operator::Lt => {
                                    let adjusted = if binary.op == Operator::Lt { value + 1 } else { *value };
                                    lower_bound = Some(lower_bound.map_or(adjusted, |lb| lb.max(adjusted)));
                                }
                                Operator::GtEq | Operator::Gt => {
                                    let adjusted = if binary.op == Operator::Gt { value - 1 } else { *value };
                                    upper_bound = Some(upper_bound.map_or(adjusted, |ub| ub.min(adjusted)));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        // If we found both bounds, return the range
        if let (Some(lower), Some(upper)) = (lower_bound, upper_bound) {
            if lower <= upper {
                return Some((lower, upper));
            }
        }

        None
    }

    /// Execute a point query using learned index
    fn execute_point_query(&self, key: i64) -> Result<RecordBatch> {
        let storage = self
            .storage
            .read()
            .map_err(|e| DataFusionError::Execution(format!("Lock error: {}", e)))?;

        match storage.point_query(key) {
            Ok(Some(value_bytes)) => {
                let value_str = String::from_utf8_lossy(&value_bytes).to_string();

                let id_array = Int64Array::from(vec![key]);
                let value_array = StringArray::from(vec![value_str]);

                let batch = RecordBatch::try_new(
                    self.schema.clone(),
                    vec![
                        Arc::new(id_array) as ArrayRef,
                        Arc::new(value_array) as ArrayRef,
                    ],
                )?;

                Ok(batch)
            }
            Ok(None) => {
                // Return empty batch
                let empty_id = Int64Array::from(Vec::<i64>::new());
                let empty_value = StringArray::from(Vec::<String>::new());

                let batch = RecordBatch::try_new(
                    self.schema.clone(),
                    vec![
                        Arc::new(empty_id) as ArrayRef,
                        Arc::new(empty_value) as ArrayRef,
                    ],
                )?;

                Ok(batch)
            }
            Err(e) => Err(DataFusionError::Execution(format!(
                "Point query failed: {}",
                e
            ))),
        }
    }

    /// Execute a range query using learned index
    fn execute_range_query(&self, start_key: i64, end_key: i64) -> Result<RecordBatch> {
        let storage = self
            .storage
            .read()
            .map_err(|e| DataFusionError::Execution(format!("Lock error: {}", e)))?;

        match storage.range_query(start_key, end_key) {
            Ok(rows) => {
                let mut ids = Vec::with_capacity(rows.len());
                let mut values = Vec::with_capacity(rows.len());

                for (id, value_bytes) in rows {
                    ids.push(id);
                    values.push(String::from_utf8_lossy(&value_bytes).to_string());
                }

                let id_array = Int64Array::from(ids);
                let value_array = StringArray::from(values);

                let batch = RecordBatch::try_new(
                    self.schema.clone(),
                    vec![
                        Arc::new(id_array) as ArrayRef,
                        Arc::new(value_array) as ArrayRef,
                    ],
                )?;

                Ok(batch)
            }
            Err(e) => Err(DataFusionError::Execution(format!(
                "Range query failed: {}",
                e
            ))),
        }
    }

    /// Execute a full table scan
    fn execute_full_scan(&self) -> Result<RecordBatch> {
        let storage = self
            .storage
            .read()
            .map_err(|e| DataFusionError::Execution(format!("Lock error: {}", e)))?;

        match storage.scan_all() {
            Ok(rows) => {
                let mut ids = Vec::with_capacity(rows.len());
                let mut values = Vec::with_capacity(rows.len());

                for (id, value_bytes) in rows {
                    ids.push(id);
                    values.push(String::from_utf8_lossy(&value_bytes).to_string());
                }

                let id_array = Int64Array::from(ids);
                let value_array = StringArray::from(values);

                let batch = RecordBatch::try_new(
                    self.schema.clone(),
                    vec![
                        Arc::new(id_array) as ArrayRef,
                        Arc::new(value_array) as ArrayRef,
                    ],
                )?;

                Ok(batch)
            }
            Err(e) => Err(DataFusionError::Execution(format!(
                "Full scan failed: {}",
                e
            ))),
        }
    }
}

#[async_trait]
impl TableProvider for RedbTable {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> Result<Vec<TableProviderFilterPushDown>> {
        use datafusion::logical_expr::Operator;

        // Check each filter to see if we can handle it
        Ok(filters
            .iter()
            .map(|filter| {
                // Check if this is a comparison on the id column
                // We handle: id = X, id >= X, id <= X, id > X, id < X
                if let Expr::BinaryExpr(binary) = filter {
                    if let Expr::Column(col) = &*binary.left {
                        if col.name == "id" {
                            match binary.op {
                                Operator::Eq | Operator::Gt | Operator::Lt |
                                Operator::GtEq | Operator::LtEq => {
                                    return TableProviderFilterPushDown::Exact;
                                }
                                _ => {}
                            }
                        }
                    }
                }

                // Check for BETWEEN expression
                if let Expr::Between(between) = filter {
                    if let Expr::Column(col) = &*between.expr {
                        if col.name == "id" {
                            return TableProviderFilterPushDown::Exact;
                        }
                    }
                }

                TableProviderFilterPushDown::Unsupported
            })
            .collect())
    }

    async fn scan(
        &self,
        _state: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        _limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        // Check if this is a point query (most specific)
        if let Some(key) = Self::is_point_query(filters) {
            // Use learned index for point query
            let batch = self.execute_point_query(key)?;

            // Apply projection if needed
            let batch = if let Some(proj) = projection {
                batch.project(proj)?
            } else {
                batch
            };

            let exec =
                MemoryExec::try_new(&[vec![batch]], self.schema.clone(), projection.cloned())?;
            return Ok(Arc::new(exec));
        }

        // Check if this is a range query
        if let Some((start_key, end_key)) = Self::is_range_query(filters) {
            // Use learned index for range query
            let batch = self.execute_range_query(start_key, end_key)?;

            // Apply projection if needed
            let batch = if let Some(proj) = projection {
                batch.project(proj)?
            } else {
                batch
            };

            let exec =
                MemoryExec::try_new(&[vec![batch]], self.schema.clone(), projection.cloned())?;
            return Ok(Arc::new(exec));
        }

        // Fall back to full scan
        let batch = self.execute_full_scan()?;

        // Apply projection if needed
        let batch = if let Some(proj) = projection {
            batch.project(proj)?
        } else {
            batch
        };

        let exec = MemoryExec::try_new(&[vec![batch]], self.schema.clone(), projection.cloned())?;
        Ok(Arc::new(exec))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redb_storage::RedbStorage;
    use datafusion::prelude::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_datafusion_point_query() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_df.redb");

        let mut storage = RedbStorage::new(&db_path).unwrap();

        // Insert test data
        storage.insert(1, b"value_1").unwrap();
        storage.insert(2, b"value_2").unwrap();
        storage.insert(3, b"value_3").unwrap();

        let storage = Arc::new(RwLock::new(storage));
        let table = RedbTable::new(storage, "test_table");

        // Create DataFusion context
        let ctx = SessionContext::new();
        ctx.register_table("test_table", Arc::new(table)).unwrap();

        // Execute point query
        let df = ctx
            .sql("SELECT * FROM test_table WHERE id = 2")
            .await
            .unwrap();
        let results = df.collect().await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].num_rows(), 1);
    }

    #[tokio::test]
    async fn test_datafusion_full_scan() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_df_scan.redb");

        let mut storage = RedbStorage::new(&db_path).unwrap();

        for i in 0..10 {
            storage
                .insert(i, format!("value_{}", i).as_bytes())
                .unwrap();
        }

        let storage = Arc::new(RwLock::new(storage));
        let table = RedbTable::new(storage, "test_table");

        let ctx = SessionContext::new();
        ctx.register_table("test_table", Arc::new(table)).unwrap();

        let df = ctx.sql("SELECT * FROM test_table").await.unwrap();
        let results = df.collect().await.unwrap();

        let total_rows: usize = results.iter().map(|b| b.num_rows()).sum();
        assert_eq!(total_rows, 10);
    }

    #[tokio::test]
    async fn test_datafusion_projection() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_df_proj.redb");

        let mut storage = RedbStorage::new(&db_path).unwrap();
        storage.insert(42, b"test_value").unwrap();

        let storage = Arc::new(RwLock::new(storage));
        let table = RedbTable::new(storage, "test_table");

        let ctx = SessionContext::new();
        ctx.register_table("test_table", Arc::new(table)).unwrap();

        let df = ctx
            .sql("SELECT id FROM test_table WHERE id = 42")
            .await
            .unwrap();
        let results = df.collect().await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].num_columns(), 1);
        assert_eq!(results[0].num_rows(), 1);
    }

    #[tokio::test]
    async fn test_datafusion_aggregation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_df_agg.redb");

        let mut storage = RedbStorage::new(&db_path).unwrap();

        for i in 1..=100 {
            storage.insert(i, b"value").unwrap();
        }

        let storage = Arc::new(RwLock::new(storage));
        let table = RedbTable::new(storage, "test_table");

        let ctx = SessionContext::new();
        ctx.register_table("test_table", Arc::new(table)).unwrap();

        let df = ctx
            .sql("SELECT COUNT(*) as count FROM test_table")
            .await
            .unwrap();
        let results = df.collect().await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].num_rows(), 1);
    }

    #[tokio::test]
    async fn test_datafusion_range_query_between() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_df_range_between.redb");

        let mut storage = RedbStorage::new(&db_path).unwrap();

        // Insert 1000 rows
        for i in 0..1000 {
            storage
                .insert(i, format!("value_{}", i).as_bytes())
                .unwrap();
        }

        let storage = Arc::new(RwLock::new(storage));
        let table = RedbTable::new(storage, "test_table");

        let ctx = SessionContext::new();
        ctx.register_table("test_table", Arc::new(table)).unwrap();

        // Test BETWEEN clause (should use learned index range_query)
        let df = ctx
            .sql("SELECT * FROM test_table WHERE id BETWEEN 400 AND 600")
            .await
            .unwrap();
        let results = df.collect().await.unwrap();

        let total_rows: usize = results.iter().map(|b| b.num_rows()).sum();
        assert_eq!(total_rows, 201, "Should return 201 rows (400-600 inclusive)");

        // Verify all returned rows are in range
        for batch in &results {
            let id_array = batch
                .column(0)
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap();
            for i in 0..id_array.len() {
                let id = id_array.value(i);
                assert!(
                    id >= 400 && id <= 600,
                    "Row id {} should be in range 400-600",
                    id
                );
            }
        }
    }

    #[tokio::test]
    async fn test_datafusion_range_query_gte_lte() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_df_range_gte_lte.redb");

        let mut storage = RedbStorage::new(&db_path).unwrap();

        for i in 0..1000 {
            storage
                .insert(i, format!("value_{}", i).as_bytes())
                .unwrap();
        }

        let storage = Arc::new(RwLock::new(storage));
        let table = RedbTable::new(storage, "test_table");

        let ctx = SessionContext::new();
        ctx.register_table("test_table", Arc::new(table)).unwrap();

        // Test >= AND <= (should use learned index range_query)
        let df = ctx
            .sql("SELECT * FROM test_table WHERE id >= 250 AND id <= 350")
            .await
            .unwrap();
        let results = df.collect().await.unwrap();

        let total_rows: usize = results.iter().map(|b| b.num_rows()).sum();
        assert_eq!(total_rows, 101, "Should return 101 rows (250-350 inclusive)");
    }

    #[tokio::test]
    async fn test_datafusion_range_query_gt_lt() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_df_range_gt_lt.redb");

        let mut storage = RedbStorage::new(&db_path).unwrap();

        for i in 0..1000 {
            storage
                .insert(i, format!("value_{}", i).as_bytes())
                .unwrap();
        }

        let storage = Arc::new(RwLock::new(storage));
        let table = RedbTable::new(storage, "test_table");

        let ctx = SessionContext::new();
        ctx.register_table("test_table", Arc::new(table)).unwrap();

        // Test > AND < (exclusive bounds, should use learned index)
        let df = ctx
            .sql("SELECT * FROM test_table WHERE id > 100 AND id < 200")
            .await
            .unwrap();
        let results = df.collect().await.unwrap();

        let total_rows: usize = results.iter().map(|b| b.num_rows()).sum();
        assert_eq!(
            total_rows, 99,
            "Should return 99 rows (101-199, exclusive bounds)"
        );
    }

    #[tokio::test]
    async fn test_datafusion_range_query_large() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_df_range_large.redb");

        let mut storage = RedbStorage::new(&db_path).unwrap();

        // Insert 10K rows for more realistic test
        let batch: Vec<(i64, Vec<u8>)> = (0..10000)
            .map(|i| (i, format!("value_{}", i).into_bytes()))
            .collect();
        storage.insert_batch(batch).unwrap();

        let storage = Arc::new(RwLock::new(storage));
        let table = RedbTable::new(storage, "test_table");

        let ctx = SessionContext::new();
        ctx.register_table("test_table", Arc::new(table)).unwrap();

        // Large range query (should be much faster with learned index than full scan)
        let df = ctx
            .sql("SELECT * FROM test_table WHERE id BETWEEN 4000 AND 6000")
            .await
            .unwrap();
        let results = df.collect().await.unwrap();

        let total_rows: usize = results.iter().map(|b| b.num_rows()).sum();
        assert_eq!(total_rows, 2001, "Should return 2001 rows");
    }

    #[tokio::test]
    async fn test_datafusion_range_query_projection() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_df_range_proj.redb");

        let mut storage = RedbStorage::new(&db_path).unwrap();

        for i in 0..100 {
            storage.insert(i, b"value").unwrap();
        }

        let storage = Arc::new(RwLock::new(storage));
        let table = RedbTable::new(storage, "test_table");

        let ctx = SessionContext::new();
        ctx.register_table("test_table", Arc::new(table)).unwrap();

        // Range query with projection
        let df = ctx
            .sql("SELECT id FROM test_table WHERE id BETWEEN 30 AND 40")
            .await
            .unwrap();
        let results = df.collect().await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].num_columns(), 1, "Should project only id column");
        assert_eq!(results[0].num_rows(), 11, "Should return 11 rows");
    }

    #[tokio::test]
    async fn test_learned_index_usage_verification() {
        use crate::metrics::*;

        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_df_learned_index_verify.redb");

        let mut storage = RedbStorage::new(&db_path).unwrap();

        // Insert 10K rows to ensure learned index is trained
        let batch: Vec<(i64, Vec<u8>)> = (0..10000)
            .map(|i| (i, format!("value_{}", i).into_bytes()))
            .collect();
        storage.insert_batch(batch).unwrap();

        let storage = Arc::new(RwLock::new(storage));
        let table = RedbTable::new(storage, "test_table");

        let ctx = SessionContext::new();
        ctx.register_table("test_table", Arc::new(table)).unwrap();

        // Record baseline metrics
        let baseline_learned_path = QUERY_PATH.with_label_values(&["learned_index"]).get();
        let baseline_searches = TOTAL_SEARCHES.get();

        // Execute range query (should use learned index)
        let df = ctx
            .sql("SELECT * FROM test_table WHERE id BETWEEN 3000 AND 4000")
            .await
            .unwrap();
        let results = df.collect().await.unwrap();

        // Verify results are correct
        let total_rows: usize = results.iter().map(|b| b.num_rows()).sum();
        assert_eq!(total_rows, 1001, "Should return 1001 rows");

        // Verify learned index path was taken (metrics should have increased)
        let learned_path_after = QUERY_PATH.with_label_values(&["learned_index"]).get();
        let searches_after = TOTAL_SEARCHES.get();

        // The range query should have used the learned_index path
        assert!(
            learned_path_after > baseline_learned_path,
            "Learned index path should have been used (path count: {} -> {})",
            baseline_learned_path,
            learned_path_after
        );

        assert!(
            searches_after > baseline_searches,
            "Search metrics should have increased (searches: {} -> {})",
            baseline_searches,
            searches_after
        );
    }
}
