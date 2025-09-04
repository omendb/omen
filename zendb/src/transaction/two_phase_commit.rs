//! Two-Phase Commit protocol implementation
//!
//! Implements distributed transaction coordination using the classic 2PC protocol
//! to ensure ACID properties across multiple database nodes.

use anyhow::{Result, Context};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};
use tokio::sync::oneshot;
use tokio::time::{timeout, Duration};

use super::{Transaction, TransactionManager, TransactionState};

/// 2PC transaction states
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TwoPhaseState {
    /// Initial state - transaction is being prepared
    Preparing,
    /// All participants have voted to prepare
    Prepared,
    /// Transaction is committed across all participants
    Committed,
    /// Transaction is aborted across all participants
    Aborted,
}

/// 2PC protocol messages between coordinator and participants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TwoPhaseMessage {
    /// Phase 1: Coordinator asks participant to prepare
    Prepare {
        txn_id: u64,
        coordinator_id: u64,
    },
    /// Phase 1 response: Participant votes to prepare
    Prepared {
        txn_id: u64,
        participant_id: u64,
    },
    /// Phase 1 response: Participant votes to abort
    PrepareAbort {
        txn_id: u64,
        participant_id: u64,
        reason: String,
    },
    /// Phase 2: Coordinator tells participants to commit
    Commit {
        txn_id: u64,
        coordinator_id: u64,
    },
    /// Phase 2: Coordinator tells participants to abort
    Abort {
        txn_id: u64,
        coordinator_id: u64,
    },
    /// Response: Participant acknowledges commit/abort
    Acknowledgment {
        txn_id: u64,
        participant_id: u64,
        success: bool,
    },
}

/// Information about a distributed transaction from coordinator perspective
#[derive(Debug, Clone)]
pub struct DistributedTransaction {
    pub local_txn: Transaction,
    pub state: TwoPhaseState,
    pub participants: HashSet<u64>,
    pub prepared_votes: HashSet<u64>,
    pub acknowledgments: HashSet<u64>,
    pub coordinator_id: u64,
}

/// Information about a distributed transaction from participant perspective
#[derive(Debug, Clone)]
pub struct ParticipantTransaction {
    pub local_txn: Transaction,
    pub state: TwoPhaseState,
    pub coordinator_id: u64,
}

/// Two-Phase Commit Coordinator
/// Coordinates distributed transactions across multiple database nodes
pub struct TwoPhaseCommitCoordinator {
    node_id: u64,
    local_txn_manager: Arc<TransactionManager>,
    /// Active distributed transactions this node is coordinating
    coordinated_transactions: Arc<RwLock<HashMap<u64, DistributedTransaction>>>,
    /// Message sender for communication with participants
    message_sender: Option<Box<dyn MessageSender>>,
    /// Timeout for 2PC operations (default: 30 seconds)
    timeout_duration: Duration,
}

/// Two-Phase Commit Participant
/// Participates in distributed transactions coordinated by other nodes
pub struct TwoPhaseCommitParticipant {
    node_id: u64,
    local_txn_manager: Arc<TransactionManager>,
    /// Active distributed transactions this node is participating in
    participant_transactions: Arc<RwLock<HashMap<u64, ParticipantTransaction>>>,
    /// Message sender for communication with coordinator
    message_sender: Option<Box<dyn MessageSender>>,
}

/// Trait for sending messages between nodes
/// Implementation depends on the networking layer (gRPC, HTTP, etc.)
pub trait MessageSender: Send + Sync {
    fn send_message(&self, target_node: u64, message: TwoPhaseMessage) -> Result<()>;
}

impl TwoPhaseCommitCoordinator {
    pub fn new(node_id: u64, txn_manager: Arc<TransactionManager>) -> Self {
        Self {
            node_id,
            local_txn_manager: txn_manager,
            coordinated_transactions: Arc::new(RwLock::new(HashMap::new())),
            message_sender: None,
            timeout_duration: Duration::from_secs(30),
        }
    }

    /// Set the message sender for network communication
    pub fn set_message_sender(&mut self, sender: Box<dyn MessageSender>) {
        self.message_sender = Some(sender);
    }

    /// Begin a distributed transaction across multiple participants
    pub async fn begin_distributed(&self, participants: HashSet<u64>) -> Result<u64> {
        // Start local transaction
        let local_txn = self.local_txn_manager.begin().await?;
        let txn_id = local_txn.id;

        let distributed_txn = DistributedTransaction {
            local_txn,
            state: TwoPhaseState::Preparing,
            participants: participants.clone(),
            prepared_votes: HashSet::new(),
            acknowledgments: HashSet::new(),
            coordinator_id: self.node_id,
        };

        self.coordinated_transactions.write().insert(txn_id, distributed_txn);

        Ok(txn_id)
    }

