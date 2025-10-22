# OmenDB MVCC Design Document

**Date**: October 21, 2025
**Version**: 0.1.0
**Target**: Snapshot Isolation for Concurrent Transactions
**Implementation**: Phase 1 (Weeks 1-3)

---

## Executive Summary

**Goal**: Implement Multi-Version Concurrency Control (MVCC) with snapshot isolation to enable safe concurrent transactions in OmenDB.

**Approach**: Timestamp-based versioning inspired by ToyDB, TiKV, and PostgreSQL implementations.

**Key Features**:
- Snapshot isolation (not full serializability for 0.1.0)
- Lock-free reads
- Optimistic concurrency control
- First-committer-wins conflict resolution
- Garbage collection for old versions

**Timeline**: 2-3 weeks (Phase 1 of 0.1.0 roadmap)

---

## Current State vs Target State

### Current (Before MVCC)

**Problem**: Single-version storage, no isolation
```rust
// Current: Direct write to storage
storage.put(key, value)?;  // Overwrites immediately
txn.commit()?;             // No version control
```

**Issues**:
- ❌ Concurrent transactions unsafe
- ❌ Dirty reads possible
- ❌ Non-repeatable reads
- ❌ Lost updates
- ❌ Phantom reads

### Target (With MVCC)

**Solution**: Multi-version storage with snapshot isolation
```rust
// Target: Versioned writes
let txn_id = oracle.begin()?;              // Allocate timestamp
let snapshot_ts = oracle.snapshot_ts();     // Read snapshot
txn.put(key, value)?;                       // Buffered write
oracle.commit(txn_id)?;                     // Create version
```

**Benefits**:
- ✅ Concurrent transactions safe
- ✅ Snapshot isolation guarantees
- ✅ Lock-free reads
- ✅ Write conflict detection
- ✅ ACID compliance

---

## Architecture Overview

### Components

```
MVCC Architecture:
├── Transaction Oracle
│   ├── Timestamp allocation (monotonic)
│   ├── Active transaction tracking
│   └── Snapshot management
├── Version Storage
│   ├── RocksDB: (key, txn_id) → value
│   ├── ALEX index: key → latest_txn_id
│   └── Version chains (append-only)
├── Visibility Engine
│   ├── Snapshot read logic
│   ├── Version visibility rules
│   └── Garbage collection
└── Conflict Detector
    ├── Write-write conflicts
    ├── First-committer-wins
    └── Retry logic
```

### Data Flow

```
BEGIN:
  1. Oracle.begin() → allocate txn_id
  2. Create snapshot (active transaction list)
  3. Transaction sees: committed data < snapshot_ts

READ:
  1. ALEX lookup → find latest version
  2. Visibility check → is version visible?
  3. Return visible version (may traverse chain)

WRITE:
  1. Buffer in transaction context
  2. Track write set (for conflicts)
  3. No immediate storage update

COMMIT:
  1. Check write conflicts
  2. Allocate commit_ts
  3. Write versions to storage
  4. Update ALEX index
  5. Mark transaction committed
```

---

## Detailed Design

### 1. Transaction Oracle

**Purpose**: Centralized timestamp allocation and transaction lifecycle management

