# HNSW Debugging Findings

## Issues Identified and Fixed

### 1. Graph Connectivity (Partially Fixed)
**Issue**: Only 60% of nodes were reachable due to limited sampling in `_fast_individual_connect`
**Fix Applied**: Increased sampling from 20 nodes to min(100, size/10) nodes
**Result**: Improved recall from 0% to 10-15% at 1000 vectors

### 2. Search Parameters (Fixed)
**Issue**: ef was set to just M (16) instead of ef_construction (200)
**Fix Applied**: Changed to use ef_construction for insertion
**Result**: Better exploration during insertion

### 3. Candidate Exploration (Fixed)
**Issue**: Artificial limit of ef//2 candidates during search
**Fix Applied**: Removed the limit, explore all ef candidates
**Result**: More thorough search

### 4. Double Size Counting (Fixed)
**Issue**: Size was being updated twice in bulk insertion
**Fix Applied**: Removed duplicate size update
**Result**: Correct size tracking

## Major Issue Still Present

### Bulk Insertion Graph Connectivity
**Root Cause**: Bulk insertion doesn't properly navigate the graph hierarchy
- Individual insertion: Navigates from entry point through all layers
- Bulk insertion: Processes each layer independently without navigation

**Impact**:
- 100 vectors: 70% recall (improved from 0%)
- 500 vectors: 15% recall
- 1000 vectors: 10% recall  
- 2000+ vectors: 0% recall

**Evidence**:
- Individual-only insertion: 100% recall
- Bulk-only insertion: 100% recall
- Mixed (individual then bulk): 30% recall for bulk part

## Recommendations

### Short Term
1. Use individual insertion when quality matters
2. Use flat buffer for <500 vectors (100% accurate, 2-4x faster)
3. Warn users about quality issues at scale

### Long Term
1. Refactor bulk insertion to navigate hierarchy properly
2. Implement proper entry point selection for each node
3. Add quality gates to prevent shipping with low recall

## Test Results

### Performance
- Individual insertion: ~5K vectors/sec
- Bulk insertion: ~10K vectors/sec
- Flat buffer: ~20K vectors/sec at small scale

### Quality
- Flat buffer: 100% recall (ground truth)
- HNSW at 100 vectors: 70% recall
- HNSW at 1000+ vectors: <10% recall (unacceptable)

## Code Locations
- Connectivity fix: hnsw.mojo:1722
- Search parameter fix: hnsw.mojo:1906
- Candidate exploration fix: hnsw.mojo:1930
- Size counting fix: hnsw.mojo:1173

## Next Steps
1. Major refactor of bulk insertion to navigate hierarchy
2. Or switch to individual insertion for quality
3. Or use flat buffer for small datasets
4. Add comprehensive quality testing before any optimization