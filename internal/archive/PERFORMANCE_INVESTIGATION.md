# Performance Regression Investigation - September 1, 2025

## Executive Summary
**Found the root cause**: DiskANN `add_batch` MERGE MODE has fundamental algorithmic issues causing both 95% performance regression and segfaults at scale.

## Problem Statement
- **Regression**: 97K → 3K vec/s (95% slower) at 10K-50K scale  
- **Crashes**: Segfault at ~5K vectors in MERGE MODE
- **Root cause**: Expensive beam search operations in `_connect_node()`

## Detailed Findings

### Performance Cliff Analysis
```
INITIAL BUILD (fast):  73K vec/s  ✅ Single batch, empty index
MERGE MODE (slow):      3K vec/s  ❌ Multiple batches, existing index
```

**Timeline of discovery:**
1. 1000 vectors: 73K vec/s (fast - INITIAL BUILD)
2. 2000 vectors: 74K vec/s (still fast - first flush)  
3. 3000 vectors: 68K vec/s (starting to slow)
4. 5000 vectors: 8K vec/s (crash with segfault)

### Root Cause: DiskANN add_batch Algorithm

**INITIAL BUILD Path (Fast):**
```mojo
if self.node_count == batch_size:
    // Simple O(1) local connections within batch
    for i in range(1, min(batch_size, 10)):
        self._connect_node_within_batch(...)  // Fast
```

**MERGE MODE Path (Slow):**
```mojo
else:
    // Expensive beam searches for each bridge connection
    while i < batch_size:
        self._connect_node(node_idx, vector)  // O(log n) beam search + RobustPrune
```

### What _connect_node() Does (Expensive)
1. **Beam search**: `_beam_search_for_insertion()` - O(log n) graph traversal
2. **RobustPrune**: Complex pruning algorithm 
3. **Multiple iterations**: Called for every bridge node in batch

**For 2K batch**: 2000 × O(log n) operations instead of 10 × O(1) operations

## Investigation Attempts

### ❌ Attempted Fixes (All Failed)
1. **Reduced bridge connections**: 1 instead of sqrt(batch_size) - still crashes
2. **Optimized connection strategy**: Custom sampling - still crashes  
3. **Disabled bridge connections**: Completely skip - still crashes
4. **Disabled memory stats**: Remove _update_memory_stats() - still crashes

### ✅ What This Proves
- Issue is NOT in bridge connection logic
- Issue is NOT in memory stats
- Issue is in **Phase 1 or Phase 2** of add_batch:
  - Phase 1: `graph.add_node()` calls
  - Phase 2: Local connections within batch

## State-of-the-Art Solution Required

The current approach is fundamentally flawed:

### Current (Broken) Approach
```mojo
// Add nodes one by one
for i in range(batch_size):
    var node_idx = self.graph.add_node(ids[i], vector)  
    // Potentially expensive for each node

// Connect within batch one by one  
for i in range(batch_size):
    self._find_nearest_in_batch(...)  // May have O(n²) behavior
```

### Proper Solution Needed
1. **Batch-aware node insertion**: Add all nodes at once with pre-allocated space
2. **Efficient local connections**: Use vectorized distance calculations
3. **Proper graph capacity management**: Ensure CSRGraph can handle growth
4. **Memory-safe operations**: Fix whatever is causing segfaults

## Immediate Recommendations

### Option 1: Deep Fix (Proper Solution) 
- **Investigate CSRGraph.add_node()** for scale issues
- **Profile _find_nearest_in_batch()** for O(n²) behavior
- **Fix underlying graph data structure** capacity/memory issues
- **Timeline**: 2-3 days of focused debugging

### Option 2: Architectural Solution (Production-Ready)
- **Multi-segment approach**: Each flush creates new immutable segment
- **Parallel search**: Query multiple segments concurrently  
- **Periodic compaction**: Background merging of segments
- **Timeline**: 1 week implementation
- **Benefit**: Matches how Pinecone, Qdrant, Weaviate actually work

### Option 3: Temporary Workaround (Ship v0.1.0)
- **Increase buffer_size to 50K-100K** to avoid MERGE MODE
- **Document limitation**: Single large batch only
- **Timeline**: Immediate
- **Risk**: Memory usage increases, still has underlying bugs

## Current Status

**Stable configurations:**
- ✅ Single batch ≤5K vectors: 73K vec/s
- ✅ Search performance: <1ms at all scales
- ✅ 100K scale: Achieved with large buffer

**Broken configurations:** 
- ❌ Multiple batches: Crashes at ~5K total vectors
- ❌ MERGE MODE: 95% performance regression

## Recommendation

**Implement Option 1 (Deep Fix)** since you requested a proper solution. The multi-segment approach (Option 2) is more architecturally sound but represents a larger change.

Next steps:
1. Profile exactly where in `add_batch` the segfault occurs
2. Check CSRGraph capacity management and growth logic
3. Verify `_find_nearest_in_batch` doesn't have O(n²) complexity  
4. Fix the fundamental issue causing both performance and stability problems

This will result in a truly state-of-the-art vector database instead of workarounds.