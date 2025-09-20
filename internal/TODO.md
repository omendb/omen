# OmenDB Development Roadmap: Qdrant Segmented HNSW
_Updated: September 20, 2025 - Research-Based Strategy_

## 🎯 **STRATEGIC DECISION: Qdrant Segmented HNSW (Not LanceDB)**
**Research Finding**: In-memory performance priority → Qdrant segmentation superior to LanceDB two-phase
- **Qdrant**: 250+ QPS in-memory (our target market)
- **LanceDB**: 178 QPS in-memory, better out-of-memory (future optimization)

## 🚨 **6-WEEK ROADMAP TO PRODUCTION (20-40K vec/s, 95% recall)**

### **WEEK 1-2: Fix Segmented HNSW Quality (CRITICAL)**
**Problem**: Current 30K+ vec/s but 0% recall due to broken bulk construction
**Solution**: Proper HNSW construction within each segment

- [ ] **Fix segment HNSW construction**
  - [x] Root cause identified: bulk construction skips layer navigation
  - [ ] Implement proper layer-by-layer construction per segment
  - [ ] Maintain hierarchical navigation invariants
  - [ ] Use individual insertion with proper distance calculations
  - [ ] **Target: 8-15K vec/s with 95% recall**

- [ ] **Validate segment result merging**
  - [ ] Fix heap-based ranking across segments
  - [ ] Ensure distance-based result ordering
  - [ ] Test quality at 1K, 5K, 10K vectors
  - [ ] **Quality Gate: ≥95% recall@10 required to proceed**

### **WEEK 3-4: Optimize Segment Parallelism**
**Goal**: True independent segment construction + optimized merging

- [ ] **Independent segment construction**
  - [ ] Build separate HNSW graphs per 10K vectors
  - [ ] Remove any shared state between segments
  - [ ] Implement true parallel segment building
  - [ ] **Target: 15-25K vec/s with 95% recall**

- [ ] **Advanced segment merging**
  - [ ] Heap-based result combination (Qdrant approach)
  - [ ] Quality filtering per segment contribution
  - [ ] Optimize memory allocation per segment
  - [ ] **Performance Gate: 15K+ vec/s required**

### **WEEK 5-6: Production Readiness**
**Goal**: Competitive performance with enterprise features

- [ ] **SIMD optimization per segment**
  - [ ] Dimension-specific distance kernels per segment
  - [ ] Vectorized distance computation within segments
  - [ ] Memory-aligned vector storage per segment
  - [ ] **Target: 20-40K vec/s with 95% recall**

- [ ] **Production features**
  - [ ] Adaptive ef_construction per segment size
  - [ ] Memory pool allocation optimization
  - [ ] Quality monitoring and validation
  - [ ] Concurrent search during construction
  - [ ] **Production Gate: Competitive with Qdrant (20K+ vec/s)**

## 🔧 **SUPPORTING OPTIMIZATIONS (After Core Quality Fixed)**

### **Proven HNSW+ Optimizations**
- [x] **ef_construction = 50** (Qdrant benchmark setting, 2-4x speedup)
- [ ] **SIMD distance kernels** (already working, optimize per segment)
  - [x] Fixed calls to use `_fast_distance_between_nodes()`
  - [ ] Dimension-specific kernels per segment (768D, 1536D)
  - [ ] Vectorized distance matrix computation per segment

### **Future Optimizations (Phase 2)**
- [ ] **Adaptive parameters per segment**
  - [ ] Dynamic ef_construction based on segment size
  - [ ] M parameter optimization per dimensionality
  - [ ] Runtime tuning for different data distributions

- [ ] **Zero-copy FFI** (10% overhead reduction)
  - [ ] NumPy buffer protocol for direct memory access
  - [ ] Eliminate Python → Mojo copy overhead
  - [ ] Memory-mapped vector access

## 📊 **CRITICAL BENCHMARKING (Weekly Gates)**

### **Week 1-2 Quality Gates**
- [ ] **Segmented HNSW validation**
  - [ ] 1K vectors: ≥99% recall@10 (individual insertion baseline)
  - [ ] 5K vectors: ≥95% recall@10 (segment construction test)
  - [ ] 10K vectors: ≥95% recall@10 (single segment validation)
  - [ ] Performance: 8-15K vec/s minimum

### **Week 3-4 Performance Gates**
- [ ] **Parallel segment validation**
  - [ ] 20K vectors (2 segments): ≥95% recall@10
  - [ ] 50K vectors (5 segments): ≥90% recall@10
  - [ ] 100K vectors (10 segments): ≥90% recall@10
  - [ ] Performance: 15-25K vec/s minimum

### **Week 5-6 Production Gates**
- [ ] **Competitive benchmarking**
  - [ ] Install local Qdrant for direct comparison
  - [ ] SIFT1M dataset validation (industry standard)
  - [ ] Memory usage within 2x of Qdrant
  - [ ] **Final Gate: 20K+ vec/s with 95% recall**

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

## 📈 **RESEARCH-VALIDATED PERFORMANCE TARGETS**

| Week | Milestone | Target Speed | Recall | Status | Approach |
|------|-----------|--------------|--------|---------|----------|
| Current | Quality Mode | 3,332 vec/s | 100% | ✅ Baseline | Individual insertion per segment |
| Current | Speed Mode | 30,000 vec/s | 0% | ❌ Broken | Bulk construction without navigation |
| 1-2 | **Fix Quality** | **8-15K vec/s** | **95%** | 🚨 Critical | Proper segment construction |
| 3-4 | **Optimize Parallel** | **15-25K vec/s** | **95%** | 🎯 Target | Independent segments + merging |
| 5-6 | **Production Ready** | **20-40K vec/s** | **95%** | 🏆 Goal | SIMD + enterprise features |
| **FINAL** | **Market Competitive** | **20K+ vec/s** | **95%+** | 🎯 **SUCCESS** | **Qdrant-level performance** |

### **Success Criteria (Non-Negotiable)**
- **Quality**: ≥95% recall@10 on all tests (no exceptions)
- **Speed**: ≥20K vec/s insertion rate (competitive threshold)
- **Memory**: Within 2x of Qdrant memory usage
- **Stability**: Zero crashes, deterministic results

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