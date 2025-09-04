# OmenDB TODO List
*Last Updated: August 29, 2025*

## 🚨 CRITICAL PATH (Aug 29, 2025 - UPDATED)

**Memory Crisis IDENTIFIED**: Mojo stdlib has 69x overhead
- Algorithm level: 136 bytes/vector ✅ (quantization works!)
- Reported memory: 372 bytes/vector (includes metadata)
- Actual OS memory: 9,339 bytes/vector ❌ (69x target!)
- **Root Cause**: Dict[String, Int] uses 8KB/entry, List[String] uses 5KB/item

**Immediate Actions Required**:
1. **Replace Dict[String, Int]**: Custom hash map implementation
2. **Replace List[String]**: Fixed-size string pool
3. **String optimization**: Use fixed char arrays, avoid String type
4. **Memory allocator**: Arena allocator for batch operations

**Expected Result**: Achieve true 136 bytes/vector by eliminating stdlib overhead

## ✅ CRITICAL FIX: 26K Vector Limit ELIMINATED!

**COMPLETED**: LibC mmap fix eliminated the crash
- **Previous**: Hard crash at 26-27K vectors (List[List[Int]] issue)
- **Solution**: Python FFI elimination fixed underlying memory corruption
- **Tested**: Successfully scaled to 60K+ vectors
- **Performance**: 70K+ vec/s sustained throughout
- **Status**: FIXED - No limit detected!

## ✅ CRITICAL FIX: Python FFI Eliminated from I/O Path

**COMPLETED**: Replaced Python mmap with LibC implementation
- **Problem**: 512 Python FFI calls per vector (2000ns per read)
- **Solution**: Direct LibC mmap with sys.ffi.external_call
- **Result**: 50x faster I/O (256μs → 5.12μs per vector)
- **Status**: Implemented and tested

## 🚨 STATUS UPDATE (Aug 29, 2025 - End of Day)

### 1. ✅ QUANTIZATION FIXED
- **DONE**: Switched from MMapGraph to VamanaGraph
- **DONE**: Quantization enables and works functionally
- **TODO**: Memory optimization (still ~2KB/vector)

### 2. ✅ PERFORMANCE CLIFF IDENTIFIED & MITIGATED
- **ROOT CAUSE FOUND**: Buffer flush adds vectors one-by-one (39ms/100 vectors)
- **QUICK FIX APPLIED**: Increased buffer_size 10K → 100K
- **RESULT**: Cliff delayed to 100K vectors
- **TODO**: Implement proper batch flush

### 3. ✅ VamanaGraph Crash Fixed
- **Issue**: Segfault at ~20K vectors
- **Root Cause**: Copy constructor allocated based on num_nodes instead of capacity
- **Solution**: Fixed memory allocation and added bounds checking
- **Status**: FIXED - Can now handle 20K+ vectors

**Previous Claims Were Misleading:**
- 70K vec/s was true at 5-10K scale, not at 100K+
- 208 bytes/vector never worked (double storage bug)
- Tests were done at toy scale, not production scale

## 🔥 IMMEDIATE FIXES REQUIRED (Blocking v0.0.1)

### Critical Memory Issues (Aug 29 Investigation)

1. **FIX QUANTIZATION DOUBLE STORAGE** [🔴 CRITICAL]
   - **Problem**: VectorBuffer stores Float32, VamanaGraph stores quantized = double storage
   - **Impact**: 648 bytes/vector instead of 136 bytes (376% overhead)
   - **Fix**: Make VectorBuffer quantization-aware, pass flag from VectorStore
   - **Priority**: HIGHEST - This alone gives 75% memory reduction

2. **FIX PRE-ALLOCATION WASTE** [🔴 CRITICAL]
   - **Problem**: 91-93% memory waste at typical scales
   - **Impact**: 11.9 MB allocated for 0.5 MB data at 1K vectors
   - **Quick Fix**: Reduce DiskANN 1M → 100K, Buffer 100K → 10K
   - **Full Fix**: Dynamic growth (2x → 1.5x → 1.125x), adaptive sizing
   - **Priority**: HIGH - Essential for embedded deployment

