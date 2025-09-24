# 🎯 OmenDB Architecture - Final Decision

**Date**: September 20, 2025
**Status**: DECIDED - Ready to implement

## The Decision

**Pure Mojo + HNSW/IVF-Flat + Python Server (when needed)**

No more debates. No more analysis paralysis. This is what we're building.

## What We're Building

### Core Engine (Pure Mojo)
```mojo
struct OmenDB:
    var storage: MMapStorage      # Memory-mapped vectors
    var index: AdaptiveIndex      # HNSW or IVF-Flat

    fn add_batch(vectors: Tensor) -> List[Int]:
        # Batch operations only (min 100)
        return self.storage.append(vectors)

    fn search(query: Tensor) -> Results:
        if GPU.available():
            return self.index.search_gpu(query)  # <1ms
        else:
            return self.index.search_cpu(query)  # 2-3ms
```

### Index Strategy
- **< 10K vectors**: Flat search (100% recall)
- **CPU (10K-1M)**: HNSW (95% recall, 2-3ms)
- **GPU (any size)**: IVF-Flat (90-95% recall, <1ms)

### API Design
```python
import omendb

# Embedded mode (primary)
db = omendb.open("vectors.db")
db.add_batch(vectors)  # 50K+ vec/s
results = db.search(query, k=10)

# Server mode (optional)
pip install omendb[server]
omendb serve --db vectors.db --port 8000
```

## Why This Architecture Wins

### 1. No FFI Overhead
- Pure Mojo = no boundary crossings
- Would lose 10-50% with Rust+Mojo hybrid

### 2. GPU Ready Today
- Mojo 25.6 supports Metal/NVIDIA/AMD
- IVF-Flat perfect for GPU parallelism
- 10-100x speedup available now

### 3. Matches Real Usage
- Users batch load millions
- Nobody inserts one vector at a time
- Explicit index building preferred

### 4. Simple Deployment
- Single binary for embedded
- Python wrapper for server
- No complex dependencies

## What About Our Concerns?

### "Won't FFI have major overhead?"
**YES** - That's why we chose pure Mojo. FFI would add 10-50% overhead.

### "Is HNSW+ still best?"
**For CPU, yes**. For GPU, IVF-Flat is better. We support both.

### "Which index for Mojo right now?"
**HNSW for CPU** (proven), **IVF-Flat for GPU** (simple, fast).

### "Embedded vs Server mode?"
- **Embedded first** - Most users start here
- **Python server** - Simple FastAPI wrapper when needed
- **Not Rust** - Unnecessary complexity for routing

### "Python or Rust for server?"
**Python** - Just routing, Mojo does work. Can migrate later if needed.

## Performance Expectations

### What We'll Achieve
```
Bulk Load: 50K+ vec/s (competitive)
Search CPU: 2-3ms (excellent)
Search GPU: <1ms (best-in-class)
Memory: 4GB per million vectors
```

### How We Compare
```
ChromaDB: 1-3K vec/s → We're 15-50x faster
Weaviate: 10-15K vec/s → We're 3-5x faster
Qdrant: 15-20K vec/s → We're 2-3x faster
+ GPU acceleration none of them have natively
```

## Implementation Plan

### Week 1: Core Engine
- [ ] MMapStorage for vectors
- [ ] HNSW implementation
- [ ] Batch operations
- [ ] SIMD distance functions

### Week 2: GPU Support
- [ ] IVF-Flat index
- [ ] GPU kernel with Mojo 25.6
- [ ] Auto-detection logic

### Week 3: Python Integration
- [ ] Zero-copy bindings
- [ ] NumPy compatibility
- [ ] Basic tests

### Week 4: Server & Polish
- [ ] FastAPI wrapper
- [ ] Documentation
- [ ] Benchmarks
- [ ] Launch

## Files to Clean Up

### Keep
- `omendb/engine/` - Mojo implementation
- `internal/MASTER_ARCHITECTURE_DECISION_2025.md` - Final architecture
- `internal/RESEARCH_CONSOLIDATED_2025.md` - Research findings

### Remove (Redundant/Outdated)
- All conflicting architecture docs
- Old Rust+Mojo hybrid plans
- Segmented HNSW experiments
- `zendb/` directory

### Defer
- `omendb/server/` - Rust server (use Python instead)
- `omendb/web/` - Marketing site (not priority)

## Critical Code Paths

### Distance Function (SIMD)
```mojo
@always_inline
fn distance_simd(a: Pointer[Float32], b: Pointer[Float32], d: Int) -> Float32:
    var sum = SIMD[DType.float32, 16](0)
    for i in range(0, d, 16):
        var diff = a.load[width=16](i) - b.load[width=16](i)
        sum += diff * diff
    return sum.reduce_add()
```

### GPU Search (IVF-Flat)
```mojo
@parameter
fn search_gpu[device: GPU.Device](query: Tensor, data: Tensor, k: Int):
    var distances = gpu.matmul(query, data.T)
    return gpu.topk(distances, k)
```

## The Bottom Line

**Decision made. Architecture final. Let's build.**

1. Pure Mojo (no FFI)
2. HNSW for CPU, IVF-Flat for GPU
3. Batch-first API
4. Python server wrapper
5. Ship in 4 weeks

Stop analyzing. Start coding.

## Next Action

```bash
cd omendb/engine
pixi run mojo build omendb/native.mojo
```

Let's ship the fastest vector database.