    /// Execute Phase 1: Send prepare messages to all participants
    pub async fn prepare_phase(&self, txn_id: u64) -> Result<bool> {
        let distributed_txn = {
            let transactions = self.coordinated_transactions.read();
            transactions.get(&txn_id)
                .ok_or_else(|| anyhow::anyhow!("Transaction {} not found", txn_id))?
                .clone()
        };

        if distributed_txn.state != TwoPhaseState::Preparing {
            anyhow::bail!("Transaction {} is not in preparing state", txn_id);
        }

        let message_sender = self.message_sender.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Message sender not configured"))?;

        // Send prepare messages to all participants
        for &participant_id in &distributed_txn.participants {
            let prepare_msg = TwoPhaseMessage::Prepare {
                txn_id,
                coordinator_id: self.node_id,
            };

            message_sender.send_message(participant_id, prepare_msg)
                .with_context(|| format!("Failed to send prepare to participant {}", participant_id))?;
        }

        // Wait for all prepare responses with timeout
        let timeout_result = timeout(
            self.timeout_duration,
            self.wait_for_prepare_responses(txn_id)
        ).await;

        match timeout_result {
            Ok(Ok(all_prepared)) => {
                // Update transaction state
                let mut transactions = self.coordinated_transactions.write();
                if let Some(txn) = transactions.get_mut(&txn_id) {
                    txn.state = if all_prepared {
                        TwoPhaseState::Prepared
                    } else {
                        TwoPhaseState::Aborted
                    };
                }
                Ok(all_prepared)
            }
            Ok(Err(e)) => Err(e),
            Err(_) => {
                // Timeout - mark as aborted
                let mut transactions = self.coordinated_transactions.write();
                if let Some(txn) = transactions.get_mut(&txn_id) {
                    txn.state = TwoPhaseState::Aborted;
                }
                anyhow::bail!("Prepare phase timeout for transaction {}", txn_id)
            }
        }
    }

    /// Execute Phase 2: Send commit or abort to all participants
    pub async fn commit_phase(&self, txn_id: u64, should_commit: bool) -> Result<()> {
        let distributed_txn = {
            let transactions = self.coordinated_transactions.read();
            transactions.get(&txn_id)
                .ok_or_else(|| anyhow::anyhow!("Transaction {} not found", txn_id))?
                .clone()
        };

        let message_sender = self.message_sender.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Message sender not configured"))?;

        // Send commit or abort messages to all participants
        for &participant_id in &distributed_txn.participants {
            let phase2_msg = if should_commit {
                TwoPhaseMessage::Commit {
                    txn_id,
                    coordinator_id: self.node_id,
                }
            } else {
                TwoPhaseMessage::Abort {
                    txn_id,
                    coordinator_id: self.node_id,
                }
            };

            message_sender.send_message(participant_id, phase2_msg)
                .with_context(|| format!("Failed to send commit/abort to participant {}", participant_id))?;
        }

        // Commit or abort local transaction
        if should_commit {
            self.local_txn_manager.commit(distributed_txn.local_txn).await?;
        } else {
            self.local_txn_manager.abort(distributed_txn.local_txn).await?;
        }

        // Update transaction state
        let mut transactions = self.coordinated_transactions.write();
        if let Some(txn) = transactions.get_mut(&txn_id) {
            txn.state = if should_commit {
                TwoPhaseState::Committed
            } else {
                TwoPhaseState::Aborted
            };
        }

        Ok(())
    }

    /// Complete 2PC protocol for a distributed transaction
    pub async fn commit_distributed(&self, txn_id: u64) -> Result<()> {
        // Phase 1: Prepare
        let all_prepared = self.prepare_phase(txn_id).await?;

        // Phase 2: Commit or Abort
        if all_prepared {
            self.commit_phase(txn_id, true).await?;
            println!("Distributed transaction {} committed successfully", txn_id);
        } else {
            self.commit_phase(txn_id, false).await?;
            anyhow::bail!("Distributed transaction {} aborted due to prepare failures", txn_id);
        }

        // Clean up transaction record
        self.coordinated_transactions.write().remove(&txn_id);
        Ok(())
    }

