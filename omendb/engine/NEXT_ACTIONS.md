# Next Actions - Realistic Path Forward

## Current Reality Check
- **Module-level vars**: 2026+ (much later than expected)
- **Parallelize**: Works for math only, not graph updates
- **Performance**: 1,400 vec/s (10% of potential)
- **Memory**: Second batch crashes

## Immediate Actions (This Week)

### 1. Test Specialized SIMD Kernels
```bash
# Integrate specialized_kernels.mojo into HNSW
# Test performance for 128D, 256D, 384D, 512D, 768D, 1536D
# Expected: 2-3x speedup for these dimensions
```

### 2. Implement Server Mode Properly
```python
# Create omendb-server that manages state
# Single long-lived process
# HTTP/gRPC interface
# This solves production use case
```

### 3. Profile Distance Calculations
```bash
# Distance calc is 80% of CPU time
# Can we optimize further?
# Test AVX-512 if available
```

## Short-Term (Next Month)

### 1. Cache-Aligned Memory
```mojo
# Align vectors to 64-byte boundaries
var aligned_ptr = UnsafePointer[Float32].alloc(size, alignment=64)
```

### 2. Memory Layout Optimization
```mojo
# Try Structure of Arrays (SoA) instead of Array of Structures
# Better cache utilization for distance calculations
```

### 3. Batch Distance Matrix
```mojo
# Compute all pairwise distances in one go
# Better cache locality
# Can use parallelize here
```

### 4. Prefetching (if available)
```mojo
# Check if Mojo has prefetch intrinsics
# Prefetch next vector while computing current
```

## Medium-Term Strategy

### 1. Build Production Server
- Rust HTTP/gRPC server
- Calls Mojo for compute
- Manages state properly
- Solves all production issues

### 2. Optimize What We Can
- Single-thread can reach 10K vec/s
- Focus on cache efficiency
- Specialized kernels for all operations

### 3. Create Benchmarking Suite
- Compare against Faiss, Qdrant
- Track progress over time
- Identify bottlenecks

## What NOT to Do
- ❌ Don't try to fix global state (impossible until 2026)
- ❌ Don't parallelize graph updates (no sync primitives)
- ❌ Don't switch to Rust (lose GPU future)
- ❌ Don't wait for Mojo fixes (too long)

## Realistic Performance Targets

### Single-Thread Achievable
- **Current**: 1,400 vec/s
- **With SIMD kernels**: 3,000 vec/s
- **With cache optimization**: 5,000 vec/s
- **With all optimizations**: 10,000 vec/s

### Server Mode
- Handle multiple clients
- Persistent state
- Production ready
- 5,000-10,000 vec/s per instance

## Success Metrics
1. **Single batch**: 10,000 vec/s
2. **Server mode**: Production ready
3. **Documentation**: Clear about limitations
4. **Benchmarks**: Competitive for single-thread

## The Path Forward

### Week 1
- [ ] Integrate specialized SIMD kernels
- [ ] Benchmark improvements
- [ ] Start server implementation

### Week 2
- [ ] Cache alignment
- [ ] Memory layout experiments
- [ ] Server HTTP interface

### Week 3
- [ ] Batch distance operations
- [ ] Profile and optimize hot paths
- [ ] Server testing

### Week 4
- [ ] Documentation update
- [ ] Benchmark suite
- [ ] Production deployment guide

## Key Insight
We can't fix the fundamental Mojo limitations, but we can:
1. **Maximize single-thread performance** (7x improvement possible)
2. **Use server mode** for production (solves state issue)
3. **Document clearly** (set expectations)
4. **Wait strategically** (GPU support worth it)

The goal isn't to match Faiss today - it's to be ready when GPU support lands.