# AI Agent Context Guide for OmenDB Development
## Optimized Documentation Structure for AI-Assisted Development

## Purpose
This document organizes all critical information needed for AI agents (Claude, GPT, etc.) to effectively work on OmenDB without making repetitive mistakes or losing context about critical invariants.

## Document Hierarchy for AI Agents

### Level 1: Critical Invariants (ALWAYS LOAD FIRST)
```
internal/
├── HNSW_CORRECTNESS_RULES.md   # What MUST NEVER be violated
├── PERFORMANCE_TARGETS.md       # Current metrics and goals
└── KNOWN_FAILURES.md           # What doesn't work and why
```

### Level 2: Architecture & Strategy
```
internal/
├── ARCHITECTURE.md             # System design and components
├── COMPETITIVE_ANALYSIS_2025.md # Market position and targets
└── HNSW_DEVELOPMENT_GUIDE.md  # How to optimize correctly
```

### Level 3: Current State
```
internal/
├── STATUS.md                   # Latest performance and progress
├── HNSW_OPTIMIZATION_FINDINGS.md # Detailed optimization attempts
└── TODO.md                     # Prioritized task list
```

### Level 4: Historical Context
```
internal/archive/
└── [dated folders]             # Past attempts and learnings
```

## Critical Information for AI Agents

### 1. ABSOLUTE INVARIANTS (Never Violate)
```yaml
HNSW Invariants:
  - MUST navigate hierarchically from entry_point down through layers
  - MUST maintain bidirectional connections (A→B and B→A)
  - MUST update entry_point to highest level node
  - MUST ensure graph validity after EACH insertion

Performance Requirements:
  - Recall@10 MUST be ≥ 95%
  - No memory corruption or segfaults
  - Must handle 100K+ vectors without crashes
```

### 2. Current Performance Baseline
```yaml
Current State (September 2025):
  - Insertion: 867 vec/s
  - Recall@10: 95.5%
  - Latency: 8.2ms
  - Status: Quality good, speed needs 23x improvement

Proven Capability:
  - Achieved: 27,604 vec/s (but 1% recall)
  - Problem: Skipped hierarchical navigation
  - Solution: Keep navigation, optimize other parts
```

### 3. What Has Been Tried (And Failed)

| Attempt | Speed | Recall | Why It Failed | Key Learning |
|---------|-------|--------|--------------|--------------|
| Simplified Insertion | 27K vec/s | 1% | Skipped hierarchical navigation | Navigation is NON-NEGOTIABLE |
| Sophisticated Bulk | 22K vec/s | 1.5% | Memory corruption at scale | Need proper memory management |
| Parallel Insertion | 18K vec/s | 0.1% | Race conditions on graph | Need lock-free or segments |
| Naive Parallel | 9K vec/s | 0.1% | Random connections | Graph structure corrupted |

### 4. The Proven Path Forward

```mojo
Week 1: Fix What We Have (867 → 2K vec/s)
  - Profile actual bottlenecks
  - Fix bulk construction memory issues
  - Batch only safe operations

Week 2: Smart Parallelism (2K → 10K vec/s)
  - Segment parallel construction
  - Independent graph segments
  - Merge at search time

Week 3: Optimize Core (10K → 15K vec/s)
  - SIMD distance calculations
  - Cache-friendly memory layout
  - Pre-allocated buffers

Week 4: Advanced (15K → 20K+ vec/s)
  - Lock-free structures
  - Zero-copy FFI
  - Final optimizations
```

## Decision Trees for AI Agents

### When Optimizing Performance

```
START: Current performance < target?
├─ YES: Profile first
│   ├─ Distance calc slow? → Try SIMD
│   ├─ Graph traversal slow? → Optimize cache locality
│   ├─ Memory allocation slow? → Pre-allocate
│   └─ Graph updates slow? → Batch or lock-free
└─ NO: Check quality maintained
    ├─ Recall ≥ 95%? → Success!
    └─ Recall < 95%? → Check invariants
        ├─ Hierarchical navigation intact?
        ├─ Bidirectional connections?
        ├─ Entry point correct?
        └─ Progressive construction?
```

### When Adding New Features

```
START: New optimization idea
├─ Does it violate HNSW invariants? → STOP
├─ Has it been tried before? → Check KNOWN_FAILURES.md
├─ Will it help bottleneck? → Profile first
└─ Can maintain quality? → Implement with tests
```

### When Debugging Issues

```
START: Performance or quality issue
├─ Segfault/Crash?
│   ├─ Check memory management
│   ├─ Check pointer validity
│   └─ Reduce batch sizes
├─ Poor Recall?
│   ├─ Check hierarchical navigation
│   ├─ Verify bidirectional connections
│   └─ Test entry point management
└─ Slow Performance?
    ├─ Profile actual bottleneck
    ├─ Check against COMPETITIVE_ANALYSIS
    └─ Try safer optimization
```

## Common AI Agent Mistakes to Avoid

### 1. Attempting Parallel Graph Updates Without Locks
```mojo
# ❌ WRONG - Causes race conditions
parallel_for i in range(n):
    graph.add_edge(i, j)  # RACE!

# ✅ CORRECT - Use segments or locks
parallel_for segment in segments:
    build_independent_graph(segment)
```

