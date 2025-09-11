# NOW - Current Sprint (Feb 2025)

## 🎯 Current Status: HNSW Quality Crisis - Systematic Debugging in Progress

### Critical Quality Issues Found (Feb 2025) 🚨

**THE REAL PROBLEM:** HNSW has catastrophic recall failure at scale

**Quality Results:**
- **100 vectors:** 70% Recall@1 (improved from 0%)
- **500 vectors:** 15% Recall@1
- **1000 vectors:** 10% Recall@1  
- **2000+ vectors:** 0% Recall@1 (complete failure)

**Root Cause Identified:** Bulk insertion doesn't navigate graph hierarchy properly

### Bugs Fixed Today (Partial Improvement):

1. **✅ Graph connectivity** → Increased sampling from 20 to 100+ nodes
2. **✅ Search parameters** → Fixed ef to use ef_construction (200) not M (16)
3. **✅ Candidate limits** → Removed artificial ef//2 restriction
4. **✅ Size counting** → Fixed double counting bug in bulk insert

### Critical Issue Remaining:

**Bulk insertion broken for existing graphs:**
- Individual insertion: Navigates hierarchy properly (100% recall)
- Bulk insertion: Doesn't navigate, creates disconnected clusters
- Mixed mode: Bulk nodes can't reach individual nodes (30% recall)

**Required Fix:** Complete refactor of bulk insertion to navigate hierarchy

### Why We Regressed:
- Moved from simple working system to over-engineered HNSW+
- Added too many "optimizations" that actually slow things down
- Fixed capacity prevents proper scaling
- Exploring 64+ candidates when 8 would suffice

### Storage Optimization Results (Feb 2025) ✅
- **96x compression working** (PQ32 properly implemented)
- **1,307 vec/s throughput** (3x improvement with batching)
- **1.00008x overhead** (dynamic growth, no pre-allocation)
- **Full recovery working** (tested at scale)
- **Direct mmap implemented** (storage_v3.mojo created)

**Key Finding**: Python I/O itself is fast (4M vec/s)
**Real bottleneck**: Python FFI overhead from Mojo (not Python I/O)
**Solution implemented**: Direct syscalls via external_call["mmap"]

### Current Performance ✅
- **4,556 vec/s** single-threaded (stable performance)
- **0.56ms** search latency (1800 QPS)
- **288 bytes/vector** memory usage
- **Multiple batches working**: Global singleton properly restored
- **No warnings**: Using __ prefix for global state

### Critical Fix Applied (Feb 2025)
**Fixed**: Restored global singleton pattern with explicit global variable
- Database was creating new instances on every call (bug introduced earlier)
- Now properly maintains single global instance (with Mojo warning)
- Module-level variables still coming 2026+ (official support)
- Thread synchronization primitives not available (limits parallelization)

### What's Working ✅
1. **Zero-copy FFI**: NumPy arrays passed directly (5x speedup)
2. **SIMD optimizations**: vectorize extensively used for distance calculations
3. **Memory pool**: Pre-allocated, no malloc overhead
4. **Parallelize for math**: Distance matrix operations use parallelize
5. **Cache optimization**: Batched neighbor computations for better locality
6. **Global state**: Working with __ prefix (suppresses warning)

### What's Limited ⛔
1. **Graph parallelization**: No mutexes for thread-safe graph updates
2. **Multiple instances**: Can't create separate DB instances (singleton only)
3. **Prefetching**: No explicit prefetch instructions available yet
4. **GPU support**: Ready but requires NVIDIA hardware

### Parallelize Usage Analysis
**Working**:
- `matrix_ops.mojo`: Distance matrix calculations ✅
- `simd.mojo`: Query processing ✅

**Not Working**:
- `hnsw.mojo`: insert_bulk_wip() crashes at 5K+ vectors ❌
- Graph updates can't be parallelized (no synchronization)

### Recent Work (Feb 2025)

#### mmap Implementation (Just completed)
1. **Created storage_v3.mojo**: Direct mmap without Python FFI
2. **Extracted patterns**: From memory_mapped.mojo (removed 64MB pre-allocation)
3. **Integrated PQ compression**: Inline SimplePQ implementation
4. **Benchmarked performance**: Python I/O achieves 4M vec/s natively
5. **Identified real issue**: FFI overhead, not I/O speed

### Previous Work
1. **Fixed NumPy detection**: Changed isinstance() method → 1,400 vec/s restored
2. **Investigated memory corruption**: Traced to global singleton pattern
3. **Created workarounds**: Process isolation (33% overhead)
4. **Added specialized SIMD kernels**: For 128D, 256D, 384D, 512D, 768D, 1536D
5. **Documented limitations**: Created CURRENT_STATUS.md and OPTIMIZATION_ANALYSIS.md

### Maximum Achievable Performance
**Single-thread potential** (with all CPU optimizations):
- Current: 4,605 vec/s (using ~30% of potential)
- Achievable: 10,000-15,000 vec/s
- With GPU (2025+): 100,000+ vec/s

### Storage Engine Status (Feb 2025)

**✅ INTEGRATED**: Memory-mapped storage connected to GlobalDatabase!
- Found sophisticated implementation in `omendb/storage/memory_mapped.mojo`
- Created `PersistentGlobalDatabase` integration layer
- Persistence and recovery working end-to-end
- Automatic checkpointing every 100 operations

**✅ Working Features**:
- Save/load vectors with minimal overhead (1.00008x)
- Recovery works perfectly (tested with 10K vectors)
- Accurate memory reporting (3,136 bytes/vector)
- Search functionality after recovery
- Clean, maintainable code (~300 lines)

**✅ Completed Steps**:
1. **Wired up compression** - PQ32 working with 96x compression
2. **Added batch writes** - 3x throughput improvement achieved
3. **Ported mmap properly** - Created storage_v3 with direct syscalls
4. **Ready for HNSW+** - Storage layer complete, can focus on algorithm

### Workarounds Available
1. **Single batch mode**: Clear DB between batches
2. **Server mode**: HTTP/gRPC server manages state
3. **Process isolation**: 930 vec/s with stability

### Strategic Decision
**Staying with Mojo** despite current limitations because:
- Python interoperability (drop-in replacement)
- SIMD by default (already fast)
- Future GPU support (100x speedup coming)
- No GC (predictable performance)

The global state issue is temporary - Mojo is actively developing needed features.