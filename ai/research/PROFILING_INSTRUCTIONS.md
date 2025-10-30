# Profiling Instructions for Fedora

**Goal**: Profile OmenDB to identify optimization opportunities beyond SIMD

**Current Performance** (with SIMD):
- 581 QPS (approaching Qdrant's 626 QPS @ 99.5% recall)
- Query latency: 1.72ms avg, 2.08ms p95
- Build speed: 6540 vec/sec

**Target**: 600-800 QPS (exceed Qdrant)

---

## Step 1: Install Profiling Tools (One-time Setup)

**On Fedora** (requires your password):
```bash
sudo bash ~/setup_profiling_tools.sh
```

**What it installs**:
- `perf`: CPU profiling (for flamegraph)
- `heaptrack`: Memory allocation profiling
- `valgrind`: Memory debugging
- Sets up permissions for non-root profiling

**Time**: ~2-3 minutes

---

## Step 2: Run Profiling Suite

**On Fedora** (no sudo needed):
```bash
cd ~/github/omendb/omen
bash run_profiling.sh
```

**What it does**:
1. **Flamegraph** (CPU hotspots) - Visual representation of where CPU time is spent
2. **Heaptrack** (memory allocations) - Identify allocation hotspots
3. **Perf stat** (performance counters) - Cache misses, branch mispredictions

**Time**: ~10-15 minutes

**Output files**:
- `flamegraph_queries.svg` - CPU hotspots (open in browser)
- `heaptrack.benchmark_pgvector_comparison.*.zst` - Memory data
- `perf_stat_output.txt` - Performance counters

---

## Step 3: Download Results (On Mac)

```bash
# Download flamegraph
scp nick@fedora:~/github/omendb/omen/flamegraph_queries.svg .

# Download perf stats
scp nick@fedora:~/github/omendb/omen/perf_stat_output.txt .

# Open flamegraph in browser
open flamegraph_queries.svg
```

---

## Step 4: Analysis (Claude will do this)

**Expected bottlenecks** (based on research):
1. **Distance calculations** (hot path) - Should be fast with SIMD, verify
2. **Memory allocations** (temporary buffers) - 10-20% improvement possible
3. **HNSW graph traversal** (cache misses) - Memory layout optimization
4. **Vector allocations** (per-query) - Object pooling opportunity

**Target optimizations**:
- Object pooling for query buffers
- Better memory layout for cache efficiency
- Reduce allocations in hot paths

**Expected improvement**: 10-30% additional (581 → 640-755 QPS)

---

## Alternative: Skip Profiling

If profiling is blocked, we can implement **known optimizations** based on research:

1. **Object pooling** (10-20% improvement)
2. **Cache optimization** (10-20% improvement)
3. **Query batching** (20-50% throughput)

**Total**: 581 QPS × 1.4-1.8x = 813-1045 QPS (beat Qdrant)

---

**Scripts created**:
- `~/setup_profiling_tools.sh` - Install profiling tools (run with sudo)
- `~/github/omendb/omen/run_profiling.sh` - Run profiling suite
