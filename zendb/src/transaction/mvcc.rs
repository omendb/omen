use anyhow::{Result, Context};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use std::collections::BTreeMap;

/// Hybrid Logical Clock for distributed timestamp ordering
/// Combines physical time (wall clock) and logical counter
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HLCTimestamp {
    /// Physical time in microseconds since Unix epoch
    pub physical: u64,
    /// Logical counter for events at same physical time
    pub logical: u32,
}

impl HLCTimestamp {
    pub fn new(physical: u64, logical: u32) -> Self {
        Self { physical, logical }
    }
    
    /// Get current wall clock time in microseconds
    fn now_micros() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64
    }
    
    /// Create a zero timestamp (beginning of time)
    pub fn zero() -> Self {
        Self { physical: 0, logical: 0 }
    }
    
    /// Create a timestamp for "now"
    pub fn now() -> Self {
        Self {
            physical: Self::now_micros(),
            logical: 0,
        }
    }
    
    /// Convert to bytes for storage
    pub fn to_bytes(&self) -> [u8; 12] {
        let mut bytes = [0u8; 12];
        bytes[0..8].copy_from_slice(&self.physical.to_be_bytes());
        bytes[8..12].copy_from_slice(&self.logical.to_be_bytes());
        bytes
    }
    
    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 12 {
            anyhow::bail!("Invalid timestamp bytes");
        }
        Ok(Self {
            physical: u64::from_be_bytes(bytes[0..8].try_into()?),
            logical: u32::from_be_bytes(bytes[8..12].try_into()?),
        })
    }
}

/// Hybrid Logical Clock implementation
/// Thread-safe clock that guarantees monotonic timestamps
pub struct HLC {
    /// Last issued timestamp
    last: Arc<RwLock<HLCTimestamp>>,
    /// Node ID for distributed systems (unused for now)
    _node_id: u64,
}

impl HLC {
    pub fn new(node_id: u64) -> Self {
        Self {
            last: Arc::new(RwLock::new(HLCTimestamp::zero())),
            _node_id: node_id,
        }
    }
    
    /// Get next timestamp, guaranteed to be monotonically increasing
    pub fn now(&self) -> HLCTimestamp {
        let mut last = self.last.write();
        let physical = HLCTimestamp::now_micros();
        
        if physical > last.physical {
            // Time has advanced, reset logical counter
            *last = HLCTimestamp::new(physical, 0);
        } else {
            // Same physical time, increment logical counter
            last.logical += 1;
            
            // Check for logical overflow (very unlikely)
            if last.logical == u32::MAX {
                // Wait 1 microsecond to advance physical time
                std::thread::sleep(std::time::Duration::from_micros(1));
                let new_physical = HLCTimestamp::now_micros();
                *last = HLCTimestamp::new(new_physical, 0);
            }
        }
        
        *last
    }
    
    /// Update clock with received timestamp (for distributed sync)
    pub fn update(&self, received: HLCTimestamp) -> HLCTimestamp {
        let mut last = self.last.write();
        let physical = HLCTimestamp::now_micros();
        
        // Maximum of local physical, received physical, and last physical
        let max_physical = physical.max(received.physical).max(last.physical);
        
        let new_timestamp = if max_physical == physical && max_physical == received.physical {
            // All three are equal, use max logical + 1
            HLCTimestamp::new(max_physical, received.logical.max(last.logical) + 1)
        } else if max_physical == physical {
            // Local physical is ahead
            HLCTimestamp::new(max_physical, 0)
        } else if max_physical == received.physical {
            // Received is ahead
            HLCTimestamp::new(max_physical, received.logical + 1)
        } else {
            // Last is ahead
            HLCTimestamp::new(max_physical, last.logical + 1)
        };
        
        *last = new_timestamp;
        new_timestamp
    }
}

/// Version information for a key-value pair
#[derive(Debug, Clone)]
pub struct Version {
    /// Timestamp when this version was created
    pub timestamp: HLCTimestamp,
    /// Transaction ID that created this version
    pub txn_id: u64,
    /// The value (None for deletions)
    pub value: Option<Vec<u8>>,
    /// Is this a committed version?
    pub committed: bool,
    /// When was this version committed (for snapshot isolation)
    pub commit_timestamp: Option<HLCTimestamp>,
}

/// Multi-Version Storage for a single key
/// Maintains multiple versions sorted by timestamp
#[derive(Debug, Clone)]
pub struct VersionedValue {
    /// All versions of this key, sorted by timestamp (newest first)
    versions: Vec<Version>,
}

