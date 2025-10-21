//! SQL execution engine for OmenDB
//! Parses and executes SQL statements using the multi-table catalog

use crate::catalog::Catalog;
use crate::metrics::{record_sql_query, record_sql_query_error};
use crate::row::Row;
use crate::value::Value;
use crate::wal::{Transaction, TransactionManager, WalManager};
use anyhow::{anyhow, Result};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use sqlparser::ast::{
    ColumnDef, DataType as SqlDataType, Expr, Ident, Join, JoinConstraint, JoinOperator,
    ObjectName, OrderByExpr, Query, Select, SelectItem, SetExpr, Statement, TableFactor, Values,
};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, instrument, warn};

/// Configuration for SQL engine execution
#[derive(Clone, Debug)]
pub struct QueryConfig {
    /// Maximum query execution time (default: 30 seconds)
    pub timeout: Duration,

    /// Maximum number of rows to return (default: 1 million)
    pub max_rows: usize,

    /// Maximum memory per query in bytes (default: 1GB)
    pub max_memory_bytes: usize,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_rows: 1_000_000,
            max_memory_bytes: 1_000_000_000, // 1GB
        }
    }
}

/// SQL execution engine with transaction support
pub struct SqlEngine {
    catalog: Catalog,
    config: QueryConfig,
    transaction_manager: Option<Arc<TransactionManager>>,
    current_transaction: Arc<Mutex<Option<Transaction>>>,
}

impl SqlEngine {
    /// Create new SQL engine with catalog and default config
    pub fn new(catalog: Catalog) -> Self {
        Self::with_config(catalog, QueryConfig::default())
    }

    /// Create new SQL engine with custom configuration
    pub fn with_config(catalog: Catalog, config: QueryConfig) -> Self {
        Self {
            catalog,
            config,
            transaction_manager: None,
            current_transaction: Arc::new(Mutex::new(None)),
        }
    }

    /// Create new SQL engine with transaction support
    pub fn with_transactions(catalog: Catalog, config: QueryConfig, wal: Arc<WalManager>) -> Self {
        let tx_manager = Arc::new(TransactionManager::new(wal));
        Self {
            catalog,
            config,
            transaction_manager: Some(tx_manager),
            current_transaction: Arc::new(Mutex::new(None)),
        }
    }

    /// Execute a SQL statement with timeout and resource limits
    #[instrument(skip(self, sql), fields(query_length = sql.len()))]
    pub fn execute(&mut self, sql: &str) -> Result<ExecutionResult> {
        let start_time = Instant::now();

        debug!(query = %sql, "Executing SQL query");

        // Check query size limit (default: 10MB)
        if sql.len() > 10_000_000 {
            error!(query_size = sql.len(), "Query size exceeds limit");
            record_sql_query_error("query_too_large");
            return Err(anyhow!("Query size exceeds limit (10MB)"));
        }

        let dialect = GenericDialect {};
        let statements = match Parser::parse_sql(&dialect, sql) {
            Ok(stmts) => {
                debug!(statement_count = stmts.len(), "SQL parsed successfully");
                stmts
            }
            Err(e) => {
                error!(error = %e, "SQL parse error");
                record_sql_query_error("parse_error");
                return Err(anyhow!("SQL parse error: {}", e));
            }
        };

        if statements.is_empty() {
            warn!("Empty SQL query received");
            record_sql_query_error("empty_query");
            return Err(anyhow!("No SQL statement found"));
        }

        if statements.len() > 1 {
            warn!(
                count = statements.len(),
                "Multiple statements not supported"
            );
            record_sql_query_error("multiple_statements");
            return Err(anyhow!("Multiple statements not supported"));
        }

        // Check timeout before execution
        if start_time.elapsed() > self.config.timeout {
            error!("Query timed out during parsing");
            record_sql_query_error("timeout");
            return Err(anyhow!("Query timed out during parsing"));
        }

        // Execute query and record metrics
        let result = match &statements[0] {
            Statement::StartTransaction { .. } => {
                info!("Beginning transaction");
                self.begin_transaction()
            }
            Statement::Commit { .. } => {
                info!("Committing transaction");
                self.commit_transaction()
            }
            Statement::Rollback { .. } => {
                info!("Rolling back transaction");
                self.rollback_transaction()
            }
            Statement::CreateTable(stmt) => {
                info!(table = %stmt.name, "Creating table");
                self.execute_create_table(&stmt.name, &stmt.columns)
            }
            Statement::Insert(stmt) => {
                debug!(table = %stmt.table_name, "Inserting data");
                self.execute_insert(&stmt.table_name, &stmt.source)
            }
            Statement::Update {
                table,
                assignments,
                selection,
                ..
            } => {
                // Extract table name from TableWithJoins
                let table_name = match &table.relation {
                    TableFactor::Table { name, .. } => name,
                    _ => return Err(anyhow!("Complex table expressions not supported in UPDATE")),
                };
                info!(table = %table_name, "Updating data");
                self.execute_update(table_name, assignments, selection.as_ref())
            }
            Statement::Delete(delete_stmt) => {
                info!("Deleting data");
                self.execute_delete(&delete_stmt.from, delete_stmt.selection.as_ref())
            }
            Statement::Query(query) => {
                debug!("Executing SELECT query");
                self.execute_query_with_limits(query, start_time)
            }
            _ => {
                warn!("Unsupported SQL statement type");
                record_sql_query_error("unsupported_statement");
                return Err(anyhow!("Unsupported SQL statement"));
            }
        };

        // Record metrics based on result
        match &result {
            Ok(exec_result) => {
                let duration = start_time.elapsed().as_secs_f64();
                let (query_type, rows) = match exec_result {
                    ExecutionResult::Created { .. } => ("CREATE_TABLE", 0),
                    ExecutionResult::Inserted { rows } => ("INSERT", *rows),
                    ExecutionResult::Updated { rows } => ("UPDATE", *rows),
                    ExecutionResult::Deleted { rows } => ("DELETE", *rows),
                    ExecutionResult::Selected { rows, .. } => ("SELECT", *rows),
                    ExecutionResult::TransactionStarted { .. } => ("BEGIN", 0),
                    ExecutionResult::TransactionCommitted { .. } => ("COMMIT", 0),
                    ExecutionResult::TransactionRolledBack { .. } => ("ROLLBACK", 0),
                };
                info!(
                    query_type = query_type,
                    rows = rows,
                    duration_ms = duration * 1000.0,
                    "Query executed successfully"
                );
                record_sql_query(query_type, duration, rows);
            }
            Err(e) => {
                let error_type = if e.to_string().contains("timeout") {
                    "timeout"
                } else if e.to_string().contains("exceeds maximum row limit") {
                    "row_limit_exceeded"
                } else if e.to_string().contains("not found") {
                    "not_found"
                } else {
                    "execution_error"
                };
                error!(error = %e, error_type = error_type, "Query execution failed");
                record_sql_query_error(error_type);
            }
        }

        result
    }

    /// Check if query has exceeded timeout
    fn check_timeout(&self, start_time: Instant) -> Result<()> {
        if start_time.elapsed() > self.config.timeout {
            return Err(anyhow!(
                "Query execution timeout ({} seconds)",
                self.config.timeout.as_secs()
            ));
        }
        Ok(())
    }

    /// Begin a new transaction
    fn begin_transaction(&self) -> Result<ExecutionResult> {
        // Check if transactions are enabled
        let tx_manager = self.transaction_manager.as_ref().ok_or_else(|| {
            anyhow!("Transactions not enabled. Use SqlEngine::with_transactions()")
        })?;

        // Check if there's already an active transaction
        let mut current_tx = self
            .current_transaction
            .lock()
            .map_err(|e| anyhow!("Transaction mutex poisoned: {}", e))?;

        if current_tx.is_some() {
            return Err(anyhow!(
                "Transaction already in progress. Commit or rollback first."
            ));
        }

        // Begin new transaction
        let transaction = tx_manager.begin()?;
        let txn_id = transaction.id();
        *current_tx = Some(transaction);

        debug!(txn_id = txn_id, "Transaction started");

        Ok(ExecutionResult::TransactionStarted { txn_id })
    }

    /// Commit the current transaction
    fn commit_transaction(&self) -> Result<ExecutionResult> {
        let mut current_tx = self
            .current_transaction
            .lock()
            .map_err(|e| anyhow!("Transaction mutex poisoned: {}", e))?;

        let transaction = current_tx
            .take()
            .ok_or_else(|| anyhow!("No active transaction to commit"))?;

        let txn_id = transaction.id();
        transaction.commit()?;

        debug!(txn_id = txn_id, "Transaction committed");

        Ok(ExecutionResult::TransactionCommitted { txn_id })
    }

    /// Rollback the current transaction
    fn rollback_transaction(&self) -> Result<ExecutionResult> {
        let mut current_tx = self
            .current_transaction
            .lock()
            .map_err(|e| anyhow!("Transaction mutex poisoned: {}", e))?;

        let transaction = current_tx
            .take()
            .ok_or_else(|| anyhow!("No active transaction to rollback"))?;

        let txn_id = transaction.id();
        transaction.rollback()?;

        debug!(txn_id = txn_id, "Transaction rolled back");

        Ok(ExecutionResult::TransactionRolledBack { txn_id })
    }

