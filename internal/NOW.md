# NOW - Current Sprint (Feb 2025)

## 🎯 This Week: Multimodal HNSW+ Implementation

### Strategic Pivot ✅
**Decision**: Building multimodal database from start (not pure vector first)
- **Why**: 10x pricing power, less competition, real market pain
- **How**: HNSW+ with integrated metadata filtering and text search

### ✅ Documentation Cleanup Complete
- Consolidated all docs to single source of truth
- Marked DiskANN as deprecated
- Archived ZenDB with preservation notice  
- Created MOJO_WORKAROUNDS.md for limitations
- Created IMPLEMENTATION_CHECKLIST.md for clear roadmap

### 🚀 Ready to Start Implementation
```bash
# Next immediate step:
cd omendb/engine
touch omendb/algorithms/hnsw.mojo

# Then follow IMPLEMENTATION_CHECKLIST.md Phase 1
```

### HNSW+ Implementation Plan
```mojo
# omendb/engine/omendb/algorithms/hnsw.mojo
struct HNSWIndex:
    var layers: List[Graph]         # Hierarchical layers
    var M: Int = 16                 # Neighbors per node
    var ef_construction: Int = 200  # Build parameter
    var entry_point: Int            # Top layer entry
    
    # Multimodal support from start
    var metadata_filter: MetadataIndex
    var text_index: BM25Index
    
    fn insert(self, vector: Vector, metadata: Dict, text: String):
        # Integrated multimodal insertion
        pass
        
    fn hybrid_search(self, vector: Vector, filters: Dict, text: String, k: Int):
        # Combined vector + metadata + text search
        pass
```

### Architecture Decisions ✅
- **Core Engine**: Mojo (CPU/GPU compilation advantage)
- **Server**: Rust HTTP/gRPC wrapper
- **Algorithm**: HNSW+ with metadata filtering
- **Storage**: Tiered (Hot: NVMe, Warm: SSD, Cold: S3)
- **Query Language**: SQL with vector extensions
- **Business Model**: Open source full multimodal, cloud GPU premium

### Success Metrics This Week
- [ ] HNSW+ structure defined
- [ ] Insert function working
- [ ] Search function working  
- [ ] Python binding operational
- [ ] Benchmark: 10K vectors insert < 1 sec

## 🚫 Blockers
None

## 📅 Next Week
- Optimize SIMD distance calculations
- Add parallel layer construction
- Benchmark against pgvector
- Add metadata filtering (multimodal prep)

## 🔧 Quick Commands
```bash
# Build
cd omendb/engine
pixi run mojo build omendb/native.mojo -o python/omendb/native.so

# Test
python -c "from omendb import Index; idx = Index(); print('OK')"

# Benchmark
pixi run benchmark-quick
```

## 📝 Notes
- Ignore DiskANN code, we're replacing it
- Reference: https://github.com/nmslib/hnswlib
- SIMD everything possible
- Focus on single-node first, distributed later