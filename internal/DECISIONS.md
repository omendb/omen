# 📊 OmenDB Decision Log
*Append-only record of architectural and technical decisions*

---

## 2025-02-04 | Stay with Mojo over Rust

### Context
25K vector bottleneck blocking production. Considering complete rewrite in Rust for better FFI performance.

### Options Considered
1. **Rust Rewrite**
   - Pros: PyO3 zero-copy, mature ecosystem, proven in Qdrant/Lance
   - Cons: Lose Mojo advantages, restart from scratch, 1-2 month effort
   
2. **Fix Mojo Architecture**
   - Pros: Keep existing code, Mojo has zero-copy available, GPU native
   - Cons: Less mature ecosystem, some language features in development

3. **Hybrid Approach**
   - Pros: Best of both worlds
   - Cons: Complexity, two codebases to maintain

### Decision
**Chosen: Fix Mojo Architecture**

### Rationale
- Zero-copy FFI exists in Mojo via `__array_interface__`
- Threading/async patterns achievable with current features
- GPU support is native in Mojo (advantage over Rust)
- Simpler to fix existing code than complete rewrite

### Consequences
- Can solve bottleneck with async buffer pattern
- Keep innovation advantage with new language
- May need workarounds for missing features
- Community/ecosystem still growing

### Review Date
After implementing async buffer fix (1 week)

---

## 2025-02-04 | Documentation Strategy for AI Agents  

### Context
Need efficient way to maintain context between AI sessions and track work.

### Options Considered
1. **GitHub Issues/Projects**
   - Pros: Standard, API access, collaboration
   - Cons: Network latency, not in context, complexity
   
2. **Database/JSON**
   - Pros: Structured, queryable
   - Cons: Overhead, schema changes, parsing

3. **Markdown in Repo**
   - Pros: Instant access, versioned, readable
   - Cons: Less structured, manual updates

### Decision  
**Chosen: Markdown files with specific structure**

### Rationale
- AI agents need immediate access without API calls
- Version control provides history automatically  
- Simple to maintain and debug
- Works offline

### Consequences
- Excellent AI workflow established
- Need discipline to maintain structure
- May need tooling for complex queries later

### Review Date
After 3 months usage

---

## 2025-02-04 | Buffer Architecture (To Be Replaced)

### Context
Current synchronous buffer flush causes 25K vector bottleneck.

### Options Considered
1. **Keep Buffer, Make Async** (FreshDiskANN pattern)
   - Pros: Proven in Chroma, immediate fix possible
   - Cons: Still has buffer overhead
   
2. **Eliminate Buffer** (IP-DiskANN pattern)  
   - Pros: State-of-art (2025), best performance
   - Cons: Major architecture change, research needed

### Decision
**Chosen: Async Buffer (short-term), then IP-DiskANN (long-term)**

### Rationale
- Need immediate fix for production
- Async buffer is well-understood pattern
- IP-DiskANN requires more research/time

### Consequences
- Two-phase migration needed
- Temporary solution will work at 100K+ scale
- Ultimate solution will handle billions

### Review Date
After async buffer implementation

---

## 2025-01-15 | DiskANN over HNSW

### Context
Choosing core algorithm for vector search.

### Options Considered
1. **HNSW** (Hierarchical Navigable Small Worlds)
   - Pros: Widely used, good recall, fast queries
   - Cons: Must fit in RAM, no built-in compression
   
2. **DiskANN** (Vamana algorithm)
   - Pros: Scales beyond RAM, built-in PQ compression
   - Cons: More complex, less adoption

3. **IVF-PQ** (Inverted File with Product Quantization)
   - Pros: Simple, good compression
   - Cons: Lower recall, requires training

### Decision
**Chosen: DiskANN**

### Rationale  
- Only algorithm that scales to billions on single machine
- PQ compression critical for memory efficiency
- Microsoft proven it in production

### Consequences
- More complex implementation
- Better long-term scalability
- Unique differentiation from competitors

### Review Date
N/A - Core architecture decision

---

## 2025-02-05 | HNSW+ over DiskANN (Major Pivot)