    /// Execute CREATE TABLE statement
    fn execute_create_table(
        &mut self,
        name: &ObjectName,
        columns: &[ColumnDef],
    ) -> Result<ExecutionResult> {
        let table_name = Self::extract_table_name(name)?;

        // Extract columns
        let mut fields = Vec::new();
        let mut primary_key = None;

        for column in columns {
            let field_name = column.name.value.clone();
            let data_type = Self::sql_type_to_arrow(&column.data_type)?;
            let nullable = !column
                .options
                .iter()
                .any(|opt| matches!(opt.option, sqlparser::ast::ColumnOption::NotNull));

            // Check if this is the primary key
            if column
                .options
                .iter()
                .any(|opt| matches!(&opt.option, sqlparser::ast::ColumnOption::Unique { .. }))
            {
                primary_key = Some(field_name.clone());
            }

            fields.push(Field::new(field_name, data_type, nullable));
        }

        // Default to first column as primary key if not specified
        let primary_key = primary_key.unwrap_or_else(|| fields[0].name().clone());

        let schema = Arc::new(Schema::new(fields));
        self.catalog
            .create_table(table_name.clone(), schema, primary_key)?;

        Ok(ExecutionResult::Created {
            message: format!("Table '{}' created", table_name),
        })
    }

    /// Execute INSERT statement
    fn execute_insert(
        &mut self,
        table_name: &ObjectName,
        source: &Option<Box<Query>>,
    ) -> Result<ExecutionResult> {
        let table_name = Self::extract_table_name(table_name)?;
        let table = self.catalog.get_table_mut(&table_name)?;
        let schema = table.schema().clone();

        // Extract values
        let source = source
            .as_ref()
            .ok_or_else(|| anyhow!("No source for INSERT"))?;

        let rows_inserted = match source.body.as_ref() {
            SetExpr::Values(Values { rows, .. }) => {
                let mut count = 0;
                for row_values in rows {
                    let mut values = Vec::new();

                    for (i, expr) in row_values.iter().enumerate() {
                        if i >= schema.fields().len() {
                            return Err(anyhow!("Too many values for INSERT"));
                        }

                        let value = Self::expr_to_value(expr, schema.field(i).data_type())?;
                        values.push(value);
                    }

                    let row = Row::new(values);
                    table.insert(row)?;
                    count += 1;
                }
                count
            }
            _ => return Err(anyhow!("Only VALUES clause supported for INSERT")),
        };

        Ok(ExecutionResult::Inserted {
            rows: rows_inserted,
        })
    }

    /// Execute UPDATE statement
    fn execute_update(
        &mut self,
        table_name: &ObjectName,
        assignments: &[sqlparser::ast::Assignment],
        selection: Option<&Expr>,
    ) -> Result<ExecutionResult> {
        let table_name = Self::extract_table_name(table_name)?;

        // Get schema and primary key info before mutable borrow
        let (schema, primary_key) = {
            let table = self.catalog.get_table(&table_name)?;
            (table.schema().clone(), table.primary_key().to_string())
        };

        // Extract WHERE clause to get primary key value
        let key_value = if let Some(where_expr) = selection {
            Self::extract_primary_key_from_where_static(where_expr, &primary_key, &schema)?
        } else {
            return Err(anyhow!("UPDATE without WHERE clause not supported yet"));
        };

        // Now get mutable table reference
        let table = self.catalog.get_table_mut(&table_name)?;

        // Get existing row
        let existing_row = table
            .get(&key_value)?
            .ok_or_else(|| anyhow!("Row with key {:?} not found", key_value))?;

        // Build updated row by applying assignments
        let mut updated_values: Vec<Value> = existing_row.values().to_vec();

        for assignment in assignments {
            let col_name = match &assignment.target {
                sqlparser::ast::AssignmentTarget::ColumnName(name) => {
                    name.0.iter().map(|i| i.value.as_str()).collect::<Vec<_>>().join(".")
                }
                _ => return Err(anyhow!("Complex assignment targets not supported")),
            };

            // PRIMARY KEY constraint: Cannot update primary key column
            if col_name == primary_key {
                return Err(anyhow!(
                    "Cannot update PRIMARY KEY column '{}'. PRIMARY KEY values are immutable.",
                    primary_key
                ));
            }

            let col_idx = schema.index_of(&col_name)?;
            let col_type = schema.field(col_idx).data_type();

            // Evaluate the new value
            let new_value = Self::expr_to_value(&assignment.value, col_type)?;
            updated_values[col_idx] = new_value;
        }

        let updated_row = Row::new(updated_values);

        // Call Table::update
        let rows_updated = table.update(&key_value, updated_row)?;

        Ok(ExecutionResult::Updated {
            rows: rows_updated,
        })
    }

    /// Execute DELETE statement
    fn execute_delete(
        &mut self,
        from: &sqlparser::ast::FromTable,
        selection: Option<&Expr>,
    ) -> Result<ExecutionResult> {
        // Extract table name from FromTable
        let table_name = match from {
            sqlparser::ast::FromTable::WithFromKeyword(tables) => {
                if tables.is_empty() {
                    return Err(anyhow!("DELETE requires table name"));
                }
                match &tables[0].relation {
                    TableFactor::Table { name, .. } => Self::extract_table_name(name)?,
                    _ => return Err(anyhow!("Complex table expressions not supported in DELETE")),
                }
            }
            sqlparser::ast::FromTable::WithoutKeyword(tables) => {
                if tables.is_empty() {
                    return Err(anyhow!("DELETE requires table name"));
                }
                match &tables[0].relation {
                    TableFactor::Table { name, .. } => Self::extract_table_name(name)?,
                    _ => return Err(anyhow!("Complex table expressions not supported in DELETE")),
                }
            }
        };

        // Get schema and primary key info before mutable borrow
        let (schema, primary_key) = {
            let table = self.catalog.get_table(&table_name)?;
            (table.schema().clone(), table.primary_key().to_string())
        };

        // Extract WHERE clause to get primary key value
        let key_value = if let Some(where_expr) = selection {
            Self::extract_primary_key_from_where_static(where_expr, &primary_key, &schema)?
        } else {
            return Err(anyhow!("DELETE without WHERE clause not supported yet"));
        };

        // Now get mutable table reference
        let table = self.catalog.get_table_mut(&table_name)?;

        // Call Table::delete
        let rows_deleted = table.delete(&key_value)?;

        Ok(ExecutionResult::Deleted {
            rows: rows_deleted,
        })
    }

    /// Extract primary key value from WHERE clause (static method)
    /// Currently supports simple equality: WHERE id = 5
    fn extract_primary_key_from_where_static(
        expr: &Expr,
        primary_key: &str,
        schema: &SchemaRef,
    ) -> Result<Value> {
        use sqlparser::ast::BinaryOperator;

        match expr {
            Expr::BinaryOp { left, op, right } if matches!(op, BinaryOperator::Eq) => {
                // Check if this is primary key equality
                if let (Expr::Identifier(col), Expr::Value(val)) = (left.as_ref(), right.as_ref())
                {
                    if col.value == primary_key {
                        let pk_field = schema.field_with_name(primary_key)?;
                        return Self::sql_value_to_value(val, pk_field.data_type());
                    }
                }
                // Try reversed: WHERE 5 = id
                if let (Expr::Value(val), Expr::Identifier(col)) = (left.as_ref(), right.as_ref())
                {
                    if col.value == primary_key {
                        let pk_field = schema.field_with_name(primary_key)?;
                        return Self::sql_value_to_value(val, pk_field.data_type());
                    }
                }
                Err(anyhow!(
                    "WHERE clause must be equality on primary key '{}' for UPDATE/DELETE",
                    primary_key
                ))
            }
            _ => Err(anyhow!(
                "Only simple WHERE primary_key = value supported for UPDATE/DELETE"
            )),
        }
    }

    /// Execute SELECT query
    /// Execute SELECT query with timeout and resource limits
    fn execute_query_with_limits(
        &self,
        query: &Query,
        start_time: Instant,
    ) -> Result<ExecutionResult> {
        // Check timeout before query execution
        self.check_timeout(start_time)?;

        let order_by = match &query.order_by {
            Some(order) => order.exprs.as_slice(),
            None => &[],
        };

        let mut result = match query.body.as_ref() {
            SetExpr::Select(select) => self.execute_select(select, order_by)?,
            _ => return Err(anyhow!("Only SELECT queries supported")),
        };

        // Check timeout after query execution
        self.check_timeout(start_time)?;

        // Apply OFFSET first, then LIMIT (standard SQL semantics)
        if let Some(offset_expr) = &query.offset {
            if let sqlparser::ast::Offset { value, .. } = offset_expr {
                if let Expr::Value(sqlparser::ast::Value::Number(n, _)) = value {
                    let offset: usize = n.parse()?;
                    if let ExecutionResult::Selected {
                        columns,
                        rows,
                        mut data,
                    } = result
                    {
                        if offset < data.len() {
                            data = data.into_iter().skip(offset).collect();
                        } else {
                            data.clear();
                        }
                        result = ExecutionResult::Selected {
                            columns,
                            rows: data.len(),
                            data,
                        };
                    }
                }
            }
        }

        // Apply LIMIT after OFFSET
        if let Some(limit_expr) = &query.limit {
            if let Expr::Value(sqlparser::ast::Value::Number(n, _)) = limit_expr {
                let limit: usize = n.parse()?;
                if let ExecutionResult::Selected {
                    columns,
                    rows,
                    mut data,
                } = result
                {
                    data.truncate(limit);
                    result = ExecutionResult::Selected {
                        columns,
                        rows: data.len(),
                        data,
                    };
                }
            }
        }

        // Enforce maximum row limit
        if let ExecutionResult::Selected {
            columns,
            rows,
            data,
        } = result
        {
            if data.len() > self.config.max_rows {
                return Err(anyhow!(
                    "Query result exceeds maximum row limit ({} rows). Use LIMIT clause to restrict results.",
                    self.config.max_rows
                ));
            }
            result = ExecutionResult::Selected {
                columns,
                rows,
                data,
            };
        }

        Ok(result)
    }

