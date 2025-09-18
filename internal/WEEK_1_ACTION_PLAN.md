# Week 1 Action Plan: Fix Bulk Construction
## Target: 867 → 2,000-5,000 vec/s while maintaining 95%+ recall

## Current Situation (September 17, 2025)
```yaml
Baseline: 867 vec/s, 95.5% recall@10 (stable, good quality)
Problem: Need 23x speedup to reach competitive 20K+ vec/s
Proven: 27,604 vec/s achieved but destroyed quality (1% recall)
Root Cause: Skipping hierarchical navigation breaks HNSW invariants
```

## Week 1 Goal
**Achieve 2,000-5,000 vec/s with 95%+ recall by fixing bulk construction without violating HNSW invariants.**

## Daily Action Plan

### Day 1 (Today): Profile Current Bottlenecks
**Objective:** Identify exactly where the 867 vec/s limitation comes from

#### Tasks:
1. **Add timing measurements to current implementation**
   ```python
   def profile_insertion_detailed():
       timings = {
           'navigation': [],
           'neighbor_search': [],
           'connection_updates': [],
           'memory_allocation': [],
           'distance_calculation': []
       }
       # Measure each component
   ```

2. **Run profiling on current working implementation**
   ```bash
   cd omendb/engine
   pixi run python benchmarks/profile_insertion.py  # Create this
   ```

3. **Expected bottleneck analysis**
   - Distance calculations: ~40% (SIMD broken)
   - Graph traversal: ~30% (cache misses)
   - Memory allocation: ~15% (hot path allocation)
   - Connection management: ~10% (lock overhead)
   - FFI overhead: ~5% (Python ↔ Mojo)

#### Success Criteria:
- [ ] Exact timing breakdown of 867 vec/s implementation
- [ ] Identified slowest component with data
- [ ] Baseline measurement for comparison

### Day 2: Fix Bulk Construction Memory Issues
**Objective:** Make sophisticated bulk construction stable at 20K+ vectors

#### Current Problem:
- Sophisticated bulk achieves 22K+ vec/s but crashes with segfault
- Memory corruption at scale (lines 1406-1577 in hnsw.mojo)

#### Tasks:
1. **Reduce chunk sizes to prevent memory explosion**
   ```mojo
   var chunk_size = 200  # Down from 500 to prevent OOM
   var max_layer_queries = 25  # Down from 50 to prevent corruption
   ```

2. **Add bounds checking everywhere**
   ```mojo
   # Before every pointer access
   if node_id >= 0 and node_id < self.capacity:
       var node = self.node_pool.get(node_id)
       if node:  # Check validity
           # Safe to use
   ```

3. **Fix entry point initialization**
   ```mojo
   # Ensure entry point is set before bulk operations
   if self.entry_point < 0 and actual_count > 0:
       self.entry_point = node_ids[0]
       self.size = 1
   ```

4. **Test incrementally**
   - 1K vectors: Should work
   - 5K vectors: Test stability
   - 10K vectors: Test performance
   - 20K vectors: Test memory limits

#### Success Criteria:
- [ ] No crashes at 20K vectors
- [ ] Maintains hierarchical navigation
- [ ] Achieves 5K+ vec/s
- [ ] Recall stays ≥95%

### Day 3: Test Segment Parallelism Approach
**Objective:** Implement Qdrant-style segment parallelism for independent scaling

#### Approach:
Build independent HNSW segments in parallel, merge results at search time

#### Tasks:
1. **Implement basic segment splitting**
   ```mojo
   fn segment_parallel_insert(vectors, n_vectors):
       var num_segments = 4  # Start small
       var segment_size = n_vectors / num_segments

       # Build independent segments (no shared state)
       for segment in range(num_segments):
           build_independent_hnsw(segment_vectors)
   ```

2. **Test segment quality**
   ```python
   # Each segment should maintain 95%+ recall
   for segment in segments:
       segment_recall = test_segment_quality(segment)
       assert segment_recall >= 0.95
   ```

3. **Implement segment search**
   ```mojo
   fn search_all_segments(query, k):
       var all_results = List[SearchResult]()
       for segment in segments:
           segment_results = segment.search(query, k)
           all_results.extend(segment_results)
       return merge_and_rank(all_results, k)
   ```

#### Success Criteria:
- [ ] Independent segments build without conflicts
- [ ] Each segment maintains 95%+ recall
- [ ] Combined search produces correct results
- [ ] 4x speedup potential demonstrated

### Day 4: Optimize Distance Calculations
**Objective:** Fix SIMD compilation or implement manual vectorization

