# omen Product Roadmap

**Last Updated**: October 28, 2025
**Current Phase**: Week 7 - Validation & Production Hardening

---

## Current Status (Week 7)

### Validation Phase Progress

**Completed:**
- ✅ Phase 1: Core HNSW + BQ implementation (98% complete, 101 tests)
- ✅ Phase 2 (60%): Edge cases, resource limits (41 tests, 40 ASAN validated)
- ✅ Total: 142 tests passing, zero memory safety issues

**Remaining (3-4 weeks):**
1. **Week 7-8**: Finish Phase 2 validation
   - Input validation fuzzing
   - Serialization fuzzing
   - Concurrency stress tests

2. **Week 9**: Phase 3 performance benchmarks
   - Benchmark vs pgvector (prove 10x claim)
   - Memory efficiency validation (28x better claim)
   - Query latency at scale (10M+ vectors)

3. **Week 10**: Production hardening
   - Monitoring and observability
   - Documentation and examples
   - Launch prep (website, marketing materials)

**Architecture Status:**
- ✅ Core modules stable (no planned architectural changes)
- ✅ RocksDB storage layer finalized
- ✅ HNSW + Binary Quantization working
- 🎯 Focus: Validation, benchmarks, and polish

---

## Future Enhancement: Time Series Module

### When: After Production-Ready (Week 11+, 3-4 weeks)

### Strategic Rationale

**Why Time Series in omen:**
- **Highly complementary**: Embeddings often have timestamps (document versions, model drift)
- **Unique positioning**: No competitor has "time-aware vector search"
- **Stronger product**: Unified database vs separate products
- **Real use cases**: Document versioning, RAG with recency, embedding drift tracking

**Competitive Landscape:**
- Pinecone: Vector only, no time series
- pgvector: Vector only, no time series
- InfluxDB: Time series only, no vectors
- **omen**: Both + 10x faster 🎯

### Proposed SQL Interface

```sql
-- Time-aware vector search (UNIQUE feature!)
SELECT * FROM documents
WHERE embedding <-> $query < 0.3
  AND timestamp > NOW() - INTERVAL '30 days'
ORDER BY timestamp DESC
LIMIT 10;

-- Time series aggregation with vectors
SELECT time_bucket('1 hour', timestamp),
       COUNT(*),
       AVG(similarity)
FROM documents
WHERE embedding <-> $query < 0.5
GROUP BY 1
ORDER BY 1;

-- Track embedding drift over time
SELECT time_bucket('1 day', timestamp),
       AVG(embedding <-> LAG(embedding) OVER (ORDER BY timestamp))
FROM document_versions
WHERE document_id = $id
GROUP BY 1;
```

### Architecture Design (Preliminary)

**Module Structure:**
```
omen/timeseries/
├── compression.rs        // Gorilla, delta encoding for time series
├── aggregation.rs        // Time windows, downsampling, percentiles
├── storage.rs            // RocksDB layout for time series data
├── query_planner.rs      // Optimize time range + vector queries
└── retention.rs          // Automatic data expiration policies
```

**Storage Layout (RocksDB):**
```
ts:{table}:{metric}:{timestamp_bucket}:{series_id} -> CompressedValues
ts_meta:{table}:{metric} -> Schema
```

**Key Features:**
- Time bucketing (1s, 1m, 1h, 1d configurable windows)
- Downsampling (avg, min, max, sum, count, percentiles)
- Compression (Gorilla algorithm for floats, delta encoding for ints)
- Automatic retention policies (expire old data)
- Combined time range + vector similarity queries

### Use Cases

**1. Document Versioning & RAG:**
```sql
-- Find similar documents from recent versions only
SELECT doc_id, version, timestamp, content
FROM document_versions
WHERE embedding <-> $query < 0.3
  AND timestamp > NOW() - INTERVAL '7 days'
ORDER BY similarity ASC, timestamp DESC
LIMIT 10;
```