    /// Execute SELECT query (legacy method for backward compatibility)
    fn execute_query(&self, query: &Query) -> Result<ExecutionResult> {
        self.execute_query_with_limits(query, Instant::now())
    }

    /// Execute SELECT statement
    fn execute_select(&self, select: &Select, order_by: &[OrderByExpr]) -> Result<ExecutionResult> {
        // Check if this is a JOIN query
        if select.from.len() == 1 && !select.from[0].joins.is_empty() {
            return self.execute_join(select, order_by);
        }

        // Extract table name (single table query)
        if select.from.len() != 1 {
            return Err(anyhow!("Only single table SELECT supported"));
        }

        let table_name = match &select.from[0].relation {
            TableFactor::Table { name, .. } => Self::extract_table_name(name)?,
            _ => return Err(anyhow!("Only table SELECT supported")),
        };

        let table = self.catalog.get_table(&table_name)?;

        // Get rows based on WHERE clause
        let mut rows = if let Some(ref selection) = select.selection {
            self.execute_where_clause(table, selection)?
        } else {
            // No WHERE clause - scan all
            table.scan_all()?
        };

        // Check if this is an aggregate query
        let has_aggregates = select
            .projection
            .iter()
            .any(|item| matches!(item, SelectItem::UnnamedExpr(Expr::Function(_))));

        if has_aggregates {
            // Handle aggregate query
            return self.execute_aggregate_query(&select.projection, rows, &select.group_by, table);
        }

        // Apply ORDER BY if present (non-aggregate queries)
        if !order_by.is_empty() {
            rows = self.apply_order_by(rows, order_by, table)?;
        }

        // Extract column names to return
        let column_names: Vec<String> = match &select.projection[0] {
            SelectItem::Wildcard(_) => table
                .schema()
                .fields()
                .iter()
                .map(|f| f.name().clone())
                .collect(),
            _ => select
                .projection
                .iter()
                .filter_map(|item| {
                    if let SelectItem::UnnamedExpr(Expr::Identifier(ident)) = item {
                        Some(ident.value.clone())
                    } else {
                        None
                    }
                })
                .collect(),
        };

        Ok(ExecutionResult::Selected {
            columns: column_names,
            rows: rows.len(),
            data: rows,
        })
    }

    /// Execute JOIN query (INNER JOIN, LEFT JOIN)
    fn execute_join(&self, select: &Select, order_by: &[OrderByExpr]) -> Result<ExecutionResult> {
        // Extract left table
        let left_table_name = match &select.from[0].relation {
            TableFactor::Table { name, .. } => Self::extract_table_name(name)?,
            _ => return Err(anyhow!("Only table JOINs supported")),
        };

        // Extract right table and join operator
        if select.from[0].joins.len() != 1 {
            return Err(anyhow!("Only single JOIN supported (no multi-way joins yet)"));
        }

        let join = &select.from[0].joins[0];
        let right_table_name = match &join.relation {
            TableFactor::Table { name, .. } => Self::extract_table_name(name)?,
            _ => return Err(anyhow!("Only table JOINs supported")),
        };

        // Get tables
        let left_table = self.catalog.get_table(&left_table_name)?;
        let right_table = self.catalog.get_table(&right_table_name)?;

        // Parse join condition
        let (left_col, right_col, is_left_join) = match &join.join_operator {
            JoinOperator::Inner(constraint) => {
                let (l, r) = Self::parse_join_condition(constraint)?;
                (l, r, false)
            }
            JoinOperator::LeftOuter(constraint) => {
                let (l, r) = Self::parse_join_condition(constraint)?;
                (l, r, true)
            }
            _ => return Err(anyhow!("Only INNER JOIN and LEFT JOIN supported")),
        };

        // Get all rows from both tables
        let left_rows = left_table.scan_all()?;
        let right_rows = right_table.scan_all()?;

        // Find column indices for join
        let left_col_idx = left_table
            .schema()
            .index_of(&left_col)
            .map_err(|_| anyhow!("Column '{}' not found in table '{}'", left_col, left_table_name))?;

        let right_col_idx = right_table
            .schema()
            .index_of(&right_col)
            .map_err(|_| anyhow!("Column '{}' not found in table '{}'", right_col, right_table_name))?;

        // Perform join (nested loop)
        let mut result_rows = Vec::new();

        for left_row in &left_rows {
            let left_join_val = left_row.get(left_col_idx)?;
            let mut found_match = false;

            for right_row in &right_rows {
                let right_join_val = right_row.get(right_col_idx)?;

                // Check if join condition matches
                if left_join_val == right_join_val {
                    // Combine rows
                    let mut combined_values = Vec::new();
                    for i in 0..left_row.len() {
                        combined_values.push(left_row.get(i)?.clone());
                    }
                    for i in 0..right_row.len() {
                        combined_values.push(right_row.get(i)?.clone());
                    }
                    result_rows.push(Row::new(combined_values));
                    found_match = true;
                }
            }

            // LEFT JOIN: emit row with NULLs if no match
            if is_left_join && !found_match {
                let mut combined_values = Vec::new();
                for i in 0..left_row.len() {
                    combined_values.push(left_row.get(i)?.clone());
                }
                // Add NULLs for right table columns
                let right_table_col_count = right_table.schema().fields().len();
                for _ in 0..right_table_col_count {
                    combined_values.push(Value::Null);
                }
                result_rows.push(Row::new(combined_values));
            }
        }

        // Build combined schema (prefix columns with table names)
        let mut combined_fields = Vec::new();
        for field in left_table.schema().fields() {
            combined_fields.push(Field::new(
                format!("{}.{}", left_table_name, field.name()),
                field.data_type().clone(),
                field.is_nullable(),
            ));
        }
        for field in right_table.schema().fields() {
            combined_fields.push(Field::new(
                format!("{}.{}", right_table_name, field.name()),
                field.data_type().clone(),
                field.is_nullable(),
            ));
        }
        let combined_schema = Arc::new(Schema::new(combined_fields));

        // Apply WHERE clause if present
        if let Some(ref selection) = select.selection {
            result_rows = self.execute_where_clause_with_schema(&combined_schema, result_rows, selection)?;
        }

        // Apply ORDER BY if present
        if !order_by.is_empty() {
            // TODO: Support ORDER BY with JOIN
            return Err(anyhow!("ORDER BY with JOIN not yet implemented"));
        }

        // Project requested columns
        let (column_names, projected_rows) = self.project_columns(
            select,
            result_rows,
            &combined_schema,
            &left_table_name,
            &right_table_name,
        )?;

        Ok(ExecutionResult::Selected {
            columns: column_names,
            rows: projected_rows.len(),
            data: projected_rows,
        })
    }

    /// Parse JOIN condition from ON clause
    fn parse_join_condition(constraint: &JoinConstraint) -> Result<(String, String)> {
        match constraint {
            JoinConstraint::On(expr) => {
                // Expect: table1.col = table2.col
                if let Expr::BinaryOp { left, op, right } = expr {
                    use sqlparser::ast::BinaryOperator;
                    if !matches!(op, BinaryOperator::Eq) {
                        return Err(anyhow!("Only equality joins (=) supported"));
                    }

                    let left_col = Self::extract_column_from_expr(left)?;
                    let right_col = Self::extract_column_from_expr(right)?;

                    Ok((left_col, right_col))
                } else {
                    Err(anyhow!("Invalid JOIN condition (expected col = col)"))
                }
            }
            _ => Err(anyhow!("Only ON clause supported for JOINs")),
        }
    }

    /// Extract column name from expression (handles table.column or column)
    fn extract_column_from_expr(expr: &Expr) -> Result<String> {
        match expr {
            Expr::Identifier(ident) => Ok(ident.value.clone()),
            Expr::CompoundIdentifier(idents) => {
                // table.column -> return column name
                if idents.len() == 2 {
                    Ok(idents[1].value.clone())
                } else {
                    Err(anyhow!("Invalid compound identifier in JOIN condition"))
                }
            }
            _ => Err(anyhow!("Invalid expression in JOIN condition")),
        }
    }

    /// Execute WHERE clause with explicit schema
    fn execute_where_clause_with_schema(
        &self,
        schema: &SchemaRef,
        rows: Vec<Row>,
        expr: &Expr,
    ) -> Result<Vec<Row>> {
        // Filter rows based on WHERE expression
        let filtered: Vec<Row> = rows
            .into_iter()
            .filter(|row| self.evaluate_where_expr_with_schema(schema, row, expr).unwrap_or(false))
            .collect();
        Ok(filtered)
    }

