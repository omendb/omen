# Concurrency Patterns for Vector Databases

*Extracted from ZenDB's multi-writer implementation*

## Multi-Writer Architecture

**Problem**: Multiple processes need to add/update vectors simultaneously
**Solution**: Page-level locking with deadlock detection

```rust
// From ZenDB - adapt for vector storage
pub struct MultiWriterPageManager {
    page_locks: Arc<DashMap<PageId, Arc<RwLock<()>>>>,
    lock_timeout: Duration,
}
```

### Vector-Specific Considerations

```mojo
# OmenDB needs different granularity
struct VectorLockManager:
    var graph_lock: RwLock      # DiskANN graph structure
    var vector_locks: Dict[String, RwLock]  # Per-vector locks
    var buffer_lock: Mutex       # Buffer flush coordination
```

## Lock Hierarchy Pattern

**Problem**: Deadlocks when acquiring multiple locks
**Solution**: Strict lock ordering

```
Lock Order (high → low priority):
1. Metadata lock (collection info)
2. Graph structure lock (DiskANN index)
3. Page locks (vector data)
4. Individual vector locks
```

**Rule**: Always acquire locks in this order, release in reverse

## Optimistic Concurrency Control

**Problem**: Pessimistic locking hurts read performance
**Solution**: Version checking on write

```rust
// Check version before update
let version = vector.version();
// ... perform calculations ...
if !vector.compare_and_swap(version, new_data) {
    // Retry with fresh data
}
```

## Read-Write Separation

**Problem**: Reads blocked by writes
**Solution**: MVCC with snapshot isolation

```mojo
struct MVCCVectorStore:
    var versions: Dict[String, List[VectorVersion]]
    var active_transactions: List[TransactionId]
    
    fn read(self, vector_id: String, tx_id: TransactionId) -> Vector:
        # Read version visible to this transaction
        return self.get_version_as_of(vector_id, tx_id.start_time)
```

## Buffer Flush Coordination

**Critical for OmenDB**: The 25K bottleneck likely here

```mojo
struct BufferFlushStrategy:
    var flush_threshold: Int = 10000  # Vectors before flush
    var flush_lock: Mutex
    var flush_in_progress: AtomicBool
    
    fn should_flush(self) -> Bool:
        # Avoid thundering herd
        if self.flush_in_progress.load():
            return False
        return self.buffer.size() > self.flush_threshold
    
    fn flush_async(self):
        # Non-blocking flush to main index
        if self.flush_in_progress.compare_exchange(False, True):
            # Spawn background task
            spawn(self._do_flush)
```

## Actionable Implementation

### Fix for 25K Bottleneck
```mojo
# Current (likely problematic):
fn add_vector(self, vector):
    self.buffer.add(vector)
    if self.buffer.size() > 25000:
        self.flush_all()  # BLOCKING! 

# Better approach:
fn add_vector(self, vector):
    self.buffer.add(vector)
    if self.should_flush():
        self.trigger_async_flush()  # Non-blocking
    return  # Don't wait
```

### Concurrent Search Pattern
```mojo
struct ConcurrentSearch:
    var read_pool: ThreadPool
    var graph: Arc[DiskANNIndex]  # Shared immutable during search
    
    fn search_parallel(self, queries: List[Vector]) -> List[Results]:
        # Parallel search across threads
        return self.read_pool.map(
            lambda q: self.graph.search(q),
            queries
        )
```

## Error → Fix Mappings

| Error | Fix | Context |
|-------|-----|---------|
| "Lock timeout exceeded" | Increase timeout or reduce transaction scope | Contention on hot vectors |
| "Deadlock detected" | Follow lock hierarchy strictly | Multiple locks acquired out of order |
| "Version conflict" | Implement retry logic with backoff | Optimistic concurrency failure |
| "Buffer overflow during flush" | Use double buffering technique | Writes blocked during flush |

## Decision Trees

```
IF operation == READ:
    → Use shared locks only
    → Never wait for writes
ELIF operation == WRITE:
    IF affecting_graph:
        → Acquire graph write lock (rare)
    ELSE:
        → Acquire page/vector locks only
ELIF operation == BULK_INSERT:
    → Use buffer with async flush
    → Batch operations
```

## Performance Patterns

### Double Buffering
```mojo
struct DoubleBuffer:
    var active: VectorBuffer
    var flushing: Optional[VectorBuffer]
    
    fn add(self, vector):
        self.active.add(vector)
        if self.active.is_full():
            # Swap buffers
            self.flushing = self.active
            self.active = VectorBuffer.new()
            # Flush in background
            spawn(self.flush_buffer)
```

### Read-Copy-Update (RCU) for Graph
```mojo
struct RCUGraph:
    var current: Arc[DiskANNGraph]
    
    fn update(self, changes):
        # Create new version
        var new_graph = self.current.copy()
        new_graph.apply(changes)
        # Atomic pointer swap
        self.current = Arc(new_graph)
        # Old version cleaned up when readers finish
```

## Benchmarking Concurrency

```bash
# Test concurrent writes
mojo run benchmarks/concurrent_writes.mojo --threads=16 --vectors=100000

# Measure lock contention
mojo run tools/lock_profiler.mojo --duration=60

# Test read-write mix
mojo run benchmarks/mixed_workload.mojo --read-ratio=0.9
```

## Key Insights for OmenDB

1. **Buffer flush is critical** - The 25K bottleneck is likely synchronous flush
2. **Graph updates are expensive** - Use RCU or versioning
3. **Page-level locking works** - But need right granularity
4. **Async is essential** - Never block writes on I/O

## Learning from Chroma's WAL Evolution

**Source**: https://trychroma.com/engineering/wal3

Chroma went through 3 WAL versions to solve this exact problem:
- v1: SQLite WAL (too slow, blocking)
- v2: Custom WAL with batching (better but still blocked)
- v3: **Async flush with double buffering** (solved it!)

**Their Solution (apply to OmenDB):**
```mojo
struct AsyncBufferManager:
    var active_buffer: VectorBuffer
    var flush_buffer: Optional[VectorBuffer]
    var flush_in_progress: AtomicBool
    
    fn add_vector(self, vector):
        self.active_buffer.add(vector)
        
        if self.should_flush() and not self.flush_in_progress:
            # Swap buffers immediately (non-blocking)
            self.flush_buffer = self.active_buffer
            self.active_buffer = VectorBuffer.new()
            self.flush_in_progress = True
            
            # Flush old buffer async
            spawn(self.flush_to_index)
        
    fn flush_to_index(self):
        # This happens in background
        self.index.add_batch(self.flush_buffer.vectors)
        self.flush_buffer = None
        self.flush_in_progress = False
```

**This is THE fix for the 25K bottleneck!**

## Recommended Architecture

```
Writers → Buffer1 → | Async Flush | → DiskANN Index
       → Buffer2 → | When B1 Full|
                    
Readers → Directly read from Index (lock-free)
```

This architecture can handle 100K+ writes/sec with proper tuning.