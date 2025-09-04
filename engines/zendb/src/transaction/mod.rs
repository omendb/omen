//! Transaction management with MVCC and Hybrid Logical Clocks
//! 
//! Implements optimistic concurrency control using HLC timestamps
//! for distributed timestamp ordering (inspired by Google Spanner).

pub mod mvcc;
pub mod two_phase_commit;

use anyhow::Result;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;

pub use mvcc::{HLCTimestamp, HLC, MVCCStorage, Version};
pub use two_phase_commit::{
    TwoPhaseCommitCoordinator, TwoPhaseCommitParticipant, 
    TwoPhaseMessage, TwoPhaseState, MessageSender,
    DistributedTransaction, ParticipantTransaction
};

/// Transaction isolation levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IsolationLevel {
    /// Read uncommitted - can see uncommitted changes
    ReadUncommitted,
    /// Read committed - only see committed changes
    ReadCommitted,
    /// Repeatable read - consistent snapshot for entire transaction
    RepeatableRead,
    /// Serializable - full serializability
    Serializable,
}

impl Default for IsolationLevel {
    fn default() -> Self {
        Self::RepeatableRead
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionState {
    Active,
    Committed,
    Aborted,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: u64,
    pub start_timestamp: HLCTimestamp,
    pub commit_timestamp: Option<HLCTimestamp>,
    pub state: TransactionState,
    pub isolation_level: IsolationLevel,
    /// Write set for conflict detection
    pub write_set: Vec<Vec<u8>>,
    /// Read set for conflict detection (serializable only)
    pub read_set: Vec<Vec<u8>>,
}

impl Transaction {
    /// Check if transaction is active
    pub fn is_active(&self) -> bool {
        self.state == TransactionState::Active
    }
    
    /// Add key to write set
    pub fn add_write(&mut self, key: Vec<u8>) {
        if !self.write_set.contains(&key) {
            self.write_set.push(key);
        }
    }
    
    /// Add key to read set
    pub fn add_read(&mut self, key: Vec<u8>) {
        if self.isolation_level == IsolationLevel::Serializable {
            if !self.read_set.contains(&key) {
                self.read_set.push(key);
            }
        }
    }
}

pub struct TransactionManager {
    /// MVCC storage layer
    mvcc: Arc<MVCCStorage>,
    /// Active transactions
    active_transactions: Arc<RwLock<HashMap<u64, Transaction>>>,
    /// Recently committed transactions (for conflict detection)
    committed_transactions: Arc<RwLock<HashMap<u64, Transaction>>>,
    /// Next transaction ID
    next_txn_id: AtomicU64,
    /// Hybrid logical clock
    clock: Arc<HLC>,
}

impl TransactionManager {
    pub fn new() -> Self {
        let mvcc = Arc::new(MVCCStorage::new());
        let clock = Arc::new(HLC::new(1)); // Node ID 1
        
        Self {
            mvcc,
            active_transactions: Arc::new(RwLock::new(HashMap::new())),
            committed_transactions: Arc::new(RwLock::new(HashMap::new())),
            next_txn_id: AtomicU64::new(1),
            clock,
        }
    }
    
    /// Begin a new transaction
    pub async fn begin(&self) -> Result<Transaction> {
        self.begin_with_isolation(IsolationLevel::default()).await
    }
    
    /// Begin a new transaction with specific isolation level
    pub async fn begin_with_isolation(&self, isolation: IsolationLevel) -> Result<Transaction> {
        // Use MVCC's transaction ID to ensure consistency
        let (tx_id, start_timestamp) = self.mvcc.begin_transaction();
        
        let transaction = Transaction {
            id: tx_id,
            start_timestamp,
            commit_timestamp: None,
            state: TransactionState::Active,
            isolation_level: isolation,
            write_set: Vec::new(),
            read_set: Vec::new(),
        };
        
        self.active_transactions.write().insert(tx_id, transaction.clone());
        Ok(transaction)
    }
    
    /// Commit a transaction
    pub async fn commit(&self, transaction: Transaction) -> Result<()> {
        if !transaction.is_active() {
            anyhow::bail!("Transaction {} is not active", transaction.id);
        }
        
        // Get the current transaction state from active_transactions (with updated write/read sets)
        let mut current_tx = {
            let active = self.active_transactions.read();
            active.get(&transaction.id)
                .ok_or_else(|| anyhow::anyhow!("Transaction {} not found in active transactions", transaction.id))?
                .clone()
        };
        
        // Check for write-write conflicts using the current transaction state
        if !self.check_conflicts(&current_tx)? {
            return self.abort(transaction).await;
        }
        
        // Get commit timestamp
        let commit_ts = self.clock.now();
        current_tx.commit_timestamp = Some(commit_ts);
        current_tx.state = TransactionState::Committed;
        
        // Commit in MVCC storage
        self.mvcc.commit(current_tx.id)?;
        
        // Move from active to committed transactions
        self.active_transactions.write().remove(&current_tx.id);
        self.committed_transactions.write().insert(current_tx.id, current_tx);
        
        Ok(())
    }
    
    /// Abort a transaction
    pub async fn abort(&self, mut transaction: Transaction) -> Result<()> {
        if !transaction.is_active() {
            anyhow::bail!("Transaction {} is not active", transaction.id);
        }
        
        transaction.state = TransactionState::Aborted;
        
        // Rollback in MVCC storage
        self.mvcc.rollback(transaction.id)?;
        
        // Remove from active transactions
        self.active_transactions.write().remove(&transaction.id);
        
        // Return error to indicate abort
        anyhow::bail!("Transaction {} aborted due to write-write conflict", transaction.id)
    }
    
    /// Check for conflicts with other transactions
    fn check_conflicts(&self, transaction: &Transaction) -> Result<bool> {
        let active = self.active_transactions.read();
        let committed = self.committed_transactions.read();
        
        // Check conflicts with active transactions
        for (_, other_tx) in active.iter() {
            if other_tx.id == transaction.id {
                continue;
            }
            
            // Check if other transaction is concurrent (started before this one)
            if other_tx.start_timestamp < transaction.start_timestamp {
                // Check for write-write conflicts
                for key in &transaction.write_set {
                    if other_tx.write_set.contains(key) {
                        return Ok(false); // Conflict detected
                    }
                }
                
                // Check for read-write conflicts (serializable only)
                if transaction.isolation_level == IsolationLevel::Serializable {
                    for key in &transaction.read_set {
                        if other_tx.write_set.contains(key) {
                            return Ok(false); // Conflict detected
                        }
                    }
                }
            }
        }
        
        // Check conflicts with recently committed transactions that overlap with our write set
        for (_, committed_tx) in committed.iter() {
            if committed_tx.id == transaction.id {
                continue;
            }
            
            // Check if the committed transaction overlaps with our transaction lifetime
            // A conflict exists if the committed transaction committed after we started
            if let Some(commit_ts) = committed_tx.commit_timestamp {
                if commit_ts > transaction.start_timestamp {
                    // This transaction committed after we started, check for write-write conflicts
                    for key in &transaction.write_set {
                        if committed_tx.write_set.contains(key) {
                            return Ok(false); // Write-write conflict detected
                        }
                    }
                }
            }
        }
        
        Ok(true) // No conflicts
    }
    
    /// Get a value within a transaction
    pub async fn get(&self, key: &[u8], txn_id: u64) -> Result<Option<Vec<u8>>> {
        // Record read in transaction
        {
            let mut active = self.active_transactions.write();
            if let Some(tx) = active.get_mut(&txn_id) {
                tx.add_read(key.to_vec());
            }
        }
        
        self.mvcc.get(key, txn_id)
    }
    
    /// Put a value within a transaction
    pub async fn put(&self, key: Vec<u8>, value: Vec<u8>, txn_id: u64) -> Result<()> {
        // Record write in transaction
        {
            let mut active = self.active_transactions.write();
            if let Some(tx) = active.get_mut(&txn_id) {
                tx.add_write(key.clone());
            }
        }
        
        self.mvcc.put(key, value, txn_id)
    }
    
    /// Delete a key within a transaction
    pub async fn delete(&self, key: Vec<u8>, txn_id: u64) -> Result<()> {
        // Record write in transaction
        {
            let mut active = self.active_transactions.write();
            if let Some(tx) = active.get_mut(&txn_id) {
                tx.add_write(key.clone());
            }
        }
        
        self.mvcc.delete(key, txn_id)
    }
    
    /// Time-travel query: get value at specific timestamp
    pub fn get_at_timestamp(&self, key: &[u8], timestamp: HLCTimestamp) -> Option<Vec<u8>> {
        self.mvcc.get_at_timestamp(key, timestamp)
    }
    
    /// Range scan within a transaction
    pub async fn range_scan(
        &self,
        start: &[u8],
        end: &[u8],
        txn_id: u64,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        self.mvcc.range_scan(start, end, txn_id)
    }
    
    /// Garbage collect old versions
    pub fn garbage_collect(&self, keep_duration_micros: u64) {
        self.mvcc.garbage_collect(keep_duration_micros);
        
        // Clean up old committed transactions (keep for conflict detection window)
        let cutoff = HLCTimestamp::new(
            (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64)
                .saturating_sub(keep_duration_micros),
            0,
        );
        
        let mut committed = self.committed_transactions.write();
        committed.retain(|_, tx| {
            if let Some(commit_ts) = tx.commit_timestamp {
                commit_ts > cutoff
            } else {
                true // Keep if no commit timestamp
            }
        });
        
        // Clean up completed transactions from active (shouldn't happen normally)
        let mut active = self.active_transactions.write();
        active.retain(|_, tx| tx.is_active());
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}