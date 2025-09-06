# Business Model & Strategy Decisions

## Feb 5, 2025: Pure Vector First, Multimodal Later

### Context  
Deciding between pure vector DB vs multimodal (vector + text + structured data) for initial release.

### Options Evaluated
1. **Pure Vector Only**
   - Pros: Focused, 4-week MVP, clear market (pgvector replacement)
   - Cons: Commoditized market, limited differentiation

2. **Multimodal from Start**
   - Pros: Huge differentiation, higher value ("MongoDB for AI")
   - Cons: 12+ weeks to MVP, complex architecture

3. **Phased Approach** ✅
   - Pros: Quick MVP, upgrade path, best of both worlds
   - Cons: Two development phases, roadmap complexity

### Decision: Phased Implementation

#### Phase 1 (Month 1): Pure Vector MVP
```python
from omendb import Index
index = Index(dimension=1536)
index.add(vectors, ids)           # Simple vector storage
results = index.search(query, k=10)  # <10ms queries
```

#### Phase 2 (Month 2): Metadata Filtering
```python
index.add(vectors, ids, metadata={"type": "product", "category": "phone"})
results = index.search(query, k=10, filter={"type": "product"})
```

#### Phase 3 (Month 3): Full Multimodal
```python
# The killer feature - unified AI data storage
db.insert({
    "text": "iPhone 15 Pro review",
    "vector": embedding,  
    "metadata": {"rating": 4.5, "price": 999},
    "image": binary_data
})

# Hybrid search across modalities
results = db.search(
    text_query="latest iPhone",           # Text search (BM25)
    vector_query=query_embedding,         # Semantic search (HNSW+) 
    filters={"price": {"$lt": 1000}}      # Structured queries
)
```

#### Rationale
- **Market validation**: Prove we can beat pgvector first (10x performance)
- **Revenue progression**: Free vector → paid metadata → premium multimodal
- **Positioning evolution**: "Fast pgvector" → "MongoDB for AI applications"
- **Implementation manageable**: Each phase builds on previous

### Success Criteria
- Phase 1: 10x faster than pgvector on standard benchmarks
- Phase 2: Metadata filtering without performance degradation
- Phase 3: Full-text + vector + structured queries in single API

**⚠️ REVIEW NOTE**: Need to re-evaluate this decision with HNSW+ pivot. HNSW may be better suited for multimodal than DiskANN was.

---

## Open Source + Cloud Model

### Open Source (CPU) ✅
- **Algorithm**: HNSW+ optimized for CPU
- **Performance**: 10x faster than pgvector
- **Bindings**: Python native, C/Rust via shared library
- **License**: Apache 2.0 (permissive)
- **Goal**: Wide adoption, developer mindshare

### Cloud Platform (GPU) ✅
- **Algorithm**: Same HNSW+, GPU-accelerated
- **Performance**: 100x faster than pgvector (10x better than CPU version)
- **Features**: Managed service, auto-scaling, enterprise SLAs
- **Pricing**: $0.50/million vectors/month (estimate)
- **Goal**: Revenue, high-performance use cases

### Why This Split Works
1. **Broad adoption**: Free CPU version builds community
2. **Clear upgrade path**: CPU → GPU for performance needs
3. **Revenue justification**: GPU costs require premium pricing
4. **Competitive moat**: Same Mojo codebase, different compilation targets

### Market Positioning

#### Phase 1: "10x Faster pgvector"
- Target: PostgreSQL users frustrated with pgvector performance
- Message: "Same API, 10x faster builds, Python native"
- Competition: pgvector (performance), ChromaDB (simplicity)

#### Phase 2: "Unified AI Data Platform" 
- Target: AI application developers
- Message: "Vector + metadata + text search in one database"
- Competition: MongoDB + vector search, specialized solutions

#### Phase 3: "Production AI Infrastructure"
- Target: Enterprise AI applications
- Message: "Scalable, managed, multi-modal AI data platform"
- Competition: Pinecone + additional services, custom solutions

---

## Go-to-Market Strategy

### Developer-Led Growth
1. **Open source adoption** (Month 1-2)
2. **PostgreSQL extension** (Month 2-3) 
3. **Cloud beta program** (Month 3-4)
4. **Enterprise sales** (Month 6+)

### Key Metrics
- **Adoption**: GitHub stars, package downloads, documentation views
- **Engagement**: Active users, query volume, retention
- **Revenue**: Cloud ARR, enterprise contracts, support subscriptions

---
*All business and strategy decisions documented here*