//! Multi-writer concurrency support for PageManager
//!
//! Implements fine-grained page-level locking to allow multiple concurrent writers
//! while maintaining ACID properties and preventing conflicts.

use anyhow::Result;
use parking_lot::RwLock;
use tokio::sync::Mutex;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use std::time::{Duration, Instant};
use crate::storage::{PageId, Page};

/// Lock modes for page access
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockMode {
    /// Shared lock for reading
    Shared,
    /// Exclusive lock for writing
    Exclusive,
}

/// Lock request information
#[derive(Debug, Clone)]
struct LockRequest {
    txn_id: u64,
    page_id: PageId,
    mode: LockMode,
    timestamp: Instant,
}

/// Information about a held lock
#[derive(Debug, Clone)]
struct HeldLock {
    txn_id: u64,
    mode: LockMode,
    acquired_at: Instant,
}

/// Page-level lock information
#[derive(Debug)]
struct PageLock {
    /// Current holders of this lock (multiple for shared, single for exclusive)
    holders: Vec<HeldLock>,
    /// Queue of waiting lock requests
    waiters: VecDeque<LockRequest>,
}

impl PageLock {
    fn new() -> Self {
        Self {
            holders: Vec::new(),
            waiters: VecDeque::new(),
        }
    }
    
    /// Check if a lock request is compatible with current holders
    fn is_compatible(&self, mode: LockMode) -> bool {
        if self.holders.is_empty() {
            return true;
        }
        
        match mode {
            LockMode::Shared => {
                // Shared locks are compatible with other shared locks
                self.holders.iter().all(|h| h.mode == LockMode::Shared)
            }
            LockMode::Exclusive => {
                // Exclusive locks are not compatible with any other locks
                false
            }
        }
    }
    
    /// Grant a lock to a transaction
    fn grant(&mut self, txn_id: u64, mode: LockMode) {
        self.holders.push(HeldLock {
            txn_id,
            mode,
            acquired_at: Instant::now(),
        });
    }
    
    /// Release a lock held by a transaction
    fn release(&mut self, txn_id: u64) -> bool {
        let before = self.holders.len();
        self.holders.retain(|h| h.txn_id != txn_id);
        self.holders.len() < before
    }
}

/// Multi-writer lock manager for page-level concurrency control
pub struct MultiWriterLockManager {
    /// Locks for each page
    page_locks: Arc<RwLock<HashMap<PageId, PageLock>>>,
    /// Locks held by each transaction (for cleanup on abort/commit)
    txn_locks: Arc<RwLock<HashMap<u64, Vec<PageId>>>>,
    /// Deadlock detector
    deadlock_detector: Arc<DeadlockDetector>,
    /// Lock timeout duration
    lock_timeout: Duration,
    /// Statistics
    stats: Arc<LockManagerStats>,
}

/// Statistics for lock manager performance monitoring
#[derive(Debug, Default)]
pub struct LockManagerStats {
    pub locks_acquired: AtomicU64,
    pub locks_released: AtomicU64,
    pub lock_conflicts: AtomicU64,
    pub deadlocks_detected: AtomicU64,
    pub lock_timeouts: AtomicU64,
}

/// Deadlock detection using wait-for graph
struct DeadlockDetector {
    /// Wait-for graph: txn A -> txn B means A is waiting for B
    wait_for_graph: Mutex<HashMap<u64, Vec<u64>>>,
}

impl DeadlockDetector {
    fn new() -> Self {
        Self {
            wait_for_graph: Mutex::new(HashMap::new()),
        }
    }
    
    /// Add a wait-for edge (waiter is waiting for holder)
    async fn add_edge(&self, waiter: u64, holder: u64) {
        let mut graph = self.wait_for_graph.lock().await;
        graph.entry(waiter).or_insert_with(Vec::new).push(holder);
    }
    
    /// Remove all edges from a transaction (when it completes)
    async fn remove_edges(&self, txn_id: u64) {
        let mut graph = self.wait_for_graph.lock().await;
        graph.remove(&txn_id);
        // Also remove from other transactions' wait lists
        for wait_list in graph.values_mut() {
            wait_list.retain(|&id| id != txn_id);
        }
    }
    
    /// Detect if adding this edge would create a cycle (deadlock)
    async fn would_deadlock(&self, waiter: u64, holder: u64) -> bool {
        let graph = self.wait_for_graph.lock().await;
        
        // Use DFS to check if holder can reach waiter (would create cycle)
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![holder];
        
        while let Some(current) = stack.pop() {
            if current == waiter {
                return true; // Found cycle
            }
            
            if visited.insert(current) {
                if let Some(neighbors) = graph.get(&current) {
                    stack.extend(neighbors.iter().copied());
                }
            }
        }
        
        false
    }
}

