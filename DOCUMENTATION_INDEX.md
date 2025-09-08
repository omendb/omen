# OmenDB Core Documentation Index

## 📋 **Current Status & Progress**

### Active Development Documents
- **[BULK_OPTIMIZATION_STATUS.md](BULK_OPTIMIZATION_STATUS.md)** - Complete bulk optimization implementation status and analysis
- **[OPTIMIZATION_STRATEGY_ANALYSIS.md](OPTIMIZATION_STRATEGY_ANALYSIS.md)** - Strategic analysis of optimization approaches and recommendations

### Project Configuration
- **[CLAUDE.md](CLAUDE.md)** - Claude Code agent instructions and project context
- **[README.md](README.md)** - Project overview and getting started

---

## 🧪 **Analysis & Testing Files**

### Performance Analysis (Root Level)
```
optimization_audit.py          # Comprehensive optimization verification
profile_detailed.py           # Detailed bottleneck analysis  
profile_insertion.py          # Insertion performance profiling
```

### Testing & Validation (Root Level)  
```
test_bulk_optimization.py     # Bulk optimization verification tests
test_large_bulk.py            # Large batch testing (crashes at 5K vectors)
test_dynamic_scaling.py       # Dynamic scaling tests
```

### Engine-Level Analysis
```
omendb/engine/optimization_audit.py  # Duplicate of root level audit
```

---

## 📊 **Key Findings Summary**

### Performance Achievements  
- ✅ **1.81x bulk speedup**: 8,658 vs 4,784 vec/s individual
- ✅ **Stable operations**: Up to 2,000 vectors
- ✅ **Working optimizations**: Dynamic growth, basic HNSW

### Critical Issues Identified
- ❌ **65x performance mystery**: 8,658 vs 133 vec/s in different tests
- ❌ **Memory overhead**: 187ms per vector (major bottleneck)  
- ❌ **SIMD regression**: 64D=8.4K vs 512D=3K vec/s
- ❌ **Segfaults**: At 5,000+ vectors with vectorization
- ❌ **Industry gap**: Need 25K+ vec/s (2.9x current best)

### Failed Experiments
- **Vectorized bulk operations**: Implemented but caused crashes and zero speedup
- **Distance matrix approach**: Algorithm overhead negated benefits
- **True bulk graph construction**: HNSW inherently requires sequential building

---

## 🎯 **Current Strategy: Investigation-First**

**Priority 1**: Debug 65x performance discrepancy (133 vs 8,658 vec/s)
**Priority 2**: Identify production vs test code path differences  
**Priority 3**: Target memory overhead reduction (187ms → <50ms)

### Next Phase Options
- **Incremental optimization**: Fix known bottlenecks (3x improvement target)
- **Algorithmic pivot**: Switch to LSH/IVF if incremental insufficient
- **Hybrid approach**: Combine investigation + optimization + decision point

---

## 🧹 **File Cleanup Status**

### To Archive/Remove
- `test_large_bulk.py` - Causes segfaults, analysis complete
- `profile_insertion.py` - Single-use analysis file
- Duplicate `omendb/engine/optimization_audit.py`

### To Keep
- `optimization_audit.py` - Comprehensive verification tool
- `profile_detailed.py` - Reusable bottleneck analysis
- `test_bulk_optimization.py` - Core verification test

---

## 📈 **Performance Targets**

| Target | Current Best | Gap | Approach |
|--------|-------------|-----|----------|
| Competitive (25K vec/s) | 8,658 vec/s | 2.9x | Investigation + incremental |
| Industry-leading (50K+ vec/s) | 8,658 vec/s | 5.8x | May need algorithmic pivot |

**Status**: Investigation phase to understand real bottlenecks before optimization