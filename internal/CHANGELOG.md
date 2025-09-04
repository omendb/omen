# OmenDB Development Changelog
*Internal development history - append only*

## v0.0.4-dev (Current)
*August 24, 2025*

### Sparse Graph Integration
- **Implemented complete sparse graph architecture**
  - Dynamic neighbor allocation (8 → ~20 neighbors)
  - Int32 indices instead of Int64 (50% reduction)
  - SparseNeighborList with 2x growth factor
  - 79.2% theoretical reduction in edge storage
- **Integrated into production**
  - Created SparseDiskANNIndex implementation
  - Replaced DiskANNIndex in native.mojo
  - Batch building optimized for 10K+ vectors
  - All APIs compatible with existing code
- **Current issues**
  - Memory still at 778MB/100K (needs debugging)
  - Individual insertion at 1.6K vec/s (was 70K+)
  - Batch performance maintained at 80K+ vec/s

### Memory Tracking Infrastructure
- Implemented idiomatic Mojo memory tracking
  - MemoryTracker struct for allocation monitoring
  - ComponentMemoryStats for breakdown by component
  - Python API: get_memory_stats()
  - Tracks allocations, deallocations, peak usage

### API Enhancements
- **Added beamwidth control**
  - `db.search(vector, beamwidth=50)` for explicit control
  - Auto-selects optimal value when not specified
  - Allows accuracy/speed tradeoff tuning

### Scale Testing
- **Validated at 500K vectors**
  - Insert: 73-75K vec/s (stable across scales)
  - Search: 1.36ms P50 (maintained at 500K)
  - Memory: 146MB per 100K vectors (pre-sparse)
  - Linear scaling confirmed

### Memory Optimization Research
- **Implemented scalar quantization: 33.6x memory compression!**
  - From 1700MB → 50.5MB per 1M vectors
  - Only 3.8% performance overhead
  - Int8 quantization with on-the-fly dequantization
- **Implemented binary quantization: 23.8x compression**
  - 1 bit per dimension storage
  - Vectors only 1.6MB for 1M @ 128D
  - Graph/metadata overhead dominates (105MB fixed)
- Fixed critical double storage bug (was storing both float32 and int8)
- Reduced graph degree R from 64 → 48 (minimal impact)

### Research Completed
- Analyzed state-of-the-art memory techniques (2024-2025)
- Studied Qdrant, MongoDB Atlas, Weaviate implementations
- Identified path to 12-15MB target via binary quantization
- String ID optimization analyzed (5MB savings, deferred)

## v0.0.3-dev
*August 23, 2025*

### Performance Breakthroughs
- Achieved instant checkpoint via buffer swap (microseconds)
- Checkpoint throughput: 46K vec/s (verified)
- Batch insert: 76K vec/s (verified)
  - Fixed copy constructor creating duplicate files (10.7x)
  - Implemented batch memory operations (1.6x)
  - Added async checkpoint with double-buffering (694x)

### Documentation
- Consolidated 30+ MD files into 6 core docs
- Created single-source-of-truth structure
- Fixed conflicting information (dates, performance numbers)

### Storage Architecture
- Discovered WAL is obsolete (2025 research)
- Implemented memory-mapped storage with double-buffering
- Replaced element-by-element copying with batch operations

## v0.0.2-dev
*August 21-22, 2025*

### Bug Fixes
- Fixed critical get_vector() stub bug - was returning None since beginning
- Fixed batch_build partial processing (only processed 1K of 9K vectors)

### Performance
- Achieved 67,065 vec/s with true zero-copy FFI (44.7x improvement)
  - Discovered unsafe_get_as_pointer() method in Modular docs
  - Eliminated Python object conversion overhead

### Testing
- Added regression test system
- Implemented standard benchmark suite

## v0.0.1-dev
*July-August 2025*

### Initial Implementation
- DiskANN algorithm implementation in Mojo
- Python bindings with numpy support
- Basic persistence with checkpoint/recovery
- Deferred indexing with 10K vector buffer

### Performance Baseline
- ~3,250 vec/s with fake zero-copy
- 0.62ms search latency
- 40MB memory per 1M vectors

### Known Issues at Start
- Element-by-element copying (100x slower)
- HNSW migration not completing
- Collections API causing crashes
- No true zero-copy from numpy

---
*Note: All versions are pre-v0.1.0 release*