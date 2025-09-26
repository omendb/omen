# OmenDB Research & State-of-the-Art Analysis

**Last Updated**: September 25, 2025
**Focus**: Cutting-edge database storage and indexing research

---

## Learned Index Research (Core Innovation)

### Foundational Papers
- **RMI (Recursive Model Index)**: Kraska et al., SIGMOD 2018 - Original learned index paper
- **ALEX**: Microsoft Research, SIGMOD 2020 - Adaptive learned index
- **RadixSpline**: SIGMOD 2020 - Radix + learned spline for robust performance
- **Tsunami**: Google, VLDB 2020 - Production learned indexes at Google

### Key Insights
- Linear models sufficient for most workloads (neural nets overkill)
- Two-stage models (RMI) balance accuracy vs memory
- Error bounds critical for correctness guarantees
- Time-series data particularly amenable to learned indexes

---

## State-of-the-Art Storage Engine Research (2024-2025)

### 1. **CXL Memory Disaggregation (BREAKTHROUGH)**
**Papers**:
- "CXL and the Return of Scale-Up Database Engines" (VLDB 2024)
- "DEX: Scalable Range Indexing on Disaggregated Memory" (VLDB 2024)

**Implications for OmenDB**:
- Learned models can live in disaggregated memory (100x capacity)
- Sub-microsecond access to remote memory (faster than SSD)
- Perfect for large ML models that don't fit in local RAM
- **Opportunity**: First learned database with CXL support

**Hardware Availability**:
- Marvell Structera (July 2024) - CXL devices now shipping
- Intel/AMD CPUs with CXL support in production

### 2. **LSM-Tree Innovations**
**Papers**:
- "Towards flexibility and robustness of LSM trees" (VLDB Journal 2024)
- "LSMGraph: High-Performance Dynamic Graph Storage" (SIGMOD 2025)
- "Endure: Tuning LSM-trees with Machine Learning" (2024)

**Key Advances**:
- ML-optimized compaction strategies (Endure/RocksDB)
- Learned bloom filters reducing false positives
- Adaptive merge policies based on workload

**For OmenDB**:
- Combine learned indexes with LSM storage
- ML-driven compaction scheduling
- Time-series optimized LSM variants

### 3. **Cloud-Native Disaggregated Architecture**
**Best Paper SIGMOD 2024**: "PolarDB-MP: Multi-Primary Cloud-Native Database"

**Other Notable Systems**:
- Amazon MemoryDB (SIGMOD 2024)
- GaussDB with compute-memory-storage disaggregation (VLDB 2024)
- CloudJump II: Optimizing for shared storage (SIGMOD 2025)

**Architecture Pattern**:
```
Compute Layer    <- Stateless, scales elastically
     ↓
Memory Layer     <- CXL-enabled, shared across compute
     ↓
Storage Layer    <- Object storage (S3), infinite capacity
```

**OmenDB Opportunity**: Learned models in memory layer, data in storage layer

### 4. **Hardware Acceleration**
**GPU Acceleration**:
- "GOLAP: GPU-in-Data-Path Architecture" (SIGMOD 2025)
- DFI: Data Flow Interface for high-speed networks (SIGMOD Best Paper)

**Persistent Memory**:
- "NV-SQL: Boosting OLTP with Non-Volatile DIMMs" (VLDB 2023)
- "ZBTree: B+-tree for Persistent Memory" (IEEE TKDE 2024)

**For Learned Indexes**:
- GPU for model training and batch inference
- Persistent memory for model storage (survives restarts)
- FPGA for ultra-low latency predictions

### 5. **Time-Series Specific Research**
**Systems**:
- TimescaleDB: Hypertables with automatic partitioning
- QuestDB: Column-based with SIMD optimization
- InfluxDB: LSM-tree with time-based compaction

**Research Gaps** (Our Opportunity):
- No time-series database uses learned indexes
- Current systems use traditional B-trees/LSM-trees
- Time-series data is highly predictable (perfect for learned indexes)

---

## Competitive Intelligence

### Who's Building What (2024-2025)

