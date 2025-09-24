# OmenDB Architecture

**Status**: FINAL - Ready to Implement
**Date**: September 20, 2025

## What We're Building

A pure Mojo vector database that's fast, simple, and works everywhere.

## Core Design

- **Language**: Pure Mojo (no FFI overhead)
- **Primary Mode**: Embedded (like SQLite)
- **Server Mode**: Python FastAPI wrapper (optional)
- **CPU Algorithm**: HNSW (95% recall, 2-3ms search)
- **GPU Algorithm**: IVF-Flat (10-100x faster)

## How It Works

```python
import omendb

# Simple embedded use
db = omendb.open("vectors.db")
db.add_batch(vectors)     # Must batch (min 100)
db.build_index()          # Explicit control
results = db.search(query) # Fast search
```

## Performance Targets

- **Bulk Load**: 50K+ vec/s
- **Search (CPU)**: 2-3ms with HNSW
- **Search (GPU)**: <1ms with IVF-Flat
- **Memory**: 4GB per million 128d vectors

## Implementation Plan

### Week 1: Core Engine
- Memory-mapped storage
- HNSW implementation
- SIMD distance functions
- Batch operations

### Week 2: GPU Support
- IVF-Flat with Mojo 25.6
- Auto-detect GPU
- Fallback to CPU

### Week 3: Python Integration
- Zero-copy bindings
- NumPy compatibility
- Basic tests

### Week 4: Polish
- FastAPI server (optional)
- Documentation
- Benchmarks
- Launch

## Key Decisions

1. **Pure Mojo**: No FFI overhead (would lose 10-50%)
2. **Batch API**: Real users batch anyway
3. **No Async**: Not needed for CPU-bound ops
4. **HNSW**: Most proven for CPU
5. **IVF-Flat**: Best for GPU parallelism
6. **Python Server**: Just routing, Mojo does work

## What Makes Us Different

- **10x faster** than ChromaDB
- **GPU native** with Mojo 25.6
- **Zero dependencies** for embedded
- **Simple API** that just works

---

*This is the plan. Let's build it.*