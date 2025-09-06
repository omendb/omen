# Algorithm Decisions

## Feb 5, 2025: HNSW+ over DiskANN (Major Pivot)

### Context
After extensive research, discovered DiskANN fundamentally incompatible with streaming updates. Need production-ready algorithm for vector database.

### Options Evaluated
1. **Fix Current DiskANN**
   - Pros: Keep existing code, minimal changes
   - Cons: Fighting algorithm's batch-oriented nature forever
   - Verdict: Technical debt nightmare

2. **Implement IP-DiskANN (2025)**
   - Pros: State-of-art streaming (400K updates/sec), Microsoft paper
   - Cons: Unproven (Feb 2025 paper only), complex bidirectional edges, no references
   - Verdict: Too risky for production

3. **Switch to HNSW+** ✅
   - Pros: Industry standard, 8+ years proven, Mojo strengths apply perfectly
   - Cons: Complete algorithm rewrite required
   - Verdict: Correct choice

### Decision: HNSW+ with Modern Optimizations

#### Rationale
- **Market reality**: pgvector, MongoDB, Redis, Elasticsearch all use HNSW
- **Production proven**: 8+ years vs IP-DiskANN (research paper only)
- **Mojo advantages**: SIMD, parallelism, future GPU all benefit HNSW more than DiskANN
- **Timeline**: 4 weeks to production vs 6+ weeks for IP-DiskANN
- **Business model**: Clear CPU open source, GPU cloud premium split
- **Benchmarking**: Can directly compare against pgvector (standard HNSW)

#### Implementation Strategy
```mojo
struct HNSWIndex:
    var layers: List[Graph]         # Hierarchical structure
    var M: Int = 16                 # Connections per layer (standard)
    var ef_construction: Int = 200  # Build parameter (quality vs speed)
    var entry_point: Int            # Top layer entry node
```

### Consequences
- Complete algorithm replacement needed (not just fixes)
- Can target pgvector benchmarks directly
- Standard algorithm = easier developer adoption
- GPU acceleration path is proven (CAGRA uses similar concepts)
- Lost 6 months of DiskANN work, but gained market-fit algorithm

### Success Metrics
- 10x faster builds than pgvector (target: 100K vectors/sec)
- Sub-10ms query latency at 95% recall
- Memory usage ≤ 2 bytes/vector
- Python native integration with zero FFI overhead

---

## Jan 15, 2025: DiskANN over HNSW (Superseded)

*This decision has been superseded by the Feb 5 HNSW+ decision above.*

### Original Context
Choosing core algorithm for billion-scale vector search.

### Original Decision: DiskANN
- **Rationale**: Only algorithm that scales beyond RAM with PQ compression
- **Problem discovered**: DiskANN designed for batch building, not streaming updates
- **Lesson**: Theoretical advantages don't matter if algorithm doesn't fit use case

### Why This Failed
1. **Streaming incompatibility**: DiskANN requires complete graph rebuilding for updates
2. **Production reality**: Even Microsoft struggles with updates in SQL Server 2025
3. **Market evidence**: Every other major vector DB chose HNSW over DiskANN

### Key Insight
Algorithm choice must match use case. For streaming vector databases:
- **HNSW**: Natural incremental updates ✅
- **DiskANN**: Batch-oriented, fights streaming ❌

---
*All major algorithm decisions should be documented here*