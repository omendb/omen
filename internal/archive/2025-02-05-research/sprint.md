# Current Sprint - Week of Feb 5, 2025

## 🎯 This Week: HNSW+ Foundation

### Priority Tasks
```bash
# 1. Refactor docs (IN PROGRESS)
mkdir -p internal/{current,decisions,knowledge}

# 2. Start HNSW+ implementation
cd omendb/engine
touch omendb/algorithms/hnsw.mojo

# 3. Define core structures
# HNSWIndex with hierarchical layers, M=16, ef=200
```

### HNSW+ Implementation Plan
```mojo
struct HNSWIndex:
    var layers: List[Graph]      # Hierarchical structure
    var entry_point: Int         # Top layer entry
    var M: Int = 16              # Neighbors per layer
    var ef_construction: Int = 200   # Build parameter
    
    fn insert(self, vector: Vector, level: Int):
        # TODO: This week
        pass
        
    fn search(self, query: Vector, k: Int) -> List[Int]:
        # TODO: This week  
        pass
```

### Success Metrics
- [ ] HNSW+ structure compiles
- [ ] Insert function working
- [ ] Search function working
- [ ] Python binding operational
- [ ] Benchmark: 10K vectors < 1 sec

## 📅 Daily Progress

### Today
- [x] Document cleanup (847 → 15 files)
- [ ] Create HNSW+ file structure
- [ ] Implement basic HNSWIndex struct

### Tomorrow
- [ ] Implement layer management
- [ ] Add neighbor selection logic
- [ ] Start distance calculations

### Rest of Week
- [ ] SIMD optimization
- [ ] Python FFI integration
- [ ] Basic benchmarking

## 🔧 Quick Commands
```bash
# Build
pixi run mojo build omendb/native.mojo -o python/omendb/native.so

# Test
python -c "from omendb import Index; print('OK')"

# Benchmark  
pixi run benchmark-quick
```