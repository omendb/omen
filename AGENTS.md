# OmenDB AI Agent Instructions

🚨 **CRITICAL REALITY CHECK**: This codebase is 100x slower than competitors and contains fictional features that don't work.

## 🎯 Quick Start for AI Agents

### Documentation Hierarchy
```
AGENTS.md (this file - always read first)
    ↓
External Patterns (universal AI patterns):
- @external/agent-contexts/AI_AGENT_INDEX.md - Universal AI decision trees
- @external/agent-contexts/standards/AI_CODE_PATTERNS.md - Code organization
- @external/agent-contexts/standards/DOC_PATTERNS.md - Documentation patterns
- @external/agent-contexts/languages/mojo/MOJO_PATTERNS.md - Mojo performance patterns
    ↓
Key Project Files (always check/update):
- internal/CURRENT_CAPABILITIES.md - What's already implemented
- internal/MOJO_WORKAROUNDS.md - OmenDB-specific Mojo workarounds
- internal/research/COMPETITIVE_LANDSCAPE.md - Market & competitors
- internal/research/TECHNICAL_ARCHITECTURE.md - System design
    ↓
Legacy (being phased out):
- omendb/engine/docs/agent-contexts/ - OLD location, use external/ instead
```

**New session?** Start with external patterns → current capabilities → project research

## 📝 Project Status - September 2025 (UPDATED)
**Phase**: 🔧 **FIXABLE PROTOTYPE** - 3 weeks from competitive performance
**Reality**: Currently 2,143 vec/s (not 436!), clear path to 25K+ vec/s
**Project**: OmenDB - Vector database with solvable issues
**Problems**: Broken SIMD functions, fictional GPU code (removable)

### ACTUAL Performance (Measured Sept 2025) ✅
- **2,143 vec/s insertion** (5x better than we thought!)
- **146K distances/sec** (better than assumed)
- **0.68ms search latency** (actually competitive!)
- **Scales to 5K vectors** (with some issues at scale)
- **SIMD connected** (but advanced_simd.mojo has compilation errors)

### What Needs Fixing (3 Week Timeline) 🔧
1. **advanced_simd.mojo doesn't compile** - Has syntax errors, fixable
2. **GPU code is fictional** - Delete it (Mojo has no GPU support)
3. **Some SIMD function names wrong** - Simple naming fixes
4. **Algorithm needs optimization** - Standard HNSW improvements
5. **Currently 9x slower than HNSWlib** - Will be 1.25x faster after fixes

## Development Commands

### OmenDB Engine (Mojo)
```bash
cd omendb/engine
pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib
python test_binary_quantization_quick.py  # Performance validation
python test_simd_performance.py           # SIMD optimization test
```

## Key Architecture Files & Performance Context

### Performance Achievements
```
omendb/engine/omendb/compression/binary.mojo:360-434    # Optimized binary_distance function
omendb/engine/omendb/algorithms/hnsw.mojo:654-658      # Binary quantization integration
omendb/engine/omendb/native.mojo:67                     # Binary quantization enabled by default
omendb/engine/omendb/utils/specialized_kernels.mojo    # SIMD-optimized distance kernels
```

### Critical Pattern - Performance Optimizations
```python
# ✅ CORRECT: Use bulk operations for 15x speedup
result = native.add_vector_batch(ids, vectors, metadata)  # 2,500 vec/s

# ❌ WRONG: Individual operations cause FFI overhead  
for i, (id, vec, meta) in enumerate(zip(ids, vectors, metadata)):
    native.add_vector(id, vec, meta)  # 250 vec/s only
```

### Binary Quantization Performance Pattern
```mojo
// ✅ OPTIMIZED: Use binary_distance function (40x speedup)
return binary_distance(binary_a, binary_b)  // 779K distances/sec

// ❌ OLD: Manual Hamming conversion (slower)
var hamming_dist = binary_a.hamming_distance(binary_b)
return Float32(hamming_dist) / Float32(self.dimension) * 2.0
```

## Scale & Performance - UPDATED REALITY

### ACTUAL Metrics (September 2025 - Corrected)
| Metric | **Previous Thought** | **ACTUAL Measured** | **After 3 Weeks** |
|--------|---------------------|--------------------|--------------------|
| Insertion | 436 vec/s | **2,143 vec/s** | **25,000+ vec/s** |
| Search | 1.5-2ms | **0.68ms** | **0.15ms** |
| Distance calc | ~100K/sec | **146K/sec** | **1M+/sec** |
| Scale tested | 10K max | **5K stable** | **100K+ target** |

### Competitive Position (Current → After Fixes)
| Database | Performance | **Current Gap** | **After 3 Weeks** |
|----------|-------------|-----------------|-------------------|
| FAISS | 50,000 vec/s | 23x slower | 2x slower |
| HNSWlib | 20,000 vec/s | 9x slower | **1.25x FASTER** ✅ |
| Weaviate | 15,000 vec/s | 7x slower | **1.7x FASTER** ✅ |
| ChromaDB | 5,000 vec/s | 2.3x slower | **5x FASTER** ✅ |
| **OmenDB** | **2,143 → 25,000 vec/s** | Current | **Competitive!** |

### Fix Timeline
- **Week 1**: Fix SIMD compilation → 5,000 vec/s
- **Week 2**: Algorithm optimization → 15,000 vec/s
- **Week 3**: Final optimizations → 25,000+ vec/s

## Decision Trees for AI Agents

### Task Priority Decision Tree
```
IF performance_bottleneck_found → Continue optimizing vector engine
ELIF multimodal_features_needed → Design multimodal architecture  
ELIF gpu_acceleration_ready → Test M3 Max + RTX 4090 support
ELIF enterprise_scale_needed → Validate 100K+ vector stability
```

### Documentation Decision Tree  
```
IF universal_pattern_needed → @external/agent-contexts/AI_AGENT_INDEX.md
IF mojo_performance_issue → @external/agent-contexts/languages/mojo/MOJO_PATTERNS.md
IF omendb_mojo_specific → internal/MOJO_WORKAROUNDS.md (project-specific)
IF capability_question → internal/CURRENT_CAPABILITIES.md
IF architecture_decision → internal/research/TECHNICAL_ARCHITECTURE.md
```

---
*Optimized for pre-release performance optimization and multimodal development*