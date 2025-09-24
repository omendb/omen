# OmenDB Storage Engine Specification
## Version 1.0 - February 2025

## Executive Summary

OmenDB will implement a custom storage engine in pure Mojo, leveraging the language's native support for concurrency primitives (Atomics, SpinLocks), file I/O (FileHandle), and FFI capabilities for memory mapping. This approach follows industry leaders like Qdrant (custom Gridstore) and Weaviate (custom LSM-tree) who moved away from generic databases for optimal performance.

## MVP Implementation Approach

### Start Simple, Iterate Fast
1. **Phase 1**: Basic file-based persistence with binary format
2. **Phase 2**: Add WAL for crash recovery
3. **Phase 3**: Implement concurrent read/write access
4. **Phase 4**: Optimize with memory mapping

### Key Principles
- **Working code over perfect design**: Get persistence working first
- **Incremental optimization**: Profile and improve bottlenecks
- **Test-driven**: Each phase must pass all tests before moving on
- **API stability**: Keep VectorStore interface unchanged

## Architecture Overview

### Core Components

1. **Write-Ahead Log (WAL)**
   - Append-only transaction log for crash recovery
   - Uses `FileHandle` with binary writes
   - Checksummed entries for corruption detection
   - Automatic compaction at configurable thresholds

2. **Block Storage Manager**
   - Fixed-size blocks (4KB default, configurable)
   - Free block tracking via bitmask
   - Coalesced writes for sequential performance
   - Block-level checksums for integrity

3. **Memory Mapping Layer**
   - Zero-copy access via `mmap` FFI
   - Lazy loading of cold data
   - Configurable cache size
   - Prefetch hints for sequential access

4. **Concurrent Access Control**
   - `BlockingSpinLock` for critical sections
   - `Atomic[DType.int64]` for counters and flags
   - Lock-free reads where possible
   - Write batching for reduced contention

## Storage Format

### File Layout
```
omendb.db (main data file)
├── Header (4KB)
│   ├── Magic bytes: "OMEN"
│   ├── Version: u32
│   ├── Block size: u32
│   ├── Total blocks: u64
│   ├── Free blocks: u64
│   └── Checksum: u32
├── Free Bitmask (variable)
│   └── 1 bit per block
├── Data Blocks (4KB each)
│   ├── Vector blocks
│   ├── ID mapping blocks
│   └── Graph edge blocks
└── Metadata Block (final)
    ├── Index configuration
    └── Statistics

omendb.wal (write-ahead log)
├── Header (512 bytes)
│   ├── Magic: "OMENWAL"
│   ├── Version: u32
│   └── Sequence: u64
└── Entries (variable)
    ├── Type: u8
    ├── Size: u32
    ├── Data: bytes
    └── CRC32: u32
```

### Block Types

#### Vector Block (Type 0x01)
```
├── Header (64 bytes)
│   ├── Type: 0x01
│   ├── Count: u16
│   ├── Dimension: u16
│   └── Next block: u32
├── Vector data (compressed)
└── Padding
```

#### ID Mapping Block (Type 0x02)
```
├── Header (64 bytes)
│   ├── Type: 0x02
│   ├── Count: u16
│   └── Next block: u32
├── Entries
│   ├── Hash: u64
│   ├── Vector offset: u32
│   └── Metadata offset: u32
└── Padding
```

#### Graph Edge Block (Type 0x03)
```
├── Header (64 bytes)
│   ├── Type: 0x03
│   ├── Node ID: u32
│   ├── Edge count: u16
│   └── Next block: u32
├── Edges
│   ├── Target ID: u32
│   └── Weight: f32
└── Padding
```

## Implementation Plan

### Phase 1: Core Storage (Week 1)
```mojo
# Start simple - file-based storage with binary format
struct StorageEngine:
    var data_file: FileHandle
    var index_file: FileHandle
    var lock: BlockingSpinLock
    var block_size: Int
    var free_blocks: List[Int]
    
    fn __init__(inout self, path: String) raises:
        self.data_file = open(path + ".db", "wb+")
        self.index_file = open(path + ".idx", "wb+")
        self.lock = BlockingSpinLock()
        self.block_size = 4096
        self.free_blocks = List[Int]()
        self.write_header()
    
    fn write_vector(inout self, id: String, vector: SIMD[DType.float32, _]) raises:
        """Simple write - can optimize later with WAL."""
        with BlockingScopedLock(self.lock):
            # Find or allocate block
            var block_id = self.allocate_block()
            var offset = block_id * self.block_size
            
            # Write vector data
            self.data_file.seek(offset)
            var bytes = vector.unsafe_ptr().bitcast[UInt8]()
            self.data_file.write(bytes, vector.size * 4)
            
            # Update index
            self.write_index_entry(id, block_id)
```