impl VersionedValue {
    pub fn new() -> Self {
        Self {
            versions: Vec::new(),
        }
    }
    
    /// Add a new version
    pub fn add_version(&mut self, version: Version) {
        // Insert maintaining sorted order (newest first)
        let pos = self.versions
            .binary_search_by(|v| version.timestamp.cmp(&v.timestamp))
            .unwrap_or_else(|e| e);
        self.versions.insert(pos, version);
        
        // Limit version history (configurable)
        const MAX_VERSIONS: usize = 100;
        if self.versions.len() > MAX_VERSIONS {
            // Keep only recent versions
            self.versions.truncate(MAX_VERSIONS);
        }
    }
    
    /// Get value at specific timestamp
    pub fn get_at(&self, timestamp: HLCTimestamp) -> Option<&Version> {
        // Find first committed version where:
        // 1. Version was created before or at query timestamp
        // 2. Version was committed before query timestamp started
        for version in &self.versions {
            if version.committed && version.timestamp <= timestamp {
                if let Some(commit_ts) = version.commit_timestamp {
                    if commit_ts <= timestamp {
                        return Some(version);
                    }
                }
            }
        }
        None
    }
    
    /// Get latest committed value
    pub fn get_latest(&self) -> Option<&Version> {
        self.versions
            .iter()
            .find(|v| v.committed)
    }
    
    /// Mark a version as committed
    pub fn commit_version(&mut self, txn_id: u64, commit_timestamp: HLCTimestamp) {
        if let Some(version) = self.versions.iter_mut().find(|v| v.txn_id == txn_id) {
            version.committed = true;
            version.commit_timestamp = Some(commit_timestamp);
        }
    }
    
    /// Remove uncommitted version
    pub fn rollback_version(&mut self, txn_id: u64) {
        self.versions.retain(|v| v.txn_id != txn_id || v.committed);
    }
    
    /// Garbage collect old versions
    pub fn gc_versions(&mut self, keep_after: HLCTimestamp) {
        // Keep at least one version even if it's old
        if self.versions.len() <= 1 {
            return;
        }
        
        // Find the cutoff point
        let cutoff = self.versions
            .iter()
            .position(|v| v.timestamp < keep_after)
            .unwrap_or(self.versions.len());
        
        // Keep at least one old version for history
        if cutoff > 1 {
            self.versions.truncate(cutoff);
        }
    }
}

/// MVCC Storage Manager
/// Manages multi-version storage for all keys
pub struct MVCCStorage {
    /// Map from key to versioned values
    data: Arc<RwLock<BTreeMap<Vec<u8>, VersionedValue>>>,
    /// Hybrid logical clock
    clock: Arc<HLC>,
    /// Next transaction ID
    next_txn_id: AtomicU64,
    /// Active transactions and their start timestamps
    active_txns: Arc<RwLock<BTreeMap<u64, HLCTimestamp>>>,
}