    /// Evaluate WHERE expression with explicit schema
    fn evaluate_where_expr_with_schema(
        &self,
        schema: &SchemaRef,
        row: &Row,
        expr: &Expr,
    ) -> Result<bool> {
        use sqlparser::ast::BinaryOperator;

        match expr {
            Expr::BinaryOp { left, op, right } => {
                match op {
                    BinaryOperator::Eq | BinaryOperator::NotEq
                    | BinaryOperator::Lt | BinaryOperator::LtEq
                    | BinaryOperator::Gt | BinaryOperator::GtEq => {
                        let left_val = self.evaluate_value_expr_with_schema(schema, row, left)?;
                        let right_val = self.evaluate_value_expr_with_schema(schema, row, right)?;

                        let result = match op {
                            BinaryOperator::Eq => left_val == right_val,
                            BinaryOperator::NotEq => left_val != right_val,
                            BinaryOperator::Gt => left_val > right_val,
                            BinaryOperator::GtEq => left_val >= right_val,
                            BinaryOperator::Lt => left_val < right_val,
                            BinaryOperator::LtEq => left_val <= right_val,
                            _ => false,
                        };
                        Ok(result)
                    }
                    _ => Err(anyhow!("Unsupported operator in WHERE clause")),
                }
            }
            _ => Err(anyhow!("Unsupported expression in WHERE clause")),
        }
    }

    /// Evaluate value expression with explicit schema
    fn evaluate_value_expr_with_schema(
        &self,
        schema: &SchemaRef,
        row: &Row,
        expr: &Expr,
    ) -> Result<Value> {
        match expr {
            Expr::Identifier(ident) => {
                // Try to find column in schema
                let col_idx = schema
                    .index_of(&ident.value)
                    .map_err(|_| anyhow!("Column '{}' not found", ident.value))?;
                Ok(row.get(col_idx)?.clone())
            }
            Expr::CompoundIdentifier(idents) => {
                // table.column
                if idents.len() == 2 {
                    let full_name = format!("{}.{}", idents[0].value, idents[1].value);
                    let col_idx = schema
                        .index_of(&full_name)
                        .map_err(|_| anyhow!("Column '{}' not found", full_name))?;
                    Ok(row.get(col_idx)?.clone())
                } else {
                    Err(anyhow!("Invalid compound identifier"))
                }
            }
            Expr::Value(val) => {
                // Convert SQL value to our Value type
                match val {
                    sqlparser::ast::Value::Number(n, _) => {
                        if n.contains('.') {
                            Ok(Value::Float64(n.parse()?))
                        } else {
                            Ok(Value::Int64(n.parse()?))
                        }
                    }
                    sqlparser::ast::Value::SingleQuotedString(s) => Ok(Value::Text(s.clone())),
                    sqlparser::ast::Value::Boolean(b) => Ok(Value::Boolean(*b)),
                    sqlparser::ast::Value::Null => Ok(Value::Null),
                    _ => Err(anyhow!("Unsupported SQL value type")),
                }
            }
            _ => Err(anyhow!("Unsupported expression in WHERE clause")),
        }
    }

    /// Project columns from joined rows
    fn project_columns(
        &self,
        select: &Select,
        rows: Vec<Row>,
        combined_schema: &SchemaRef,
        left_table: &str,
        right_table: &str,
    ) -> Result<(Vec<String>, Vec<Row>)> {
        // Handle wildcard (SELECT *)
        if matches!(&select.projection[0], SelectItem::Wildcard(_)) {
            let column_names: Vec<String> = combined_schema
                .fields()
                .iter()
                .map(|f| f.name().clone())
                .collect();
            return Ok((column_names, rows));
        }

        // Extract requested columns
        let mut column_indices = Vec::new();
        let mut column_names = Vec::new();

        for item in &select.projection {
            match item {
                SelectItem::UnnamedExpr(Expr::Identifier(ident)) => {
                    // Simple column name - could be ambiguous
                    // Try both tables
                    let full_left = format!("{}.{}", left_table, ident.value);
                    let full_right = format!("{}.{}", right_table, ident.value);

                    if let Ok(idx) = combined_schema.index_of(&full_left) {
                        column_indices.push(idx);
                        column_names.push(ident.value.clone());
                    } else if let Ok(idx) = combined_schema.index_of(&full_right) {
                        column_indices.push(idx);
                        column_names.push(ident.value.clone());
                    } else {
                        return Err(anyhow!("Column '{}' not found in either table", ident.value));
                    }
                }
                SelectItem::UnnamedExpr(Expr::CompoundIdentifier(idents)) => {
                    // table.column
                    if idents.len() == 2 {
                        let full_name = format!("{}.{}", idents[0].value, idents[1].value);
                        let idx = combined_schema
                            .index_of(&full_name)
                            .map_err(|_| anyhow!("Column '{}' not found", full_name))?;
                        column_indices.push(idx);
                        column_names.push(idents[1].value.clone());
                    } else {
                        return Err(anyhow!("Invalid column reference"));
                    }
                }
                _ => return Err(anyhow!("Unsupported SELECT item in JOIN")),
            }
        }

        // Project rows
        let projected_rows: Vec<Row> = rows
            .into_iter()
            .map(|row| {
                let values: Vec<Value> = column_indices
                    .iter()
                    .map(|&idx| row.get(idx).unwrap().clone())
                    .collect();
                Row::new(values)
            })
            .collect();

        Ok((column_names, projected_rows))
    }

    /// Execute aggregate query (COUNT, SUM, AVG, MIN, MAX)
    fn execute_aggregate_query(
        &self,
        projection: &[SelectItem],
        rows: Vec<Row>,
        group_by: &sqlparser::ast::GroupByExpr,
        table: &crate::table::Table,
    ) -> Result<ExecutionResult> {
        
        use std::collections::HashMap;

        // Parse GROUP BY columns
        let group_by_cols: Vec<String> = match group_by {
            sqlparser::ast::GroupByExpr::Expressions(exprs, _) => exprs
                .iter()
                .filter_map(|expr| {
                    if let Expr::Identifier(ident) = expr {
                        Some(ident.value.clone())
                    } else {
                        None
                    }
                })
                .collect(),
            sqlparser::ast::GroupByExpr::All(_) => Vec::new(),
        };

        // Group rows if GROUP BY is present
        // Use string keys since Value doesn't implement Eq+Hash (contains f64)
        let groups: HashMap<String, (Vec<Value>, Vec<Row>)> = if !group_by_cols.is_empty() {
            let mut groups = HashMap::new();

            for row in rows {
                let mut key_values = Vec::new();
                let mut key_str = String::new();

                for col_name in &group_by_cols {
                    let col_idx = table.schema().index_of(col_name)?;
                    let val = row.get(col_idx)?.clone();
                    key_str.push_str(&format!("{:?}|", val));
                    key_values.push(val);
                }

                let entry = groups
                    .entry(key_str)
                    .or_insert_with(|| (key_values.clone(), Vec::new()));
                entry.1.push(row);
            }
            groups
        } else {
            // No GROUP BY - treat all rows as single group
            let mut groups = HashMap::new();
            groups.insert(String::new(), (Vec::new(), rows));
            groups
        };

        // Process each group and compute aggregates
        let mut result_rows = Vec::new();
        let mut column_names = Vec::new();

        for (_group_key, (group_values, group_row_data)) in groups {
            let mut result_values = Vec::new();

            // Add GROUP BY columns first
            result_values.extend(group_values);

            // Process each projection item
            for item in projection {
                match item {
                    SelectItem::UnnamedExpr(Expr::Function(func)) => {
                        let agg_value = self.compute_aggregate(func, &group_row_data, table)?;
                        result_values.push(agg_value);

                        // Add column name for first group only
                        if column_names.len() < group_by_cols.len() + projection.len() {
                            // Extract argument description
                            let arg_desc = match &func.args {
                                sqlparser::ast::FunctionArguments::List(list) => {
                                    if list.args.is_empty() {
                                        "*"
                                    } else {
                                        "column"
                                    }
                                }
                                _ => "column",
                            };
                            column_names.push(format!(
                                "{}({})",
                                func.name.0[0].value.to_uppercase(),
                                arg_desc
                            ));
                        }
                    }
                    SelectItem::UnnamedExpr(Expr::Identifier(ident)) => {
                        // Non-aggregate column (must be in GROUP BY)
                        if column_names.len() < group_by_cols.len() {
                            column_names.push(ident.value.clone());
                        }
                    }
                    _ => return Err(anyhow!("Unsupported projection in aggregate query")),
                }
            }

            result_rows.push(Row::new(result_values));
        }

        Ok(ExecutionResult::Selected {
            columns: column_names,
            rows: result_rows.len(),
            data: result_rows,
        })
    }