### Phase 2: Write-Ahead Logging (Week 2)
```mojo
struct WAL:
    var file: FileHandle
    var sequence: Atomic[DType.int64]
    
    fn __init__(inout self, path: String) raises:
        self.file = open(path, "ab+")  # Append mode
        self.sequence = Atomic[DType.int64](0)
        
    fn append_entry(inout self, op_type: UInt8, data: DTypePointer[DType.uint8], size: Int) raises:
        """Append operation to WAL."""
        var seq = self.sequence.fetch_add(1)
        
        # Simple format: [seq:8][type:1][size:4][data:N][crc:4]
        self.file.write(seq.unsafe_ptr().bitcast[UInt8](), 8)
        self.file.write(UnsafePointer.address_of(op_type).bitcast[UInt8](), 1)
        self.file.write(size.unsafe_ptr().bitcast[UInt8](), 4)
        self.file.write(data, size)
        
        # TODO: Add CRC32 checksum
        var crc = UInt32(0)  # Placeholder
        self.file.write(crc.unsafe_ptr().bitcast[UInt8](), 4)
        self.file.flush()
    
    fn replay(inout self) raises -> Int:
        """Replay WAL and return number of entries processed."""
        self.file.seek(0)
        var count = 0
        
        # Read entries until EOF or corruption
        while self.file.tell() < self.file.size():
            # Read and validate entry
            # Apply to storage engine
            count += 1
        
        return count
```

### Phase 3: Concurrent Access (Week 3)
```mojo
struct ConcurrentStorage:
    var engine: StorageEngine
    var read_lock: BlockingSpinLock
    var write_lock: BlockingSpinLock
    var active_readers: Atomic[DType.int64]
    
    fn read_vector(self, id: String) raises -> SIMD[DType.float32, _]:
        """Lock-free read path."""
        _ = self.active_readers.fetch_add(1)
        defer:
            _ = self.active_readers.fetch_sub(1)
        
        # Read without lock (data is immutable once written)
        return self.engine.read_vector_direct(id)
    
    fn write_batch(inout self, ids: List[String], vectors: List[SIMD[DType.float32, _]]) raises:
        """Batched writes with single lock acquisition."""
        with BlockingScopedLock(self.write_lock):
            # Wait for readers to finish if doing compaction
            while self.active_readers.load() > 0:
                # Spin wait or yield
                pass
            
            # Batch write all vectors
            for i in range(len(ids)):
                self.engine.write_vector(ids[i], vectors[i])
```

### Phase 4: Memory Mapping (Week 4)
```mojo
struct MemoryMap:
    var fd: Int
    var ptr: UnsafePointer[UInt8]
    var size: Int
    
    fn __init__(inout self, fd: Int, size: Int):
        self.fd = fd
        self.size = size
        
        # Call mmap via FFI
        var result = external_call[
            "mmap",
            UnsafePointer[UInt8],
        ](
            0,  # addr (let kernel choose)
            size,
            3,  # PROT_READ | PROT_WRITE
            1,  # MAP_SHARED
            fd,
            0   # offset
        )
        
        if Int(result) == -1:
            raise Error("mmap failed")
        
        self.ptr = result
    
    fn read[T: AnyType](self, offset: Int) -> T:
        """Zero-copy read from mapped memory."""
        return UnsafePointer[T](self.ptr + offset).load()
    
    fn write[T: AnyType](inout self, offset: Int, value: T):
        """Direct write to mapped memory."""
        UnsafePointer[T](self.ptr + offset).store(value)
```

## Performance Targets

### MVP Targets (Phase 1-2)
- **Write throughput**: 10K vectors/second (file-based)
- **Read latency**: < 10ms average
- **Recovery time**: < 10 seconds for 100K vectors
- **Memory overhead**: < 50% above raw data

### Production Targets (Phase 3-4)
- **Write throughput**: 50K vectors/second (with mmap)
- **Read latency**: < 1ms p99
- **Recovery time**: < 5 seconds for 1M vectors
- **Memory overhead**: < 20% above raw data

### Future Optimizations
1. **Batch writes**: Group WAL entries, coalesce block writes
2. **Lock-free reads**: Use RCU-style patterns where possible
3. **Prefetching**: Predict access patterns, prefetch blocks
4. **Compression**: Optional PQ/SQ compression for vectors
5. **Direct I/O**: Bypass OS cache for predictable latency

## Testing Strategy