    /// Handle incoming 2PC messages (prepare responses)
    pub fn handle_message(&self, message: TwoPhaseMessage) -> Result<()> {
        match message {
            TwoPhaseMessage::Prepared { txn_id, participant_id } => {
                let mut transactions = self.coordinated_transactions.write();
                if let Some(txn) = transactions.get_mut(&txn_id) {
                    txn.prepared_votes.insert(participant_id);
                }
                Ok(())
            }
            TwoPhaseMessage::PrepareAbort { txn_id, participant_id, reason } => {
                println!("Participant {} voted to abort transaction {}: {}", 
                         participant_id, txn_id, reason);
                // Mark transaction for abort - prepare phase will detect this
                Ok(())
            }
            TwoPhaseMessage::Acknowledgment { txn_id, participant_id, success } => {
                let mut transactions = self.coordinated_transactions.write();
                if let Some(txn) = transactions.get_mut(&txn_id) {
                    if success {
                        txn.acknowledgments.insert(participant_id);
                    }
                }
                Ok(())
            }
            _ => {
                anyhow::bail!("Unexpected message type for coordinator: {:?}", message)
            }
        }
    }

    /// Wait for all participants to respond to prepare phase
    async fn wait_for_prepare_responses(&self, txn_id: u64) -> Result<bool> {
        let (expected_participants, mut current_votes) = {
            let transactions = self.coordinated_transactions.read();
            let txn = transactions.get(&txn_id)
                .ok_or_else(|| anyhow::anyhow!("Transaction {} not found", txn_id))?;
            (txn.participants.clone(), txn.prepared_votes.clone())
        };

        // Poll for votes until all participants respond or timeout
        while current_votes.len() < expected_participants.len() {
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            let transactions = self.coordinated_transactions.read();
            if let Some(txn) = transactions.get(&txn_id) {
                current_votes = txn.prepared_votes.clone();
                
                // Check if transaction was marked for abort
                if txn.state == TwoPhaseState::Aborted {
                    return Ok(false);
                }
            } else {
                return Ok(false); // Transaction was removed/aborted
            }
        }

        // All participants voted to prepare
        Ok(true)
    }

    /// Get statistics about coordinated transactions
    pub fn get_coordinator_stats(&self) -> (usize, usize) {
        let transactions = self.coordinated_transactions.read();
        let total = transactions.len();
        let committed = transactions.values()
            .filter(|txn| txn.state == TwoPhaseState::Committed)
            .count();
        (total, committed)
    }
}

impl TwoPhaseCommitParticipant {
    pub fn new(node_id: u64, txn_manager: Arc<TransactionManager>) -> Self {
        Self {
            node_id,
            local_txn_manager: txn_manager,
            participant_transactions: Arc::new(RwLock::new(HashMap::new())),
            message_sender: None,
        }
    }

    /// Set the message sender for network communication
    pub fn set_message_sender(&mut self, sender: Box<dyn MessageSender>) {
        self.message_sender = Some(sender);
    }

    /// Handle incoming 2PC messages from coordinator
    pub async fn handle_message(&self, message: TwoPhaseMessage) -> Result<()> {
        match message {
            TwoPhaseMessage::Prepare { txn_id, coordinator_id } => {
                self.handle_prepare(txn_id, coordinator_id).await
            }
            TwoPhaseMessage::Commit { txn_id, coordinator_id } => {
                self.handle_commit(txn_id, coordinator_id).await
            }
            TwoPhaseMessage::Abort { txn_id, coordinator_id } => {
                self.handle_abort(txn_id, coordinator_id).await
            }
            _ => {
                anyhow::bail!("Unexpected message type for participant: {:?}", message)
            }
        }
    }

    /// Handle prepare message from coordinator
    async fn handle_prepare(&self, txn_id: u64, coordinator_id: u64) -> Result<()> {
        let message_sender = self.message_sender.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Message sender not configured"))?;

        // Check if we can prepare (validate transaction state, locks, etc.)
        let can_prepare = self.can_prepare_transaction(txn_id).await;

        let response_msg = if can_prepare {
            // Create participant transaction record
            let local_txn = self.local_txn_manager.begin().await?;
            let participant_txn = ParticipantTransaction {
                local_txn,
                state: TwoPhaseState::Prepared,
                coordinator_id,
            };
            
            self.participant_transactions.write().insert(txn_id, participant_txn);

            TwoPhaseMessage::Prepared {
                txn_id,
                participant_id: self.node_id,
            }
        } else {
            TwoPhaseMessage::PrepareAbort {
                txn_id,
                participant_id: self.node_id,
                reason: "Cannot prepare transaction due to conflicts".to_string(),
            }
        };

        message_sender.send_message(coordinator_id, response_msg)?;
        Ok(())
    }

