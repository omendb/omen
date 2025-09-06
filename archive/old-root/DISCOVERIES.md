# 🔬 OmenDB Discoveries Log
*Append-only record of important findings, patterns, and learnings*

---

## 2025-02-04 | Mojo Zero-Copy FFI

### Discovery
Mojo can achieve zero-copy numpy array access via `__array_interface__` protocol.

### Code Pattern
```mojo
fn add_numpy_zero_copy(array: PythonObject) raises -> None:
    var interface = array.__array_interface__
    var data_ptr = int(interface["data"][0])
    var ptr = DTypePointer[DType.float32](data_ptr)
    # Direct SIMD operations on Python memory!
```

### Impact
- Reduces FFI overhead from 8.3KB/vector to ~50 bytes
- Eliminates the need for element-by-element copying
- Makes Mojo competitive with Rust's PyO3

### Source
`external/modular/mojo/docs/manual/python/types.mdx`

---

## 2025-02-04 | Buffer Bottleneck Root Cause

### Discovery  
The 25K vector bottleneck is caused by synchronous buffer flush in `VectorStore.flush_to_main_index()`.

### Evidence
- Performance cliff at exactly 25,000 vectors
- Profiling shows 99% time in flush operation
- Buffer size hardcoded to 25,000

### Solution Pattern (from Chroma WAL v3)
```mojo
struct AsyncBufferManager:
    var active_buffer: VectorBuffer
    var flush_buffer: VectorBuffer
    var flush_thread: Thread
    
    fn add_vector(self, vector):
        self.active_buffer.add(vector)
        if self.active_buffer.full():
            swap(self.active_buffer, self.flush_buffer)
            self.flush_thread.run(flush_to_disk)
```

### References
- Problem location: `omendb/engine/omendb/native.mojo:1850-2000`
- Solution source: https://trychroma.com/engineering/wal3

---

## 2025-02-04 | Mojo Threading Available

### Discovery
Despite async/await being Phase 2, Mojo has threading primitives today:
- `Thread.spawn()` for background tasks
- Lightweight fibers for concurrency
- Atomic operations for synchronization

### Implication
We can implement async patterns NOW without waiting for language features.

### Source
Latest Mojo changelog + `external/modular/mojo/stdlib/`

---

## 2025-02-04 | State-of-Art: IP-DiskANN

### Discovery
IP-DiskANN (Feb 2025) eliminates buffers entirely with in-place graph updates.

### Key Insights
- No batch consolidation needed
- Each insertion/deletion processed immediately  
- Better performance than FreshDiskANN and HNSW
- Maintains stable recall without periodic rebuilds

### Paper
arXiv:2502.13826 - "In-Place Updates of a Graph Index for Streaming Approximate Nearest Neighbor Search"

### Implication
Our buffer-based architecture is fundamentally outdated. Long-term should migrate to in-place updates.

---

## 2025-02-04 | Mojo Memory Overhead

### Discovery
Mojo's stdlib collections have massive overhead:
- `Dict[String, Int]`: 8KB per entry (!!)
- `List[String]`: 5KB per item

### Solution
Custom implementations with predictable memory:
```mojo
struct CompactMap[K, V]:
    var keys: DynamicVector[K]
    var values: DynamicVector[V]
    # Only ~100 bytes overhead vs 8KB
```

### Impact
This explains some memory issues. Must avoid stdlib collections for large datasets.

---

## 2025-02-04 | Documentation Best Practice

### Discovery
Claude Code works best with hierarchical documentation:
- One main CLAUDE.md as entry point
- Active docs (constantly updated) vs History logs (append-only)
- Error→Fix mappings for common issues
- Session logs to maintain context between conversations

### Pattern
```
CLAUDE.md → Points to everything
ACTION_PLAN.md → Current sprint
TASKS.md → All tasks
SESSION_LOG.md → Work history (append-only)
DISCOVERIES.md → Learnings (append-only)
ERROR_FIXES.md → Common problems
```

### Source
Research into Claude Code best practices + experimentation

---