# OmenDB Internal Documentation
## AI-Optimized Organization for State-of-the-Art Vector Database Development

## Quick Start for AI Agents

### 🚨 MANDATORY: Load These First
1. **`HNSW_CORRECTNESS_RULES.md`** - What MUST NEVER be violated
2. **`STATUS.md`** - Current performance (867 vec/s, 95.5% recall)
3. **`COMPETITIVE_ANALYSIS_2025.md`** - Market targets (20K+ vec/s needed)
4. **`AI_AGENT_CONTEXT.md`** - This guide and decision trees

### 📊 Current State Summary (September 2025)
```yaml
Performance: 867 vec/s insertion, 95.5% recall@10
Target: 20,000+ vec/s insertion, 95% recall@10
Proven: 27,604 vec/s achieved (but 1% recall due to broken navigation)
Timeline: 3-4 weeks to competitive performance
Status: Quality excellent, speed needs 23x improvement
```

## Document Hierarchy

### 🔴 Critical (Always Load First)
```
internal/
├── HNSW_CORRECTNESS_RULES.md   # ⛔ NEVER violate these rules
├── STATUS.md                   # 📊 Latest performance and progress
├── COMPETITIVE_ANALYSIS_2025.md # 🎯 Market targets and strategy
└── AI_AGENT_CONTEXT.md         # 🤖 Guide for AI development
```

### 🟡 Reference (Load When Needed)
```
internal/
├── ARCHITECTURE.md             # System design overview
├── HNSW_DEVELOPMENT_GUIDE.md   # How to optimize correctly
├── HNSW_OPTIMIZATION_FINDINGS.md # Detailed attempt analysis
└── RESEARCH.md                 # Academic background
```

### 🟢 Archives (Historical Context)
```
internal/archive/
├── 2025-02-05-cleanup/         # Legacy cleanup
├── 2025-10-legacy-root/        # Old documentation
└── [dated folders]/            # Past experiments
```

### 📂 Research (Supporting Material)
```
internal/research/
├── HNSW_OPTIMIZATIONS_2025.md
├── TECHNICAL_ARCHITECTURE.md
└── COMPETITOR_ANALYSIS.md
```

## Key Learnings (AI Must Know)

### 🎯 The Core Problem
**We can achieve 27K vec/s but destroy quality (1% recall) by skipping hierarchical navigation. The challenge is maintaining HNSW invariants while optimizing performance.**

### 🔧 What Works
```yaml
Quality:
  - Hierarchical navigation (entry_point → target_layer)
  - Bidirectional connections (A↔B)
  - Progressive construction (valid after each insert)

Performance:
  - Batched processing (867 → 2K vec/s possible)
  - Segment parallelism (independent graphs)
  - SIMD distance calculations (4-8x speedup)
  - Memory pre-allocation (avoid hot-path allocation)
```

### ❌ What Fails
```yaml
Broken Attempts:
  - Simplified insertion: 27K vec/s, 1% recall (skipped navigation)
  - Sophisticated bulk: 22K vec/s, crashes (memory corruption)
  - Parallel graph updates: 18K vec/s, 0.1% recall (race conditions)
  - Naive parallelism: 9K vec/s, 0.1% recall (random connections)
```

### 🏆 Competitor Benchmarks
```yaml
Must Beat:
  - Qdrant: 20,000-50,000 vec/s, 95% recall
  - Weaviate: 15,000-25,000 vec/s, 95% recall
  - Pinecone: 10,000-30,000 vec/s, 95% recall

Can Beat:
  - Chroma: 5,000-10,000 vec/s, 90% recall
  - Local implementations: <1,000 vec/s

Advantages:
  - Mojo performance ceiling (theoretical 100K+ vec/s)
  - No GIL limitations
  - Manual memory control
  - SIMD without overhead
```

## AI Agent Best Practices

### Effective Prompts
```
✅ "Profile the bottleneck in the current 867 vec/s implementation"
✅ "Fix bulk construction memory issues while preserving 95% recall"
✅ "Implement Qdrant-style segment parallelism for 10K+ vec/s"
✅ "Add SIMD distance calculations without breaking invariants"
```

### Ineffective Prompts
```
❌ "Make it faster" (too vague)
❌ "Parallelize everything" (will break invariants)
❌ "Copy Chroma's approach" (Python GIL limitations)
❌ "Simplify the algorithm" (will destroy quality)
```

## Development Workflow

### For New Optimizations
1. **Read correctness rules** (`HNSW_CORRECTNESS_RULES.md`)
2. **Check if tried before** (`HNSW_OPTIMIZATION_FINDINGS.md`)
3. **Profile current performance** (identify real bottleneck)
4. **Implement with tests** (maintain quality)
5. **Benchmark and document** (update `STATUS.md`)

### Success Metrics
- **Performance:** 20,000+ vec/s insertion rate
- **Quality:** 95%+ recall@10 on standard benchmarks
- **Stability:** Handle 1M+ vectors without crashes
- **Latency:** <10ms query response time

---
*Last Updated: September 2025*
*Next Review: After achieving 2K+ vec/s milestone*