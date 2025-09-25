# Algorithm Migration Status

**Date**: July 29, 2025  
**Status**: RoarGraph → HNSW Migration Complete

## Migration Summary

### What Happened
- **RoarGraph** was our initial algorithm choice claiming "5-10x faster than HNSW"
- Testing revealed RoarGraph takes **minutes** to build (473s for 50K vectors)
- Faiss is **4,470x faster** than our RoarGraph implementation
- We migrated to standard HNSW in the public omenDB repo

### Current State
- **Public omenDB**: Uses HNSW (brute force < 5K vectors, HNSW ≥ 5K)
- **Private omendb-server**: RoarGraph archived here for potential GPU optimization
- **Performance**: Query latency good (0.2ms) but construction slow (144 vec/s)

### Critical Issues Found
1. **Memory Leaks**:
   - 10MB leaked per DB instance creation
   - 246MB leaked during data churn operations
   - 6.7GB memory usage for just 50K vectors

2. **Old native.so Issue**:
   - Compiled binaries can contain old algorithms
   - Always rebuild after algorithm changes
   - Check file dates: `ls -la python/omendb/*.so`

3. **Performance Reality**:
   - We claimed 5K vec/s but that's only for brute force
   - HNSW construction is 144 vec/s (needs 1000+ target)
   - Migration process is blocking and slow

## Lessons Learned

### Algorithm Development
- **Test at scale early** - RoarGraph worked for small tests but failed at scale
- **Benchmark honestly** - Compare against established solutions (Faiss, HNSW)
- **Memory profiling critical** - Leaks compound quickly in production

### Architecture Decisions
- Keep experimental algorithms in private repo
- Use proven algorithms in public releases
- Don't claim performance without thorough testing

## Future Strategy

### For Public omenDB
1. Fix HNSW memory leaks
2. Optimize construction speed
3. Consider pure brute force for v0.1.0 if HNSW isn't ready

### For Private omendb-server
1. GPU-accelerated RoarGraph research
2. Explore hybrid CPU/GPU approaches
3. Keep experimental work isolated

## Technical Details

### HNSW Parameters (current)
```mojo
M=32                    # Connections per layer
ef_construction=400     # Construction search width
ef=32                   # Query search width
max_M=64               # Max connections
```

### Memory Usage Breakdown
- Base overhead: ~6GB (Python + libraries)
- Per vector: ~134KB (way too high!)
- Target: <1KB per vector

### Build Process
```bash
# Always rebuild native module after algorithm changes
pixi run mojo build omendb/native.mojo --emit shared-lib -o python/omendb/native.so

# Verify the build date
ls -la python/omendb/native.so
```

## Recommendations

**For v0.1.0 Release**:
1. Consider pure brute force only (remove HNSW migration)
2. Be honest about performance limitations
3. Fix memory leaks before any release
4. Remove false "5-10x faster" claims

**Long Term**:
1. Properly optimize HNSW or find alternative
2. Establish rigorous benchmarking process
3. Keep experimental work in private repo