```rust
pub struct TransactionOracle {
    /// Monotonically increasing counter
    next_txn_id: AtomicU64,

    /// Active transactions (txn_id, start_ts)
    active_txns: RwLock<HashMap<u64, TransactionState>>,

    /// Recently committed (for visibility checks)
    recent_commits: RwLock<VecDeque<CommittedTransaction>>,

    /// Garbage collection watermark
    gc_watermark: AtomicU64,
}

impl TransactionOracle {
    /// Begin a new transaction
    pub fn begin(&self) -> Result<Transaction> {
        let txn_id = self.next_txn_id.fetch_add(1, Ordering::SeqCst);
        let snapshot_ts = txn_id;

        // Capture active transactions for snapshot
        let active_txns = self.active_txns.read().unwrap();
        let snapshot = active_txns.keys()
            .filter(|&&id| id < snapshot_ts)
            .copied()
            .collect::<Vec<u64>>();

        let state = TransactionState {
            txn_id,
            start_ts: snapshot_ts,
            snapshot: snapshot,
            write_set: HashSet::new(),
            read_set: HashSet::new(),
            status: TxnStatus::Active,
        };

        self.active_txns.write().unwrap().insert(txn_id, state.clone());

        Ok(Transaction::new(txn_id, snapshot_ts, snapshot, self.clone()))
    }

    /// Commit a transaction
    pub fn commit(&self, txn_id: u64, write_set: HashSet<Vec<u8>>) -> Result<u64> {
        // Check for write conflicts
        self.check_conflicts(txn_id, &write_set)?;

        // Allocate commit timestamp
        let commit_ts = self.next_txn_id.fetch_add(1, Ordering::SeqCst);

        // Mark transaction as committed
        let mut active = self.active_txns.write().unwrap();
        if let Some(state) = active.get_mut(&txn_id) {
            state.status = TxnStatus::Committed(commit_ts);
        }

        // Move to recent commits
        let mut recent = self.recent_commits.write().unwrap();
        recent.push_back(CommittedTransaction {
            txn_id,
            commit_ts,
            write_set,
        });

        // Limit recent commits buffer (e.g., 10000)
        while recent.len() > 10000 {
            recent.pop_front();
        }

        // Remove from active
        active.remove(&txn_id);

        Ok(commit_ts)
    }

    /// Abort a transaction
    pub fn abort(&self, txn_id: u64) {
        let mut active = self.active_txns.write().unwrap();
        active.remove(&txn_id);
    }

    /// Check for write-write conflicts
    fn check_conflicts(&self, txn_id: u64, write_set: &HashSet<Vec<u8>>) -> Result<()> {
        let active = self.active_txns.read().unwrap();
        let my_txn = active.get(&txn_id)
            .ok_or_else(|| anyhow!("Transaction not found"))?;

        // Check if any committed transaction after my snapshot wrote to same keys
        let recent = self.recent_commits.read().unwrap();
        for committed in recent.iter() {
            // Only check transactions committed after my snapshot
            if committed.commit_ts <= my_txn.start_ts {
                continue;
            }

            // Check for overlapping writes
            if !committed.write_set.is_disjoint(write_set) {
                return Err(anyhow!("Write conflict detected"));
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
struct TransactionState {
    txn_id: u64,
    start_ts: u64,
    snapshot: Vec<u64>,  // Active txn IDs at start
    write_set: HashSet<Vec<u8>>,
    read_set: HashSet<Vec<u8>>,
    status: TxnStatus,
}

#[derive(Clone)]
enum TxnStatus {
    Active,
    Committed(u64),  // commit_ts
    Aborted,
}

struct CommittedTransaction {
    txn_id: u64,
    commit_ts: u64,
    write_set: HashSet<Vec<u8>>,
}
```

### 2. Version Storage

**Purpose**: Store multiple versions of each key with transaction IDs

#### RocksDB Key Format

**Current**:
```
key → value
```

**With MVCC**:
```
(key, txn_id) → (value, begin_ts, end_ts)
```

**Implementation**:
```rust
pub struct VersionedKey {
    key: Vec<u8>,
    txn_id: u64,
}

impl VersionedKey {
    /// Encode as RocksDB key: key_bytes + txn_id (big-endian, inverted)
    pub fn encode(&self) -> Vec<u8> {
        let mut encoded = self.key.clone();
        // Invert txn_id so newer versions come first in iterator
        let inverted_txn_id = u64::MAX - self.txn_id;
        encoded.extend_from_slice(&inverted_txn_id.to_be_bytes());
        encoded
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 8 {
            return Err(anyhow!("Invalid versioned key"));
        }

        let key_len = bytes.len() - 8;
        let key = bytes[..key_len].to_vec();
        let inverted_txn_id = u64::from_be_bytes(bytes[key_len..].try_into()?);
        let txn_id = u64::MAX - inverted_txn_id;

        Ok(VersionedKey { key, txn_id })
    }
}

pub struct VersionedValue {
    value: Vec<u8>,
    begin_ts: u64,   // Transaction that created this version
    end_ts: Option<u64>,  // Transaction that deleted this (None = current)
}

impl VersionedValue {
    pub fn encode(&self) -> Vec<u8> {
        let mut encoded = Vec::new();
        // Format: value_len (4 bytes) | value | begin_ts (8 bytes) | end_ts_flag (1 byte) | end_ts (8 bytes if flag=1)
        encoded.extend_from_slice(&(self.value.len() as u32).to_be_bytes());
        encoded.extend_from_slice(&self.value);
        encoded.extend_from_slice(&self.begin_ts.to_be_bytes());

        if let Some(end_ts) = self.end_ts {
            encoded.push(1);
            encoded.extend_from_slice(&end_ts.to_be_bytes());
        } else {
            encoded.push(0);
        }

        encoded
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 13 {  // min: 4 + 0 + 8 + 1
            return Err(anyhow!("Invalid versioned value"));
        }

        let value_len = u32::from_be_bytes(bytes[0..4].try_into()?) as usize;
        let value = bytes[4..4+value_len].to_vec();
        let begin_ts = u64::from_be_bytes(bytes[4+value_len..4+value_len+8].try_into()?);
        let end_ts_flag = bytes[4+value_len+8];

        let end_ts = if end_ts_flag == 1 {
            Some(u64::from_be_bytes(bytes[4+value_len+9..4+value_len+17].try_into()?))
        } else {
            None
        };

        Ok(VersionedValue { value, begin_ts, end_ts })
    }
}
```

