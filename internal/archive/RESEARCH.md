# HNSW Research Findings

## Competitor Algorithm Analysis

### Qdrant (20-50K vec/s)
- **Segmented HNSW**: Independent 10K vector segments, parallel construction
- **Proper bulk construction**: Distance computation → graph building (two phases)
- **Result merging**: Heap-based combination across segments
- **Parameters**: ef_construction=50-100 in production (not 200)

### Weaviate (15-25K vec/s)
- **Lock-free reads**: Concurrent search during construction
- **Memory pools**: Pre-allocated node pools (avoid allocation overhead)
- **Adaptive ef_construction**: Dynamic parameter tuning

### Pinecone (10-30K vec/s)
- **Hierarchical merging**: Multi-level segment combination
- **Quality monitoring**: Continuous recall validation

## Critical HNSW Construction Rules (Literature)

### Why Bulk Insertion Fails
1. **Navigation requirement**: Must traverse entry_point → target_layer
2. **Connection order**: Bidirectional links must be maintained
3. **Layer structure**: Upper layers are "highways" to distant regions

### Proven Approaches That Work
- **Individual insertion**: Always works, maintains invariants
- **Segment parallelism**: Independent graphs avoid race conditions
- **Two-phase bulk**: Compute distances first, then build connections

### Parameters That Matter
- **ef_construction=50**: Qdrant production setting (not 200)
- **M=16**: Connections per layer
- **Segment size=10K**: Balance parallelism vs memory

## Key Research Insights

### Speed vs Quality Trade-off
Can get 30K+ vec/s by skipping navigation, but destroys recall. All successful implementations preserve HNSW invariants.

### Code Analysis (Sept 24, 2025)
**File**: `omendb/algorithms/hnsw.mojo`
- **Line 1271**: `insert_bulk()` function exists but...
- **Line 1435**: Falls back to individual `_insert_node()` for each vector
- **Line 1452**: `if False:` - actual bulk optimization code is DISABLED
- **Line 2866-2868**: `_insert_node()` properly navigates entry_point → target_layer (works)
- **Line 3174**: `_insert_node_simplified()` exists but unused

**Root Cause**: The optimized bulk code that would give 30K+ vec/s is completely disabled. Currently using individual insertion which maintains quality but is slow (3.3K vec/s).

### GPU Support Reality (Sept 2025)
- **Mojo Apple Silicon GPU**: Announced Sept 21, 2025 (experimental)
- **Current limitations**: No AI models, limited operations, basic compute only
- **Vector DB implications**: Not viable for production yet
- **Strategy**: Focus on CPU optimization first, GPU later when mature

### Apple Silicon GPU Potential
- **Hardware**: M1-M4 have powerful unified memory architecture
- **Advantage**: No PCIe bottleneck, shared memory with CPU
- **Challenge**: Mojo GPU support still experimental (can't even run PyTorch)
- **Timeline**: Likely 1-2 years before production-ready for vector operations