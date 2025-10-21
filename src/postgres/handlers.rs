//! PostgreSQL wire protocol handlers for OmenDB

use super::auth::OmenDbAuthSource;
use super::encoding::record_batches_to_query_response;
use super::extended::OmenDbExtendedQueryHandler;
use crate::constraints::ConstraintManager;
use crate::metrics;
use crate::transaction::{BufferedOperation, TransactionContext};
use async_trait::async_trait;
use datafusion::prelude::*;
use pgwire::api::auth::noop::NoopStartupHandler;
use pgwire::api::auth::scram::SASLScramAuthStartupHandler;
use pgwire::api::auth::{DefaultServerParameterProvider, StartupHandler};
use pgwire::api::copy::NoopCopyHandler;
use pgwire::api::query::SimpleQueryHandler;
use pgwire::api::results::{Response, Tag};
use pgwire::api::{ClientInfo, PgWireHandlerFactory};
use pgwire::error::{ErrorInfo, PgWireError, PgWireResult};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// OmenDB query handler that executes queries using DataFusion
pub struct OmenDbQueryHandler {
    /// DataFusion session context for query execution
    ctx: Arc<RwLock<SessionContext>>,
    /// Transaction context for ACID compliance
    tx_ctx: Arc<RwLock<TransactionContext>>,
    /// Constraint manager for PRIMARY KEY validation
    constraint_mgr: Arc<ConstraintManager>,
}

impl OmenDbQueryHandler {
    pub fn new(ctx: Arc<RwLock<SessionContext>>) -> Self {
        let constraint_mgr = Arc::new(ConstraintManager::new(ctx.clone()));
        Self {
            ctx,
            tx_ctx: Arc::new(RwLock::new(TransactionContext::new())),
            constraint_mgr,
        }
    }

    pub fn new_with_tx(ctx: Arc<RwLock<SessionContext>>, tx_ctx: Arc<RwLock<TransactionContext>>) -> Self {
        let constraint_mgr = Arc::new(ConstraintManager::new(ctx.clone()));
        Self {
            ctx,
            tx_ctx,
            constraint_mgr,
        }
    }

    /// Check if a query is a special PostgreSQL command that we should handle separately
    fn is_special_command(query: &str) -> bool {
        let upper = query.trim().to_uppercase();
        upper.starts_with("SET")
            || upper.starts_with("SHOW")
            || upper.starts_with("BEGIN")
            || upper.starts_with("COMMIT")
            || upper.starts_with("ROLLBACK")
            || upper.starts_with("START TRANSACTION")
    }

    /// Extract table name from DROP TABLE statement
    fn extract_drop_table_name(query: &str) -> Option<String> {
        let upper = query.trim().to_uppercase();
        let drop_table = "DROP TABLE ";

        if let Some(pos) = upper.find(drop_table) {
            let after_drop = &query[pos + drop_table.len()..];

            // Handle "DROP TABLE IF EXISTS"
            let after_if_exists = if after_drop.trim().to_uppercase().starts_with("IF EXISTS") {
                after_drop[9..].trim()
            } else {
                after_drop.trim()
            };

            // Get table name (until space, semicolon, or end)
            if let Some(end) = after_if_exists.find([' ', ';', ',']) {
                return Some(after_if_exists[..end].trim().to_string());
            } else {
                // Rest of string is the table name
                return Some(after_if_exists.trim().to_string());
            }
        }

        None
    }

    /// Classify query type for metrics tracking
    fn classify_query(query: &str) -> String {
        let upper = query.trim().to_uppercase();
        if upper.starts_with("SELECT") {
            "SELECT".to_string()
        } else if upper.starts_with("INSERT") {
            "INSERT".to_string()
        } else if upper.starts_with("UPDATE") {
            "UPDATE".to_string()
        } else if upper.starts_with("DELETE") {
            "DELETE".to_string()
        } else if upper.starts_with("CREATE TABLE") {
            "CREATE_TABLE".to_string()
        } else if upper.starts_with("CREATE") {
            "CREATE".to_string()
        } else if upper.starts_with("DROP TABLE") {
            "DROP_TABLE".to_string()
        } else if upper.starts_with("DROP") {
            "DROP".to_string()
        } else if upper.starts_with("ALTER") {
            "ALTER".to_string()
        } else {
            "OTHER".to_string()
        }
    }