3. **FIX TEST INFRASTRUCTURE** [⚠️ IMPORTANT]
   - **Problem**: All Mojo tests use outdated import paths
   - **Impact**: Can't validate memory optimizations
   - **Fix**: Update imports, create memory validation tests
   - **Priority**: MEDIUM - Need tests to verify fixes

### Completed Fixes (Aug 29):

✅ **VAMANAGRAPH CRASH FIXED** - Fixed memory allocation bug, scales beyond 20K
✅ **BATCH FLUSH IMPLEMENTED** - Flush uses batch operations, major speedup
✅ **100K SCALE ACHIEVED** - Fixed buffer config and parameter mismatch

## 🎯 Pre-Release Priorities (v0.0.1 Targets)

### Architecture Clarity (Already Decided)
1. ✅ **Thread Model** - Single-writer embedded (SQLite model), Rust server for concurrency
2. ✅ **Data Integrity** - Checkpoint-based durability, no ACID needed for vectors (like ChromaDB)
3. ✅ **Configuration** - Simplified to 5 essential fields, connected to Python
4. ✅ **Performance** - 70K vec/s sustained, removed 10% checksum overhead

### Must Complete Before v1.0
1. [x] **Panic Elimination** - ✅ All 16 .value() calls replaced with safe_unwrap
2. [x] **Python API Polish** - ✅ Fixed batch_add parameter order bug
3. [ ] **Memory Safety Audit** - Document all unsafe operations, justify each one
4. [ ] **Monitoring Hooks** - Add OpenTelemetry support (industry standard)
5. [ ] **Documentation** - Complete API docs, architecture guide, benchmarks
6. [ ] **Comprehensive Tests** - Edge cases, error conditions, stress tests

## What Can We Fix Now vs Blocked by Mojo

### ✅ Fixable Now (No Language Limitations)

| Priority | Task | Effort | Impact | Status |
|----------|------|--------|--------|---------|
| ~~0~~ | ~~Fix 26K Limit~~ | ~~High~~ | ~~CRITICAL~~ | ✅ FIXED (scales to 200K+) |
| ~~1~~ | ~~Configuration System~~ | ~~Medium~~ | ~~High~~ | ✅ SIMPLIFIED to Config with 5 fields |
| ~~2~~ | ~~Remove Checksums~~ | ~~Low~~ | ~~High~~ | ✅ REMOVED (+10% performance) |
| ~~3~~ | ~~Fix Result Type~~ | ~~Low~~ | ~~Medium~~ | ✅ FIXED discriminated union |
| 1 | Error Propagation | High | Critical | Replace all panics with Result<T,E> |
| 2 | Memory Safety Audit | Medium | High | Document unsafe operations |
| 3 | OpenTelemetry Support | Medium | High | Industry standard monitoring |
| 4 | Complete Documentation | Medium | Critical | API docs, architecture guide |

### ❌ Blocked by Mojo (Language Limitations)

| Feature | Blocker | Workaround | ETA |
|---------|---------|------------|-----|
| Thread Safety | No Arc, Mutex, atomics | Single-threaded only | Mojo 2026 |
| Module Splitting | Import system limitations | Keep 3300-line monolith | Unknown |
| SIMD Optimizations | Compiler bugs | Use scalar (auto-vectorized) | Next release |
| Async/Await | Not in language | Manual double-buffering | On roadmap |
| Generic Collections | Limited generics | Type-specific implementations | In progress |

## ✅ O(n) to O(1) Optimization - COMPLETED (Aug 28, 2025)

**UPDATE**: Investigation revealed these optimizations were already implemented!
- VectorBuffer has `id_to_index: Dict[String, Int]` at line 25
- BruteForceIndex has `id_to_index: Dict[String, Int]` at line 24
- Both use O(1) lookups in their delete()/remove() methods
- TODO.md was outdated - always verify current implementation

**Result**: ID lookups are already O(1), no optimization needed

## Recent Fixes (August 27-28, 2025)