impl MultiWriterLockManager {
    pub fn new() -> Self {
        Self {
            page_locks: Arc::new(RwLock::new(HashMap::new())),
            txn_locks: Arc::new(RwLock::new(HashMap::new())),
            deadlock_detector: Arc::new(DeadlockDetector::new()),
            lock_timeout: Duration::from_secs(5),
            stats: Arc::new(LockManagerStats::default()),
        }
    }
    
    /// Acquire a lock on a page for a transaction
    pub async fn acquire_lock(
        &self,
        txn_id: u64,
        page_id: PageId,
        mode: LockMode,
    ) -> Result<()> {
        let start_time = Instant::now();
        
        loop {
            // Check for timeout
            if start_time.elapsed() > self.lock_timeout {
                self.stats.lock_timeouts.fetch_add(1, Ordering::Relaxed);
                anyhow::bail!("Lock acquisition timeout for page {}", page_id);
            }
            
            // Try to acquire the lock
            let (can_grant, holders_to_check, need_to_wait) = {
                let mut locks = self.page_locks.write();
                let page_lock = locks.entry(page_id).or_insert_with(PageLock::new);
                
                // Check if we already hold this lock
                if page_lock.holders.iter().any(|h| h.txn_id == txn_id) {
                    // Already hold the lock, nothing to do
                    return Ok(());
                }
                
                // Remove from waiters if present (for retry logic)
                page_lock.waiters.retain(|r| r.txn_id != txn_id);
                
                if page_lock.is_compatible(mode) {
                    // We can grant - collect holders to check for deadlock
                    let holders: Vec<u64> = page_lock.holders.iter().map(|h| h.txn_id).collect();
                    (true, holders, false)
                } else {
                    // Need to wait - add to queue
                    page_lock.waiters.push_back(LockRequest {
                        txn_id,
                        page_id,
                        mode,
                        timestamp: Instant::now(),
                    });
                    
                    let holders: Vec<u64> = page_lock.holders.iter().map(|h| h.txn_id).collect();
                    self.stats.lock_conflicts.fetch_add(1, Ordering::Relaxed);
                    (false, holders, true)
                }
            };
            
            // Check for deadlock outside of lock
            if can_grant {
                let mut would_deadlock = false;
                for holder_id in holders_to_check {
                    if self.deadlock_detector.would_deadlock(txn_id, holder_id).await {
                        would_deadlock = true;
                        break;
                    }
                }
                
                if would_deadlock {
                    self.stats.deadlocks_detected.fetch_add(1, Ordering::Relaxed);
                    anyhow::bail!("Deadlock detected for transaction {}", txn_id);
                }
                
                // Grant the lock
                {
                    let mut locks = self.page_locks.write();
                    if let Some(page_lock) = locks.get_mut(&page_id) {
                        page_lock.grant(txn_id, mode);
                    }
                }
                
                // Track lock for this transaction
                {
                    let mut txn_locks = self.txn_locks.write();
                    txn_locks.entry(txn_id).or_insert_with(Vec::new).push(page_id);
                }
                
                self.stats.locks_acquired.fetch_add(1, Ordering::Relaxed);
                return Ok(());
            } else if need_to_wait {
                // Add wait-for edges for deadlock detection
                for holder_id in holders_to_check {
                    self.deadlock_detector.add_edge(txn_id, holder_id).await;
                }
            }
            
            // Wait before retrying
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
    
    /// Release a lock on a page
    pub fn release_lock(&self, txn_id: u64, page_id: PageId) -> Result<()> {
        let mut locks = self.page_locks.write();
        
        if let Some(page_lock) = locks.get_mut(&page_id) {
            if page_lock.release(txn_id) {
                self.stats.locks_released.fetch_add(1, Ordering::Relaxed);
                
                // Clean up empty lock entries
                if page_lock.holders.is_empty() && page_lock.waiters.is_empty() {
                    locks.remove(&page_id);
                }
            }
        }
        
        // Remove from transaction's lock list
        let mut txn_locks = self.txn_locks.write();
        if let Some(pages) = txn_locks.get_mut(&txn_id) {
            pages.retain(|&p| p != page_id);
        }
        
        Ok(())
    }
    
    /// Release all locks held by a transaction (on commit/abort)
    pub async fn release_all_locks(&self, txn_id: u64) -> Result<()> {
        let pages = {
            let mut txn_locks = self.txn_locks.write();
            txn_locks.remove(&txn_id).unwrap_or_default()
        };
        
        for page_id in pages {
            self.release_lock(txn_id, page_id)?;
        }
        
        // Clean up deadlock detection edges
        self.deadlock_detector.remove_edges(txn_id).await;
        
        Ok(())
    }
    
    /// Get lock manager statistics
    pub fn get_stats(&self) -> LockManagerStats {
        LockManagerStats {
            locks_acquired: AtomicU64::new(self.stats.locks_acquired.load(Ordering::Relaxed)),
            locks_released: AtomicU64::new(self.stats.locks_released.load(Ordering::Relaxed)),
            lock_conflicts: AtomicU64::new(self.stats.lock_conflicts.load(Ordering::Relaxed)),
            deadlocks_detected: AtomicU64::new(self.stats.deadlocks_detected.load(Ordering::Relaxed)),
            lock_timeouts: AtomicU64::new(self.stats.lock_timeouts.load(Ordering::Relaxed)),
        }
    }
    
    /// Upgrade a shared lock to exclusive (for read-modify-write operations)
    pub async fn upgrade_lock(
        &self,
        txn_id: u64,
        page_id: PageId,
    ) -> Result<()> {
        // First release the shared lock
        self.release_lock(txn_id, page_id)?;
        
        // Then acquire exclusive lock
        self.acquire_lock(txn_id, page_id, LockMode::Exclusive).await
    }
}

/// Multi-writer page manager wrapper
/// Provides concurrent write access to pages with fine-grained locking
pub struct MultiWriterPageManager {
    /// Lock manager for concurrency control
    lock_manager: Arc<MultiWriterLockManager>,
    /// Next transaction ID
    next_txn_id: AtomicU64,
}

impl MultiWriterPageManager {
    pub fn new(lock_manager: Arc<MultiWriterLockManager>) -> Self {
        Self {
            lock_manager,
            next_txn_id: AtomicU64::new(1),
        }
    }
    
    /// Begin a new transaction
    pub fn begin_transaction(&self) -> u64 {
        self.next_txn_id.fetch_add(1, Ordering::SeqCst)
    }
    
    /// Read a page with shared lock
    pub async fn read_page(&self, txn_id: u64, page_id: PageId) -> Result<Page> {
        // Acquire shared lock
        self.lock_manager.acquire_lock(txn_id, page_id, LockMode::Shared).await?;
        
        // TODO: Actually read the page from underlying storage
        // For now, return a placeholder
        Ok(Page::new(page_id))
    }
    
    /// Write a page with exclusive lock
    pub async fn write_page(&self, txn_id: u64, page: &Page) -> Result<()> {
        // Acquire exclusive lock
        self.lock_manager.acquire_lock(txn_id, page.id, LockMode::Exclusive).await?;
        
        // TODO: Actually write the page to underlying storage
        // For now, just return success
        Ok(())
    }
    
    /// Commit a transaction (release all locks)
    pub async fn commit_transaction(&self, txn_id: u64) -> Result<()> {
        self.lock_manager.release_all_locks(txn_id).await
    }
    
    /// Abort a transaction (release all locks)
    pub async fn abort_transaction(&self, txn_id: u64) -> Result<()> {
        self.lock_manager.release_all_locks(txn_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_shared_locks_compatible() {
        let manager = MultiWriterLockManager::new();
        
        // Two transactions should be able to acquire shared locks
        manager.acquire_lock(1, 100, LockMode::Shared).await.unwrap();
        manager.acquire_lock(2, 100, LockMode::Shared).await.unwrap();
        
        // Clean up
        manager.release_lock(1, 100).unwrap();
        manager.release_lock(2, 100).unwrap();
    }
    
    #[tokio::test]
    async fn test_exclusive_lock_blocks() {
        let manager = Arc::new(MultiWriterLockManager::new());
        
        // First transaction acquires exclusive lock
        manager.acquire_lock(1, 100, LockMode::Exclusive).await.unwrap();
        
        // Second transaction should timeout trying to acquire
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            // This should timeout or wait
            let result = tokio::time::timeout(
                Duration::from_millis(100),
                manager_clone.acquire_lock(2, 100, LockMode::Exclusive)
            ).await;
            
            assert!(result.is_err()); // Should timeout
        });
        
        handle.await.unwrap();
        
        // Clean up
        manager.release_lock(1, 100).unwrap();
    }
    
    #[tokio::test]
    async fn test_deadlock_detection() {
        let detector = DeadlockDetector::new();
        
        // Create a potential deadlock scenario: 1 -> 2 -> 3 -> 1
        detector.add_edge(1, 2).await;
        detector.add_edge(2, 3).await;
        
        // This would create a cycle
        assert!(detector.would_deadlock(3, 1).await);
        
        // This would not create a cycle
        assert!(!detector.would_deadlock(4, 1).await);
    }
    
    #[tokio::test]
    async fn test_lock_release_wakes_waiters() {
        let manager = Arc::new(MultiWriterLockManager::new());
        
        // First transaction acquires exclusive lock
        manager.acquire_lock(1, 100, LockMode::Exclusive).await.unwrap();
        
        // Start second transaction trying to acquire
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            // This will wait until lock is released
            manager_clone.acquire_lock(2, 100, LockMode::Exclusive).await
        });
        
        // Give some time for the second transaction to start waiting
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Release the first lock
        manager.release_lock(1, 100).unwrap();
        
        // Second transaction should now succeed
        let result = tokio::time::timeout(Duration::from_millis(100), handle).await;
        assert!(result.is_ok());
        
        // Clean up
        manager.release_lock(2, 100).unwrap();
    }
}