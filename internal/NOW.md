# NOW - Current Sprint (Feb 2025)

## 🎯 Current Status: HNSW+ Feature Parity Required

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

### ✅ HNSW+ Core Complete (Feb 6)
```bash
# Created: omendb/engine/omendb/algorithms/hnsw.mojo
# Status: Core working, but API incompatible with DiskANN
# Test: 100 vectors @ 622 inserts/sec, search in 0.05ms
```

**Completed:**
- ✅ HNSW core algorithm with layers
- ✅ Priority queue for O(log n) search
- ✅ Diversity-based neighbor selection
- ✅ Basic test suite passing
- ✅ Migration strategy documented

**Blocked - API Incompatibility:**
- ❌ No string ID support (only numeric)
- ❌ No batch operations
- ❌ No graph traversal API
- ❌ No quantization support
- ❌ No save/load functionality

**Required for Migration:**
- Add string ID mapping layer
- Implement batch insert
- Create DiskANN-compatible adapter
- Add persistence support

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
- [x] HNSW+ structure defined
- [x] Insert function working
- [x] Search function working  
- [ ] ⚠️ Python binding blocked (API incompatible)
- [x] Benchmark: 100 vectors working

### Migration Path (See HNSW_MIGRATION_STRATEGY.md)
1. **Phase 1**: Add missing features to HNSW
2. **Phase 2**: Create adapter layer
3. **Phase 3**: Gradual migration with testing

## 🚫 Blockers
- HNSW lacks DiskANN API compatibility
- Cannot directly replace in native.mojo
- Need feature parity before migration

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
- **IMPORTANT**: Keep DiskANN until HNSW has feature parity
- Reference: /internal/HNSW_MIGRATION_STRATEGY.md
- Migration requires adapter layer approach
- Test both algorithms in parallel before switching