//! Transaction management for ACID compliance
//!
//! Provides write buffering and rollback support for transactions.
//! Operations within a transaction are buffered until COMMIT, or discarded on ROLLBACK.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Transaction state for a single session
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionState {
    /// No active transaction
    Idle,
    /// Transaction in progress, buffering writes
    InProgress { tx_id: u64 },
    /// Transaction committed (brief state before returning to Idle)
    Committed,
    /// Transaction rolled back (brief state before returning to Idle)
    RolledBack,
}

/// A single buffered operation in a transaction
#[derive(Debug, Clone)]
pub enum BufferedOperation {
    /// INSERT statement
    Insert {
        table_name: String,
        query: String,
    },
    /// UPDATE statement (when implemented)
    Update {
        table_name: String,
        query: String,
    },
    /// DELETE statement (when implemented)
    Delete {
        table_name: String,
        query: String,
    },
}

/// Transaction context for a single database session
///
/// Each client connection has its own transaction context.
/// Tracks transaction state and buffers operations until COMMIT.
#[derive(Debug, Clone)]
pub struct TransactionContext {
    /// Current transaction state
    state: TransactionState,
    /// Buffered operations (applied on COMMIT, discarded on ROLLBACK)
    buffer: Vec<BufferedOperation>,
    /// Transaction ID counter (for debugging)
    next_tx_id: u64,
}

impl TransactionContext {
    /// Create a new transaction context
    pub fn new() -> Self {
        Self {
            state: TransactionState::Idle,
            buffer: Vec::new(),
            next_tx_id: 1,
        }
    }

    /// Check if currently in a transaction
    pub fn is_in_transaction(&self) -> bool {
        matches!(self.state, TransactionState::InProgress { .. })
    }

    /// Get current transaction state
    pub fn state(&self) -> &TransactionState {
        &self.state
    }

    /// Begin a new transaction
    ///
    /// Returns error if already in a transaction (nested transactions not supported yet)
    pub fn begin(&mut self) -> Result<u64> {
        match &self.state {
            TransactionState::Idle | TransactionState::Committed | TransactionState::RolledBack => {
                let tx_id = self.next_tx_id;
                self.next_tx_id += 1;
                self.state = TransactionState::InProgress { tx_id };
                self.buffer.clear();

                info!("Transaction {} started", tx_id);
                Ok(tx_id)
            }
            TransactionState::InProgress { tx_id } => {
                warn!("BEGIN called while transaction {} is already active", tx_id);
                // PostgreSQL behavior: BEGIN within transaction is allowed, just continues current tx
                Ok(*tx_id)
            }
        }
    }

    /// Buffer an operation (will be applied on COMMIT)
    pub fn buffer_operation(&mut self, operation: BufferedOperation) -> Result<()> {
        match &self.state {
            TransactionState::InProgress { tx_id } => {
                debug!("Buffering operation in transaction {}: {:?}", tx_id, operation);
                self.buffer.push(operation);
                Ok(())
            }
            _ => {
                // Auto-commit mode: operation should be executed immediately
                // This is handled by the caller
                debug!("No active transaction, operation will be auto-committed");
                Ok(())
            }
        }
    }

    /// Get buffered operations for commit
    ///
    /// Returns all buffered operations and transitions to Committed state
    pub fn prepare_commit(&mut self) -> Result<Vec<BufferedOperation>> {
        match &self.state {
            TransactionState::InProgress { tx_id } => {
                info!("Committing transaction {} ({} operations)", tx_id, self.buffer.len());
                self.state = TransactionState::Committed;
                Ok(self.buffer.clone())
            }
            _ => {
                debug!("COMMIT called outside transaction (no-op)");
                Ok(Vec::new())
            }
        }
    }

    /// Finalize commit (return to Idle state)
    pub fn finalize_commit(&mut self) {
        self.buffer.clear();
        self.state = TransactionState::Idle;
    }

    /// Rollback transaction (discard all buffered operations)
    pub fn rollback(&mut self) -> Result<()> {
        match &self.state {
            TransactionState::InProgress { tx_id } => {
                info!("Rolling back transaction {} ({} operations discarded)", tx_id, self.buffer.len());
                self.buffer.clear();
                self.state = TransactionState::RolledBack;

                // Return to idle immediately after rollback
                self.state = TransactionState::Idle;
                Ok(())
            }
            _ => {
                debug!("ROLLBACK called outside transaction (no-op)");
                Ok(())
            }
        }
    }

    /// Get number of buffered operations
    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }

    /// Get read-only access to buffered operations
    /// Used for constraint validation within transactions
    pub fn buffered_operations(&self) -> &[BufferedOperation] {
        &self.buffer
    }
}

