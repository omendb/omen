# OmenDB Architectural Decision Log
_Append-only - never delete past decisions_

## 2025-09-19: Use ef_construction=50 instead of 200
**Context**: HNSW construction was too slow, using ef_construction=200
**Decision**: Reduce to ef_construction=50 (Qdrant benchmark setting)
**Rationale**:
- Qdrant uses 50-100 in production benchmarks
- 200 provides <1% quality improvement for 2-4x slowdown
- Testing shows 95%+ recall maintained with ef=50
**Consequences**: Expected 2-4x speedup in construction
**Status**: Implemented

---

## 2025-09-19: Individual Insertion for Segmented HNSW
**Context**: Bulk insertion in segments caused 0% recall
**Decision**: Use individual insertion within each segment
**Rationale**:
- Bulk insertion creates disconnected graphs
- Individual insertion maintains proper HNSW connectivity
- Quality more important than speed for now
**Consequences**: Slower (3.3K vec/s) but perfect recall
**Status**: Temporary - need proper bulk implementation

---

## 2025-09-19: Quality Filtering for Segment Search
**Context**: Each segment returned its "best" match even if terrible
**Decision**: Filter matches using 3x best distance threshold
**Rationale**:
- Prevents segments from contributing bad matches
- Special case for perfect matches (distance < 0.01)
- Maintains high recall while merging segments
**Consequences**: Slightly more computation but correct results
**Status**: Permanent solution

---

## 2025-09-18: Lazy Initialization for SegmentedHNSW
**Context**: Memory corruption when migrating from flat buffer to HNSW
**Decision**: Delay HNSWIndex creation until first use
**Rationale**:
- Migration corrupts global memory state
- Creating objects immediately after causes crashes
- Lazy init avoids corrupted state window
**Consequences**: Slightly more complex initialization logic
**Status**: Permanent fix

---

## 2025-09-18: Use SIMD Distance Functions
**Context**: Distance calculations using slow scalar implementation
**Decision**: Fix calls to use _fast_distance_between_nodes()
**Rationale**:
- SIMD provides 4-8x speedup on distance calc
- Distance computation is 70% of insertion time
- Code was already written, just not being called
**Consequences**: 6.15x overall speedup achieved
**Status**: Permanent

---

## 2025-09-17: Choose HNSW over Vamana/DiskANN
**Context**: Selecting core vector index algorithm
**Decision**: HNSW with modifications
**Rationale**:
- Most mature and proven in production
- Best recall/speed trade-off for in-memory
- Extensive research and optimizations available
- Used by Qdrant, Weaviate, Milvus
**Consequences**: Must respect hierarchical navigation invariants
**Status**: Core architecture decision

---

## 2025-09-16: Segment-Based Architecture
**Context**: Need to achieve 20K+ vec/s for competitiveness
**Decision**: Implement segmented HNSW similar to Qdrant
**Rationale**:
- Only way to achieve true parallelism
- Proven to scale to 50K+ vec/s
- Independent segments avoid lock contention
- Linear scaling with CPU cores
**Consequences**: More complex query merging, slightly lower recall
**Status**: In development

---

## 2025-09-15: Binary Quantization for Memory
**Context**: Memory usage too high for large datasets
**Decision**: Implement binary quantization with reranking
**Rationale**:
- 32x memory reduction (512 bytes → 16 bytes per vector)
- Hamming distance is fast (popcount instruction)
- Reranking with full precision maintains quality
- Industry standard approach
**Consequences**: Slightly more complex search path
**Status**: Implemented, needs verification

---

## 2025-09-10: Pure Mojo Implementation
**Context**: Choosing implementation language for vector engine
**Decision**: Pure Mojo with Python bindings
**Rationale**:
- True parallelism without GIL
- SIMD support built-in
- Memory control for performance
- Zero-copy potential with Python
**Consequences**:
- Limited ecosystem
- Compiler bugs and missing features
- No GPU support currently
**Status**: Committed

---

## 2025-09-05: Monorepo Structure
**Context**: Organizing multiple components (engine, server, web)
**Decision**: Single repository with clear separation
**Rationale**:
- Easier coordination between components
- Single source of truth for versions
- Simplified CI/CD
- Better for small team
**Consequences**: Larger repository, need good organization
**Status**: Implemented

---

## 2025-09-01: Target Competitive Performance
**Context**: Setting performance goals for MVP
**Decision**: Target 20K+ vec/s with 90%+ recall
**Rationale**:
- Matches Qdrant/Weaviate range
- Achievable without GPU
- Sufficient for most use cases
- Competitive for market entry
**Consequences**: Must implement segment parallelism
**Status**: Active goal

---

## 2025-08-15: Focus on Single-Node Performance
**Context**: Distributed vs single-node architecture
**Decision**: Optimize single-node first
**Rationale**:
- Simpler to implement and debug
- Most users don't need distribution
- Can add distribution later
- Reduces complexity for MVP
**Consequences**: Limited to single machine scale initially
**Status**: Current focus

---

## 2025-08-01: Open Source Strategy
**Context**: Business model and licensing
**Decision**: Open source with commercial cloud offering
**Rationale**:
- Builds trust and community
- Follows successful precedents (Qdrant, Weaviate)
- Cloud offering for revenue
- Enterprise features for monetization
**Consequences**: Need strong documentation and community building
**Status**: Planned for stable release

---

## Technical Debt Decisions

## 2025-09-19: Accept Temporary Performance Regression
**Context**: Choosing between speed and quality
**Decision**: Accept 3.3K vec/s to maintain 100% recall
**Rationale**:
- Quality critical for trust
- Speed without recall is useless
- Proper fix coming with DiskANN approach
- Better to be slow and correct
**Consequences**: Not competitive until bulk fixed
**Status**: Temporary acceptance

---

## 2025-09-18: Defer Zero-Copy FFI
**Context**: 10% overhead from Python→Mojo copying
**Decision**: Defer until core algorithm optimized
**Rationale**:
- Bigger gains from algorithmic improvements
- Zero-copy complex with current Mojo limitations
- Not the primary bottleneck
- Can add later without breaking changes
**Consequences**: Accepting 10% overhead temporarily
**Status**: On roadmap

---

## Failed Approaches (Learning)

## 2025-09-17: Sophisticated Bulk Construction - FAILED
**Context**: Trying to speed up bulk insertion
**Decision**: Complex batched graph construction
**Outcome**: 0% recall - creates disconnected graphs
**Learning**: Can't skip hierarchical navigation in HNSW

---

## 2025-09-16: Lock-Free Parallel Updates - FAILED
**Context**: Attempting parallel graph updates
**Decision**: Lock-free data structures
**Outcome**: 0.1% recall - race conditions corrupt graph
**Learning**: Need independent segments, not shared graph

---

## 2025-09-15: ef_construction=200 - WRONG
**Context**: Pursuing maximum quality
**Decision**: Use very high ef_construction
**Outcome**: 2-4x slower for <1% quality gain
**Learning**: Diminishing returns after ef=50-100

---

_This log helps understand why decisions were made and their outcomes_