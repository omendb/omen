# Storage Patterns for Vector Databases

*Extracted from ZenDB's production-grade storage engine*

## Memory-Mapped I/O Pattern

**Problem**: Need zero-copy access to large vector datasets
**Solution**: Memory-mapped files with page-based organization

```rust
// From ZenDB - directly applicable to vectors
pub const PAGE_SIZE: usize = 16384; // 16KB pages optimal for vectors
pub const HEADER_SIZE: usize = 4096; // 4KB header for metadata

// For OmenDB vectors (1536 dims × 4 bytes = 6KB per vector)
// 16KB page holds ~2 vectors + metadata
```

### Implementation in Mojo
```mojo
from sys.ffi import DLHandle, c_void_ptr
from memory import mmap, MAP_SHARED, PROT_READ, PROT_WRITE

struct VectorPage:
    var page_id: UInt64
    var vector_count: UInt32
    var vectors: DTypePointer[DType.float32]
    
    fn __init__(inout self, page_size: Int = 16384):
        self.vectors = mmap(page_size, PROT_READ | PROT_WRITE, MAP_SHARED)
```

## LRU Cache Pattern

**Problem**: Can't keep all vectors in memory
**Solution**: Bounded LRU cache with automatic eviction

```rust
// ZenDB's approach - keep hot vectors in memory
pub struct LRUCache {
    cache: HashMap<PageId, Arc<Page>>,
    access_order: VecDeque<PageId>,
    max_size: usize,  // ~16MB default
}
```

**For OmenDB**: 
- Cache size = 16MB holds ~2,700 vectors
- Evict least recently used when full
- Track access patterns for predictive loading

## Free Page Management

**Problem**: Vector deletions create holes in storage
**Solution**: Linked list of free pages in the data file

```rust
// Store free list directly in unused pages
struct FreePageNode {
    next_free_page: PageId,  // 0 means end
    _padding: [u8; PAGE_SIZE - 8],
}
```

**Key insight**: Reuse deleted vector space immediately, no external metadata

## Page Compression Pattern

**Problem**: Vector data is large (6KB per vector)
**Solution**: LZ4 compression with selective exclusion

```rust
// Compress data pages but not index pages
if !is_index_page(page_id) {
    compressed = lz4::compress(&page.data)?;
    // 30-70% reduction on vector data
}
```

**For OmenDB**:
- Compress vector pages (30-70% reduction)
- Keep DiskANN graph uncompressed (needs random access)
- Decompress on page load, not per-query

## WAL (Write-Ahead Logging) Pattern

**Problem**: Vector insertions must be durable
**Solution**: Log operations before modifying data

```rust
pub enum WALEntry {
    AddVector { id: String, vector: Vec<f32> },
    DeleteVector { id: String },
    UpdateVector { id: String, vector: Vec<f32> },
}

// Write to WAL before modifying index
wal.append(WALEntry::AddVector { ... })?;
index.add_vector(...);
wal.commit()?;
```

**Recovery process**:
1. Read WAL from last checkpoint
2. Replay operations on index
3. Mark checkpoint after successful recovery

## Actionable Commands

```bash
# Test memory-mapped performance
mojo run benchmarks/mmap_test.mojo --page-size=16384

# Monitor cache hit rates
watch -n 1 'cat /tmp/omendb_metrics | grep cache_hit_rate'

# Verify WAL integrity
mojo run tools/wal_validator.mojo --wal-file=vectors.wal

# Compress existing database
mojo run tools/compress_db.mojo --input=vectors.db --output=vectors_compressed.db
```

## Error → Fix Mappings

| Error | Fix | Context |
|-------|-----|---------|
| "mmap failed: Cannot allocate memory" | Reduce cache size or increase ulimits | PAGE_SIZE × cache_pages > available |
| "Page checksum mismatch" | Restore from WAL | Corruption detected |
| "Cache thrashing detected" | Increase cache size or optimize access pattern | Hit rate < 50% |
| "WAL replay failed" | Check disk space, run recovery tool | Incomplete write |

## Decision Trees

```
IF vector_count < 100K:
    → Keep all in memory
ELIF vector_count < 1M:
    → Use LRU cache with mmap
ELIF vector_count < 10M:
    → Add compression + larger pages
ELSE:
    → Consider sharding/distribution
```

## Performance Characteristics

- **Page Size**: 16KB optimal for 1536-dim vectors
- **Cache Size**: 16MB serves 90% of queries for 100K vectors
- **Compression**: 30-70% reduction with LZ4
- **WAL Overhead**: ~10% write performance cost for durability

## Integration with DiskANN

DiskANN requires different access patterns:
- **Graph pages**: Keep uncompressed (random access)
- **Vector pages**: Can compress (sequential during search)
- **Cache priority**: Graph pages > hot vectors > cold vectors

## Next Steps for OmenDB

1. Implement memory-mapped vector storage
2. Add LRU cache for hot vectors
3. Integrate WAL for crash recovery
4. Add compression for cold vectors
5. Test with 1M+ vector datasets