    /// Compute single aggregate function
    fn compute_aggregate(
        &self,
        func: &sqlparser::ast::Function,
        rows: &[Row],
        table: &crate::table::Table,
    ) -> Result<Value> {
        use sqlparser::ast::{FunctionArg, FunctionArguments};

        let func_name = func.name.0[0].value.to_uppercase();

        match func_name.as_str() {
            "COUNT" => {
                // Extract args from FunctionArguments enum
                let args = match &func.args {
                    FunctionArguments::List(list) => &list.args,
                    _ => return Err(anyhow!("Invalid function arguments")),
                };

                if args.is_empty()
                    || matches!(
                        &args[0],
                        FunctionArg::Unnamed(sqlparser::ast::FunctionArgExpr::Wildcard)
                    )
                {
                    // COUNT(*) or COUNT()
                    Ok(Value::Int64(rows.len() as i64))
                } else {
                    // COUNT(column) - count non-null values
                    let col_idx = self.extract_column_index(&args[0], table)?;
                    let count = rows
                        .iter()
                        .filter(|row| !matches!(row.get(col_idx), Ok(Value::Null)))
                        .count();
                    Ok(Value::Int64(count as i64))
                }
            }
            "SUM" => {
                let args = match &func.args {
                    FunctionArguments::List(list) => &list.args,
                    _ => return Err(anyhow!("Invalid function arguments")),
                };
                let col_idx = self.extract_column_index(&args[0], table)?;
                let mut sum = 0.0;
                for row in rows {
                    match row.get(col_idx)? {
                        Value::Int64(n) => sum += *n as f64,
                        Value::Float64(f) => sum += f,
                        Value::Null => continue,
                        _ => return Err(anyhow!("SUM requires numeric column")),
                    }
                }
                Ok(Value::Float64(sum))
            }
            "AVG" => {
                let args = match &func.args {
                    FunctionArguments::List(list) => &list.args,
                    _ => return Err(anyhow!("Invalid function arguments")),
                };
                let col_idx = self.extract_column_index(&args[0], table)?;
                let mut sum = 0.0;
                let mut count = 0;
                for row in rows {
                    match row.get(col_idx)? {
                        Value::Int64(n) => {
                            sum += *n as f64;
                            count += 1;
                        }
                        Value::Float64(f) => {
                            sum += f;
                            count += 1;
                        }
                        Value::Null => continue,
                        _ => return Err(anyhow!("AVG requires numeric column")),
                    }
                }
                if count == 0 {
                    Ok(Value::Null)
                } else {
                    Ok(Value::Float64(sum / count as f64))
                }
            }
            "MIN" => {
                let args = match &func.args {
                    FunctionArguments::List(list) => &list.args,
                    _ => return Err(anyhow!("Invalid function arguments")),
                };
                let col_idx = self.extract_column_index(&args[0], table)?;
                let mut min_val: Option<Value> = None;
                for row in rows {
                    let val = row.get(col_idx)?;
                    if matches!(val, Value::Null) {
                        continue;
                    }
                    min_val = match min_val {
                        None => Some(val.clone()),
                        Some(ref current) => {
                            if Self::compare_values(val, current)? < 0 {
                                Some(val.clone())
                            } else {
                                Some(current.clone())
                            }
                        }
                    };
                }
                Ok(min_val.unwrap_or(Value::Null))
            }
            "MAX" => {
                let args = match &func.args {
                    FunctionArguments::List(list) => &list.args,
                    _ => return Err(anyhow!("Invalid function arguments")),
                };
                let col_idx = self.extract_column_index(&args[0], table)?;
                let mut max_val: Option<Value> = None;
                for row in rows {
                    let val = row.get(col_idx)?;
                    if matches!(val, Value::Null) {
                        continue;
                    }
                    max_val = match max_val {
                        None => Some(val.clone()),
                        Some(ref current) => {
                            if Self::compare_values(val, current)? > 0 {
                                Some(val.clone())
                            } else {
                                Some(current.clone())
                            }
                        }
                    };
                }
                Ok(max_val.unwrap_or(Value::Null))
            }
            _ => Err(anyhow!("Unsupported aggregate function: {}", func_name)),
        }
    }

    /// Extract column index from function argument
    fn extract_column_index(
        &self,
        arg: &sqlparser::ast::FunctionArg,
        table: &crate::table::Table,
    ) -> Result<usize> {
        use sqlparser::ast::FunctionArg;

        match arg {
            FunctionArg::Unnamed(sqlparser::ast::FunctionArgExpr::Expr(Expr::Identifier(
                ident,
            ))) => Ok(table.schema().index_of(&ident.value)?),
            _ => Err(anyhow!("Aggregate function requires column name")),
        }
    }

    /// Apply ORDER BY clause to rows
    fn apply_order_by(
        &self,
        mut rows: Vec<Row>,
        order_by: &[OrderByExpr],
        table: &crate::table::Table,
    ) -> Result<Vec<Row>> {
        if order_by.is_empty() {
            return Ok(rows);
        }

        // Get the column to order by (only support single column for now)
        let order_expr = &order_by[0];
        let column_name = match &order_expr.expr {
            Expr::Identifier(ident) => ident.value.clone(),
            _ => return Err(anyhow!("ORDER BY only supports column names")),
        };

        let column_idx = table.schema().index_of(&column_name)?;
        let is_asc = order_expr.asc.unwrap_or(true); // Default to ASC

        // Sort the rows
        rows.sort_by(|a, b| {
            let val_a = a.get(column_idx).ok();
            let val_b = b.get(column_idx).ok();

            let cmp = match (val_a, val_b) {
                (Some(a), Some(b)) => match Self::compare_values(a, b) {
                    Ok(c) if c < 0 => std::cmp::Ordering::Less,
                    Ok(c) if c > 0 => std::cmp::Ordering::Greater,
                    _ => std::cmp::Ordering::Equal,
                },
                (Some(_), None) => std::cmp::Ordering::Greater,
                (None, Some(_)) => std::cmp::Ordering::Less,
                (None, None) => std::cmp::Ordering::Equal,
            };

            if is_asc {
                cmp
            } else {
                cmp.reverse()
            }
        });

        Ok(rows)
    }

    /// Execute WHERE clause with learned index optimization
    fn execute_where_clause(&self, table: &crate::table::Table, expr: &Expr) -> Result<Vec<Row>> {
        use sqlparser::ast::BinaryOperator;

        match expr {
            // Primary key equality: WHERE id = 5 (use learned index point query)
            Expr::BinaryOp { left, op, right } if matches!(op, BinaryOperator::Eq) => {
                if let (Expr::Identifier(col), Expr::Value(val)) = (left.as_ref(), right.as_ref()) {
                    if col.value == table.primary_key() {
                        let value = Self::sql_value_to_value(
                            val,
                            table.schema().field_with_name(&col.value)?.data_type(),
                        )?;
                        if let Some(row) = table.get(&value)? {
                            return Ok(vec![row]);
                        } else {
                            return Ok(vec![]);
                        }
                    }
                }
                // Fall through to scan + filter
                self.scan_and_filter(table, expr)
            }

            // Range query: WHERE id > 10 AND id < 20 (use learned index range query)
            Expr::BinaryOp {
                left,
                op: BinaryOperator::And,
                right,
            } => {
                // Try to extract range bounds with operator info
                if let (
                    Some((col, start_val, start_inclusive)),
                    Some((col2, end_val, end_inclusive)),
                ) = (
                    Self::extract_range_with_op(left),
                    Self::extract_range_with_op(right),
                ) {
                    if col == col2 && col == table.primary_key() {
                        let start = Self::sql_value_to_value(
                            &start_val,
                            table.schema().field_with_name(&col)?.data_type(),
                        )?;
                        let end = Self::sql_value_to_value(
                            &end_val,
                            table.schema().field_with_name(&col)?.data_type(),
                        )?;

                        // Get range (inclusive), then filter for exclusive bounds
                        let mut rows = table.range_query(&start, &end)?;

                        // Filter out boundaries if needed
                        let pk_idx = table.schema().index_of(&col)?;
                        rows.retain(|row| {
                            let pk_val = row.get(pk_idx).ok();
                            if let Some(val) = pk_val {
                                let include_start = start_inclusive || val != &start;
                                let include_end = end_inclusive || val != &end;
                                include_start && include_end
                            } else {
                                false
                            }
                        });

                        return Ok(rows);
                    }
                }
                // Fall through to scan + filter
                self.scan_and_filter(table, expr)
            }

            // Greater than: WHERE id > 10 or WHERE id >= 10
            Expr::BinaryOp { left, op, right }
                if matches!(op, BinaryOperator::Gt | BinaryOperator::GtEq) =>
            {
                if let (Expr::Identifier(col), Expr::Value(val)) = (left.as_ref(), right.as_ref()) {
                    if col.value == table.primary_key() {
                        let start_val = Self::sql_value_to_value(
                            val,
                            table.schema().field_with_name(&col.value)?.data_type(),
                        )?;
                        let max_val = Value::Int64(i64::MAX);
                        let mut rows = table.range_query(&start_val, &max_val)?;

                        // For >, exclude the start value
                        if matches!(op, BinaryOperator::Gt) {
                            let pk_idx = table.schema().index_of(&col.value)?;
                            rows.retain(|row| row.get(pk_idx).ok() != Some(&start_val));
                        }

                        return Ok(rows);
                    }
                }
                self.scan_and_filter(table, expr)
            }

            // Less than: WHERE id < 20 or WHERE id <= 20
            Expr::BinaryOp { left, op, right }
                if matches!(op, BinaryOperator::Lt | BinaryOperator::LtEq) =>
            {
                if let (Expr::Identifier(col), Expr::Value(val)) = (left.as_ref(), right.as_ref()) {
                    if col.value == table.primary_key() {
                        let end_val = Self::sql_value_to_value(
                            val,
                            table.schema().field_with_name(&col.value)?.data_type(),
                        )?;
                        let min_val = Value::Int64(i64::MIN);
                        let mut rows = table.range_query(&min_val, &end_val)?;

                        // For <, exclude the end value
                        if matches!(op, BinaryOperator::Lt) {
                            let pk_idx = table.schema().index_of(&col.value)?;
                            rows.retain(|row| row.get(pk_idx).ok() != Some(&end_val));
                        }

                        return Ok(rows);
                    }
                }
                self.scan_and_filter(table, expr)
            }

            // Other expressions: fall back to scan + filter
            _ => self.scan_and_filter(table, expr),
        }
    }

