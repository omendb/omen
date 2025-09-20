# OmenDB Active Development Tasks
_Updated: September 19, 2025_

## 🚨 High Priority - Production Performance

### Fix HNSW Bulk Insertion (CRITICAL)
- [ ] Fix monolithic HNSW bulk construction (currently 0% recall)
  - [x] Identified issue: sophisticated bulk skips navigation
  - [ ] Implement DiskANN two-phase approach
  - [ ] Test recall at 1K, 5K, 10K vectors
  - [ ] Target: 8-12K vec/s with 95% recall

### Enable True Segment Parallelism
- [ ] Fix segmented HNSW at scale (breaks at 3K+ vectors)
  - [x] Individual insertion per segment working (100% recall)
  - [ ] Replace with proper bulk per segment
  - [ ] Enable actual parallel processing (not sequential)
  - [ ] Target: 15-25K vec/s with 90%+ recall

## 🔧 In Progress - Core Optimizations

### Performance Optimizations
- [x] Reduce ef_construction from 200 → 100 → 50 (Qdrant setting)
- [ ] Implement proper bulk insertion with navigation
  - [ ] Pre-allocate all memory upfront
  - [ ] Build graph layer-by-layer (maintain hierarchy)
  - [ ] Batch distance computations

### SIMD & Hardware
- [ ] Fix SIMD distance kernels
  - [ ] Migrate from advanced_simd.mojo to specialized_kernels.mojo
  - [ ] Verify AVX-512 usage on supported hardware
  - [ ] Target: 4-8x speedup on distance calc

### Zero-Copy FFI
- [ ] Implement NumPy buffer protocol
  - [ ] Direct memory access from Python
  - [ ] Eliminate copy overhead (currently 10%)
  - [ ] Target: 50% reduction in overhead

## 📊 Testing & Validation

### Benchmarking
- [ ] Create comprehensive benchmark suite
  - [ ] Compare against Qdrant locally
  - [ ] Test with SIFT1M dataset
  - [ ] Measure insertion rate + recall@10
  - [ ] Document memory usage

### Quality Assurance
- [ ] Validate recall at different scales
  - [ ] 1K vectors: >99% recall required
  - [ ] 10K vectors: >95% recall required
  - [ ] 100K vectors: >90% recall required
  - [ ] 1M vectors: >85% recall acceptable

## 🗂️ Documentation & Cleanup

### Documentation Updates
- [x] Update agent-contexts submodule
- [x] Create internal/TODO.md
- [ ] Update internal/STATUS.md with latest findings
- [ ] Consolidate research into internal/RESEARCH.md
- [ ] Create internal/DECISIONS.md
- [ ] Archive weekly logs

### Code Cleanup
- [ ] Remove test files (test_*.py)
- [ ] Clean up debug output from production code
- [ ] Remove WIP implementations that failed

## ✅ Completed This Session

### September 19, 2025
- [x] Fixed segmented HNSW recall with quality filtering (100% recall achieved)
- [x] Identified root cause: segments returning bad matches
- [x] Fixed with smart quality threshold (absolute for close matches)
- [x] Fixed with individual insertion per segment (maintains connectivity)
- [x] Achieved 3,332 vec/s with 100% recall at 2500 vectors
- [x] Reduced ef_construction from 100 to 50 (Qdrant optimization)

### Previous Session
- [x] Fixed memory corruption in SegmentedHNSW (lazy initialization)
- [x] Fixed SIMD bottleneck (6.15x speedup: 867 → 5,329 vec/s)
- [x] Identified bulk insertion breaks graph connectivity

## 📈 Performance Targets

| Milestone | Target Speed | Recall | Status |
|-----------|--------------|--------|---------|
| Current Baseline | 867 vec/s | 95.5% | ✅ Stable |
| With ef=50 | 2,000 vec/s | 95% | 🔄 Testing |
| DiskANN Bulk | 8-12K vec/s | 95% | ⏳ Pending |
| Segment Parallel | 15-25K vec/s | 90%+ | ⏳ Pending |
| Production Ready | 20K+ vec/s | 90%+ | 🎯 Target |

## 🔍 Known Issues

1. **Monolithic HNSW bulk**: 0% recall with sophisticated bulk construction
2. **Segmented at scale**: 0% recall at 3000+ vectors (accumulation issue)
3. **No real parallelism**: Everything sequential despite "parallel" names
4. **Memory overhead**: Still copying despite zero-copy attempts

## 💡 Research Questions

- [ ] Why does bulk insertion break even with proper navigation?
- [ ] Can we use lock-free structures without breaking connectivity?
- [ ] Is there a better segment merging strategy than Qdrant's?
- [ ] Should we try Vamana instead of HNSW for Mojo?

---
_Note: Tasks marked with 🚨 are blocking production readiness_