**2. Embedding Drift Detection:**
```sql
-- Monitor how embeddings change over time
SELECT time_bucket('1 day', timestamp) as day,
       AVG(embedding <-> prev_embedding) as avg_drift
FROM (
  SELECT timestamp,
         embedding,
         LAG(embedding) OVER (PARTITION BY doc_id ORDER BY timestamp) as prev_embedding
  FROM document_embeddings
) WHERE prev_embedding IS NOT NULL
GROUP BY 1
ORDER BY 1;
```

**3. Time-Weighted Similarity:**
```sql
-- Bias search results toward recent documents
SELECT *,
       (embedding <-> $query) * EXP(-age_days / 30.0) as weighted_score
FROM documents
WHERE embedding <-> $query < 0.5
ORDER BY weighted_score ASC
LIMIT 10;
```

**4. Anomaly Detection Over Time:**
```sql
-- Find unusual embedding patterns in time windows
SELECT time_bucket('1 hour', timestamp) as hour,
       COUNT(*) as count,
       AVG(embedding <-> $baseline) as avg_distance,
       STDDEV(embedding <-> $baseline) as stddev
FROM events
WHERE timestamp > NOW() - INTERVAL '24 hours'
GROUP BY 1
HAVING AVG(embedding <-> $baseline) > $threshold
ORDER BY 1;
```

### Technical Design Decisions

**1. Compression Strategy:**
- **Floats**: Gorilla algorithm (used by Facebook, InfluxDB)
  - XOR-based delta compression
  - 12x compression ratio typical
- **Integers**: Delta-of-delta encoding
  - Store differences between differences
  - 20x compression ratio typical
- **Timestamps**: Delta encoding with variable-length encoding

**2. Time Bucketing:**
- Configurable bucket sizes: 1s, 1m, 1h, 1d, 1w
- Automatic downsampling to coarser buckets (1h → 1d → 1w)
- Retention policies per bucket size

**3. Query Optimization:**
- Index on (table, metric, timestamp) for range scans
- Combine time range filter with HNSW vector search
- Push down time filters before vector similarity
- Bloom filters for metric existence checks

**4. Integration with Vector Search:**
- Extend query planner to handle time predicates
- Optimize: Filter by time first, then vector similarity
- Avoid scanning vectors outside time range
- Use HNSW ef_search adjustment based on time range size

### Implementation Timeline

**Week 11-12: Design & Architecture (2 weeks)**
- Finalize storage layout (integrate with stable omen)
- SQL syntax design and parser updates
- Compression algorithm implementation
- Query planner integration design

**Week 13-14: Core Implementation (2 weeks)**
- Time series storage module
- SQL engine extensions (time_bucket, aggregations)
- Compression and decompression
- Basic time range + vector queries

**Week 15: Integration & Testing (1 week)**
- Combined time series + vector query optimization
- Benchmark vs InfluxDB (compression, query speed)
- Documentation and examples
- Integration tests

**Total: 3-4 weeks after omen v0.1 production-ready**

### Why Wait Until Week 11+

**Avoid Conflicts with Current Work:**
- Time series would touch: sql_engine, storage, query_planner, rocks_storage
- omen architecture still stabilizing (validation phase)
- Risk breaking working code during critical validation
- Context switching during complex debugging

**Better Integration on Stable Foundation:**
- Storage layout finalized (no conflicts)
- Query planner proven (easier to extend)
- Less rework, faster implementation
- Full attention to architectural decisions

### Success Metrics

**Performance:**
- 10x better compression than uncompressed storage
- Query speed competitive with InfluxDB for time range queries
- Combined vector + time queries < 2x overhead vs vector-only

**Features:**
- All SQL time functions (time_bucket, window functions, aggregations)
- Automatic retention policies working
- Smooth integration with existing vector queries

**Market:**
- Unique positioning: "Time-aware vector search"
- Clear use cases documented (RAG versioning, drift detection)
- Compelling benchmarks vs competitors