    /// Extract range bound from expression like "id > 10" or "id >= 10"
    fn extract_range_bound(
        expr: &Expr,
        op1: sqlparser::ast::BinaryOperator,
        op2: sqlparser::ast::BinaryOperator,
    ) -> Option<(String, sqlparser::ast::Value)> {
        

        if let Expr::BinaryOp { left, op, right } = expr {
            if matches!(op, x if *x == op1 || *x == op2) {
                if let (Expr::Identifier(col), Expr::Value(val)) = (left.as_ref(), right.as_ref()) {
                    return Some((col.value.clone(), val.clone()));
                }
            }
        }
        None
    }

    /// Extract range bound with operator info (column, value, is_inclusive)
    fn extract_range_with_op(expr: &Expr) -> Option<(String, sqlparser::ast::Value, bool)> {
        use sqlparser::ast::BinaryOperator;

        if let Expr::BinaryOp { left, op, right } = expr {
            if let (Expr::Identifier(col), Expr::Value(val)) = (left.as_ref(), right.as_ref()) {
                match op {
                    BinaryOperator::Gt => return Some((col.value.clone(), val.clone(), false)),
                    BinaryOperator::GtEq => return Some((col.value.clone(), val.clone(), true)),
                    BinaryOperator::Lt => return Some((col.value.clone(), val.clone(), false)),
                    BinaryOperator::LtEq => return Some((col.value.clone(), val.clone(), true)),
                    _ => {}
                }
            }
        }
        None
    }

    /// Scan table and filter rows based on WHERE expression
    fn scan_and_filter(&self, table: &crate::table::Table, expr: &Expr) -> Result<Vec<Row>> {
        let all_rows = table.scan_all()?;
        let mut filtered = Vec::new();

        for row in all_rows {
            if self.evaluate_expr(expr, &row, table.schema())? {
                filtered.push(row);
            }
        }

        Ok(filtered)
    }

    /// Evaluate expression against a row
    fn evaluate_expr(
        &self,
        expr: &Expr,
        row: &Row,
        schema: &arrow::datatypes::SchemaRef,
    ) -> Result<bool> {
        use sqlparser::ast::BinaryOperator;

        match expr {
            Expr::BinaryOp { left, op, right } => match op {
                BinaryOperator::Eq => {
                    let left_val = self.evaluate_value_expr(left, row, schema)?;
                    let right_val = self.evaluate_value_expr(right, row, schema)?;
                    Ok(left_val == right_val)
                }
                BinaryOperator::Gt => {
                    let left_val = self.evaluate_value_expr(left, row, schema)?;
                    let right_val = self.evaluate_value_expr(right, row, schema)?;
                    Ok(Self::compare_values(&left_val, &right_val)? > 0)
                }
                BinaryOperator::Lt => {
                    let left_val = self.evaluate_value_expr(left, row, schema)?;
                    let right_val = self.evaluate_value_expr(right, row, schema)?;
                    Ok(Self::compare_values(&left_val, &right_val)? < 0)
                }
                BinaryOperator::GtEq => {
                    let left_val = self.evaluate_value_expr(left, row, schema)?;
                    let right_val = self.evaluate_value_expr(right, row, schema)?;
                    Ok(Self::compare_values(&left_val, &right_val)? >= 0)
                }
                BinaryOperator::LtEq => {
                    let left_val = self.evaluate_value_expr(left, row, schema)?;
                    let right_val = self.evaluate_value_expr(right, row, schema)?;
                    Ok(Self::compare_values(&left_val, &right_val)? <= 0)
                }
                BinaryOperator::And => {
                    let left_result = self.evaluate_expr(left, row, schema)?;
                    let right_result = self.evaluate_expr(right, row, schema)?;
                    Ok(left_result && right_result)
                }
                BinaryOperator::Or => {
                    let left_result = self.evaluate_expr(left, row, schema)?;
                    let right_result = self.evaluate_expr(right, row, schema)?;
                    Ok(left_result || right_result)
                }
                _ => Err(anyhow!("Unsupported operator: {:?}", op)),
            },
            _ => Err(anyhow!("Unsupported expression in WHERE clause")),
        }
    }

    /// Evaluate expression to get a Value
    fn evaluate_value_expr(
        &self,
        expr: &Expr,
        row: &Row,
        schema: &arrow::datatypes::SchemaRef,
    ) -> Result<Value> {
        match expr {
            Expr::Identifier(ident) => {
                let col_idx = schema.index_of(&ident.value)?;
                Ok(row.get(col_idx)?.clone())
            }
            Expr::Value(val) => {
                // Convert SQL value to our Value type (simplified - assumes Int64)
                match val {
                    sqlparser::ast::Value::Number(n, _) => {
                        if n.contains('.') {
                            Ok(Value::Float64(n.parse()?))
                        } else {
                            Ok(Value::Int64(n.parse()?))
                        }
                    }
                    sqlparser::ast::Value::SingleQuotedString(s) => Ok(Value::Text(s.clone())),
                    sqlparser::ast::Value::Boolean(b) => Ok(Value::Boolean(*b)),
                    _ => Err(anyhow!("Unsupported value type in WHERE clause")),
                }
            }
            Expr::UnaryOp { op, expr } => {
                // Handle negative numbers in WHERE clause
                use sqlparser::ast::UnaryOperator;
                match op {
                    UnaryOperator::Minus => {
                        // Special case for i64::MIN
                        if let Expr::Value(sqlparser::ast::Value::Number(n, _)) = expr.as_ref() {
                            if n == "9223372036854775808" {
                                return Ok(Value::Int64(i64::MIN));
                            }
                        }

                        let value = self.evaluate_value_expr(expr, row, schema)?;
                        match value {
                            Value::Int64(n) => Ok(Value::Int64(-n)),
                            Value::Float64(f) => Ok(Value::Float64(-f)),
                            Value::Timestamp(t) => Ok(Value::Timestamp(-t)),
                            _ => Err(anyhow!("Cannot negate {:?}", value)),
                        }
                    }
                    UnaryOperator::Plus => self.evaluate_value_expr(expr, row, schema),
                    _ => Err(anyhow!(
                        "Unsupported unary operator in WHERE clause: {:?}",
                        op
                    )),
                }
            }
            _ => Err(anyhow!("Unsupported expression type")),
        }
    }

    /// Compare two values
    fn compare_values(left: &Value, right: &Value) -> Result<i32> {
        match (left, right) {
            (Value::Int64(a), Value::Int64(b)) => Ok(a.cmp(b) as i32),
            (Value::Float64(a), Value::Float64(b)) => {
                if a < b {
                    Ok(-1)
                } else if a > b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            (Value::Text(a), Value::Text(b)) => Ok(a.cmp(b) as i32),
            (Value::Timestamp(a), Value::Timestamp(b)) => Ok(a.cmp(b) as i32),
            _ => Err(anyhow!("Cannot compare values of different types")),
        }
    }

    /// Convert SQL value to our Value type
    fn sql_value_to_value(val: &sqlparser::ast::Value, expected_type: &DataType) -> Result<Value> {
        match val {
            sqlparser::ast::Value::Number(n, _) => match expected_type {
                DataType::Int64 => Ok(Value::Int64(n.parse()?)),
                DataType::Float64 => Ok(Value::Float64(n.parse()?)),
                DataType::Timestamp(_, _) => Ok(Value::Timestamp(n.parse()?)),
                _ => Err(anyhow!("Cannot convert number to {:?}", expected_type)),
            },
            sqlparser::ast::Value::SingleQuotedString(s) => match expected_type {
                DataType::Utf8 => Ok(Value::Text(s.clone())),
                _ => Err(anyhow!("Cannot convert string to {:?}", expected_type)),
            },
            sqlparser::ast::Value::Boolean(b) => Ok(Value::Boolean(*b)),
            _ => Err(anyhow!("Unsupported SQL value type")),
        }
    }

    /// Convert SQL data type to Arrow data type
    fn sql_type_to_arrow(sql_type: &SqlDataType) -> Result<DataType> {
        match sql_type {
            SqlDataType::BigInt(_) | SqlDataType::Int8(_) | SqlDataType::Int64 => {
                Ok(DataType::Int64)
            }
            SqlDataType::Double | SqlDataType::Float8 | SqlDataType::Float64 => {
                Ok(DataType::Float64)
            }
            SqlDataType::Varchar(_) | SqlDataType::Text | SqlDataType::String(_) => {
                Ok(DataType::Utf8)
            }
            SqlDataType::Timestamp(_, _) => Ok(DataType::Timestamp(
                arrow::datatypes::TimeUnit::Microsecond,
                None,
            )),
            SqlDataType::Boolean => Ok(DataType::Boolean),
            _ => Err(anyhow!("Unsupported SQL data type: {:?}", sql_type)),
        }
    }

    /// Convert SQL expression to Value
    fn expr_to_value(expr: &Expr, expected_type: &DataType) -> Result<Value> {
        match expr {
            Expr::Value(sqlparser::ast::Value::Number(n, _)) => match expected_type {
                DataType::Int64 => Ok(Value::Int64(n.parse()?)),
                DataType::Float64 => Ok(Value::Float64(n.parse()?)),
                DataType::Timestamp(_, _) => Ok(Value::Timestamp(n.parse()?)),
                _ => Err(anyhow!("Cannot convert number to {:?}", expected_type)),
            },
            Expr::Value(sqlparser::ast::Value::SingleQuotedString(s)) => match expected_type {
                DataType::Utf8 => Ok(Value::Text(s.clone())),
                _ => Err(anyhow!("Cannot convert string to {:?}", expected_type)),
            },
            Expr::Value(sqlparser::ast::Value::Boolean(b)) => match expected_type {
                DataType::Boolean => Ok(Value::Boolean(*b)),
                _ => Err(anyhow!("Cannot convert boolean to {:?}", expected_type)),
            },
            Expr::Value(sqlparser::ast::Value::Null) => Ok(Value::Null),
            Expr::UnaryOp { op, expr } => {
                // Handle negative numbers: -50, -3.14
                use sqlparser::ast::UnaryOperator;
                match op {
                    UnaryOperator::Minus => {
                        // Special case: i64::MIN cannot be parsed as positive then negated
                        // because i64::MAX + 1 overflows
                        if let Expr::Value(sqlparser::ast::Value::Number(n, _)) = expr.as_ref() {
                            if matches!(expected_type, DataType::Int64)
                                && n == "9223372036854775808"
                            {
                                return Ok(Value::Int64(i64::MIN));
                            }
                        }

                        let value = Self::expr_to_value(expr, expected_type)?;
                        match value {
                            Value::Int64(n) => Ok(Value::Int64(-n)),
                            Value::Float64(f) => Ok(Value::Float64(-f)),
                            Value::Timestamp(t) => Ok(Value::Timestamp(-t)),
                            _ => Err(anyhow!("Cannot negate {:?}", value)),
                        }
                    }
                    UnaryOperator::Plus => {
                        // Unary plus is a no-op
                        Self::expr_to_value(expr, expected_type)
                    }
                    _ => Err(anyhow!("Unsupported unary operator: {:?}", op)),
                }
            }
            _ => Err(anyhow!("Unsupported expression: {:?}", expr)),
        }
    }

    /// Extract table name from ObjectName
    fn extract_table_name(name: &ObjectName) -> Result<String> {
        if name.0.is_empty() {
            return Err(anyhow!("Empty table name"));
        }
        Ok(name.0[name.0.len() - 1].value.clone())
    }

    /// Get reference to catalog
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    /// Get mutable reference to catalog
    pub fn catalog_mut(&mut self) -> &mut Catalog {
        &mut self.catalog
    }
}

/// Result of SQL execution
#[derive(Debug)]
pub enum ExecutionResult {
    /// Table created
    Created { message: String },