impl Default for TransactionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Global transaction manager
///
/// Manages transaction contexts for all active sessions.
/// Each session is identified by a connection ID.
pub struct TransactionManager {
    /// Map of connection ID -> transaction context
    sessions: Arc<RwLock<HashMap<u64, Arc<RwLock<TransactionContext>>>>>,
    /// Next connection ID
    next_conn_id: Arc<RwLock<u64>>,
}

impl TransactionManager {
    /// Create a new transaction manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            next_conn_id: Arc::new(RwLock::new(1)),
        }
    }

    /// Register a new session and return its connection ID
    pub async fn register_session(&self) -> u64 {
        let mut next_id = self.next_conn_id.write().await;
        let conn_id = *next_id;
        *next_id += 1;

        let mut sessions = self.sessions.write().await;
        sessions.insert(conn_id, Arc::new(RwLock::new(TransactionContext::new())));

        info!("Registered session {}", conn_id);
        conn_id
    }

    /// Get transaction context for a session
    pub async fn get_context(&self, conn_id: u64) -> Option<Arc<RwLock<TransactionContext>>> {
        let sessions = self.sessions.read().await;
        sessions.get(&conn_id).cloned()
    }

    /// Remove a session (on disconnect)
    pub async fn remove_session(&self, conn_id: u64) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(&conn_id);
        info!("Removed session {}", conn_id);
    }

    /// Get active session count
    pub async fn active_sessions(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_lifecycle() {
        let mut ctx = TransactionContext::new();

        // Initially idle
        assert_eq!(ctx.state(), &TransactionState::Idle);
        assert!(!ctx.is_in_transaction());

        // Begin transaction
        let tx_id = ctx.begin().unwrap();
        assert!(ctx.is_in_transaction());
        assert_eq!(ctx.state(), &TransactionState::InProgress { tx_id });

        // Buffer operations
        ctx.buffer_operation(BufferedOperation::Insert {
            table_name: "users".to_string(),
            query: "INSERT INTO users VALUES (1, 'Alice')".to_string(),
        }).unwrap();

        assert_eq!(ctx.buffer_size(), 1);

        // Commit
        let ops = ctx.prepare_commit().unwrap();
        assert_eq!(ops.len(), 1);
        ctx.finalize_commit();

        assert_eq!(ctx.state(), &TransactionState::Idle);
        assert_eq!(ctx.buffer_size(), 0);
    }

    #[test]
    fn test_rollback() {
        let mut ctx = TransactionContext::new();

        // Begin and buffer operations
        ctx.begin().unwrap();
        ctx.buffer_operation(BufferedOperation::Insert {
            table_name: "users".to_string(),
            query: "INSERT INTO users VALUES (1, 'Alice')".to_string(),
        }).unwrap();
        ctx.buffer_operation(BufferedOperation::Insert {
            table_name: "users".to_string(),
            query: "INSERT INTO users VALUES (2, 'Bob')".to_string(),
        }).unwrap();

        assert_eq!(ctx.buffer_size(), 2);

        // Rollback
        ctx.rollback().unwrap();

        // Buffer should be empty, back to idle
        assert_eq!(ctx.buffer_size(), 0);
        assert_eq!(ctx.state(), &TransactionState::Idle);
    }

    #[test]
    fn test_auto_commit_mode() {
        let mut ctx = TransactionContext::new();

        // Not in transaction - operation should not be buffered
        let result = ctx.buffer_operation(BufferedOperation::Insert {
            table_name: "users".to_string(),
            query: "INSERT INTO users VALUES (1, 'Alice')".to_string(),
        });

        assert!(result.is_ok());
        assert_eq!(ctx.buffer_size(), 0); // Not buffered in auto-commit mode
    }

    #[tokio::test]
    async fn test_transaction_manager() {
        let manager = TransactionManager::new();

        // Register sessions
        let conn1 = manager.register_session().await;
        let conn2 = manager.register_session().await;

        assert_eq!(manager.active_sessions().await, 2);

        // Get contexts
        let ctx1 = manager.get_context(conn1).await.unwrap();
        let ctx2 = manager.get_context(conn2).await.unwrap();

        // Start transaction in one session
        ctx1.write().await.begin().unwrap();

        // Should be independent
        assert!(ctx1.read().await.is_in_transaction());
        assert!(!ctx2.read().await.is_in_transaction());

        // Remove session
        manager.remove_session(conn1).await;
        assert_eq!(manager.active_sessions().await, 1);
    }
}
