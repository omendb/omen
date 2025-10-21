//! Extended Query Protocol support for OmenDB
//!
//! Implements prepared statements with parameter binding ($1, $2, etc.)

use super::encoding::{record_batches_to_query_response_with_format, arrow_to_pg_type};
use crate::constraints::ConstraintManager;
use crate::transaction::{BufferedOperation, TransactionContext};
use async_trait::async_trait;
use datafusion::prelude::*;
use pgwire::api::portal::Portal;
use pgwire::api::query::ExtendedQueryHandler;
use pgwire::api::results::{Response, Tag, FieldInfo};
use pgwire::api::stmt::QueryParser;
use pgwire::api::ClientInfo;
use pgwire::error::{ErrorInfo, PgWireError, PgWireResult};
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// Parsed SQL statement for extended query protocol
#[derive(Clone, Debug)]
pub struct OmenDbStatement {
    /// Original SQL query with parameter placeholders ($1, $2, etc.)
    pub query: String,

    /// Number of parameters in the query
    pub param_count: usize,
}

/// Query parser for OmenDB statements
pub struct OmenDbQueryParser;

#[async_trait]
impl QueryParser for OmenDbQueryParser {
    type Statement = OmenDbStatement;

    /// Parse SQL query and extract parameter count
    async fn parse_sql(&self, sql: &str, _types: &[pgwire::api::Type]) -> PgWireResult<Self::Statement> {
        // Count parameter placeholders ($1, $2, etc.)
        let param_count = count_parameters(sql);

        info!("[EXTENDED] Parsed SQL with {} parameters: {}", param_count, sql);

        Ok(OmenDbStatement {
            query: sql.to_string(),
            param_count,
        })
    }
}

/// Extended query handler for OmenDB
pub struct OmenDbExtendedQueryHandler {
    /// DataFusion session context for query execution
    ctx: Arc<RwLock<SessionContext>>,

    /// Transaction context for ACID compliance
    tx_ctx: Arc<RwLock<TransactionContext>>,

    /// Constraint manager for PRIMARY KEY validation
    constraint_mgr: Arc<ConstraintManager>,

    /// Query parser
    parser: Arc<OmenDbQueryParser>,
}

impl OmenDbExtendedQueryHandler {
    pub fn new(ctx: Arc<RwLock<SessionContext>>, tx_ctx: Arc<RwLock<TransactionContext>>) -> Self {
        let constraint_mgr = Arc::new(ConstraintManager::new(ctx.clone()));
        Self {
            ctx,
            tx_ctx,
            constraint_mgr,
            parser: Arc::new(OmenDbQueryParser),
        }
    }

    /// Execute SQL query with parameter substitution
    async fn execute_with_params(
        &self,
        query: &str,
        params: &[Option<String>],
        format: pgwire::api::results::FieldFormat,
    ) -> PgWireResult<Response<'static>> {
        // Substitute parameters in query
        let substituted_query = substitute_parameters(query, params)?;

        debug!("Executing parameterized query: {}", substituted_query);

        // Handle special PostgreSQL commands
        if Self::is_special_command(&substituted_query) {
            return self.handle_special_command(&substituted_query).await;
        }

        // Check if we're in a transaction and this is a DML query
        let upper = substituted_query.trim().to_uppercase();
        let is_dml = upper.starts_with("INSERT") || upper.starts_with("UPDATE") || upper.starts_with("DELETE");

        // Register table schema if this is a CREATE TABLE
        if upper.starts_with("CREATE TABLE") {
            self.constraint_mgr.register_table_schema(&substituted_query).await.map_err(|e| {
                error!("[EXTENDED] Failed to register table schema: {}", e);
                PgWireError::UserError(Box::new(ErrorInfo::new(
                    "ERROR".to_owned(),
                    "XX000".to_owned(),
                    format!("Schema registration error: {}", e),
                )))
            })?;
        }

        // Remove constraints if this is a DROP TABLE
        if upper.starts_with("DROP TABLE") {
            if let Some(table_name) = Self::extract_drop_table_name(&substituted_query) {
                let constraints_arc = self.constraint_mgr.constraints();
                let mut constraints = constraints_arc.write().await;
                constraints.remove_table(&table_name);
                debug!("[EXTENDED] Removed constraints for table: {}", table_name);
            }
        }

