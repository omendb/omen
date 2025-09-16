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

## 📝 ACTUAL Project Status (December 2024)
**Phase**: 🚫 **RESEARCH PROTOTYPE** - Not usable for any production purpose
**Reality**: 100x slower than ALL competitors, fundamental architecture flaws
**Project**: OmenDB - Failed vector database attempt
**Problems**: Wrong technology choices, fictional features, unachievable targets

### ACTUAL Performance (Not Fantasies) ❌
- **436 vec/s insertion** (claimed 2,500 - **5.7x lie**)
- **~100K distances/sec** (claimed 779K - **7.8x lie**)
- **Crashes beyond 10K vectors** (claimed 75K - **7.5x lie**)
- **1.5-2ms search latency** (vs 0.08ms competition - **20x slower**)
- **NO GPU support exists** (all GPU code is fake)

### What's Actually Broken 🚫
1. **GPU acceleration is COMPLETELY FICTIONAL** - Mojo has no GPU support
2. **"SOTA" optimizations don't compile** - Missing Mojo features
3. **Not actually parallel** - "Lock-free" code is sequential
4. **FFI overhead kills performance** - 50-70% time wasted
5. **100x slower than FAISS** - Architectural failure

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

## Scale & Performance - REALITY

### ACTUAL Metrics (December 2024)
| Metric | **Claimed** | **ACTUAL** | **Reality** |
|--------|------------|------------|-------------|
| Insertion | 2,500 vec/s | **436 vec/s** | ❌ 5.7x lie |
| Distance calc | 779,000/sec | **~100K/sec** | ❌ 7.8x lie |
| Memory | 32x reduction | Untested | ❓ Unverified |
| Stability | 75K+ vectors | **10K max** | ❌ 7.5x lie |
| Search | 0.649ms | **1.5-2ms** | ❌ 2.3x slower |

### Competitive Reality
| Database | Performance | **We Are** |
|----------|------------|------------|
| FAISS | 50,000 vec/s | **115x slower** |
| HNSWlib | 20,000 vec/s | **46x slower** |
| Weaviate | 15,000 vec/s | **34x slower** |
| **OmenDB** | **436 vec/s** | **Dead last** |

### Hardware Reality
- **M3 Max GPU**: Cannot be used (Mojo has no GPU support)
- **RTX 4090**: Cannot be used (Mojo has no GPU support)
- **CPU only**: And we're terrible at that too

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