---

## Monetization Strategy

### Free Tier (Embedded Library)
- Embedded omen library (unlimited usage)
- All core vector database features
- **Time series features included** (when released)
- PostgreSQL wire protocol
- Single-node only

### Paid Tier (omen-server - Managed Service)

**Pricing: $99-999/month based on:**
- Data volume (vectors + time series points stored)
- Query volume (queries per month)
- Retention period (time series storage duration)
- Number of nodes (distributed mode)
- Support level

**Example Tiers:**
```
Starter: $99/mo
  - 10M vectors
  - 100M time series points
  - 30-day retention
  - Community support

Pro: $299/mo
  - 100M vectors
  - 1B time series points
  - 90-day retention
  - Email support

Scale: $999/mo
  - 1B+ vectors
  - Unlimited time series
  - Custom retention
  - Priority support + SLA
```

**Value Proposition:**
- Competitors: Pinecone ($100+/mo) + InfluxDB ($50+/mo) = $150+/mo
- omen: Vector + Time Series = $99-999/mo (better value)

### Competitive Pricing Comparison

| Feature | Pinecone | InfluxDB | pgvector | omen |
|---------|----------|----------|----------|------|
| Vector search | ✅ $70-1000+/mo | ❌ | ✅ Free (need PG) | ✅ Free embedded |
| Time series | ❌ | ✅ $50-450/mo | ❌ | ✅ Included |
| Embedded | ❌ | ❌ | ✅ | ✅ |
| Managed | ✅ | ✅ | Via providers | ✅ omen-server |
| **Total Cost** | $70+ | $50+ | PG hosting | **$0-999** |

---

## Market Positioning

### Current (omen v0.1)
**"pgvector but embedded and 10x faster"**
- Target: AI/ML companies doing RAG, semantic search
- Advantage: Embeddable, no PostgreSQL needed, faster
- Launch: Week 10 (end of current validation phase)

### Future (omen v0.2 with time series)
**"Vector search with time-travel"**
- Target: AI companies doing document versioning, RAG with recency
- Unique: Only vector DB with native time series support
- Advantage: Unified database (vectors + time + relational)
- Launch: Week 15 (3-4 weeks after v0.1)

### Differentiators

**vs Pinecone:**
- ✅ Embeddable (they're cloud-only)
- ✅ Time series support (they have none)
- ✅ Cheaper ($0 embedded vs $70+/mo)

**vs pgvector:**
- ✅ 10x faster (learned indexes + BQ)
- ✅ Time series support (PostgreSQL has basic)
- ✅ Standalone (no PostgreSQL needed)

**vs InfluxDB:**
- ✅ Vector search (they have none)
- ✅ Combined queries (time + vectors)
- ✅ Embedded (they're server-only)

**Unique Position:**
- **Only** embedded vector database with native time series
- **Only** time-aware vector search capability
- **Only** unified vector + time + relational database

---

## Next Steps

### Immediate (Current Session)
- ✅ Document time series roadmap
- 🎯 Continue omen validation work (Phase 2)

### Week 7-10: Focus on omen v0.1
- Finish Phase 2 validation (input fuzzing, serialization tests)
- Complete Phase 3 benchmarks (prove 10x claim)
- Production hardening (docs, examples, monitoring)
- Launch omen v0.1

### Week 11+: Add Time Series Module
- Design time series architecture (on stable omen foundation)
- Implement time series storage and compression
- Extend SQL engine with time functions
- Benchmark and launch omen v0.2

---

## References

- omen repository: https://github.com/omendb/omen
- omen-server repository: https://github.com/omendb/omen-server
- omen-core repository: https://github.com/omendb/omen-core (private)
- ai/STATUS.md: Current status and recent work
- ai/DECISIONS.md: Architectural decisions with rationale
- CLAUDE.md: Agent instructions and project overview