        // Validate INSERT statements against PRIMARY KEY constraints
        if upper.starts_with("INSERT") {
            let tx = self.tx_ctx.read().await;
            let buffered_inserts = if tx.is_in_transaction() {
                self.extract_buffered_pk_values(&tx).await
            } else {
                Vec::new()
            };
            drop(tx);

            self.validate_insert_with_transaction(&substituted_query, &buffered_inserts).await.map_err(|e| {
                error!("[EXTENDED] Constraint violation: {}", e);
                PgWireError::UserError(Box::new(ErrorInfo::new(
                    "ERROR".to_owned(),
                    "23505".to_owned(),
                    e.to_string(),
                )))
            })?;
        }

        if is_dml {
            let tx = self.tx_ctx.read().await;
            if tx.is_in_transaction() {
                // Buffer the operation instead of executing
                drop(tx);
                let mut tx = self.tx_ctx.write().await;

                let operation = if upper.starts_with("INSERT") {
                    BufferedOperation::Insert {
                        table_name: "unknown".to_string(),
                        query: substituted_query.clone(),
                    }
                } else if upper.starts_with("UPDATE") {
                    BufferedOperation::Update {
                        table_name: "unknown".to_string(),
                        query: substituted_query.clone(),
                    }
                } else {
                    BufferedOperation::Delete {
                        table_name: "unknown".to_string(),
                        query: substituted_query.clone(),
                    }
                };

                tx.buffer_operation(operation).map_err(|e| {
                    error!("Failed to buffer operation: {}", e);
                    PgWireError::UserError(Box::new(ErrorInfo::new(
                        "ERROR".to_owned(),
                        "25000".to_owned(),
                        format!("Failed to buffer operation: {}", e),
                    )))
                })?;

                info!("[EXTENDED] Buffered {} operation (transaction in progress)",
                    if upper.starts_with("INSERT") { "INSERT" }
                    else if upper.starts_with("UPDATE") { "UPDATE" }
                    else { "DELETE" });

                // Return success without executing
                let tag = if upper.starts_with("INSERT") {
                    Tag::new("INSERT").with_oid(0).with_rows(1)
                } else if upper.starts_with("UPDATE") {
                    Tag::new("UPDATE").with_rows(1)
                } else {
                    Tag::new("DELETE").with_rows(1)
                };

                return Ok(Response::Execution(tag));
            }
        }

        // Auto-commit mode or non-DML: execute immediately
        let ctx = self.ctx.read().await;

        // Check if this is a DDL statement (CREATE, DROP, ALTER)
        let is_ddl = upper.starts_with("CREATE")
            || upper.starts_with("DROP")
            || upper.starts_with("ALTER");

        // Log catalog state BEFORE executing DDL
        if is_ddl {
            let table_names: Vec<String> = ctx.catalog("datafusion").unwrap()
                .schema("public").unwrap()
                .table_names()
                .into_iter()
                .map(|s| s.to_string())
                .collect();
            info!("[EXTENDED] BEFORE {}: Catalog has {} tables: {:?}",
                upper.split_whitespace().next().unwrap_or("DDL"),
                table_names.len(),
                table_names);
        }

        // Execute query with DataFusion
        let df = ctx.sql(&substituted_query).await.map_err(|e| {
            error!("DataFusion SQL error: {}", e);
            PgWireError::UserError(Box::new(ErrorInfo::new(
                "ERROR".to_owned(),
                "42601".to_owned(),
                format!("SQL execution error: {}", e),
            )))
        })?;

        // Collect results into RecordBatches
        // NOTE: This is required even for DDL statements (CREATE, DROP, ALTER)
        // DataFusion requires .collect() to actually execute the catalog changes
        let batches = df.collect().await.map_err(|e| {
            error!("DataFusion collect error: {}", e);
            PgWireError::UserError(Box::new(ErrorInfo::new(
                "ERROR".to_owned(),
                "XX000".to_owned(),
                format!("Query execution error: {}", e),
            )))
        })?;

