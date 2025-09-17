# HNSW+ Migration Strategy

## Status: February 6, 2025

### ✅ Completed
1. **Core HNSW Implementation** (`omendb/algorithms/hnsw.mojo`)
   - Basic insert/search working
   - Priority queue integrated for O(log n) operations
   - Diversity-based neighbor selection
   - Test suite passing (100 vectors @ 622 inserts/sec)

2. **Compilation Issues Fixed**
   - Added `__copyinit__` and `__moveinit__` for Mojo compatibility
   - Fixed time imports (`perf_counter_ns` instead of `time_ns`)
   - Simplified SIMD (needs optimization later)

### ❌ Migration Blockers

The direct migration from DiskANN to HNSW in native.mojo failed due to fundamental API differences:

| Feature | DiskANN | HNSW (current) | Gap |
|---------|---------|----------------|-----|
| ID Management | String IDs via `add(id, vector)` | Numeric only via `insert()` | Need ID mapping |
| Batch Operations | `add_batch()` | None | Need batch insert |
| Graph Access | `graph.get_node_index()` | Internal only | Need accessor methods |
| Quantization | Built-in PQ support | None | Need to add |
| Iteration | Can iterate nodes | No support | Need iterator |
| Persistence | Supports save/load | None | Need serialization |

### 🎯 Recommended Migration Path

#### Phase 1: Feature Parity (Week 1)
```mojo
# Extend HNSW with required features
struct HNSWIndex:
    var id_map: Dict[String, Int]  # String ID to numeric ID
    
    fn add(self, id: String, vector: Pointer) -> Bool:
        var numeric_id = self.insert(vector)
        self.id_map[id] = numeric_id
        return True
    
    fn add_batch(self, ids: List[String], vectors: Pointer, count: Int):
        for i in range(count):
            self.add(ids[i], vectors.offset(i * self.dimension))
```

#### Phase 2: Adapter Layer (Week 2)
```mojo
# Create adapter that wraps HNSW with DiskANN-compatible API
struct HNSWAdapter:
    var index: HNSWIndex
    var id_mapping: SparseMap[String, Int]
    
    # Implement full DiskANN API
    fn add(self, id: String, vector) -> Bool
    fn add_batch(self, ids, vectors, count)
    fn search(self, query, k) -> List[SearchResult]
    fn save(self, path: String)
    fn load(self, path: String)
```

#### Phase 3: Gradual Migration (Week 3)
1. Keep both algorithms available
2. Add runtime flag to choose algorithm
3. Run parallel tests to validate correctness
4. Profile performance differences
5. Switch default once stable

### 🔧 Implementation Tasks

1. **Immediate (Before Integration)**
   - [ ] Add string ID mapping to HNSW
   - [ ] Implement batch operations
   - [ ] Add save/load functionality
   - [ ] Create comprehensive test suite

2. **Integration Phase**
   - [ ] Create HNSWAdapter with DiskANN API
   - [ ] Update native.mojo to use adapter
   - [ ] Maintain backward compatibility
   - [ ] Add algorithm selection flag

3. **Optimization Phase**
   - [ ] Re-enable SIMD optimizations
   - [ ] Implement RobustPrune
   - [ ] Add quantization support
   - [ ] Profile and tune parameters

### 📊 Success Metrics
- [ ] Feature parity with DiskANN
- [ ] No regression in Python API
- [ ] Better streaming insertion performance
- [ ] Recall@10 > 95%
- [ ] Memory usage < DiskANN

### ⚠️ Lessons Learned
1. **Don't rush migration** - API compatibility is critical
2. **Test incrementally** - Each feature needs validation
3. **Maintain compatibility** - Users depend on current API
4. **Document differences** - Clear migration guide needed

---
*Use this strategy for systematic HNSW integration*