#### Current Problem:
- SIMD compilation randomly breaks
- Distance calculation is 40% of runtime
- 4-8x speedup possible with working SIMD

#### Tasks:
1. **Test SIMD compilation status**
   ```mojo
   fn test_simd_simple():
       var a = SIMD[DType.float32, 8](1.0)
       var b = SIMD[DType.float32, 8](2.0)
       var c = a + b  # Does this compile?
   ```

2. **If SIMD broken, implement manual batching**
   ```mojo
   fn batch_distance_calc(vectors_a, vectors_b, count):
       # Process 8 distances at once without SIMD
       for i in range(0, count, 8):
           batch_end = min(i + 8, count)
           process_distance_batch(vectors_a[i:batch_end], vectors_b[i:batch_end])
   ```

3. **Optimize memory layout for cache efficiency**
   ```mojo
   # Structure of Arrays instead of Array of Structures
   var x_coords = UnsafePointer[Float32].alloc(n)
   var y_coords = UnsafePointer[Float32].alloc(n)
   # vs interleaved x,y,x,y,...
   ```

#### Success Criteria:
- [ ] Distance calculations 2-4x faster
- [ ] No quality regression
- [ ] Memory layout cache-friendly

### Day 5: Integration and Validation
**Objective:** Combine optimizations and validate against targets

#### Tasks:
1. **Integrate all Week 1 improvements**
   - Profiling-guided optimization
   - Fixed bulk construction
   - Segment parallelism (if working)
   - Optimized distance calculations

2. **Comprehensive testing**
   ```bash
   # Test all scenarios
   pixi run python benchmarks/final_validation.py

   # Specific Week 1 targets
   python test_week1_targets.py  # Create this
   ```

3. **Document results**
   ```markdown
   ## Week 1 Results
   - Baseline: 867 vec/s, 95.5% recall
   - Optimized: X vec/s, Y% recall
   - Improvements: [list what worked]
   - Blockers: [list what didn't work]
   ```

#### Success Criteria:
- [ ] Achieved 2,000+ vec/s (minimum)
- [ ] Maintained 95%+ recall
- [ ] No crashes at 20K+ vectors
- [ ] Clear path to Week 2 goals

## Implementation Guidelines

### HNSW Invariants (NEVER VIOLATE)
1. **Hierarchical Navigation**: Always navigate from entry_point down through layers
2. **Bidirectional Connections**: Maintain A↔B connections
3. **Progressive Construction**: Graph valid after each insertion
4. **Quality Threshold**: Recall@10 ≥ 95%

### Validation After Every Change
```python
def validate_optimization():
    # Test performance
    rate = benchmark_insertion()
    assert rate > 867  # Must improve

    # Test quality
    recall = measure_recall_at_10()
    assert recall >= 0.95  # Must maintain

    # Test stability
    test_20k_vectors()  # Must not crash
```

### If Something Breaks
1. **Revert immediately** - Don't debug broken optimizations
2. **Check invariants** - What constraint was violated?
3. **Smaller steps** - Reduce optimization scope
4. **Test incrementally** - 1K → 5K → 10K → 20K vectors

## Week 1 Success Definition

### Minimum Success
- **2,000+ vec/s** (2.3x improvement)
- **95%+ recall@10** (quality maintained)
- **No crashes at 20K vectors** (stability proven)

### Target Success
- **5,000+ vec/s** (5.8x improvement)
- **96%+ recall@10** (quality improved)
- **Segment parallelism working** (foundation for Week 2)

### Stretch Success
- **8,000+ vec/s** (9.2x improvement)
- **Working SIMD** (foundation for Week 3)
- **Clear path to 20K+ vec/s** (competitive viability)

## Risk Mitigation

### If Bulk Construction Still Crashes
- Fall back to safer batching (current working approach)
- Focus on segment parallelism instead
- Still target 2K+ vec/s with segments

### If SIMD Completely Broken
- Implement manual vectorization
- Focus on memory layout optimization
- Save SIMD for future Mojo compiler fix

### If Quality Degrades
- **STOP immediately** - Speed without quality is useless
- Identify which invariant was violated
- Revert to last known good state
- Try smaller optimization steps

## Week 2 Preview (If Week 1 Succeeds)

**Goal**: 5,000 → 10,000+ vec/s with segment parallelism
- Perfect segment independence
- Lock-free data structures
- Parallel segment construction
- Merge optimization

**Goal**: Position for Week 3 SIMD optimization (10K → 15K vec/s)

---
*Week 1 is about building a stable foundation for Week 2-4 optimizations*
*Success means we can scale to competitive performance without breaking quality*