#### ALEX Index Integration

**Challenge**: ALEX stores `key → position`, but we need `key → [positions]` for versions

**Solution**: Store latest version position only, traverse RocksDB for older versions

```rust
impl AlexTree {
    /// Lookup returns position of LATEST version
    pub fn lookup(&self, key: i64) -> Option<usize> {
        // Existing ALEX logic returns latest version
        self.internal_lookup(key)
    }

    /// For MVCC, we update ALEX on each commit
    pub fn update_latest_version(&mut self, key: i64, txn_id: u64) -> Result<()> {
        // Update ALEX to point to new version
        // Position encoding includes txn_id for version chain traversal
        self.insert(key, txn_id as usize)?;
        Ok(())
    }
}
```

### 3. Visibility Engine

**Purpose**: Determine which version of a key is visible to a transaction

#### Visibility Rules (Snapshot Isolation)

```rust
impl Transaction {
    /// Check if a version is visible to this transaction
    pub fn is_visible(&self, version: &VersionedValue) -> bool {
        let begin_ts = version.begin_ts;

        // Rule 1: Version created by me
        if begin_ts == self.txn_id {
            return true;
        }

        // Rule 2: Version created after my snapshot
        if begin_ts > self.snapshot_ts {
            return false;
        }

        // Rule 3: Version created by transaction active at my snapshot
        if self.snapshot.contains(&begin_ts) {
            return false;
        }

        // Rule 4: Version was deleted
        if let Some(end_ts) = version.end_ts {
            // Deleted by me
            if end_ts == self.txn_id {
                return false;
            }

            // Deleted before my snapshot
            if end_ts <= self.snapshot_ts && !self.snapshot.contains(&end_ts) {
                return false;
            }
        }

        // Version is visible
        true
    }

    /// Read a key at snapshot
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        // Check write buffer first (read-your-own-writes)
        if let Some(value) = self.write_buffer.get(key) {
            return Ok(Some(value.clone()));
        }

        // Lookup in ALEX
        let key_i64 = i64::from_be_bytes(key[..8].try_into()?);
        let latest_txn_id = self.storage.alex.lookup(key_i64)
            .ok_or_else(|| anyhow!("Key not found"))?;

        // Start from latest version, traverse chain
        let mut current_txn_id = latest_txn_id as u64;
        loop {
            let versioned_key = VersionedKey { key: key.to_vec(), txn_id: current_txn_id };
            let encoded_key = versioned_key.encode();

            let value_bytes = self.storage.db.get(&encoded_key)?
                .ok_or_else(|| anyhow!("Version not found"))?;

            let version = VersionedValue::decode(&value_bytes)?;

            // Check visibility
            if self.is_visible(&version) {
                return Ok(Some(version.value));
            }

            // Move to previous version (txn_id - 1, scan backwards)
            // In reality, we'd scan RocksDB iterator backwards
            if current_txn_id == 0 {
                break;
            }
            current_txn_id -= 1;
        }

        Ok(None)
    }
}
```

#### Snapshot Read Optimization