**Learned Indexes in Production**:
- **Google**: Tsunami in production (not public)
- **Microsoft**: ALEX research (not productized)
- **Amazon**: Research only, no products
- **OmenDB**: First commercial learned database (our opportunity)

**New Database Architectures**:
- **Neon**: Serverless Postgres with disaggregated storage ($104M raised)
- **PlanetScale**: MySQL with branching ($105M raised)
- **ClickHouse**: Column-store analytics ($6.35B valuation)

**Key Insight**: Nobody combining learned indexes + modern storage

---

## Technical Integration Opportunities

### The "Perfect Stack" (What We Could Build)
```
1. Query Layer:      Learned indexes (RMI/LinearIndex)
2. Compute Layer:    PostgreSQL-compatible parser
3. Memory Layer:     CXL disaggregated memory for models
4. Storage Layer:    LSM-tree with learned compaction
5. Acceleration:     GPU for training, FPGA for inference
```

### Realistic MVP Stack
```
1. Query Layer:      Simple learned indexes (done)
2. Compute Layer:    PostgreSQL extension (done)
3. Memory Layer:     In-process memory (current)
4. Storage Layer:    Sorted vectors (current)
5. Acceleration:     CPU-only (current)
```

### Evolution Path
- **Phase 1**: Current MVP (2x speedup)
- **Phase 2**: Add LSM storage (durability)
- **Phase 3**: Add GPU acceleration (10x speedup)
- **Phase 4**: Add CXL memory (100x capacity)
- **Phase 5**: Full disaggregation (cloud-native)

---

## Research-Driven Differentiation

### What Makes OmenDB Unique
1. **First commercial learned database** (others are research)
2. **Time-series optimization** (unexplored niche)
3. **PostgreSQL compatibility** (easy adoption)
4. **Modern storage integration** (CXL, GPU ready)

### Defensible Technical Moats
1. **Learned index expertise** (rare skill)
2. **Time-series specialization** (domain knowledge)
3. **Performance leadership** (2-10x faster)
4. **Patent potential** (novel combinations)

---

## Open Research Questions

### Worth Investigating
1. **Learned indexes for time-series joins** - Nobody's done this
2. **CXL-native learned models** - Designed for disaggregated memory
3. **Streaming model updates** - Continuous learning without retraining
4. **Hybrid storage engines** - LSM for writes, learned for reads

### Probably Too Complex
1. Neural network indexes (linear models work fine)
2. Distributed learned indexes (coordination overhead)
3. Multi-dimensional learned indexes (curse of dimensionality)
4. Fully homomorphic encryption (performance killer)

---

## Key Takeaways for OmenDB

### Immediate Opportunities
1. **Time-series learned indexes** - Clear market gap
2. **PostgreSQL extension** - Proven adoption path
3. **Proprietary DBaaS** - Proven business model

### Medium-Term Advantages
1. **LSM + learned indexes** - Novel combination
2. **GPU acceleration** - 10x additional speedup
3. **Cloud-native architecture** - Modern deployment

### Long-Term Vision
1. **CXL memory disaggregation** - 100x model capacity
2. **Full disaggregated architecture** - Infinite scale
3. **Hardware acceleration** - 1000x vs traditional

---

## References

### Must-Read Papers
1. Kraska et al. "The Case for Learned Index Structures" (SIGMOD 2018)
2. "PolarDB-MP" (SIGMOD 2024 Best Paper)
3. "CXL and the Return of Scale-Up Database Engines" (VLDB 2024)
4. "Endure: Machine Learning for LSM-trees" (2024)

### Key Conferences to Track
- SIGMOD 2025 (Berlin) - Accepted papers list available
- VLDB 2025 (London) - CFP open
- CIDR 2025 - Vision papers
- SysML 2025 - ML systems

### Industry Blogs
- Databricks Engineering Blog (acquired Neon)
- ClickHouse Blog (performance insights)
- PostgreSQL Hackers List (upcoming features)

---

*This research document represents the cutting edge of database systems as of September 2025. The convergence of learned indexes, disaggregated memory, and hardware acceleration presents a unique opportunity for OmenDB.*