    /// Handle special PostgreSQL commands
    async fn handle_special_command(&self, query: &str) -> PgWireResult<Vec<Response<'_>>> {
        let upper = query.trim().to_uppercase();

        if upper.starts_with("SET") {
            debug!("Handling SET command: {}", query);
            Ok(vec![Response::Execution(Tag::new("SET"))])
        } else if upper.starts_with("SHOW") {
            debug!("Handling SHOW command: {}", query);
            // Return empty result for SHOW commands
            Ok(vec![Response::EmptyQuery])
        } else if upper.starts_with("BEGIN") || upper.starts_with("START TRANSACTION") {
            info!("BEGIN transaction");
            let mut tx = self.tx_ctx.write().await;
            match tx.begin() {
                Ok(tx_id) => {
                    info!("Transaction {} started", tx_id);
                    Ok(vec![Response::Execution(Tag::new("BEGIN"))])
                }
                Err(e) => {
                    error!("Failed to start transaction: {}", e);
                    Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                        "ERROR".to_owned(),
                        "25001".to_owned(),
                        format!("Failed to start transaction: {}", e),
                    ))))
                }
            }
        } else if upper.starts_with("COMMIT") {
            info!("COMMIT transaction");
            let mut tx = self.tx_ctx.write().await;

            // Get buffered operations
            let operations = match tx.prepare_commit() {
                Ok(ops) => ops,
                Err(e) => {
                    error!("Failed to prepare commit: {}", e);
                    return Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                        "ERROR".to_owned(),
                        "25000".to_owned(),
                        format!("Failed to commit: {}", e),
                    ))));
                }
            };

            // Execute all buffered operations
            if !operations.is_empty() {
                info!("Committing {} buffered operations", operations.len());
                for op in operations {
                    let query = match op {
                        BufferedOperation::Insert { query, .. } => query,
                        BufferedOperation::Update { query, .. } => query,
                        BufferedOperation::Delete { query, .. } => query,
                    };

                    // Execute the query
                    if let Err(e) = self.execute_query_direct(&query).await {
                        error!("Failed to execute buffered operation: {}", e);
                        // Transaction failed - rollback
                        tx.rollback().ok();
                        return Err(e);
                    }
                }
            }

            // Finalize commit
            tx.finalize_commit();
            info!("Transaction committed successfully");
            Ok(vec![Response::Execution(Tag::new("COMMIT"))])
        } else if upper.starts_with("ROLLBACK") {
            info!("ROLLBACK transaction");
            let mut tx = self.tx_ctx.write().await;
            match tx.rollback() {
                Ok(()) => {
                    info!("Transaction rolled back successfully");
                    Ok(vec![Response::Execution(Tag::new("ROLLBACK"))])
                }
                Err(e) => {
                    error!("Failed to rollback: {}", e);
                    Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                        "ERROR".to_owned(),
                        "25000".to_owned(),
                        format!("Failed to rollback: {}", e),
                    ))))
                }
            }
        } else {
            Ok(vec![Response::EmptyQuery])
        }
    }

    /// Extract primary key values from buffered INSERT operations
    async fn extract_buffered_pk_values(&self, tx: &TransactionContext) -> Vec<(String, Vec<String>)> {
        let mut buffered_pks = Vec::new();

        for op in tx.buffered_operations() {
            if let BufferedOperation::Insert { query, .. } = op {
                // Parse table name and PK values from buffered INSERT
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
                    // Check if this PK value is already in buffered operations
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

    /// Execute a SQL query using DataFusion
    async fn execute_query<'a>(&self, query: &'a str) -> PgWireResult<Vec<Response<'a>>> {
        let start_time = Instant::now();

        // Check if we're in a transaction and this is a DML query
        let upper = query.trim().to_uppercase();
        let is_dml = upper.starts_with("INSERT") || upper.starts_with("UPDATE") || upper.starts_with("DELETE");

        // Register table schema if this is a CREATE TABLE
        if upper.starts_with("CREATE TABLE") {
            self.constraint_mgr.register_table_schema(query).await.map_err(|e| {
                error!("Failed to register table schema: {}", e);
                PgWireError::UserError(Box::new(ErrorInfo::new(
                    "ERROR".to_owned(),
                    "XX000".to_owned(),
                    format!("Schema registration error: {}", e),
                )))
            })?;
        }

        // Remove constraints if this is a DROP TABLE
        if upper.starts_with("DROP TABLE") {
            if let Some(table_name) = Self::extract_drop_table_name(query) {
                let constraints_arc = self.constraint_mgr.constraints();
                let mut constraints = constraints_arc.write().await;
                constraints.remove_table(&table_name);
                debug!("Removed constraints for table: {}", table_name);
            }
        }

        // Validate INSERT statements against PRIMARY KEY constraints
        if upper.starts_with("INSERT") {
            // Check if we're in a transaction and get buffered operations
            let tx = self.tx_ctx.read().await;
            let buffered_inserts = if tx.is_in_transaction() {
                // Extract primary key values from buffered INSERT operations
                self.extract_buffered_pk_values(&tx).await
            } else {
                Vec::new()
            };
            drop(tx); // Release lock before constraint validation

            // Validate against both committed data and buffered operations
            self.validate_insert_with_transaction(query, &buffered_inserts).await.map_err(|e| {
                error!("Constraint violation: {}", e);
                PgWireError::UserError(Box::new(ErrorInfo::new(
                    "ERROR".to_owned(),
                    "23505".to_owned(), // unique_violation error code
                    e.to_string(),
                )))
            })?;
        }

        if is_dml {
            let tx = self.tx_ctx.read().await;
            if tx.is_in_transaction() {
                // Buffer the operation instead of executing
                drop(tx); // Release read lock
                let mut tx = self.tx_ctx.write().await;

                let operation = if upper.starts_with("INSERT") {
                    BufferedOperation::Insert {
                        table_name: "unknown".to_string(), // TODO: Parse table name
                        query: query.to_string(),
                    }
                } else if upper.starts_with("UPDATE") {
                    BufferedOperation::Update {
                        table_name: "unknown".to_string(),
                        query: query.to_string(),
                    }
                } else {
                    BufferedOperation::Delete {
                        table_name: "unknown".to_string(),
                        query: query.to_string(),
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

                info!("Buffered {} operation (transaction in progress)", if upper.starts_with("INSERT") { "INSERT" } else if upper.starts_with("UPDATE") { "UPDATE" } else { "DELETE" });

                // Return success without executing
                let tag = if upper.starts_with("INSERT") {
                    Tag::new("INSERT").with_oid(0).with_rows(1)
                } else if upper.starts_with("UPDATE") {
                    Tag::new("UPDATE").with_rows(1)
                } else {
                    Tag::new("DELETE").with_rows(1)
                };

                return Ok(vec![Response::Execution(tag)]);
            }
        }

        // Auto-commit mode or non-DML: execute immediately
        let ctx = self.ctx.read().await;

        // Determine query type for metrics
        let query_type = Self::classify_query(query);

        // Execute query with DataFusion
        let df = ctx.sql(query).await.map_err(|e| {
            error!("DataFusion SQL error: {}", e);
            metrics::record_sql_query_error("execution_error");
            PgWireError::UserError(Box::new(ErrorInfo::new(
                "ERROR".to_owned(),
                "42601".to_owned(),
                format!("SQL execution error: {}", e),
            )))
        })?;

        // Collect results into RecordBatches
        let batches = df.collect().await.map_err(|e| {
            error!("DataFusion collect error: {}", e);
            metrics::record_sql_query_error("collect_error");
            PgWireError::UserError(Box::new(ErrorInfo::new(
                "ERROR".to_owned(),
                "XX000".to_owned(),
                format!("Query execution error: {}", e),
            )))
        })?;

        // Calculate total rows
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();

        // Record successful query metrics
        let duration = start_time.elapsed().as_secs_f64();
        metrics::record_sql_query(&query_type, duration, total_rows);

        // Check if this is a DML query (INSERT, UPDATE, DELETE)
        let upper = query.trim().to_uppercase();
        if upper.starts_with("INSERT") || upper.starts_with("UPDATE") || upper.starts_with("DELETE")
        {
            // For DML queries, count affected rows
            let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();

            let tag = if upper.starts_with("INSERT") {
                Tag::new("INSERT").with_oid(0).with_rows(total_rows)
            } else if upper.starts_with("UPDATE") {
                Tag::new("UPDATE").with_rows(total_rows)
            } else {
                Tag::new("DELETE").with_rows(total_rows)
            };

            return Ok(vec![Response::Execution(tag)]);
        }

        // For SELECT and other queries, return result set
        record_batches_to_query_response(batches)
    }
}

#[async_trait]
impl SimpleQueryHandler for OmenDbQueryHandler {
    async fn do_query<'a, 'b: 'a, C>(
        &'b self,
        _client: &mut C,
        query: &'a str,
    ) -> PgWireResult<Vec<Response<'a>>>
    where
        C: ClientInfo + futures::Sink<pgwire::messages::PgWireBackendMessage> + Unpin + Send + Sync,
        C::Error: Debug,
        PgWireError: From<<C as futures::Sink<pgwire::messages::PgWireBackendMessage>>::Error>,
    {
        info!("Executing query: {}", query);

        // Handle special PostgreSQL commands
        if Self::is_special_command(query) {
            return self.handle_special_command(query).await;
        }

        // Handle empty query
        if query.trim().is_empty() {
            return Ok(vec![Response::EmptyQuery]);
        }

        // Execute query with DataFusion
        self.execute_query(query).await
    }
}

/// Startup handler for OmenDB (no authentication)
pub struct OmenDbNoopStartupHandler;

#[async_trait]
impl NoopStartupHandler for OmenDbNoopStartupHandler {
    async fn post_startup<C>(
        &self,
        client: &mut C,
        _message: pgwire::messages::PgWireFrontendMessage,
    ) -> PgWireResult<()>
    where
        C: ClientInfo + futures::Sink<pgwire::messages::PgWireBackendMessage> + Unpin + Send,
        C::Error: Debug,
        PgWireError: From<<C as futures::Sink<pgwire::messages::PgWireBackendMessage>>::Error>,
    {
        info!(
            "Client connected: {} (secure: {})",
            client.socket_addr(),
            client.is_secure()
        );
        Ok(())
    }
}

/// Configuration for creating startup handlers
pub enum OmenDbStartupHandlerConfig {
    Noop,
    Scram {
        auth_source: Arc<OmenDbAuthSource>,
        parameter_provider: Arc<DefaultServerParameterProvider>,
    },
}

/// Enum to hold either authentication type
pub enum OmenDbStartupHandler {
    Noop(Arc<OmenDbNoopStartupHandler>),
    Scram(Arc<SASLScramAuthStartupHandler<OmenDbAuthSource, DefaultServerParameterProvider>>),
}

impl OmenDbStartupHandlerConfig {
    pub fn noop() -> Self {
        Self::Noop
    }

    pub fn scram(auth_source: Arc<OmenDbAuthSource>) -> Self {
        let parameter_provider = Arc::new(DefaultServerParameterProvider::default());
        Self::Scram {
            auth_source,
            parameter_provider,
        }
    }

    /// Create a new handler instance (called per connection)
    pub fn create_handler(&self) -> OmenDbStartupHandler {
        match self {
            Self::Noop => OmenDbStartupHandler::Noop(Arc::new(OmenDbNoopStartupHandler)),
            Self::Scram {
                auth_source,
                parameter_provider,
            } => {
                // Create new handler with fresh state for this connection
                let handler = SASLScramAuthStartupHandler::new(
                    auth_source.clone(),
                    parameter_provider.clone(),
                );
                OmenDbStartupHandler::Scram(Arc::new(handler))
            }
        }
    }
}

/// Handler factory for OmenDB
pub struct OmenDbHandlerFactory {
    simple_handler: Arc<OmenDbQueryHandler>,
    extended_handler: Arc<OmenDbExtendedQueryHandler>,
    startup_config: OmenDbStartupHandlerConfig,
}

impl OmenDbHandlerFactory {
    /// Create factory without authentication
    pub fn new(ctx: Arc<RwLock<SessionContext>>) -> Self {
        // Create a shared transaction context for both handlers
        let tx_ctx = Arc::new(RwLock::new(TransactionContext::new()));

        Self {
            simple_handler: Arc::new(OmenDbQueryHandler::new_with_tx(ctx.clone(), tx_ctx.clone())),
            extended_handler: Arc::new(OmenDbExtendedQueryHandler::new(ctx, tx_ctx)),
            startup_config: OmenDbStartupHandlerConfig::noop(),
        }
    }

    /// Create factory with SCRAM-SHA-256 authentication
    pub fn new_with_auth(
        ctx: Arc<RwLock<SessionContext>>,
        auth_source: Arc<OmenDbAuthSource>,
    ) -> Self {
        // Create a shared transaction context for both handlers
        let tx_ctx = Arc::new(RwLock::new(TransactionContext::new()));

        Self {
            simple_handler: Arc::new(OmenDbQueryHandler::new_with_tx(ctx.clone(), tx_ctx.clone())),
            extended_handler: Arc::new(OmenDbExtendedQueryHandler::new(ctx, tx_ctx)),
            startup_config: OmenDbStartupHandlerConfig::scram(auth_source),
        }
    }
}

#[async_trait]
impl StartupHandler for OmenDbStartupHandler {
    async fn on_startup<C>(
        &self,
        client: &mut C,
        message: pgwire::messages::PgWireFrontendMessage,
    ) -> PgWireResult<()>
    where
        C: ClientInfo + futures::Sink<pgwire::messages::PgWireBackendMessage> + Unpin + Send,
        C::Error: Debug,
        PgWireError: From<<C as futures::Sink<pgwire::messages::PgWireBackendMessage>>::Error>,
    {
        match self {
            Self::Noop(handler) => handler.on_startup(client, message).await,
            Self::Scram(handler) => handler.on_startup(client, message).await,
        }
    }
}

impl PgWireHandlerFactory for OmenDbHandlerFactory {
    type StartupHandler = OmenDbStartupHandler;
    type SimpleQueryHandler = OmenDbQueryHandler;
    type ExtendedQueryHandler = OmenDbExtendedQueryHandler;
    type CopyHandler = NoopCopyHandler;

    fn simple_query_handler(&self) -> Arc<Self::SimpleQueryHandler> {
        info!("[FACTORY] Returning simple query handler");
        self.simple_handler.clone()
    }

    fn extended_query_handler(&self) -> Arc<Self::ExtendedQueryHandler> {
        info!("[FACTORY] Returning extended query handler");
        self.extended_handler.clone()
    }

    fn startup_handler(&self) -> Arc<Self::StartupHandler> {
        info!("[FACTORY] Creating fresh startup handler for new connection");
        // Create a new handler for each connection (fresh SCRAM state)
        Arc::new(self.startup_config.create_handler())
    }

    fn copy_handler(&self) -> Arc<Self::CopyHandler> {
        info!("[FACTORY] Returning copy handler");
        Arc::new(NoopCopyHandler)
    }
}