### ✅ Completed
1. **Python FFI Elimination**: Replaced Python mmap with LibC (50x faster I/O)
2. **DiskANN Compilation**: Fixed all type errors and method signatures
3. **Priority Queue**: Standardized to push/pop interface
4. **O(n²) → O(log n)**: Fixed DiskANN performance (20x improvement)
5. **Iterative Algorithms**: Converted recursion to iteration
   - MinHeapPriorityQueue heapify (was recursive)
   - Quicksort (was recursive)
   - Search now uses heap instead of repeated sorting
6. **Documentation**: Updated STATUS.md, DECISIONS.md with findings
7. **String Serialization**: Fixed persistence - IDs now correctly saved/loaded (Aug 28)
8. **Memory Stats Crash**: Fixed Dict iteration bus error (Aug 28)
9. **Scale Testing**: Verified stability to 100K+ vectors (Aug 28)
10. **O(1) Lookups**: Confirmed already implemented in VectorBuffer and BruteForceIndex (Aug 28)

### 🚧 In Progress
1. **File Creation Fix**: LibC open() with O_CREAT needs adjustment
2. **26K Limit Fix**: Need to integrate FlatAdjacencyList
3. **WAL Python FFI**: Replace Python I/O in storage/wal.mojo

## ~~🔴 CRITICAL: Quantization Completely Broken~~
**UPDATE**: Quantization fixed as of August 26!

### Issues (See docs/CRITICAL_AUDIT_FINDINGS.md)
1. **Double Storage**: Vectors stored in BOTH Dict and CSRGraph
2. **Double Counting**: Memory counted twice in stats  
3. **Data Loss**: Cannot retrieve vectors when quantized
4. **Crash**: Persistence fails with .value() on empty Optional

### Fix Plan
1. [ ] Remove quantized_vectors Dict - let CSRGraph handle it
2. [ ] Fix memory counting - don't double count
3. [ ] Fix get_vector() to work with quantized CSRGraph
4. [ ] Fix persistence Optional crash
5. [ ] Test at 100K scale to verify memory savings

**Expected Impact**: 75% memory reduction (currently -388% WORSE!)

## 🔥 CRITICAL: Fix Recent "Improvements" (1-2 Days)

### 1. Simplify Configuration
- [ ] Reduce OmenDBConfig to 5 essential fields
- [ ] Wire up to Python API properly
- [ ] Test it actually works
- [ ] Remove unused profiles

### 2. Optimize Checksums  
- [ ] Replace CRC32 with xxHash64 (3-5x faster)
- [ ] Make validation opt-in per operation
- [ ] Add SIMD if possible
- [ ] Test actual overhead

### 3. Fix Result Type
- [ ] Remove Optional wrapper (redundant)
- [ ] Use discriminated union pattern
- [ ] Fix map() to work with closures
- [ ] Add and_then(), or_else() methods

### 4. Actually Test Everything
- [ ] Run benchmarks before/after each change
- [ ] Verify configuration works from Python
- [ ] Test checksum correctness
- [ ] Measure real performance impact

## Immediate Priority (After Fixes)

### 1. Data Integrity & Checksums ✅ COMPLETED
- [x] Added CRC32 checksums for all data blocks
- [x] Implemented checksum validation on read (optional)
- [x] Added corruption detection with DataIntegrityManager
- [x] Created basic recovery (skip corrupted blocks)
- [ ] Future: Add redundancy/parity for full recovery

### 2. Error Handling Improvements 🟡 HIGH
- [ ] Replace 11 unsafe_value() calls
- [ ] Implement Result type pattern
- [ ] Add graceful error recovery
- [ ] Create error reporting system

### 2. ~~Fix Vector Normalization~~ ✅ FIXED (Previous)
- [x] CSRGraph now stores both original and normalized
- [x] Retrieval returns original vectors
- [x] User data preserved correctly

### 3. ~~Apply Quantization~~ ✅ PARTIAL (Aug 26)
- [x] Quantization is being applied correctly
- [x] 42% memory reduction achieved
- [ ] **TODO**: Only 22% of theoretical max
- [ ] **TODO**: Implement on-the-fly dequantization
- [ ] **TODO**: Could achieve 8x reduction