### Context
After extensive research, discovered DiskANN fundamentally incompatible with streaming updates. Need production-ready algorithm.

### Options Considered
1. **Fix Current DiskANN**
   - Pros: Keep existing code
   - Cons: Fighting algorithm's batch-oriented nature forever
   
2. **Implement IP-DiskANN** 
   - Pros: State-of-art streaming (400K updates/sec)
   - Cons: Unproven (Feb 2025 paper), complex, no references

3. **Switch to HNSW+**
   - Pros: Industry standard, proven, Mojo strengths apply
   - Cons: Complete rewrite required

### Decision
**Chosen: HNSW+ with optimizations**

### Rationale
- **Market reality**: pgvector, MongoDB, Redis all use HNSW
- **Production proven**: 8+ years vs IP-DiskANN (paper only)
- **Mojo advantages**: SIMD, parallelism, future GPU all benefit HNSW
- **Timeline**: 4 weeks to production vs 6+ for IP-DiskANN
- **Business model**: Clear CPU open source, GPU cloud split

### Consequences
- Complete algorithm replacement needed
- Can target pgvector benchmarks directly
- Standard algorithm = easier adoption
- GPU acceleration path clear

### Review Date
After HNSW+ implementation (4 weeks)

---

## 2025-02-05 | Pure Vector First, Multimodal Later

### Context  
Deciding between pure vector DB vs multimodal (vector + text + structured).

### Options Considered
1. **Pure Vector Only**
   - Pros: Focused, 4 week MVP, clear market
   - Cons: Commoditized, limited differentiation

2. **Multimodal from Start**
   - Pros: Huge differentiation, higher value
   - Cons: 12+ weeks to MVP, complex

3. **Phased Approach**
   - Pros: Quick MVP, upgrade path, best of both
   - Cons: Two development phases

### Decision
**Chosen: Phased - Pure vector first, add multimodal**

### Rationale
- Get to market in 4 weeks with pure HNSW+
- Prove performance vs pgvector first
- Add metadata filtering (month 2)
- Full multimodal "MongoDB for AI" (month 3)
- Clear value progression and pricing tiers

### Consequences
- Can ship MVP quickly
- Future differentiation secured
- More complex roadmap
- Clear upgrade path for customers

### Review Date
After pure vector MVP ships

---
## 2025-02-05 | Multimodal Database from Start (Revised)

### Context
After extensive research on HNSW+ capabilities and competitive analysis, reconsidering pure vector vs multimodal strategy.

### Options Considered
1. **Pure Vector First (Original Plan)**
   - Pros: Quick to market (4 weeks), simpler implementation
   - Cons: Commoditized market, 20+ competitors, price race

2. **Multimodal from Start**
   - Pros: 10x pricing power, less competition, real market pain
   - Cons: 3x complexity, longer development (6-8 weeks)

3. **Hybrid Development**
   - Pros: Cover both markets
   - Cons: Split resources, confusing positioning

### Decision
**Chosen: Multimodal from Start**

