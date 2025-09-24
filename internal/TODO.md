# OmenDB Implementation Tasks

**Current State**: Main HNSW bulk construction fixed → 6K vec/s, 99% recall ✅
**Problem**: SegmentedHNSW doesn't use bulk fix → still using individual insertion
**Goal**: Apply bulk construction to segments → 20K+ vec/s target
**Architecture**: CPU-only (Mojo GPU support experimental)

## Phase 2: Apply Bulk Fix to SegmentedHNSW

### [x] 1. Fixed Main HNSW Bulk Construction ✅
- [x] Enabled bulk code (`if False:` → `if True:`)
- [x] Lowered flat buffer threshold (10K → 100)
- [x] Verified: 6K vec/s with 99% recall

### [ ] 2. Apply Fix to SegmentedHNSW
- [ ] **Examine**: `segmented_hnsw.mojo` insertion logic
- [ ] **Apply**: Same bulk construction fix to segments
- [ ] **Test**: Performance and quality at 10K+ vectors

### [ ] 3. Validate Quality at Each Step
- [ ] Test with 100, 1K, 5K vectors
- [ ] Measure recall@10 after each change
- [ ] **Quality gate**: Must maintain >95% recall to proceed

## Phase 2: Segment Parallelism (Week 2-3)

### [ ] 4. Implement Independent Segments
- [ ] Build separate HNSW graphs per 10K vectors
- [ ] Remove any shared state between segments
- [ ] Ensure each segment maintains HNSW invariants

### [ ] 5. Result Merging Across Segments
- [ ] Heap-based combination (Qdrant approach)
- [ ] Distance-based ranking
- [ ] Quality filtering (avoid bad segment matches)

## Phase 3: Optimization (Week 4)

### [ ] 6. Parameter Tuning
- [ ] Use ef_construction=50 (Qdrant production setting)
- [ ] Test M=16 connections per layer
- [ ] Optimize for target 20K+ vec/s

### [ ] 7. Performance Validation
- [ ] Benchmark against Qdrant/Weaviate
- [ ] Measure insertion rate and search recall
- [ ] **Success criteria**: 20K+ vec/s with 95%+ recall

## Success Metrics
- **Quality**: >95% recall@10 on standard benchmarks
- **Speed**: >20K vec/s insertion rate
- **Stability**: Handle 100K+ vectors without crashes

## Future: GPU Support (2026+)
**When Mojo GPU matures** (currently experimental as of Sept 2025):
- [ ] Evaluate Apple Silicon unified memory for vector ops
- [ ] Implement IVF-Flat or similar GPU-friendly algorithm
- [ ] Compare CPU vs GPU performance for different scales
- [ ] **Note**: Mojo can't even run AI models on GPU yet, so this is 1-2 years out