### 3. Configuration System Enhancement
- [x] Created OmenDBConfig struct
- [x] Added configuration profiles
- [ ] Implement environment variable loading
- [ ] Add configuration file support (TOML/JSON)
- [ ] Create runtime configuration API
- [ ] Document all configuration options

### 5. Fix Memory Tracking 🔥
- [x] Fixed double counting issue
- [ ] **BROKEN**: Shows 0 MB for small datasets
- [ ] **BROKEN**: Graph memory not updating with quantization
- [ ] Need proper memory accounting

### 4. ~~Segment Merging~~ ✅ FIXED
- [x] Fixed main index replacement issue
- [x] Implemented proper merge logic
- [x] No more duplicates
- [x] Count now accurate

### 5. ~~Memory Optimization~~ ✅ MAJOR PROGRESS
- [x] **FIXED DOUBLE STORAGE**: 26.4x reduction (778MB → 29MB/100K)
- [x] Removed duplicate storage in VectorStore
- [ ] Quantization implemented but not working
- [x] Sparse graph integrated

### 6. Performance Recovery
- [ ] Profile sparse insertion bottlenecks
- [ ] Batch buffer operations properly
- [ ] Consider CSR format for edges
- [ ] Memory pool for neighbor lists

## Production Requirements (Enterprise Grade)

### Phase 1: Safety & Reliability 🔴 CRITICAL
- [ ] **Thread Safety** - Add mutexes/atomics for concurrent access
- [ ] **Error Handling** - Replace all unsafe_value() and panics
- [ ] **Memory Safety** - Wrap all unsafe operations
- [ ] **Data Integrity** - Full WAL with checksums
- [ ] **Crash Recovery** - Consistent state after crashes
- [ ] **Atomic Operations** - No partial updates

### Phase 2: Observability 🟡 HIGH
- [ ] **Metrics** - Prometheus/OpenTelemetry integration
- [ ] **Logging** - Structured logging with levels
- [ ] **Tracing** - Distributed tracing support
- [ ] **Dashboard** - Admin UI for monitoring
- [ ] **Alerts** - Configurable alerting rules
- [ ] **Profiling** - CPU/memory profiling endpoints

### Phase 3: Enterprise Features 🟡 HIGH
- [ ] **Authentication** - User/API key management
- [ ] **Authorization** - Role-based access control
- [ ] **Encryption** - At rest and in transit
- [ ] **Audit Logging** - Compliance tracking
- [ ] **Backup/Restore** - Online backup utilities
- [ ] **Replication** - Master-slave replication
- [ ] **High Availability** - Failover support

### Phase 4: Configuration & Management
- [ ] **Dynamic Config** - Hot reload configuration
- [ ] **Resource Limits** - Memory/CPU limits
- [ ] **Rate Limiting** - Request throttling
- [ ] **Circuit Breakers** - Failure isolation
- [ ] **Health Checks** - Liveness/readiness probes
- [ ] **Graceful Shutdown** - Clean connection draining

## Next Sprint

### Performance Improvements
- [ ] **Beamwidth control** - 30% search speedup potential
- [ ] **Graph pruning** - Try R=32 or R=24 for memory
- [ ] **Angular diversity pruning** - 60% path reduction
- [ ] **Selective neighbor exploration** - Reduce distance calcs

### Search Optimization
- [ ] Target <0.5ms latency @ 100K vectors
- [ ] Implement beam width tuning
- [ ] Cache-friendly graph layout
- [ ] Prefetching for graph traversal

### Production Hardening
- [ ] Thread safety validation
- [ ] Concurrent reader/writer tests
- [ ] Crash recovery testing
- [ ] Error handling improvements

## Blocked by Mojo

### SIMD Implementation
- ⏸️ Waiting for Mojo compiler fixes
- Expected: 2-3x performance improvement
- Distance calculations primary target

### True Async Support
- ⏸️ Waiting for Mojo async/await
- Current workaround: double-buffering
- Would improve checkpoint further