**Optimization 1**: Read-only transactions skip conflict detection
```rust
pub enum TransactionMode {
    ReadWrite,
    ReadOnly,
}

// Read-only transactions don't need write conflict checks
if txn.mode == TransactionMode::ReadOnly {
    oracle.commit_readonly(txn_id)?;  // No conflict check
}
```

**Optimization 2**: Cache recent versions
```rust
pub struct VersionCache {
    cache: LruCache<(Vec<u8>, u64), Vec<u8>>,  // (key, txn_id) → value
}
```

### 4. Write Path with Conflict Detection

```rust
impl Transaction {
    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        // Buffer write
        self.write_buffer.insert(key.clone(), value);

        // Track write set (for conflict detection)
        self.write_set.insert(key);

        Ok(())
    }

    pub fn delete(&mut self, key: Vec<u8>) -> Result<()> {
        // Mark as deleted in buffer
        self.write_buffer.insert(key.clone(), TOMBSTONE.to_vec());
        self.write_set.insert(key);
        Ok(())
    }

    pub fn commit(self) -> Result<()> {
        // 1. Check conflicts
        let commit_ts = self.oracle.commit(self.txn_id, self.write_set.clone())?;

        // 2. Write versions to storage
        for (key, value) in self.write_buffer.iter() {
            let versioned_key = VersionedKey {
                key: key.clone(),
                txn_id: commit_ts,
            };

            let versioned_value = VersionedValue {
                value: value.clone(),
                begin_ts: commit_ts,
                end_ts: None,
            };

            self.storage.db.put(
                &versioned_key.encode(),
                &versioned_value.encode()
            )?;

            // 3. Update ALEX index
            let key_i64 = i64::from_be_bytes(key[..8].try_into()?);
            self.storage.alex.update_latest_version(key_i64, commit_ts)?;
        }

        Ok(())
    }
}
```

### 5. Garbage Collection

**Problem**: Old versions accumulate, waste space

**Solution**: Periodic GC based on watermark

```rust
pub struct GarbageCollector {
    oracle: Arc<TransactionOracle>,
    storage: Arc<RwLock<RocksStorage>>,
}

impl GarbageCollector {
    /// Calculate GC watermark: min(active transaction start_ts)
    pub fn calculate_watermark(&self) -> u64 {
        let active = self.oracle.active_txns.read().unwrap();

        if active.is_empty() {
            // No active transactions, can GC everything before latest commit
            return self.oracle.next_txn_id.load(Ordering::SeqCst);
        }

        // Find oldest active transaction
        active.values()
            .map(|state| state.start_ts)
            .min()
            .unwrap_or(0)
    }

    /// GC versions older than watermark
    pub fn collect(&self) -> Result<usize> {
        let watermark = self.calculate_watermark();
        let mut deleted = 0;

        // Iterate RocksDB, delete versions with begin_ts < watermark and end_ts < watermark
        let storage = self.storage.read().unwrap();
        let iter = storage.db.iterator(rocksdb::IteratorMode::Start);

        for (key_bytes, value_bytes) in iter {
            let versioned_key = VersionedKey::decode(&key_bytes)?;
            let version = VersionedValue::decode(&value_bytes)?;

            // Keep if:
            // - Version is the latest (end_ts = None)
            // - Version created after watermark
            // - Version ended after watermark

            let should_keep = version.end_ts.is_none()
                || version.begin_ts >= watermark
                || version.end_ts.map_or(false, |end| end >= watermark);

            if !should_keep {
                storage.db.delete(&key_bytes)?;
                deleted += 1;
            }
        }

        Ok(deleted)
    }

    /// Run GC periodically
    pub fn run_background(&self, interval: Duration) {
        loop {
            thread::sleep(interval);

            match self.collect() {
                Ok(deleted) => {
                    if deleted > 0 {
                        info!("GC collected {} old versions", deleted);
                    }
                }
                Err(e) => {
                    warn!("GC failed: {}", e);
                }
            }
        }
    }
}
```

---

## Integration with Existing Code

### Changes to `src/transaction.rs`