    /// Rows inserted
    Inserted { rows: usize },

    /// Rows updated
    Updated { rows: usize },

    /// Rows deleted
    Deleted { rows: usize },

    /// Rows selected
    Selected {
        columns: Vec<String>,
        rows: usize,
        data: Vec<Row>,
    },

    /// Transaction started
    TransactionStarted { txn_id: u64 },

    /// Transaction committed
    TransactionCommitted { txn_id: u64 },

    /// Transaction rolled back
    TransactionRolledBack { txn_id: u64 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_table() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        let sql = "CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))";
        let result = engine.execute(sql).unwrap();

        match result {
            ExecutionResult::Created { message } => {
                assert!(message.contains("users"));
            }
            _ => panic!("Expected Created result"),
        }

        assert!(engine.catalog().table_exists("users"));
    }

    #[test]
    fn test_insert_and_select() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        // Create table
        engine
            .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))")
            .unwrap();

        // Insert data
        let result = engine
            .execute("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')")
            .unwrap();
        match result {
            ExecutionResult::Inserted { rows } => assert_eq!(rows, 2),
            _ => panic!("Expected Inserted result"),
        }

        // Select data
        let result = engine.execute("SELECT * FROM users").unwrap();
        match result {
            ExecutionResult::Selected { columns, rows, .. } => {
                assert_eq!(columns, vec!["id", "name"]);
                assert_eq!(rows, 2);
            }
            _ => panic!("Expected Selected result"),
        }
    }

    #[test]
    fn test_multiple_data_types() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        let sql = "CREATE TABLE metrics (
            timestamp BIGINT PRIMARY KEY,
            value DOUBLE,
            label VARCHAR(100),
            active BOOLEAN
        )";
        engine.execute(sql).unwrap();

        let sql = "INSERT INTO metrics VALUES (1000, 1.5, 'test', true)";
        engine.execute(sql).unwrap();

        let result = engine.execute("SELECT * FROM metrics").unwrap();
        match result {
            ExecutionResult::Selected { rows, data, .. } => {
                assert_eq!(rows, 1);
                assert_eq!(data[0].get(0).unwrap(), &Value::Int64(1000));
                assert_eq!(data[0].get(1).unwrap(), &Value::Float64(1.5));
                assert_eq!(data[0].get(2).unwrap(), &Value::Text("test".to_string()));
                assert_eq!(data[0].get(3).unwrap(), &Value::Boolean(true));
            }
            _ => panic!("Expected Selected result"),
        }
    }

    #[test]
    fn test_where_clause_point_query() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        // Create and populate table
        engine
            .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))")
            .unwrap();
        engine
            .execute("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie')")
            .unwrap();

        // Point query using learned index: WHERE id = 2
        let result = engine.execute("SELECT * FROM users WHERE id = 2").unwrap();
        match result {
            ExecutionResult::Selected { rows, data, .. } => {
                assert_eq!(rows, 1);
                assert_eq!(data[0].get(0).unwrap(), &Value::Int64(2));
                assert_eq!(data[0].get(1).unwrap(), &Value::Text("Bob".to_string()));
            }
            _ => panic!("Expected Selected result"),
        }

        // Non-existent key
        let result = engine.execute("SELECT * FROM users WHERE id = 99").unwrap();
        match result {
            ExecutionResult::Selected { rows, .. } => {
                assert_eq!(rows, 0);
            }
            _ => panic!("Expected Selected result"),
        }
    }

    #[test]
    fn test_where_clause_range_query() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        // Create and populate table
        engine
            .execute("CREATE TABLE events (id BIGINT PRIMARY KEY, event_type VARCHAR(100))")
            .unwrap();
        for i in 0..20 {
            let sql = format!("INSERT INTO events VALUES ({}, 'event_{}')", i, i);
            engine.execute(&sql).unwrap();
        }

        // Range query: WHERE id > 5 AND id < 10
        let result = engine
            .execute("SELECT * FROM events WHERE id > 5 AND id < 10")
            .unwrap();
        match result {
            ExecutionResult::Selected { rows, data, .. } => {
                assert_eq!(rows, 4); // 6, 7, 8, 9
                for row in data {
                    if let Value::Int64(id) = row.get(0).unwrap() {
                        assert!(*id > 5 && *id < 10);
                    }
                }
            }
            _ => panic!("Expected Selected result"),
        }
    }

    #[test]
    fn test_where_clause_greater_than() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        engine
            .execute("CREATE TABLE data (id BIGINT PRIMARY KEY, value DOUBLE)")
            .unwrap();
        for i in 0..10 {
            let sql = format!("INSERT INTO data VALUES ({}, {})", i, i as f64 * 1.5);
            engine.execute(&sql).unwrap();
        }

        // WHERE id > 7
        let result = engine.execute("SELECT * FROM data WHERE id > 7").unwrap();
        match result {
            ExecutionResult::Selected { rows, data, .. } => {
                assert_eq!(rows, 2); // 8, 9
                for row in data {
                    if let Value::Int64(id) = row.get(0).unwrap() {
                        assert!(*id > 7);
                    }
                }
            }
            _ => panic!("Expected Selected result"),
        }
    }

    #[test]
    fn test_where_clause_less_than() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        engine
            .execute("CREATE TABLE data (id BIGINT PRIMARY KEY, value DOUBLE)")
            .unwrap();
        for i in 0..10 {
            let sql = format!("INSERT INTO data VALUES ({}, {})", i, i as f64 * 1.5);
            engine.execute(&sql).unwrap();
        }

        // WHERE id < 3
        let result = engine.execute("SELECT * FROM data WHERE id < 3").unwrap();
        match result {
            ExecutionResult::Selected { rows, data, .. } => {
                assert_eq!(rows, 3); // 0, 1, 2
                for row in data {
                    if let Value::Int64(id) = row.get(0).unwrap() {
                        assert!(*id < 3);
                    }
                }
            }
            _ => panic!("Expected Selected result"),
        }
    }

    #[test]
    fn test_where_clause_non_primary_key() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        engine
            .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))")
            .unwrap();
        engine
            .execute("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Alice')")
            .unwrap();

        // WHERE name = 'Alice' (scan + filter, not learned index)
        let result = engine
            .execute("SELECT * FROM users WHERE name = 'Alice'")
            .unwrap();
        match result {
            ExecutionResult::Selected { rows, data, .. } => {
                assert_eq!(rows, 2); // id=1 and id=3
                for row in data {
                    assert_eq!(row.get(1).unwrap(), &Value::Text("Alice".to_string()));
                }
            }
            _ => panic!("Expected Selected result"),
        }
    }

    #[test]
    fn test_where_clause_greater_equal() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        engine
            .execute("CREATE TABLE data (id BIGINT PRIMARY KEY, value DOUBLE)")
            .unwrap();
        for i in 0..5 {
            let sql = format!("INSERT INTO data VALUES ({}, {})", i, i as f64);
            engine.execute(&sql).unwrap();
        }

        // WHERE id >= 3
        let result = engine.execute("SELECT * FROM data WHERE id >= 3").unwrap();
        match result {
            ExecutionResult::Selected { rows, data, .. } => {
                assert_eq!(rows, 2); // 3, 4
                for row in data {
                    if let Value::Int64(id) = row.get(0).unwrap() {
                        assert!(*id >= 3);
                    }
                }
            }
            _ => panic!("Expected Selected result"),
        }
    }

    #[test]
    fn test_where_clause_less_equal() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        engine
            .execute("CREATE TABLE data (id BIGINT PRIMARY KEY, value DOUBLE)")
            .unwrap();
        for i in 0..5 {
            let sql = format!("INSERT INTO data VALUES ({}, {})", i, i as f64);
            engine.execute(&sql).unwrap();
        }

        // WHERE id <= 2
        let result = engine.execute("SELECT * FROM data WHERE id <= 2").unwrap();
        match result {
            ExecutionResult::Selected { rows, data, .. } => {
                assert_eq!(rows, 3); // 0, 1, 2
                for row in data {
                    if let Value::Int64(id) = row.get(0).unwrap() {
                        assert!(*id <= 2);
                    }
                }
            }
            _ => panic!("Expected Selected result"),
        }
    }

    #[test]
    fn test_where_clause_mixed_types() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        engine.execute("CREATE TABLE metrics (timestamp BIGINT PRIMARY KEY, value DOUBLE, status VARCHAR(50))").unwrap();
        engine.execute("INSERT INTO metrics VALUES (1000, 1.5, 'ok'), (2000, 2.5, 'warning'), (3000, 3.5, 'ok')").unwrap();

        // Point query on primary key
        let result = engine
            .execute("SELECT * FROM metrics WHERE timestamp = 2000")
            .unwrap();
        match result {
            ExecutionResult::Selected { rows, data, .. } => {
                assert_eq!(rows, 1);
                assert_eq!(data[0].get(0).unwrap(), &Value::Int64(2000));
                assert_eq!(data[0].get(2).unwrap(), &Value::Text("warning".to_string()));
            }
            _ => panic!("Expected Selected result"),
        }

        // Scan + filter on non-primary key
        let result = engine
            .execute("SELECT * FROM metrics WHERE status = 'ok'")
            .unwrap();
        match result {
            ExecutionResult::Selected { rows, data, .. } => {
                assert_eq!(rows, 2);
                for row in data {
                    assert_eq!(row.get(2).unwrap(), &Value::Text("ok".to_string()));
                }
            }
            _ => panic!("Expected Selected result"),
        }
    }

    #[test]
    #[ignore] // Large test - run with: cargo test test_where_clause_large_scale -- --ignored --nocapture
    fn test_where_clause_large_scale() {
        use std::time::Instant;

        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        println!("\n📊 Large-scale WHERE clause test (10,000 rows)");

        // Create table
        engine
            .execute("CREATE TABLE large_data (id BIGINT PRIMARY KEY, value DOUBLE)")
            .unwrap();

        // Insert 10K rows
        println!("  Inserting 10,000 rows...");
        let start = Instant::now();
        for i in 0..10_000 {
            let sql = format!("INSERT INTO large_data VALUES ({}, {})", i, i as f64 * 1.5);
            engine.execute(&sql).unwrap();
        }
        let insert_time = start.elapsed();
        println!("  ✅ Inserted 10K rows in {:?}", insert_time);

        // Point query
        println!("  Testing point query...");
        let start = Instant::now();
        let result = engine
            .execute("SELECT * FROM large_data WHERE id = 5000")
            .unwrap();
        let point_time = start.elapsed();
        match result {
            ExecutionResult::Selected { rows, .. } => {
                assert_eq!(rows, 1);
                println!("  ✅ Point query: {} row in {:?}", rows, point_time);
            }
            _ => panic!("Expected Selected result"),
        }

        // Range query
        println!("  Testing range query...");
        let start = Instant::now();
        let result = engine
            .execute("SELECT * FROM large_data WHERE id > 8000 AND id < 9000")
            .unwrap();
        let range_time = start.elapsed();
        match result {
            ExecutionResult::Selected { rows, .. } => {
                assert_eq!(rows, 999);
                println!("  ✅ Range query: {} rows in {:?}", rows, range_time);
            }
            _ => panic!("Expected Selected result"),
        }

        // Full scan for comparison
        println!("  Testing full scan...");
        let start = Instant::now();
        let result = engine.execute("SELECT * FROM large_data").unwrap();
        let scan_time = start.elapsed();
        match result {
            ExecutionResult::Selected { rows, .. } => {
                assert_eq!(rows, 10_000);
                println!("  ✅ Full scan: {} rows in {:?}", rows, scan_time);
            }
            _ => panic!("Expected Selected result"),
        }

        // Analysis
        let point_speedup = scan_time.as_micros() as f64 / point_time.as_micros() as f64;
        let range_speedup = scan_time.as_micros() as f64 / range_time.as_micros() as f64;
        println!();
        println!("  📈 Analysis:");
        println!(
            "     Point query speedup: {:.2}x vs full scan",
            point_speedup
        );
        println!(
            "     Range query speedup: {:.2}x vs full scan",
            range_speedup
        );

        // Assert meaningful speedup
        assert!(
            point_speedup > 2.0,
            "Point query should be at least 2x faster than full scan"
        );
        assert!(
            range_speedup > 2.0,
            "Range query should be at least 2x faster than full scan"
        );

        println!("  ✅ Learned index providing significant speedup!");
    }

    #[test]
    fn test_update_statement() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        // Create and populate table
        engine
            .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255), age BIGINT)")
            .unwrap();
        engine
            .execute("INSERT INTO users VALUES (1, 'Alice', 30), (2, 'Bob', 25), (3, 'Charlie', 35)")
            .unwrap();

        // Update a single row
        let result = engine
            .execute("UPDATE users SET name = 'Alice Smith', age = 31 WHERE id = 1")
            .unwrap();

        match result {
            ExecutionResult::Updated { rows } => {
                assert_eq!(rows, 1);
            }
            _ => panic!("Expected Updated result"),
        }

        // Verify update worked
        let result = engine.execute("SELECT * FROM users WHERE id = 1").unwrap();
        match result {
            ExecutionResult::Selected { rows, data, .. } => {
                assert_eq!(rows, 1);
                assert_eq!(data[0].get(1).unwrap(), &Value::Text("Alice Smith".to_string()));
                assert_eq!(data[0].get(2).unwrap(), &Value::Int64(31));
            }
            _ => panic!("Expected Selected result"),
        }
    }

    #[test]
    fn test_delete_statement() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        // Create and populate table
        engine
            .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))")
            .unwrap();
        engine
            .execute("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie')")
            .unwrap();

        // Delete a single row
        let result = engine.execute("DELETE FROM users WHERE id = 2").unwrap();

        match result {
            ExecutionResult::Deleted { rows } => {
                assert_eq!(rows, 1);
            }
            _ => panic!("Expected Deleted result"),
        }

        // Verify delete worked
        let result = engine.execute("SELECT * FROM users").unwrap();
        match result {
            ExecutionResult::Selected { rows, data, .. } => {
                assert_eq!(rows, 2); // Only 2 rows remain
                // Verify Bob is gone
                for row in data {
                    let name = row.get(1).unwrap();
                    assert_ne!(name, &Value::Text("Bob".to_string()));
                }
            }
            _ => panic!("Expected Selected result"),
        }
    }

    #[test]
    fn test_update_then_delete() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        // Create and populate table
        engine
            .execute("CREATE TABLE products (id BIGINT PRIMARY KEY, name VARCHAR(255), price DOUBLE)")
            .unwrap();
        engine
            .execute("INSERT INTO products VALUES (1, 'Widget', 10.99), (2, 'Gadget', 25.50)")
            .unwrap();

        // Update price
        engine
            .execute("UPDATE products SET price = 15.99 WHERE id = 1")
            .unwrap();

        // Verify update
        let result = engine.execute("SELECT * FROM products WHERE id = 1").unwrap();
        match result {
            ExecutionResult::Selected { data, .. } => {
                assert_eq!(data[0].get(2).unwrap(), &Value::Float64(15.99));
            }
            _ => panic!("Expected Selected result"),
        }

        // Delete the updated row
        let result = engine.execute("DELETE FROM products WHERE id = 1").unwrap();
        match result {
            ExecutionResult::Deleted { rows } => assert_eq!(rows, 1),
            _ => panic!("Expected Deleted result"),
        }

        // Verify deletion
        let result = engine.execute("SELECT * FROM products").unwrap();
        match result {
            ExecutionResult::Selected { rows, .. } => {
                assert_eq!(rows, 1); // Only product 2 remains
            }
            _ => panic!("Expected Selected result"),
        }
    }

    #[test]
    fn test_update_nonexistent_row() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        engine
            .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))")
            .unwrap();

        // Try to update nonexistent row
        let result = engine.execute("UPDATE users SET name = 'Test' WHERE id = 999");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_delete_nonexistent_row() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        engine
            .execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))")
            .unwrap();

        // Try to delete nonexistent row
        let result = engine.execute("DELETE FROM users WHERE id = 999");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