### Memory Pool
- ⏸️ Disabled due to thread safety issues
- 30% performance impact
- Waiting for Mojo Arc/Mutex support

## Version Roadmap

### v0.1.0-dev: Basic Correctness (Target: Q4 2025)
**Goal**: Zero crashes, complete correctness, single-threaded stability

**Must Have:**
- [x] NO panics or crashes
- [x] 100% recovery success rate
- [x] Accurate memory reporting
- [ ] Optimized quantization (8x reduction)
- [ ] O(1) ID lookups (Dict-based)
- [ ] Configuration file support
- [ ] Basic error types (no unwrap/panic)

### v0.2.0: Feature Parity with Chroma (Target: Q1 2026)
**Goal**: Usable for development and prototyping

- [ ] Metadata filtering during search
- [ ] Collections support
- [ ] Batch update/delete
- [ ] Range/radius search
- [ ] Import/export utilities
- [ ] Basic monitoring dashboard
- [ ] Docker container

### v0.3.0: Production Readiness (Target: Q2 2026)
**Goal**: Safe for single-instance production use

- [ ] Thread safety (if Mojo supports)
- [ ] REST API with OpenAPI spec
- [ ] Prometheus metrics
- [ ] Backup/restore tools
- [ ] Performance guarantees (<10ms P95)
- [ ] 99.9% uptime capability
- [ ] Comprehensive test suite

### v1.0.0: Enterprise Grade (Target: Q3 2026)
**Goal**: Competitive with Qdrant

- [ ] Horizontal scaling/sharding
- [ ] Replication and HA
- [ ] Advanced filtering
- [ ] Multiple language SDKs
- [ ] Authentication/authorization
- [ ] Encryption at rest
- [ ] Audit logging

## Bug Fixes

### Critical (DATA LOSS/CORRUPTION)
- [ ] Memory-mapped recovery NOT IMPLEMENTED (TODO stubs)
- [ ] Vector normalization changes user data silently
- [ ] Quantization never applied despite API
- [x] ~~Segment merging broken~~ FIXED: proper merge implemented

### High Priority
- [ ] Quantization not working - missing 4-32x savings
- [x] ~~Memory usage high~~ IMPROVED: 26x reduction (29MB/100K)
- [ ] Memory tracking wildly inaccurate (1476% off)
- [ ] FFI overhead limiting individual inserts

### Low Priority
- [ ] tcmalloc warnings from global variables
- [ ] Doc string warnings during compilation

## Testing Gaps

### Performance Tests Needed
- [ ] Checkpoint with concurrent writes
- [ ] Search during checkpoint
- [ ] Memory usage over time
- [ ] Large vector dimensions (768D, 1536D)

### Correctness Tests Needed
- [ ] Graph connectivity validation
- [ ] Recall measurement at scale
- [ ] Edge case handling
- [ ] Data corruption scenarios

## Research & Analysis

### Competitor Analysis
- [ ] Benchmark against Qdrant 1.11
- [ ] Compare with Weaviate 1.26
- [ ] Test LanceDB v0.13
- [ ] Document advantages/disadvantages

### Algorithm Research
- [ ] Review latest DiskANN papers
- [ ] Investigate DiskANN++ improvements
- [ ] Consider filtered search optimization
- [ ] Explore learned indices

## Release Checklist (v0.3.0)

### Must Have
- [ ] 739K vec/s checkpoint validated
- [ ] Scale tested to 1M vectors
- [ ] Memory optimization implemented
- [ ] Documentation complete

### Nice to Have
- [ ] SIMD if Mojo supports it
- [ ] GPU exploration started
- [ ] Benchmark suite automated
- [ ] CI/CD pipeline

## Questions Requiring Decisions

1. **API Stability**: Freeze Python API now or iterate?
2. **Quantization Method**: Binary, PQ, or SQ?
3. **GPU Priority**: Focus on CPU optimization or explore GPU?
4. **Open Source**: Full OSS or proprietary optimizations?

---

*Use this TODO list to track progress. Update status weekly.*