# OmenDB AI Agent Instructions

⚠️ **PRE-RELEASE STATUS**: All code in this repository is under active development and not ready for production use.

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

## 📝 Current Project Status (Sept 2025)
**Phase**: 🚧 PRE-RELEASE DEVELOPMENT - Not production ready
**Project**: OmenDB - Multimodal database (vectors + text + metadata)
**Strategy**: Build multimodal from start (10x better business than pure vector)
**Algorithm**: HNSW+ with integrated metadata filtering
**Architecture**: Mojo core + Rust server + Python/C bindings

### Major Performance Achievements ✅
- **2,500 vec/s insertion rate** (10x improvement from optimizations)
- **779,000 distances/sec** with binary quantization (approaching 40x target)  
- **32x memory reduction** with binary quantization
- **Segfaults completely fixed** - can process 75K+ vectors stably
- **SIMD optimizations active** - 1.4x speedup on specialized kernels

### Current Development Priorities
1. **Continue vector engine optimization** (approaching enterprise performance)
2. **Multimodal architecture design** (strategic differentiation vs Weaviate)
3. **GPU acceleration testing** (M3 Max + RTX 4090 preliminary support)
4. **100K+ vector scale validation** (enterprise readiness)

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

## Scale & Performance Context

### Current Metrics (Sept 2025)
| Metric | Current Performance | Target | Status |
|--------|-------------------|--------|---------|
| Insertion | 2,500 vec/s | 10,000 vec/s | 🟡 25% of target |
| Distance calc | 779,000/sec | 1M+/sec | 🟢 77% of target |
| Memory | 32x reduction | 32x reduction | ✅ Achieved |
| Stability | 75K+ vectors | 100K+ vectors | 🟡 75% of target |

### Business Context
- **$2.2B → $10.6B multimodal market** (2024-2032, 21% growth)
- **Strategic positioning**: Multimodal beats pure vector databases 10x
- **Competition**: Direct challenge to Weaviate's 1M+ Docker pull leadership
- **Hardware advantages**: M3 Max + RTX 4090 for performance testing

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