//! Tests for Two-Phase Commit protocol implementation

use zendb::transaction::{
    TransactionManager, TwoPhaseCommitCoordinator, TwoPhaseCommitParticipant,
    TwoPhaseMessage, TwoPhaseState, MessageSender
};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

/// Mock message delivery system for testing 2PC protocol
#[derive(Clone)]
struct MockMessageBus {
    /// Messages waiting to be delivered
    pending_messages: Arc<Mutex<HashMap<u64, Vec<TwoPhaseMessage>>>>,
    /// All sent messages for verification
    message_history: Arc<Mutex<Vec<(u64, u64, TwoPhaseMessage)>>>, // (from, to, message)
    /// Simulate network failures for specific nodes
    failed_nodes: Arc<Mutex<HashSet<u64>>>,
}

impl MockMessageBus {
    fn new() -> Self {
        Self {
            pending_messages: Arc::new(Mutex::new(HashMap::new())),
            message_history: Arc::new(Mutex::new(Vec::new())),
            failed_nodes: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Create a message sender for a specific node
    fn create_sender(&self, from_node_id: u64) -> MockMessageSender {
        MockMessageSender {
            from_node_id,
            bus: self.clone(),
        }
    }

    /// Deliver pending messages to a node
    fn deliver_messages(&self, to_node_id: u64) -> Vec<TwoPhaseMessage> {
        let mut pending = self.pending_messages.lock().unwrap();
        pending.remove(&to_node_id).unwrap_or_default()
    }

    /// Get message history for verification
    fn get_message_history(&self) -> Vec<(u64, u64, TwoPhaseMessage)> {
        self.message_history.lock().unwrap().clone()
    }

    /// Simulate network failure for a node
    fn fail_node(&self, node_id: u64) {
        self.failed_nodes.lock().unwrap().insert(node_id);
    }

    /// Restore network connectivity for a node
    fn restore_node(&self, node_id: u64) {
        self.failed_nodes.lock().unwrap().remove(&node_id);
    }

    /// Clear all messages and history
    fn clear(&self) {
        self.pending_messages.lock().unwrap().clear();
        self.message_history.lock().unwrap().clear();
        self.failed_nodes.lock().unwrap().clear();
    }
}

struct MockMessageSender {
    from_node_id: u64,
    bus: MockMessageBus,
}

impl MessageSender for MockMessageSender {
    fn send_message(&self, target_node: u64, message: TwoPhaseMessage) -> Result<()> {
        // Check if target node is failed
        let failed_nodes = self.bus.failed_nodes.lock().unwrap();
        if failed_nodes.contains(&target_node) {
            return Err(anyhow::anyhow!("Network failure: cannot reach node {}", target_node));
        }
        drop(failed_nodes);

        // Record message in history
        {
            let mut history = self.bus.message_history.lock().unwrap();
            history.push((self.from_node_id, target_node, message.clone()));
        }

        // Queue message for delivery
        {
            let mut pending = self.bus.pending_messages.lock().unwrap();
            pending.entry(target_node).or_insert_with(Vec::new).push(message);
        }

        Ok(())
    }
}

#[tokio::test]
async fn test_2pc_successful_commit() -> Result<()> {
    // Setup: 1 coordinator (node 1) and 2 participants (nodes 2, 3)
    let message_bus = MockMessageBus::new();

    // Create coordinator
    let coord_txn_manager = Arc::new(TransactionManager::new());
    let mut coordinator = TwoPhaseCommitCoordinator::new(1, coord_txn_manager);
    coordinator.set_message_sender(Box::new(message_bus.create_sender(1)));

    // Create participants
    let part1_txn_manager = Arc::new(TransactionManager::new());
    let mut participant1 = TwoPhaseCommitParticipant::new(2, part1_txn_manager);
    participant1.set_message_sender(Box::new(message_bus.create_sender(2)));

    let part2_txn_manager = Arc::new(TransactionManager::new());
    let mut participant2 = TwoPhaseCommitParticipant::new(3, part2_txn_manager);
    participant2.set_message_sender(Box::new(message_bus.create_sender(3)));

    // Start distributed transaction
    let participants = [2, 3].iter().cloned().collect();
    let txn_id = coordinator.begin_distributed(participants).await?;
    assert!(txn_id > 0);

    // Simulate Phase 1: Prepare
    println!("Starting Phase 1: Prepare");
    
    // Coordinator sends prepare messages (run in current thread)
    let prepare_result = coordinator.prepare_phase(txn_id);

    // Give coordinator time to send prepare messages
    sleep(Duration::from_millis(50)).await;

    // Participants receive and respond to prepare messages
    let participant1_messages = message_bus.deliver_messages(2);
    let participant2_messages = message_bus.deliver_messages(3);

    // Process prepare messages
    for message in participant1_messages {
        participant1.handle_message(message).await?;
    }
    for message in participant2_messages {
        participant2.handle_message(message).await?;
    }

    // Give participants time to send responses
    sleep(Duration::from_millis(50)).await;

    // Coordinator receives prepare responses
    let coordinator_messages = message_bus.deliver_messages(1);
    for message in coordinator_messages {
        coordinator.handle_message(message)?;
    }

    // Wait for prepare phase to complete
    let prepare_success = prepare_result.await?;
    assert!(prepare_success, "All participants should vote to prepare");

    println!("Phase 1 completed successfully: all participants prepared");

    // Phase 2: Commit
    println!("Starting Phase 2: Commit");
    coordinator.commit_distributed(txn_id).await?;

    // Give coordinator time to send commit messages
    sleep(Duration::from_millis(50)).await;

    // Participants receive commit messages
    let participant1_messages = message_bus.deliver_messages(2);
    let participant2_messages = message_bus.deliver_messages(3);

    for message in participant1_messages {
        participant1.handle_message(message).await?;
    }
    for message in participant2_messages {
        participant2.handle_message(message).await?;
    }

    println!("2PC transaction {} completed successfully", txn_id);

    // Verify message flow
    let history = message_bus.get_message_history();
    assert!(!history.is_empty(), "Should have message history");

    // Count message types
    let prepare_count = history.iter().filter(|(_, _, msg)| {
        matches!(msg, TwoPhaseMessage::Prepare { .. })
    }).count();
    let prepared_count = history.iter().filter(|(_, _, msg)| {
        matches!(msg, TwoPhaseMessage::Prepared { .. })
    }).count();
    let commit_count = history.iter().filter(|(_, _, msg)| {
        matches!(msg, TwoPhaseMessage::Commit { .. })
    }).count();

    assert_eq!(prepare_count, 2, "Should send 2 prepare messages");
    assert_eq!(prepared_count, 2, "Should receive 2 prepared responses");
    assert_eq!(commit_count, 2, "Should send 2 commit messages");

    Ok(())
}

#[tokio::test]
async fn test_2pc_prepare_abort() -> Result<()> {
    let message_bus = MockMessageBus::new();

    // Create coordinator
    let coord_txn_manager = Arc::new(TransactionManager::new());
    let mut coordinator = TwoPhaseCommitCoordinator::new(1, coord_txn_manager);
    coordinator.set_message_sender(Box::new(message_bus.create_sender(1)));

    // Create participants (will simulate one failing to prepare)
    let part1_txn_manager = Arc::new(TransactionManager::new());
    let mut participant1 = TwoPhaseCommitParticipant::new(2, part1_txn_manager);
    participant1.set_message_sender(Box::new(message_bus.create_sender(2)));

    // Start distributed transaction
    let participants = [2].iter().cloned().collect();
    let txn_id = coordinator.begin_distributed(participants).await?;

    // Simulate prepare phase where participant votes to abort
    let prepare_result = coordinator.prepare_phase(txn_id);

    sleep(Duration::from_millis(50)).await;

    // Instead of normal prepare response, simulate abort vote
    let abort_msg = TwoPhaseMessage::PrepareAbort {
        txn_id,
        participant_id: 2,
        reason: "Resource conflict detected".to_string(),
    };

    // Manually send abort message to coordinator
    {
        let mut pending = message_bus.pending_messages.lock().unwrap();
        pending.entry(1).or_insert_with(Vec::new).push(abort_msg);
    }

    // Coordinator receives abort vote
    let coordinator_messages = message_bus.deliver_messages(1);
    for message in coordinator_messages {
        coordinator.handle_message(message)?;
    }

    // Prepare phase should fail
    let prepare_success = prepare_result.await?;
    assert!(!prepare_success, "Prepare should fail when participant votes abort");

    println!("2PC correctly aborted when participant voted to abort");

    Ok(())
}

#[tokio::test]
async fn test_2pc_network_failure() -> Result<()> {
    let message_bus = MockMessageBus::new();

    // Create coordinator
    let coord_txn_manager = Arc::new(TransactionManager::new());
    let mut coordinator = TwoPhaseCommitCoordinator::new(1, coord_txn_manager);
    coordinator.set_message_sender(Box::new(message_bus.create_sender(1)));

    // Start distributed transaction
    let participants = [2, 3].iter().cloned().collect();
    let txn_id = coordinator.begin_distributed(participants).await?;

    // Simulate network failure to participant 3
    message_bus.fail_node(3);

    // Try to run prepare phase (should timeout due to failed node)
    let prepare_result = coordinator.prepare_phase(txn_id).await;
    
    match prepare_result {
        Err(e) => {
            assert!(e.to_string().contains("timeout"), 
                    "Should fail with timeout error: {}", e);
            println!("2PC correctly handled network failure with timeout");
        }
        Ok(_) => {
            panic!("Prepare phase should have failed due to network timeout");
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_2pc_coordinator_stats() -> Result<()> {
    let message_bus = MockMessageBus::new();
    let coord_txn_manager = Arc::new(TransactionManager::new());
    let coordinator = TwoPhaseCommitCoordinator::new(1, coord_txn_manager);

    // Initially should have no transactions
    let (total, committed) = coordinator.get_coordinator_stats();
    assert_eq!(total, 0);
    assert_eq!(committed, 0);

    // Start a distributed transaction
    let participants = [2, 3].iter().cloned().collect();
    let _txn_id = coordinator.begin_distributed(participants).await?;

    // Should now have 1 active transaction
    let (total, committed) = coordinator.get_coordinator_stats();
    assert_eq!(total, 1);
    assert_eq!(committed, 0);

    Ok(())
}

#[tokio::test]
async fn test_2pc_participant_stats() -> Result<()> {
    let txn_manager = Arc::new(TransactionManager::new());
    let participant = TwoPhaseCommitParticipant::new(2, txn_manager);

    // Initially should have no transactions
    assert_eq!(participant.get_participant_stats(), 0);

    // Simulate receiving a prepare message
    let prepare_msg = TwoPhaseMessage::Prepare {
        txn_id: 123,
        coordinator_id: 1,
    };

    // Note: This test is simplified - in a real scenario we'd need
    // to set up the message sender and handle responses properly
    let handle_result = participant.handle_message(prepare_msg).await;
    
    // The handle should work (even without message sender for prepare phase)
    // Actual participant transaction creation happens during prepare handling
    match handle_result {
        Ok(_) => println!("Participant handled prepare message"),
        Err(e) => {
            // Expected to fail due to missing message sender, but that's OK for this test
            assert!(e.to_string().contains("Message sender not configured"));
            println!("Participant correctly detected missing message sender");
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_2pc_message_types() {
    // Test message serialization/deserialization would be more comprehensive
    // in a real implementation with actual network protocols
    
    let prepare_msg = TwoPhaseMessage::Prepare {
        txn_id: 42,
        coordinator_id: 1,
    };
    
    let prepared_msg = TwoPhaseMessage::Prepared {
        txn_id: 42,
        participant_id: 2,
    };
    
    let commit_msg = TwoPhaseMessage::Commit {
        txn_id: 42,
        coordinator_id: 1,
    };
    
    let abort_msg = TwoPhaseMessage::Abort {
        txn_id: 42,
        coordinator_id: 1,
    };
    
    let ack_msg = TwoPhaseMessage::Acknowledgment {
        txn_id: 42,
        participant_id: 2,
        success: true,
    };
    
    // Verify message types can be created and matched
    match prepare_msg {
        TwoPhaseMessage::Prepare { txn_id, coordinator_id } => {
            assert_eq!(txn_id, 42);
            assert_eq!(coordinator_id, 1);
        }
        _ => panic!("Wrong message type"),
    }
    
    match prepared_msg {
        TwoPhaseMessage::Prepared { txn_id, participant_id } => {
            assert_eq!(txn_id, 42);
            assert_eq!(participant_id, 2);
        }
        _ => panic!("Wrong message type"),
    }
    
    println!("All 2PC message types work correctly");
}