```rust
// Current (Before MVCC)
pub struct TransactionContext {
    pub transaction_buffer: HashMap<String, Vec<TableRow>>,
    pub in_transaction: bool,
}

// After MVCC
pub struct TransactionContext {
    pub txn_id: u64,
    pub snapshot_ts: u64,
    pub snapshot: Vec<u64>,  // Active txn IDs at start
    pub write_buffer: HashMap<Vec<u8>, Vec<u8>>,
    pub write_set: HashSet<Vec<u8>>,
    pub read_set: HashSet<Vec<u8>>,
    pub oracle: Arc<TransactionOracle>,
    pub mode: TransactionMode,
}

impl TransactionContext {
    pub fn begin(oracle: Arc<TransactionOracle>) -> Result<Self> {
        let txn = oracle.begin()?;
        Ok(txn)
    }

    pub fn commit(self) -> Result<()> {
        // Call oracle.commit, write versions
        self.oracle.commit(self.txn_id, self.write_set)?;
        // ... write to storage
        Ok(())
    }

    pub fn rollback(self) {
        self.oracle.abort(self.txn_id);
    }
}
```

### Changes to `src/rocks_storage.rs`

```rust
pub struct RocksStorage {
    db: Arc<DB>,
    alex: AlexTree,
    oracle: Arc<TransactionOracle>,  // NEW
    gc: Arc<GarbageCollector>,       // NEW
    value_cache: LruCache<(Vec<u8>, u64), Vec<u8>>,  // Version cache
}

impl RocksStorage {
    pub fn new(path: &str) -> Result<Self> {
        let db = Arc::new(DB::open_default(path)?);
        let alex = AlexTree::new();
        let oracle = Arc::new(TransactionOracle::new());
        let gc = Arc::new(GarbageCollector::new(oracle.clone(), /* storage */));

        // Start GC background thread
        let gc_clone = gc.clone();
        thread::spawn(move || {
            gc_clone.run_background(Duration::from_secs(60));  // Every minute
        });

        Ok(Self {
            db,
            alex,
            oracle,
            gc,
            value_cache: LruCache::new(NonZeroUsize::new(1_000_000).unwrap()),
        })
    }

    /// Point query with snapshot isolation
    pub fn point_query(&self, key: i64, snapshot: &Snapshot) -> Result<Option<Vec<u8>>> {
        // Use visibility engine to find visible version
        snapshot.get(&key.to_be_bytes())
    }
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_snapshot_isolation_basic() {
        let oracle = TransactionOracle::new();

        // T1: Begin, write x=1
        let mut t1 = oracle.begin().unwrap();
        t1.put(b"x".to_vec(), b"1".to_vec()).unwrap();

        // T2: Begin (should not see T1's write)
        let t2 = oracle.begin().unwrap();
        assert_eq!(t2.get(b"x").unwrap(), None);

        // T1: Commit
        t1.commit().unwrap();

        // T2: Still should not see x=1 (snapshot isolation)
        assert_eq!(t2.get(b"x").unwrap(), None);

        // T3: Begin (should see x=1)
        let t3 = oracle.begin().unwrap();
        assert_eq!(t3.get(b"x").unwrap(), Some(b"1".to_vec()));
    }

    #[test]
    fn test_write_conflict_detection() {
        let oracle = TransactionOracle::new();

        // T1 and T2 both try to write same key
        let mut t1 = oracle.begin().unwrap();
        let mut t2 = oracle.begin().unwrap();

        t1.put(b"x".to_vec(), b"1".to_vec()).unwrap();
        t2.put(b"x".to_vec(), b"2".to_vec()).unwrap();

        // T1 commits
        t1.commit().unwrap();

        // T2 should fail with conflict
        assert!(t2.commit().is_err());
    }

    #[test]
    fn test_read_your_own_writes() {
        let oracle = TransactionOracle::new();

        let mut t1 = oracle.begin().unwrap();
        t1.put(b"x".to_vec(), b"1".to_vec()).unwrap();

        // Should see own uncommitted write
        assert_eq!(t1.get(b"x").unwrap(), Some(b"1".to_vec()));
    }

    #[test]
    fn test_version_garbage_collection() {
        let oracle = TransactionOracle::new();
        let gc = GarbageCollector::new(oracle.clone());

        // Create many versions
        for i in 0..100 {
            let mut txn = oracle.begin().unwrap();
            txn.put(b"x".to_vec(), format!("{}", i).as_bytes().to_vec()).unwrap();
            txn.commit().unwrap();
        }

        // GC should clean old versions
        let deleted = gc.collect().unwrap();
        assert!(deleted > 0);
    }
}
```