### Rationale
- Research shows real market pain ("architectural cobwebs" of multiple DBs)
- HNSW+ proven for multimodal (MongoDB Atlas, Redis, Elasticsearch use it)
- 10x pricing power ($500-50K/mo vs $50-500/mo for pure vector)
- Only MongoDB Atlas really competing (and they're slow/expensive)
- All components well-understood (HNSW, BM25, B-trees)
- Mojo's GPU compilation gives unique advantage

### Consequences
- 6-8 week development instead of 4 weeks
- More complex architecture but manageable
- Higher value capture potential
- Clear differentiation from day one
- Need query planner and storage layer design

### Review Date
After MVP ships (Month 2)

---

## 2025-02-05 | Stick with Mojo Despite Limitations

### Context
Mojo missing some features (async/await, limited stdlib). Considering Rust rewrite for stability.

### Options Considered
1. **Switch to Rust**
   - Pros: Mature, stable, great ecosystem
   - Cons: No GPU compilation, needs FFI for Python

2. **Mojo with Workarounds**
   - Pros: GPU compilation path, Python-native, SIMD built-in
   - Cons: Missing features, smaller ecosystem

3. **Hybrid Mojo Core + Rust Server**
   - Pros: Best of both worlds
   - Cons: Added complexity

### Decision
**Chosen: Mojo Core + Rust Server**

### Rationale
- GPU compilation is killer feature (100x performance potential)
- Python-native means zero FFI overhead
- Modular likely to provide development support
- Missing features have workarounds (thread pools for async)
- Rust server handles HTTP/gRPC professionally
- Long-term bet on Mojo ecosystem growth

### Consequences
- Need workarounds for missing features
- Potential Modular partnership opportunity
- Unique performance advantages
- First-mover in Mojo database space
- Some development friction initially

### Review Date
After implementing core HNSW+ (Month 1)

---

## 2025-02-06 | Clean Rebuild Over Migration Approach

### Context
After implementing HNSW+ foundation and user feedback on migration complexity, need to decide between gradual DiskANN→HNSW+ migration vs complete clean rebuild.

### Options Considered
1. **Gradual Migration**
   - Pros: Keep some existing code, incremental progress
   - Cons: API incompatibility, complex refactoring, migration bugs
   
2. **Complete Clean Rebuild**
   - Pros: State-of-the-art implementation, no compatibility overhead, Mojo-optimized
   - Cons: Start from scratch, throw away some existing work

3. **Hybrid Keep-Core-Rewrite-Interface**
   - Pros: Keep proven algorithms
   - Cons: Still has compatibility issues, partial benefits

### Decision
**Chosen: Complete Clean Rebuild**

### Rationale
- DiskANN and HNSW have fundamentally different APIs (batch vs streaming, string vs numeric IDs)
- No backward compatibility required (not in production)
- Can optimize from ground up for Mojo's strengths (SIMD, GPU)
- User feedback: "just rewrite it correctly for hnsw+. remove refactor or start fresh"
- Clean architecture enables state-of-the-art optimizations

### Consequences
- Archive entire DiskANN implementation for reference only
- Build HNSW+ with priority queue, SIMD optimization, GPU kernels
- 4-phase development: Foundation (✅) → Optimizations (🚧) → Multimodal (🔮) → Enterprise (🏭)
- Target: 10x better performance than competitors
- Reference archived DiskANN for algorithm insights only

### Review Date
After Phase 2 optimizations complete

---

## 2025-02-06 | Temporary Minimal Implementation Due to HNSW+ Memory Issues

### Context
HNSW+ implementation has critical memory allocation issues causing std::bad_alloc on second vector insertion. After consolidating heap implementations using state-of-the-art patterns from Modular's MAX kernels, the issue persists in the HNSW graph construction itself.

### Options Considered
1. **Debug HNSW+ incrementally** - Time consuming, blocks progress
2. **Use existing DiskANN** - Already deprecated, wrong architecture
3. **Switch to minimal implementation** - Quick, enables forward progress
4. **Port from server/hnsw_index.mojo** - May have same issues

### Decision
Use minimal linear search implementation (native_minimal.mojo) as temporary solution while debugging HNSW+ memory issues in parallel.

### Rationale
- Unblocks Python integration testing and Rust server development
- Provides stable base for API development
- Performance adequate for development (2,896 vec/s, 0.04ms search)
- Clear separation between working code and debugging effort
- Can swap implementations transparently once fixed

### Technical Details
**Root causes identified in HNSW+:**
- Node list growing with List[HNSWNode]()
- Each node has List[List[Int]] for connections per layer
- Visited array allocating self.size bools per search
- Exponential memory growth during graph construction

**Consolidated heap implementations created:**
- DynamicMinHeap - For search candidates (grows as needed)
- FixedMaxHeap - For top-k results (fixed size, evicts worst)
- BatchHeap - For bulk operations (pre-sized)

### Consequences
- Can continue development without blocking on memory debugging
- Need to maintain two implementations temporarily
- Must fix HNSW+ before production deployment
- Performance benchmarks not representative of final system

### Review Date
After HNSW+ memory issues resolved

---