### Unit Tests
```mojo
fn test_wal_recovery():
    var wal = WAL("test.wal")
    wal.append(WALEntry.vector("v1", vector1))
    wal.append(WALEntry.vector("v2", vector2))
    
    # Simulate crash
    wal = WAL("test.wal")
    var entries = wal.recover()
    assert_equal(len(entries), 2)

fn test_concurrent_allocation():
    var mgr = ConcurrentBlockManager(1000)
    var allocated = List[Int]()
    
    # Spawn threads to allocate blocks
    @parameter
    async fn allocate_task():
        for _ in range(100):
            allocated.append(mgr.allocate())
    
    var tg = TaskGroup()
    for _ in range(10):
        tg.create_task(allocate_task())
    tg.wait()
    
    # Verify no duplicates
    var seen = Set[Int]()
    for block_id in allocated:
        assert_false(block_id in seen)
        seen.add(block_id)
```

### Integration Tests
- Crash recovery scenarios
- Concurrent read/write stress tests
- Memory pressure tests
- Corruption detection tests

### Benchmarks
```mojo
fn benchmark_write_throughput():
    var engine = StorageEngine("bench.db")
    var vectors = generate_vectors(100_000, 768)
    
    var start = now()
    for i in range(len(vectors)):
        engine.write_vector(String(i), vectors[i])
    var elapsed = now() - start
    
    print("Throughput:", len(vectors) / elapsed.seconds(), "vec/s")

fn benchmark_read_latency():
    # Measure p50, p95, p99 latencies
    var latencies = List[Float64]()
    for _ in range(10_000):
        var start = now()
        _ = engine.read_vector(random_id())
        latencies.append((now() - start).microseconds())
    
    latencies.sort()
    print("p50:", latencies[5000], "μs")
    print("p99:", latencies[9900], "μs")
```

## Migration Path

### From Current Implementation
1. Keep existing `VectorStore` API unchanged
2. Add `StorageEngine` as optional persistence layer
3. Gradually migrate operations to use storage
4. Remove in-memory only code paths

### Compatibility
- Export/import tools for other formats
- Optional SQLite metadata storage
- REST API for language-agnostic access

## Risk Mitigation

### Technical Risks
1. **Mojo stdlib limitations**: Use FFI for missing features
2. **Performance regression**: Extensive benchmarking, rollback plan
3. **Data corruption**: Checksums, redundancy, recovery tools
4. **Memory leaks**: RAII patterns, automated testing

### Mitigation Strategies
- Incremental development with testing at each phase
- Performance benchmarks as regression tests
- Fuzz testing for corruption scenarios
- Memory sanitizers in CI pipeline

## Success Criteria

### Must Have
- ✅ Crash recovery via WAL
- ✅ Concurrent read/write support
- ✅ Zero-copy memory mapping
- ✅ 100K vec/s write throughput

### Should Have
- ⏳ Compression support
- ⏳ Online backup capability
- ⏳ Incremental snapshots
- ⏳ Multi-version concurrency

### Nice to Have
- 🔮 Distributed replication
- 🔮 Point-in-time recovery
- 🔮 Storage tiering (hot/warm/cold)

## Appendix A: Mojo Primitives Reference

### Available in Mojo stdlib
- `FileHandle`: Binary file I/O with `read_bytes()`, `write()`, `seek()`
- `Atomic[DType]`: Lock-free atomic operations
- `BlockingSpinLock`: Basic mutual exclusion
- `TaskGroup`: Async task management
- `external_call`: FFI for system calls like `mmap`

### Patterns from stdlib/test/runtime/test_locks.mojo
```mojo
var lock = BlockingSpinLock()
var counter = Atomic[DType.int64](0)

@parameter
async fn increment():
    with BlockingScopedLock(lock):
        _ = counter.fetch_add(1)

var tg = TaskGroup()
for _ in range(1000):
    tg.create_task(increment())
tg.wait()
```

## Appendix B: Competitor Analysis

### Qdrant
- Custom "Gridstore" engine
- Memory-mapped segments
- Async I/O with tokio
- Copy-on-write snapshots

### Weaviate
- Custom LSM-tree implementation
- Write-optimized with compaction
- Bloom filters for fast lookups
- Tiered storage (memory → SSD)

### Pinecone
- Proprietary distributed storage
- S3-backed with local caching
- Serverless architecture
- Pod-based isolation

## Appendix C: File Format Evolution

### Version 1.0 (Current)
- Basic block storage
- Simple WAL
- Single-threaded checkpoint

### Version 2.0 (Planned)
- Compression support
- Multi-version blocks
- Parallel checkpoint

### Version 3.0 (Future)
- Distributed WAL
- Cross-region replication
- Storage tiering