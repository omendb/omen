//! PostgreSQL wire protocol handlers for OmenDB

use super::encoding::{record_batches_to_query_response};
use async_trait::async_trait;
use datafusion::prelude::*;
use pgwire::api::auth::noop::NoopStartupHandler;
use pgwire::api::auth::{AuthSource, DefaultServerParameterProvider, LoginInfo, Password, StartupHandler};
use pgwire::api::copy::NoopCopyHandler;
use pgwire::api::query::{SimpleQueryHandler, ExtendedQueryHandler, PlaceholderExtendedQueryHandler};
use pgwire::api::results::{Response, Tag};
use pgwire::api::{ClientInfo, PgWireHandlerFactory};
use pgwire::error::{ErrorInfo, PgWireError, PgWireResult};
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// OmenDB query handler that executes queries using DataFusion
pub struct OmenDbQueryHandler {
    /// DataFusion session context for query execution
    ctx: Arc<RwLock<SessionContext>>,
}

impl OmenDbQueryHandler {
    pub fn new(ctx: Arc<RwLock<SessionContext>>) -> Self {
        Self { ctx }
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

    /// Handle special PostgreSQL commands
    fn handle_special_command(query: &str) -> PgWireResult<Vec<Response<'_>>> {
        let upper = query.trim().to_uppercase();

        if upper.starts_with("SET") {
            debug!("Handling SET command: {}", query);
            Ok(vec![Response::Execution(Tag::new("SET"))])
        } else if upper.starts_with("SHOW") {
            debug!("Handling SHOW command: {}", query);
            // Return empty result for SHOW commands
            Ok(vec![Response::EmptyQuery])
        } else if upper.starts_with("BEGIN") || upper.starts_with("START TRANSACTION") {
            debug!("Handling BEGIN command");
            Ok(vec![Response::Execution(Tag::new("BEGIN"))])
        } else if upper.starts_with("COMMIT") {
            debug!("Handling COMMIT command");
            Ok(vec![Response::Execution(Tag::new("COMMIT"))])
        } else if upper.starts_with("ROLLBACK") {
            debug!("Handling ROLLBACK command");
            Ok(vec![Response::Execution(Tag::new("ROLLBACK"))])
        } else {
            Ok(vec![Response::EmptyQuery])
        }
    }

    /// Execute a SQL query using DataFusion
    async fn execute_query<'a>(&self, query: &'a str) -> PgWireResult<Vec<Response<'a>>> {
        let ctx = self.ctx.read().await;

        // Execute query with DataFusion
        let df = ctx
            .sql(query)
            .await
            .map_err(|e| {
                error!("DataFusion SQL error: {}", e);
                PgWireError::UserError(Box::new(ErrorInfo::new(
                    "ERROR".to_owned(),
                    "42601".to_owned(),
                    format!("SQL execution error: {}", e),
                )))
            })?;

        // Collect results into RecordBatches
        let batches = df
            .collect()
            .await
            .map_err(|e| {
                error!("DataFusion collect error: {}", e);
                PgWireError::UserError(Box::new(ErrorInfo::new(
                    "ERROR".to_owned(),
                    "XX000".to_owned(),
                    format!("Query execution error: {}", e),
                )))
            })?;

        // Check if this is a DML query (INSERT, UPDATE, DELETE)
        let upper = query.trim().to_uppercase();
        if upper.starts_with("INSERT") || upper.starts_with("UPDATE") || upper.starts_with("DELETE") {
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
    async fn do_query<'a, 'b: 'a, C>(&'b self, _client: &mut C, query: &'a str) -> PgWireResult<Vec<Response<'a>>>
    where
        C: ClientInfo + futures::Sink<pgwire::messages::PgWireBackendMessage> + Unpin + Send + Sync,
        C::Error: Debug,
        PgWireError: From<<C as futures::Sink<pgwire::messages::PgWireBackendMessage>>::Error>,
    {
        info!("Executing query: {}", query);

        // Handle special PostgreSQL commands
        if Self::is_special_command(query) {
            return Self::handle_special_command(query);
        }

        // Handle empty query
        if query.trim().is_empty() {
            return Ok(vec![Response::EmptyQuery]);
        }

        // Execute query with DataFusion
        self.execute_query(query).await
    }
}

/// Startup handler for OmenDB (no authentication for MVP)
pub struct OmenDbStartupHandler;

#[async_trait]
impl NoopStartupHandler for OmenDbStartupHandler {
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

/// Handler factory for OmenDB
pub struct OmenDbHandlerFactory {
    handler: Arc<OmenDbQueryHandler>,
}

impl OmenDbHandlerFactory {
    pub fn new(ctx: Arc<RwLock<SessionContext>>) -> Self {
        Self {
            handler: Arc::new(OmenDbQueryHandler::new(ctx)),
        }
    }
}

impl PgWireHandlerFactory for OmenDbHandlerFactory {
    type StartupHandler = OmenDbStartupHandler;
    type SimpleQueryHandler = OmenDbQueryHandler;
    type ExtendedQueryHandler = PlaceholderExtendedQueryHandler;
    type CopyHandler = NoopCopyHandler;

    fn simple_query_handler(&self) -> Arc<Self::SimpleQueryHandler> {
        self.handler.clone()
    }

    fn extended_query_handler(&self) -> Arc<Self::ExtendedQueryHandler> {
        Arc::new(PlaceholderExtendedQueryHandler)
    }

    fn startup_handler(&self) -> Arc<Self::StartupHandler> {
        Arc::new(OmenDbStartupHandler)
    }

    fn copy_handler(&self) -> Arc<Self::CopyHandler> {
        Arc::new(NoopCopyHandler)
    }
}
