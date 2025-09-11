# ACTION PLAN - Getting Back to Good Performance
*2025-02-11 - Clear steps to fix our regression*

## 📊 Current Situation

We had a working system at 3,000-5,000 vec/s but regressed to 907 vec/s with HNSW+.
The issue is NOT the algorithm - it's our implementation.

## 🎯 Target: 20,000+ vec/s (20x improvement)

This is achievable because:
- Competitors achieve 25K-500K vec/s with HNSW
- Our earlier version was 3-5x faster
- Issues are implementation, not fundamental

## 📋 IMMEDIATE FIXES (Do Now)

### 1. Simplify Graph Search (1 hour)
**File:** `omendb/algorithms/hnsw.mojo`
**Line:** 1927
```mojo
# Change from:
var ef = max(M * 2, 32)  # Current - exploring 32+ candidates

# To:
var ef = M  # Only explore 8 candidates
```
**Expected Impact:** 2-4x speedup

### 2. Remove Unnecessary Features (1 hour)
**Disable these "optimizations" that slow us down:**
- Hub Highway (lines 698-796) - Complex and unproven
- Smart distance switching (lines 603-612) - Overhead > benefit
- VSAG optimizations - Not helping

**Just use simple L2 distance:**
```mojo
fn distance(self, a: UnsafePointer[Float32], b: UnsafePointer[Float32]) -> Float32:
    var sum = Float32(0)
    for i in range(self.dimension):
        var diff = a[i] - b[i]
        sum += diff * diff
    return sqrt(sum)
```

### 3. Fix Binary Quantization (2 hours)
**Current:** Creating binary vectors eagerly during insertion
**Fix:** Only create when actually used in search
```mojo
# Remove from insert()
# Add lazy creation in search when needed
```

## 🔧 MEDIUM-TERM FIXES (This Week)

### 1. Dynamic Capacity (Partially Done)
- Created `hnsw_dynamic.mojo` with growth logic
- Need to integrate and test
- **Impact:** Removes scale limitations

### 2. Object Pooling (Started)
- Pool search buffers to avoid allocations
- Reuse KNNBuffer objects
- **Impact:** 10-20% speedup

### 3. Batch Insertion
- Process multiple vectors together
- Amortize graph update costs
- **Impact:** 2-3x speedup

## 🚀 OPTIMIZATION PHASE (Next Week)

### 1. True SIMD Distance
```mojo
@always_inline
fn simd_l2_distance(a: UnsafePointer[Float32], b: UnsafePointer[Float32], 
                     dimension: Int) -> Float32:
    alias simd_width = simdwidthof[DType.float32]()
    var sum = SIMD[DType.float32, simd_width](0)
    
    for i in range(0, dimension, simd_width):
        var va = a.load[width=simd_width](i)
        var vb = b.load[width=simd_width](i)
        var diff = va - vb
        sum += diff * diff
    
    return sqrt(sum.reduce_add())
```

### 2. Cache-Aligned Memory
- Align vectors to 64-byte boundaries
- Better cache utilization
- **Impact:** 10-15% speedup

### 3. Parallel Graph Updates
- When Mojo threading improves
- Update different graph regions concurrently
- **Impact:** 4-8x speedup

## 📈 Expected Results Timeline

| Phase | Time | Insert Rate | Improvement |
|-------|------|-------------|-------------|
| Current | Now | 907 vec/s | Baseline |
| Quick Fixes | 2 hours | 3,000 vec/s | 3x |
| Medium Fixes | 2 days | 10,000 vec/s | 11x |
| Optimizations | 1 week | 20,000+ vec/s | 22x |

## ⚠️ What NOT to Do

1. **Don't wait for Mojo improvements** - We can fix this now
2. **Don't add more features** - Simplify first
3. **Don't switch algorithms** - HNSW is proven
4. **Don't over-optimize** - Get basics working first

## 🎯 Success Metrics

- [ ] Insert rate > 20,000 vec/s
- [ ] Search latency < 0.2ms (maintain current)
- [ ] Scale to 1M+ vectors
- [ ] Memory < 8KB/vector
- [ ] No crashes or memory issues

## 💡 Key Insight

**We made it worse by trying to make it better.** 

The original simple implementation was faster because it didn't have:
- Complex hub highways
- Smart distance switching
- Over-engineered graph navigation
- Fixed capacity limits

**Solution: Simplify, fix basics, then optimize carefully.**