impl MVCCStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(BTreeMap::new())),
            clock: Arc::new(HLC::new(1)), // Node ID 1 for now
            next_txn_id: AtomicU64::new(1),
            active_txns: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }
    
    /// Begin a new transaction
    pub fn begin_transaction(&self) -> (u64, HLCTimestamp) {
        let txn_id = self.next_txn_id.fetch_add(1, Ordering::SeqCst);
        let timestamp = self.clock.now();
        
        self.active_txns.write().insert(txn_id, timestamp);
        
        (txn_id, timestamp)
    }
    
    /// Get value for a key at transaction's snapshot
    pub fn get(&self, key: &[u8], txn_id: u64) -> Result<Option<Vec<u8>>> {
        let active_txns = self.active_txns.read();
        let snapshot_ts = *active_txns
            .get(&txn_id)
            .context("Transaction not found")?;
        
        let data = self.data.read();
        
        if let Some(versioned) = data.get(key) {
            if let Some(version) = versioned.get_at(snapshot_ts) {
                return Ok(version.value.clone());
            }
        }
        
        Ok(None)
    }
    
    /// Get value at specific timestamp (for time-travel queries)
    pub fn get_at_timestamp(&self, key: &[u8], timestamp: HLCTimestamp) -> Option<Vec<u8>> {
        let data = self.data.read();
        
        if let Some(versioned) = data.get(key) {
            if let Some(version) = versioned.get_at(timestamp) {
                return version.value.clone();
            }
        }
        
        None
    }
    
    /// Put a new value (creates new version)
    pub fn put(&self, key: Vec<u8>, value: Vec<u8>, txn_id: u64) -> Result<()> {
        let timestamp = self.clock.now();
        
        let version = Version {
            timestamp,
            txn_id,
            value: Some(value),
            committed: false, // Not committed yet
            commit_timestamp: None,
        };
        
        let mut data = self.data.write();
        data.entry(key)
            .or_insert_with(VersionedValue::new)
            .add_version(version);
        
        Ok(())
    }
    
    /// Delete a key (creates tombstone version)
    pub fn delete(&self, key: Vec<u8>, txn_id: u64) -> Result<()> {
        let timestamp = self.clock.now();
        
        let version = Version {
            timestamp,
            txn_id,
            value: None, // Tombstone
            committed: false,
            commit_timestamp: None,
        };
        
        let mut data = self.data.write();
        data.entry(key)
            .or_insert_with(VersionedValue::new)
            .add_version(version);
        
        Ok(())
    }
    
    /// Commit a transaction
    pub fn commit(&self, txn_id: u64) -> Result<()> {
        // Get commit timestamp
        let commit_timestamp = self.clock.now();
        
        // Remove from active transactions
        self.active_txns.write().remove(&txn_id);
        
        // Mark all versions from this transaction as committed
        let mut data = self.data.write();
        for versioned in data.values_mut() {
            versioned.commit_version(txn_id, commit_timestamp);
        }
        
        Ok(())
    }
    
    /// Rollback a transaction
    pub fn rollback(&self, txn_id: u64) -> Result<()> {
        // Remove from active transactions
        self.active_txns.write().remove(&txn_id);
        
        // Remove all uncommitted versions from this transaction
        let mut data = self.data.write();
        for versioned in data.values_mut() {
            versioned.rollback_version(txn_id);
        }
        
        Ok(())
    }
    
    /// Range scan with MVCC
    pub fn range_scan(
        &self,
        start: &[u8],
        end: &[u8],
        txn_id: u64,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let active_txns = self.active_txns.read();
        let snapshot_ts = *active_txns
            .get(&txn_id)
            .context("Transaction not found")?;
        
        let data = self.data.read();
        let mut results = Vec::new();
        
        for (key, versioned) in data.range(start.to_vec()..=end.to_vec()) {
            if let Some(version) = versioned.get_at(snapshot_ts) {
                if let Some(value) = &version.value {
                    results.push((key.clone(), value.clone()));
                }
            }
        }
        
        Ok(results)
    }
    
    /// Garbage collect old versions
    pub fn garbage_collect(&self, keep_duration_micros: u64) {
        let cutoff = HLCTimestamp::new(
            HLCTimestamp::now_micros().saturating_sub(keep_duration_micros),
            0,
        );
        
        let mut data = self.data.write();
        
        // GC each key's versions
        for versioned in data.values_mut() {
            versioned.gc_versions(cutoff);
        }
        
        // Remove keys with no versions
        data.retain(|_, v| !v.versions.is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hlc_monotonic() {
        let clock = HLC::new(1);
        
        let mut timestamps = Vec::new();
        for _ in 0..100 {
            timestamps.push(clock.now());
        }
        
        // Verify monotonic increase
        for i in 1..timestamps.len() {
            assert!(timestamps[i] > timestamps[i-1]);
        }
    }
    
    #[test]
    fn test_mvcc_basic() {
        let storage = MVCCStorage::new();
        
        // Start transaction 1
        let (txn1, _ts1) = storage.begin_transaction();
        
        // Write some data
        storage.put(b"key1".to_vec(), b"value1".to_vec(), txn1).unwrap();
        
        // Start transaction 2 (should not see uncommitted data)
        let (txn2, _ts2) = storage.begin_transaction();
        assert_eq!(storage.get(b"key1", txn2).unwrap(), None);
        
        // Commit transaction 1
        storage.commit(txn1).unwrap();
        
        // Transaction 2 still shouldn't see it (snapshot isolation)
        assert_eq!(storage.get(b"key1", txn2).unwrap(), None);
        
        // Start transaction 3 after commit (should see committed data)
        let (txn3, _ts3) = storage.begin_transaction();
        assert_eq!(storage.get(b"key1", txn3).unwrap(), Some(b"value1".to_vec()));
    }
}