### Integration Tests

```rust
#[test]
fn test_concurrent_transactions() {
    let storage = RocksStorage::new("/tmp/mvcc_test").unwrap();

    // Spawn 100 concurrent transactions
    let handles: Vec<_> = (0..100).map(|i| {
        let storage = storage.clone();
        thread::spawn(move || {
            let mut txn = storage.oracle.begin().unwrap();
            txn.put(format!("key{}", i).as_bytes().to_vec(),
                    format!("value{}", i).as_bytes().to_vec()).unwrap();
            txn.commit().unwrap();
        })
    }).collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all writes succeeded
    let txn = storage.oracle.begin().unwrap();
    for i in 0..100 {
        let value = txn.get(format!("key{}", i).as_bytes()).unwrap().unwrap();
        assert_eq!(value, format!("value{}", i).as_bytes());
    }
}
```

---

## Performance Considerations

### MVCC Overhead

**Expected**: 10-20% latency overhead (industry standard)

**Optimizations**:
1. **Read-only transactions**: Skip conflict detection
2. **Version caching**: LRU cache for recent versions
3. **GC tuning**: Collect old versions before they accumulate
4. **Index integration**: ALEX tracks latest version only

### Benchmarking

Compare performance before/after MVCC:
- Point queries: <20% regression
- Concurrent writes: Should improve (lock-free)
- Read-heavy workloads: Minimal impact

---

## Implementation Plan (Phase 1)

### Week 1: Transaction Oracle & Version Storage

**Days 1-3**:
- [ ] Implement `TransactionOracle` (timestamp allocation, lifecycle)
- [ ] Add `VersionedKey` and `VersionedValue` encoding
- [ ] Unit tests for oracle

**Days 4-5**:
- [ ] Integrate versioned storage with RocksDB
- [ ] Update ALEX to track latest version
- [ ] Unit tests for version storage

### Week 2: Visibility Engine & Conflict Detection

**Days 1-3**:
- [ ] Implement visibility rules (`is_visible`)
- [ ] Snapshot read logic
- [ ] Read-your-own-writes

**Days 4-5**:
- [ ] Write conflict detection
- [ ] First-committer-wins resolution
- [ ] Unit tests for conflicts

### Week 3: Integration & Testing

**Days 1-2**:
- [ ] Integrate with `TransactionContext`
- [ ] Update `BEGIN/COMMIT/ROLLBACK` handlers
- [ ] Update PostgreSQL protocol integration

**Days 3-5**:
- [ ] Comprehensive MVCC tests (100+ test cases)
- [ ] Concurrent transaction stress tests
- [ ] Performance validation
- [ ] Documentation

---

## Success Criteria

### Functional ✅
- [ ] Snapshot isolation working correctly
- [ ] No dirty reads, no lost updates
- [ ] Write conflicts detected and handled
- [ ] Read-your-own-writes
- [ ] 100+ MVCC unit tests passing

### Performance ✅
- [ ] <20% overhead vs non-MVCC
- [ ] Concurrent transactions don't block reads
- [ ] 1000+ concurrent transactions stable

### Quality ✅
- [ ] Zero data corruption
- [ ] All anomaly tests passing
- [ ] Clean code (no clippy warnings)
- [ ] Complete documentation

---

## References

**Implementations Studied**:
- ToyDB: https://github.com/erikgrinaker/toydb (timestamp-based, Rust)
- TiKV: Percolator model, snapshot isolation
- PostgreSQL: MVCC with xmin/xmax
- Mini-LSM: Snapshot read implementation
- SlateDB: Optimistic transactions

**Papers**:
- "A Critique of ANSI SQL Isolation Levels" (Berenson et al.)
- "Large-scale Incremental Processing Using Distributed Transactions and Notifications" (Percolator, Google)

---

**Status**: Design complete, ready for implementation
**Next**: Week 1 implementation (Transaction Oracle + Version Storage)
**Timeline**: 2-3 weeks to production-ready MVCC
**Date**: October 21, 2025