### 2. Skipping Hierarchical Navigation
```mojo
# ❌ WRONG - Destroys recall
neighbors = find_neighbors_at_layer(target_layer)

# ✅ CORRECT - Navigate from top
curr = entry_point
for layer in range(top_layer, target_layer, -1):
    curr = search_layer(query, curr, layer)
neighbors = find_neighbors_at_layer(target_layer, curr)
```

### 3. Optimizing Without Profiling
```mojo
# ❌ WRONG - Guessing at bottlenecks
"Let's parallelize everything!"

# ✅ CORRECT - Measure first
profile_results = profile_insertion()
optimize_slowest_component(profile_results)
```

### 4. Sacrificing Quality for Speed
```mojo
# ❌ WRONG - 1% recall is useless
simplify_algorithm()  # 27K vec/s but 1% recall

# ✅ CORRECT - Maintain invariants
optimize_while_preserving_invariants()  # 20K vec/s with 95% recall
```

## Key Context for Specific Files

### `/omendb/algorithms/hnsw.mojo`
- Lines 1343-1390: Batched insertion (current production)
- Lines 2872-2921: `_insert_node_simplified` (broken, 1% recall)
- Lines 1406-1577: Sophisticated bulk (memory issues)
- Lines 2781-2870: `_insert_node` (correct but slow)

### `/benchmarks/final_validation.py`
- Tests both insertion rate and recall
- 10K vectors: baseline test
- 20K vectors: scale test
- Must maintain 95%+ recall

### `/omendb/native.mojo`
- Lines 614-621: Switches between segmented/monolithic
- Line 196: Segmented search integration
- Python/Mojo FFI interface

## Environment Constraints

### Mojo Language Limitations
```yaml
Known Issues:
  - SIMD compilation randomly breaks
  - No GPU support (discovered Oct 2025)
  - Limited debugging tools
  - Memory management is manual
  - Compiler sometimes optimizes incorrectly

Workarounds:
  - Test SIMD with simple examples first
  - CPU-only optimizations
  - Add extensive logging
  - Careful pointer lifecycle management
  - Disable aggressive optimizations
```

### Build & Test Commands
```bash
# Build
pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib -I omendb

# Test Performance
pixi run python benchmarks/final_validation.py

# Quick Test
pixi run python test_binary_quantization_quick.py
```

## Success Metrics

### Minimum Viable
- 5,000+ vec/s insertion
- 95%+ recall@10
- No crashes at 100K vectors

### Competitive Target
- 20,000+ vec/s insertion
- 95%+ recall@10
- <10ms query latency
- Linear scaling with cores

### State-of-the-Art
- 25,000+ vec/s insertion
- 96%+ recall@10
- <5ms query latency
- GPU acceleration (when available)

## Communication Guidelines for AI Agents

### When Reporting Issues
```markdown
1. Current metrics: X vec/s, Y% recall
2. What was attempted: [specific change]
3. What broke: [specific failure]
4. Hypothesis: [why it failed]
5. Next step: [proposed fix]
```

### When Proposing Optimizations
```markdown
1. Bottleneck identified: [profiling data]
2. Proposed solution: [specific approach]
3. Expected improvement: X → Y vec/s
4. Quality impact: Will/won't affect recall
5. Risk assessment: [potential issues]
```

### When Achieving Milestones
```markdown
1. Baseline: X vec/s, Y% recall
2. Current: X' vec/s, Y'% recall
3. Improvement: Z% speedup
4. Method: [what was changed]
5. Verified: [test results]
```

## File Organization Best Practices

### For New Documentation
```
internal/
├── FINDING_YYYY_MM_DD.md      # Specific discoveries
├── OPTIMIZATION_[NAME].md      # Optimization attempts
└── archive/YYYY-MM-DD/         # Move outdated docs here
```

### For Code Changes
```
1. Read current implementation first
2. Check HNSW_CORRECTNESS_RULES.md
3. Profile before optimizing
4. Test quality after EVERY change
5. Commit working baselines
```

## Prompt Engineering for OmenDB

### Effective Prompts
```
"Profile the current implementation and identify the slowest component"
"Fix the bulk construction memory issue while maintaining 95% recall"
"Implement segment parallelism without violating HNSW invariants"
```

### Ineffective Prompts
```
"Make it faster" (too vague)
"Parallelize everything" (will break invariants)
"Simplify the algorithm" (will destroy quality)
```

## Conclusion

**For AI agents to succeed with OmenDB:**

1. **Always load invariants first** - Know what can't change
2. **Profile before optimizing** - Don't guess bottlenecks
3. **Test quality continuously** - Catch breaks early
4. **Learn from failures** - Check what's been tried
5. **Follow the proven path** - Week-by-week plan exists

**The goal is clear:** 20,000+ vec/s with 95% recall in 3-4 weeks.

**The path is proven:** We achieved 27K vec/s, just need quality fixes.

**The tools exist:** Mojo has the performance ceiling we need.

---
*This guide ensures AI agents have complete context to avoid repeating mistakes*