        // Log catalog state AFTER executing DDL
        if is_ddl {
            let table_names: Vec<String> = ctx.catalog("datafusion").unwrap()
                .schema("public").unwrap()
                .table_names()
                .into_iter()
                .map(|s| s.to_string())
                .collect();
            info!("[EXTENDED] AFTER {}: Catalog has {} tables: {:?}",
                upper.split_whitespace().next().unwrap_or("DDL"),
                table_names.len(),
                table_names);

            // DDL statements don't return result data, just status
            return Ok(Response::Execution(Tag::new("OK")));
        }

        // Check if this is a DML query (INSERT, UPDATE, DELETE)
        if is_dml {
            // For DML queries, count affected rows
            let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();

            let tag = if upper.starts_with("INSERT") {
                Tag::new("INSERT").with_oid(0).with_rows(total_rows)
            } else if upper.starts_with("UPDATE") {
                Tag::new("UPDATE").with_rows(total_rows)
            } else {
                Tag::new("DELETE").with_rows(total_rows)
            };

            return Ok(Response::Execution(tag));
        }

        // For SELECT and other queries, log catalog state and return result set
        let table_names: Vec<String> = ctx.catalog("datafusion").unwrap()
            .schema("public").unwrap()
            .table_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        info!("[EXTENDED] SELECT query - Catalog has {} tables: {:?}",
            table_names.len(),
            table_names);

