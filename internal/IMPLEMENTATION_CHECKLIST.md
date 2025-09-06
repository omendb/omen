# Implementation Checklist for HNSW+ Multimodal Database

## Pre-Implementation ✅ COMPLETE
- [x] Documentation consolidated and organized
- [x] DiskANN code marked as deprecated
- [x] ZenDB archived with preservation notice
- [x] Mojo workarounds documented
- [x] Architecture designed

## Phase 1: HNSW+ Core (Week 1)

### File Structure Setup
```bash
# Create new algorithm file
touch omendb/engine/omendb/algorithms/hnsw.mojo

# Create multimodal storage
mkdir -p omendb/engine/omendb/storage/multimodal
touch omendb/engine/omendb/storage/multimodal/metadata.mojo
touch omendb/engine/omendb/storage/multimodal/text_index.mojo
```

### Core HNSW+ Implementation
- [ ] Define HNSWIndex struct with layers
- [ ] Implement level assignment (exponential decay)
- [ ] Create insert function with neighbor selection
- [ ] Build hierarchical search function
- [ ] Add SIMD distance calculations

### Integration Points
- [ ] Update native.mojo to use HNSW instead of DiskANN
- [ ] Modify Python bindings for new API
- [ ] Ensure zero-copy FFI working

## Phase 2: Multimodal Features (Week 2)

### Metadata Filtering
- [ ] Create MetadataStore with columnar storage
- [ ] Implement B-tree indexes for fast filtering
- [ ] Integrate filtering with HNSW traversal
- [ ] Test filter-first vs vector-first performance

### Text Search (BM25)
- [ ] Implement inverted index structure
- [ ] Add BM25 scoring algorithm
- [ ] Create text tokenization (use Python for now)
- [ ] Integrate with vector search

## Phase 3: Query Planning (Week 3)

### Query Optimizer
- [ ] Implement selectivity estimation
- [ ] Create cost model for each operation
- [ ] Build adaptive query planner
- [ ] Add query statistics collection

### SQL Interface
- [ ] Design SQL extensions for vectors
- [ ] Create basic SQL parser (or use Python lib)
- [ ] Map SQL to internal operations
- [ ] Test hybrid queries

## Phase 4: Production Features (Week 4)

### Storage Optimization
- [ ] Implement tiered storage (hot/warm/cold)
- [ ] Add memory-mapped file support
- [ ] Create compaction process
- [ ] Test with 1M+ vectors

### Performance Tuning
- [ ] Benchmark against pgvector
- [ ] Profile and optimize hot paths
- [ ] Add parallel insertion support
- [ ] Implement batch operations

## Testing Strategy

### Unit Tests
```python
# test_hnsw.py
def test_insertion():
    index = HNSWIndex(dimension=128, M=16)
    vectors = generate_random_vectors(1000, 128)
    index.add_batch(vectors)
    assert index.size() == 1000

def test_search_accuracy():
    # Test recall@10 >= 95%
    pass

def test_metadata_filtering():
    # Test filter-first execution
    pass
```

### Benchmarks
```python
# benchmark_multimodal.py
def benchmark_hybrid_search():
    # Vector + text + metadata
    # Target: <10ms for 1M vectors
    pass
```

## Code Patterns to Follow

### Mojo Patterns
```mojo
# Always use SIMD for distance calculations
alias simd_width = simdwidthof[DType.float32]()

# Use RAII for memory management
struct Buffer:
    fn __del__(owned self):
        self.data.free()

# Batch operations for FFI
fn add_batch(vectors: List[Vector]):
    # Single FFI call, not loop
```

### Avoid These
- ❌ Mojo stdlib Dict/List (huge overhead)
- ❌ Scalar loops (use SIMD)
- ❌ Multiple FFI calls (batch instead)
- ❌ DiskANN patterns (different algorithm)

## Success Metrics

### Week 1
- [ ] 10K vectors insert < 1 second
- [ ] Search recall@10 > 90%
- [ ] Python bindings working

### Week 2
- [ ] Metadata filtering functional
- [ ] Text search returning results
- [ ] Combined queries working

### Week 3
- [ ] Query planner choosing optimal path
- [ ] SQL interface parsing queries
- [ ] <10ms latency for hybrid queries

### Week 4
- [ ] 1M vectors handled efficiently
- [ ] Memory usage < 4 bytes/vector
- [ ] Benchmarks beating pgvector

## Common Pitfalls

1. **Don't use DiskANN code** - Different algorithm entirely
2. **Check MOJO_WORKAROUNDS.md** - For language limitations
3. **Batch everything** - Single operations have FFI overhead
4. **Test incrementally** - Don't wait until end
5. **Profile early** - Find bottlenecks before optimizing

## Resources

- **Architecture**: `/internal/architecture/MULTIMODAL.md`
- **Mojo patterns**: `/internal/KNOWLEDGE.md#mojo-patterns`
- **Workarounds**: `/internal/MOJO_WORKAROUNDS.md`
- **Decisions**: `/internal/DECISIONS.md`
- **Current tasks**: `/internal/NOW.md`

## Questions to Answer

Before implementing each component, ask:
1. Is there existing code to reuse? (Check non-deprecated files)
2. What's the Mojo workaround? (Check MOJO_WORKAROUNDS.md)
3. How does this integrate? (Check architecture docs)
4. What's the performance target? (Check success metrics)

---
*Start with Phase 1, test incrementally, ask for help when blocked*