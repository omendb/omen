# Research

_Index of research findings with key takeaways_

---

## Multi-Level ALEX Performance (researched 2025-10-14)

**Sources**:
- internal/research/MULTI_LEVEL_RESULTS.md
- internal/research/100M_SCALE_RESULTS.md
- internal/research/ALEX_PERFORMANCE_VALIDATION.md

**Key Findings**:
- Linear scaling validated to 100M+ rows
- 1.24μs query latency at 100M scale (memory: 1.50 bytes/key)
- 28x memory efficient vs PostgreSQL (42 bytes/key)
- Fixed fanout (64 keys/leaf) optimal for cache locality

**Relevance**: Core index structure proven scalable
**Decision**: Adopted multi-level ALEX over DiskANN → ai/DECISIONS.md

---

## Cache Effectiveness Validation (researched 2025-10-21)

**Sources**:
- internal/CACHE_DAY_1-5_VALIDATION.md
- internal/CACHE_TUNING_GUIDE.md

**Key Findings**:
- Zipfian workload (80% queries hit 10% data): 90% hit rate
- Optimal cache size: 1-10% of dataset (not 50%)
- 2-3x speedup at 100K-1M scale
- Larger cache paradox: 50% cache slower than 1% (memory pressure)

**Applied**:
- LRU cache implementation (1-10GB configurable)
- Default 100K entries ≈ 1GB for 10KB rows
- Cache invalidation on UPDATE/DELETE

→ Details: internal/CACHE_IMPLEMENTATION_PLAN.md

---

## HN Database Architecture Insights (researched 2025-10-21)

**Sources**:
- internal/research/HN_DATABASE_INSIGHTS_ANALYSIS.md
- HN #45657827
- "Designing Data-Intensive Applications" Ch. 3

**Key Findings**:
- **80x gap**: In-memory vs disk access speed
- **LSM trees**: Power DynamoDB (80M req/s), Cassandra, RocksDB
- **Sparse indices**: ALEX approach validated by DB fundamentals
- **Immutable records**: MVCC append-only pattern is best practice

**Validation**: OmenDB architecture aligns with industry best practices
**Impact**: Confirms RocksDB + ALEX + MVCC + Cache stack

→ Details: internal/research/HN_DATABASE_INSIGHTS_ANALYSIS.md

---

## Honest Benchmarking Methodology (researched 2025-10-14)

**Sources**:
- internal/research/10M_SCALE_VALIDATION.md
- internal/technical/BENCHMARK_VARIANCE_ANALYSIS_OCT_21.md

**Key Findings**:
- 3-run minimum with variance reporting
- Outliers investigated (not dismissed)
- Same features: ACID, durability, persistence
- Same workload: sequential vs random distribution
- Worst-case documented (10M sequential: 1.93x vs target 2x)

**Applied**:
- Claim "1.5-3x faster" (validated small-medium scale)
- Report "1.2x faster at 10M" (honest, needs optimization)
- Never claim unvalidated projections as facts

→ Details: internal/research/ALEX_SQLITE_BENCHMARK_RESULTS.md

---

## MVCC Implementation Patterns (researched 2025-10-20)

**Sources**:
- ToyDB: Timestamp-based MVCC in Rust
- TiKV: Percolator model
- PostgreSQL: xmin/xmax visibility
- Mini-LSM: Snapshot read implementation

**Key Findings**:
- Inverted txn_id (u64::MAX - txn_id) for newest-first sorting
- First-committer-wins prevents write conflicts
- Read-your-own-writes essential for usability
- Garbage collection via watermark (oldest active txn)

**Applied**:
- 6 MVCC components (oracle, storage, visibility, conflict, transaction, GC)
- 85 tests (62 unit + 23 integration)
- Production-ready snapshot isolation

→ Details: internal/technical/MVCC_DESIGN.md

---

## Custom Storage Analysis (researched 2025-10-21)

**Sources**:
- internal/research/CUSTOM_STORAGE_ANALYSIS.md
- SlateDB, SurrealDB, TiKV architecture reviews

**Key Findings**:
- Custom storage = 6-12 months engineering effort
- RocksDB already optimized (10+ years, production-proven)
- Performance gains: 2-3x possible, not 10x
- Complexity/risk trade-off not worth it for 0.1.0

**Decision**: Defer custom storage to post-0.1.0
**Mitigation**: Large cache layer addresses RocksDB overhead

→ Details: internal/research/CUSTOM_STORAGE_ANALYSIS.md

---

## Competitive Landscape (researched 2025-10-08)

**Sources**:
- internal/research/COMPETITIVE_ASSESSMENT_POST_ALEX.md
- internal/COMPETITIVE_ANALYSIS.md

**Key Findings**:
- **vs SQLite**: 1.5-3x faster (validated) ✅
- **vs PostgreSQL**: 28x memory efficient (ALEX vs B-tree)
- **vs CockroachDB**: 10-50x single-node writes (projected, needs validation)
- **vs TiDB**: No replication lag, simpler architecture
- **vs SingleStore**: Multi-level ALEX advantage over B-tree

**Market Position**: High-performance single-node HTAP database
**Target**: Developers who outgrow SQLite, need PostgreSQL compatibility

→ Details: internal/research/COMPETITIVE_ASSESSMENT_POST_ALEX.md

---

## Open Questions

- [ ] MVCC overhead measurement (target: <20%)
- [ ] RocksDB tuning parameters (reduce 77% overhead to <30%)
- [ ] Optimal compaction strategy for our workload
- [ ] CockroachDB single-node benchmark validation
- [ ] Window functions implementation approach

---

## Research Archive

Completed research moved to internal/research/ for permanent reference:
- 100M_SCALE_RESULTS.md
- ALEX_PERFORMANCE_VALIDATION.md
- HN_DATABASE_INSIGHTS_ANALYSIS.md
- CUSTOM_STORAGE_ANALYSIS.md
- MVCC implementation references (ToyDB, TiKV, PostgreSQL)
- Competitive analysis (SQLite, PostgreSQL, CockroachDB, TiDB)

_For detailed technical analysis, see internal/research/ directory_