        // For Extended Query Protocol, use format-aware encoding
        // This respects the client's requested format (Binary or Text)
        info!("[EXTENDED] Encoding {} batches for response (format: {:?})", batches.len(), format);
        let responses = record_batches_to_query_response_with_format(batches, format)?;
        Ok(responses.into_iter().next().unwrap_or(Response::EmptyQuery))
    }

    /// Check if a query is a special PostgreSQL command
    fn is_special_command(query: &str) -> bool {
        let upper = query.trim().to_uppercase();
        upper.starts_with("SET")
            || upper.starts_with("SHOW")
            || upper.starts_with("BEGIN")
            || upper.starts_with("COMMIT")
            || upper.starts_with("ROLLBACK")
            || upper.starts_with("START TRANSACTION")
    }

    /// Execute a query directly (for COMMIT - apply buffered operations)
    async fn execute_query_direct(&self, query: &str) -> PgWireResult<()> {
        let ctx = self.ctx.read().await;

        // Execute with DataFusion
        let df = ctx.sql(query).await.map_err(|e| {
            error!("DataFusion SQL error: {}", e);
            PgWireError::UserError(Box::new(ErrorInfo::new(
                "ERROR".to_owned(),
                "42601".to_owned(),
                format!("SQL execution error: {}", e),
            )))
        })?;

        // IMPORTANT: Must call .collect() to actually execute ALL queries
        // This applies to both DML (INSERT/UPDATE/DELETE) and DDL (CREATE/DROP/ALTER)
        // Without this, queries are planned but never executed
        df.collect().await.map_err(|e| {
            error!("DataFusion collect error: {}", e);
            PgWireError::UserError(Box::new(ErrorInfo::new(
                "ERROR".to_owned(),
                "XX000".to_owned(),
                format!("Query execution error: {}", e),
            )))
        })?;

        Ok(())
    }

    /// Handle special PostgreSQL commands
    async fn handle_special_command(&self, query: &str) -> PgWireResult<Response<'static>> {
        let upper = query.trim().to_uppercase();

        if upper.starts_with("SET") {
            debug!("[EXTENDED] Handling SET command: {}", query);
            Ok(Response::Execution(Tag::new("SET")))
        } else if upper.starts_with("SHOW") {
            debug!("[EXTENDED] Handling SHOW command: {}", query);
            Ok(Response::EmptyQuery)
        } else if upper.starts_with("BEGIN") || upper.starts_with("START TRANSACTION") {
            info!("[EXTENDED] BEGIN transaction");
            let mut tx = self.tx_ctx.write().await;
            match tx.begin() {
                Ok(tx_id) => {
                    info!("[EXTENDED] Transaction {} started", tx_id);
                    Ok(Response::Execution(Tag::new("BEGIN")))
                }
                Err(e) => {
                    error!("[EXTENDED] Failed to start transaction: {}", e);
                    Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                        "ERROR".to_owned(),
                        "25001".to_owned(),
                        format!("Failed to start transaction: {}", e),
                    ))))
                }
            }
        } else if upper.starts_with("COMMIT") {
            info!("[EXTENDED] COMMIT transaction");
            let mut tx = self.tx_ctx.write().await;

            let operations = match tx.prepare_commit() {
                Ok(ops) => ops,
                Err(e) => {
                    error!("[EXTENDED] Failed to prepare commit: {}", e);
                    return Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                        "ERROR".to_owned(),
                        "25000".to_owned(),
                        format!("Failed to commit: {}", e),
                    ))));
                }
            };

            if !operations.is_empty() {
                info!("[EXTENDED] Committing {} buffered operations", operations.len());
                for op in operations {
                    let query = match op {
                        BufferedOperation::Insert { query, .. } => query,
                        BufferedOperation::Update { query, .. } => query,
                        BufferedOperation::Delete { query, .. } => query,
                    };

                    if let Err(e) = self.execute_query_direct(&query).await {
                        error!("[EXTENDED] Failed to execute buffered operation: {}", e);
                        tx.rollback().ok();
                        return Err(e);
                    }
                }
            }

            tx.finalize_commit();
            info!("[EXTENDED] Transaction committed successfully");
            Ok(Response::Execution(Tag::new("COMMIT")))
        } else if upper.starts_with("ROLLBACK") {
            info!("[EXTENDED] ROLLBACK transaction");
            let mut tx = self.tx_ctx.write().await;
            match tx.rollback() {
                Ok(()) => {
                    info!("[EXTENDED] Transaction rolled back successfully");
                    Ok(Response::Execution(Tag::new("ROLLBACK")))
                }
                Err(e) => {
                    error!("[EXTENDED] Failed to rollback: {}", e);
                    Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                        "ERROR".to_owned(),
                        "25000".to_owned(),
                        format!("Failed to rollback: {}", e),
                    ))))
                }
            }
        } else {
            Ok(Response::EmptyQuery)
        }
    }

    /// Extract table name from DROP TABLE statement
    fn extract_drop_table_name(query: &str) -> Option<String> {
        let upper = query.trim().to_uppercase();
        let drop_table = "DROP TABLE ";

        if let Some(pos) = upper.find(drop_table) {
            let after_drop = &query[pos + drop_table.len()..];

            let after_if_exists = if after_drop.trim().to_uppercase().starts_with("IF EXISTS") {
                after_drop[9..].trim()
            } else {
                after_drop.trim()
            };

            if let Some(end) = after_if_exists.find([' ', ';', ',']) {
                return Some(after_if_exists[..end].trim().to_string());
            } else {
                return Some(after_if_exists.trim().to_string());
            }
        }

        None
    }

    /// Extract primary key values from buffered INSERT operations
    async fn extract_buffered_pk_values(&self, tx: &TransactionContext) -> Vec<(String, Vec<String>)> {
        let mut buffered_pks = Vec::new();

        for op in tx.buffered_operations() {
            if let BufferedOperation::Insert { query, .. } = op {
                if let Some(table_name) = Self::extract_insert_table_name_simple(query) {
                    let constraints_arc = self.constraint_mgr.constraints();
                    let constraints = constraints_arc.read().await;

                    if let Some(pk_columns) = constraints.get_primary_key(&table_name) {
                        if let Some(values) = Self::extract_insert_values_simple(query, pk_columns.len()) {
                            buffered_pks.push((table_name.clone(), values));
                        }
                    }
                }
            }
        }

        buffered_pks
    }

    /// Simple table name extraction for INSERT
    fn extract_insert_table_name_simple(query: &str) -> Option<String> {
        let upper = query.trim().to_uppercase();
        if let Some(pos) = upper.find("INSERT INTO ") {
            let after_insert = &query[pos + 12..];
            if let Some(end) = after_insert.find([' ', '(']) {
                return Some(after_insert[..end].trim().to_string());
            }
        }
        None
    }

    /// Simple value extraction for INSERT
    fn extract_insert_values_simple(query: &str, num_pk_cols: usize) -> Option<Vec<String>> {
        let upper = query.trim().to_uppercase();
        if let Some(values_pos) = upper.find("VALUES") {
            let after_values = &query[values_pos + 6..].trim();
            if let Some(open_paren) = after_values.find('(') {
                if let Some(close_paren) = after_values.find(')') {
                    let values_str = &after_values[open_paren + 1..close_paren];
                    let values: Vec<String> = values_str
                        .split(',')
                        .take(num_pk_cols)
                        .map(|s| s.trim().trim_matches('\'').to_string())
                        .collect();
                    if values.len() == num_pk_cols {
                        return Some(values);
                    }
                }
            }
        }
        None
    }

    /// Validate INSERT with transaction context (checks buffered operations too)
    async fn validate_insert_with_transaction(
        &self,
        query: &str,
        buffered_inserts: &[(String, Vec<String>)],
    ) -> Result<(), anyhow::Error> {
        // First validate against committed data
        self.constraint_mgr.validate_insert(query).await?;

        // Then check against buffered operations in this transaction
        if let Some(table_name) = Self::extract_insert_table_name_simple(query) {
            let constraints_arc = self.constraint_mgr.constraints();
            let constraints = constraints_arc.read().await;

            if let Some(pk_columns) = constraints.get_primary_key(&table_name) {
                if let Some(new_values) = Self::extract_insert_values_simple(query, pk_columns.len()) {
                    for (buffered_table, buffered_values) in buffered_inserts {
                        if buffered_table == &table_name && buffered_values == &new_values {
                            return Err(anyhow::anyhow!(
                                "duplicate key value violates unique constraint: Key ({})=({}) already exists in transaction buffer",
                                pk_columns.join(", "),
                                new_values.join(", ")
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl ExtendedQueryHandler for OmenDbExtendedQueryHandler {
    type Statement = OmenDbStatement;
    type QueryParser = OmenDbQueryParser;

    fn query_parser(&self) -> Arc<Self::QueryParser> {
        self.parser.clone()
    }

    async fn do_query<'a, 'b: 'a, C>(
        &'b self,
        _client: &mut C,
        portal: &'a Portal<Self::Statement>,
        _max_rows: usize,
    ) -> PgWireResult<Response<'a>>
    where
        C: ClientInfo
            + pgwire::api::ClientPortalStore
            + futures::Sink<pgwire::messages::PgWireBackendMessage>
            + Unpin
            + Send
            + Sync,
        C::PortalStore: pgwire::api::store::PortalStore<Statement = Self::Statement>,
        C::Error: Debug,
        PgWireError: From<<C as futures::Sink<pgwire::messages::PgWireBackendMessage>>::Error>,
    {
        let stmt = &portal.statement.statement;
        info!("[EXTENDED] do_query called for: {}", stmt.query);
        info!("[EXTENDED] Portal result format: {:?}", portal.result_column_format);

        // Determine result format from portal
        // For now, use the format for column 0 (most clients use unified format anyway)
        let result_format = portal.result_column_format.format_for(0);
        info!("[EXTENDED] Using result format: {:?}", result_format);

        // Extract parameters from portal
        let params: Vec<Option<String>> = portal
            .parameters
            .iter()
            .map(|p| p.as_ref().map(|v| String::from_utf8_lossy(v).to_string()))
            .collect();

        debug!(
            "Parameters: {:?} (expected: {})",
            params,
            stmt.param_count
        );

        // Validate parameter count
        if params.len() != stmt.param_count {
            return Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                "ERROR".to_owned(),
                "08P01".to_owned(),
                format!(
                    "Expected {} parameters, got {}",
                    stmt.param_count,
                    params.len()
                ),
            ))));
        }

        // Execute query with parameters
        self.execute_with_params(&stmt.query, &params, result_format).await
    }

    async fn do_describe_statement<C>(
        &self,
        _client: &mut C,
        statement: &pgwire::api::stmt::StoredStatement<Self::Statement>,
    ) -> PgWireResult<pgwire::api::results::DescribeStatementResponse>
    where
        C: ClientInfo
            + pgwire::api::ClientPortalStore
            + futures::Sink<pgwire::messages::PgWireBackendMessage>
            + Unpin
            + Send
            + Sync,
        C::PortalStore: pgwire::api::store::PortalStore<Statement = Self::Statement>,
        C::Error: Debug,
        PgWireError: From<<C as futures::Sink<pgwire::messages::PgWireBackendMessage>>::Error>,
    {
        info!("[EXTENDED] do_describe_statement called for: {}", statement.statement.query);

        // Create parameter descriptions for each parameter in the query
        let param_types: Vec<pgwire::api::Type> = (0..statement.statement.param_count)
            .map(|_| pgwire::api::Type::UNKNOWN)
            .collect();

        // Try to get result schema for SELECT queries
        let query = &statement.statement.query;
        let upper = query.trim().to_uppercase();

        let field_descriptions = if upper.starts_with("SELECT") {
            // For SELECT queries, get the schema by planning the query
            match self.ctx.read().await.sql(query).await {
                Ok(df) => {
                    let arrow_schema = df.schema();
                    let fields: Vec<FieldInfo> = arrow_schema
                        .fields()
                        .iter()
                        .map(|field| {
                            let pg_type = arrow_to_pg_type(field.data_type()).unwrap_or(pgwire::api::Type::UNKNOWN);
                            FieldInfo::new(
                                field.name().clone(),
                                None,
                                None,
                                pg_type,
                                pgwire::api::results::FieldFormat::Text,
                            )
                        })
                        .collect();
                    info!("[EXTENDED] do_describe_statement returning {} field descriptions", fields.len());
                    fields
                }
                Err(e) => {
                    error!("[EXTENDED] Failed to plan query for describe_statement: {}", e);
                    vec![]
                }
            }
        } else {
            vec![]
        };

        Ok(pgwire::api::results::DescribeStatementResponse::new(
            param_types,
            field_descriptions,
        ))
    }

    async fn do_describe_portal<C>(
        &self,
        _client: &mut C,
        portal: &Portal<Self::Statement>,
    ) -> PgWireResult<pgwire::api::results::DescribePortalResponse>
    where
        C: ClientInfo
            + pgwire::api::ClientPortalStore
            + futures::Sink<pgwire::messages::PgWireBackendMessage>
            + Unpin
            + Send
            + Sync,
        C::PortalStore: pgwire::api::store::PortalStore<Statement = Self::Statement>,
        C::Error: Debug,
        PgWireError: From<<C as futures::Sink<pgwire::messages::PgWireBackendMessage>>::Error>,
    {
        info!("[EXTENDED] do_describe_portal called for: {}", portal.statement.statement.query);

        // Get the query from the portal
        let query = &portal.statement.statement.query;

        // Check if this is a DDL or DML query - these don't return result sets
        let upper = query.trim().to_uppercase();
        if upper.starts_with("CREATE") || upper.starts_with("DROP") || upper.starts_with("ALTER")
            || upper.starts_with("INSERT") || upper.starts_with("UPDATE") || upper.starts_with("DELETE")
            || upper.starts_with("BEGIN") || upper.starts_with("COMMIT") || upper.starts_with("ROLLBACK")
            || upper.starts_with("SET") || upper.starts_with("SHOW") {
            // These queries don't return result sets
            return Ok(pgwire::api::results::DescribePortalResponse::new(vec![]));
        }

        // Substitute parameters to get the actual query
        let params: Vec<Option<String>> = portal
            .parameters
            .iter()
            .map(|p| p.as_ref().map(|v| String::from_utf8_lossy(v).to_string()))
            .collect();

        let substituted_query = substitute_parameters(query, &params)?;

        // Get schema from DataFusion by planning the query
        let ctx = self.ctx.read().await;
        let df = ctx.sql(&substituted_query).await.map_err(|e| {
            error!("Failed to plan query for describe: {}", e);
            PgWireError::UserError(Box::new(ErrorInfo::new(
                "ERROR".to_owned(),
                "42601".to_owned(),
                format!("SQL planning error: {}", e),
            )))
        })?;

        // Get the schema from the DataFrame
        let arrow_schema = df.schema();

        // Convert Arrow schema to PostgreSQL field descriptions
        let field_descriptions: Vec<FieldInfo> = arrow_schema
            .fields()
            .iter()
            .map(|field| {
                let pg_type = arrow_to_pg_type(field.data_type()).unwrap_or(pgwire::api::Type::UNKNOWN);
                FieldInfo::new(
                    field.name().clone(),
                    None, // table_oid
                    None, // col_id
                    pg_type,
                    pgwire::api::results::FieldFormat::Text,
                )
            })
            .collect();

        info!("[EXTENDED] Describe portal returning {} fields", field_descriptions.len());
        Ok(pgwire::api::results::DescribePortalResponse::new(field_descriptions))
    }
}

/// Count parameter placeholders in SQL query
fn count_parameters(sql: &str) -> usize {
    let mut max_param = 0;
    let mut chars = sql.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            // Check if next characters are digits
            let mut num_str = String::new();
            while let Some(&next_ch) = chars.peek() {
                if next_ch.is_ascii_digit() {
                    num_str.push(next_ch);
                    chars.next();
                } else {
                    break;
                }
            }

            if !num_str.is_empty() {
                if let Ok(param_num) = num_str.parse::<usize>() {
                    max_param = max_param.max(param_num);
                }
            }
        }
    }

    max_param
}

/// Substitute parameter placeholders with actual values
fn substitute_parameters(sql: &str, params: &[Option<String>]) -> PgWireResult<String> {
    let mut result = String::with_capacity(sql.len() + params.len() * 10);
    let mut chars = sql.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            // Check if next characters are digits
            let mut num_str = String::new();
            while let Some(&next_ch) = chars.peek() {
                if next_ch.is_ascii_digit() {
                    num_str.push(next_ch);
                    chars.next();
                } else {
                    break;
                }
            }

            if num_str.is_empty() {
                // Not a parameter, just a dollar sign
                result.push('$');
            } else {
                // Parse parameter number
                let param_num: usize = num_str.parse().map_err(|_| {
                    PgWireError::UserError(Box::new(ErrorInfo::new(
                        "ERROR".to_owned(),
                        "08P01".to_owned(),
                        format!("Invalid parameter number: ${}", num_str),
                    )))
                })?;

                // Get parameter value (1-indexed)
                if param_num == 0 || param_num > params.len() {
                    return Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                        "ERROR".to_owned(),
                        "08P01".to_owned(),
                        format!("Parameter ${} out of range", param_num),
                    ))));
                }

                // Substitute parameter value
                match &params[param_num - 1] {
                    Some(value) => {
                        // Quote string values, numeric values as-is
                        if value.parse::<i64>().is_ok() || value.parse::<f64>().is_ok() {
                            result.push_str(value);
                        } else if value.eq_ignore_ascii_case("null") {
                            result.push_str("NULL");
                        } else if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false") {
                            result.push_str(value);
                        } else {
                            // String value - quote and escape
                            result.push('\'');
                            result.push_str(&value.replace('\'', "''"));
                            result.push('\'');
                        }
                    }
                    None => {
                        result.push_str("NULL");
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_parameters() {
        assert_eq!(count_parameters("SELECT * FROM users WHERE id = $1"), 1);
        assert_eq!(
            count_parameters("INSERT INTO users VALUES ($1, $2, $3)"),
            3
        );
        assert_eq!(count_parameters("SELECT * FROM users"), 0);
        assert_eq!(
            count_parameters("SELECT * FROM users WHERE id = $1 OR id = $2 OR id = $1"),
            2
        );
    }

    #[test]
    fn test_substitute_parameters() {
        let params = vec![Some("42".to_string()), Some("Alice".to_string())];

        let result =
            substitute_parameters("SELECT * FROM users WHERE id = $1 AND name = $2", &params)
                .unwrap();
        assert_eq!(
            result,
            "SELECT * FROM users WHERE id = 42 AND name = 'Alice'"
        );
    }

    #[test]
    fn test_substitute_parameters_with_null() {
        let params = vec![Some("42".to_string()), None];

        let result =
            substitute_parameters("INSERT INTO users VALUES ($1, $2)", &params).unwrap();
        assert_eq!(result, "INSERT INTO users VALUES (42, NULL)");
    }

    #[test]
    fn test_substitute_parameters_quote_escape() {
        let params = vec![Some("O'Brien".to_string())];

        let result = substitute_parameters("SELECT * FROM users WHERE name = $1", &params).unwrap();
        assert_eq!(result, "SELECT * FROM users WHERE name = 'O''Brien'");
    }

    #[test]
    fn test_substitute_parameters_numeric() {
        let params = vec![Some("123".to_string()), Some("45.67".to_string())];

        let result = substitute_parameters("SELECT * FROM data WHERE id = $1 AND value = $2", &params).unwrap();
        assert_eq!(result, "SELECT * FROM data WHERE id = 123 AND value = 45.67");
    }
}