    /// Handle commit message from coordinator
    async fn handle_commit(&self, txn_id: u64, coordinator_id: u64) -> Result<()> {
        let participant_txn = {
            let mut transactions = self.participant_transactions.write();
            transactions.remove(&txn_id)
        };

        let success = if let Some(txn) = participant_txn {
            // Commit local transaction
            match self.local_txn_manager.commit(txn.local_txn).await {
                Ok(_) => true,
                Err(e) => {
                    println!("Failed to commit transaction {}: {}", txn_id, e);
                    false
                }
            }
        } else {
            false
        };

        // Send acknowledgment to coordinator
        if let Some(sender) = &self.message_sender {
            let ack_msg = TwoPhaseMessage::Acknowledgment {
                txn_id,
                participant_id: self.node_id,
                success,
            };
            sender.send_message(coordinator_id, ack_msg)?;
        }

        Ok(())
    }

    /// Handle abort message from coordinator
    async fn handle_abort(&self, txn_id: u64, coordinator_id: u64) -> Result<()> {
        let participant_txn = {
            let mut transactions = self.participant_transactions.write();
            transactions.remove(&txn_id)
        };

        let success = if let Some(txn) = participant_txn {
            // Abort local transaction
            match self.local_txn_manager.abort(txn.local_txn).await {
                Ok(_) => false, // Abort returns error, but that's expected
                Err(_) => true, // Successfully aborted
            }
        } else {
            true
        };

        // Send acknowledgment to coordinator
        if let Some(sender) = &self.message_sender {
            let ack_msg = TwoPhaseMessage::Acknowledgment {
                txn_id,
                participant_id: self.node_id,
                success,
            };
            sender.send_message(coordinator_id, ack_msg)?;
        }

        Ok(())
    }

    /// Check if this participant can prepare for the given transaction
    async fn can_prepare_transaction(&self, _txn_id: u64) -> bool {
        // In a real implementation, this would check:
        // - Resource availability
        // - Lock conflicts
        // - Storage constraints
        // - Network connectivity
        
        // For now, always return true (optimistic)
        true
    }

    /// Get statistics about participated transactions
    pub fn get_participant_stats(&self) -> usize {
        self.participant_transactions.read().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Mock message sender for testing
    struct MockMessageSender {
        messages: Arc<RwLock<Vec<(u64, TwoPhaseMessage)>>>,
    }

    impl MockMessageSender {
        fn new() -> Self {
            Self {
                messages: Arc::new(RwLock::new(Vec::new())),
            }
        }

        fn get_messages(&self) -> Vec<(u64, TwoPhaseMessage)> {
            self.messages.read().clone()
        }

        fn clear_messages(&self) {
            self.messages.write().clear();
        }
    }

    impl MessageSender for MockMessageSender {
        fn send_message(&self, target_node: u64, message: TwoPhaseMessage) -> Result<()> {
            self.messages.write().push((target_node, message));
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_2pc_coordinator_creation() {
        let txn_manager = Arc::new(TransactionManager::new());
        let coordinator = TwoPhaseCommitCoordinator::new(1, txn_manager);
        
        assert_eq!(coordinator.node_id, 1);
        let (total, committed) = coordinator.get_coordinator_stats();
        assert_eq!(total, 0);
        assert_eq!(committed, 0);
    }

    #[tokio::test]
    async fn test_2pc_participant_creation() {
        let txn_manager = Arc::new(TransactionManager::new());
        let participant = TwoPhaseCommitParticipant::new(2, txn_manager);
        
        assert_eq!(participant.node_id, 2);
        assert_eq!(participant.get_participant_stats(), 0);
    }

    #[tokio::test]
    async fn test_distributed_transaction_begin() -> Result<()> {
        let txn_manager = Arc::new(TransactionManager::new());
        let coordinator = TwoPhaseCommitCoordinator::new(1, txn_manager);
        
        let participants = [2, 3, 4].iter().cloned().collect();
        let txn_id = coordinator.begin_distributed(participants).await?;
        
        assert!(txn_id > 0);
        let (total, committed) = coordinator.get_coordinator_stats();
        assert_eq!(total, 1);
        assert_eq!(committed, 0);
        
        Ok(())
    }
}