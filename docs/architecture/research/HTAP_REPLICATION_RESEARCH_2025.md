# HTAP Replication & Query Routing Research (2025)

**Date**: January 2025
**Purpose**: Research foundation for Phase 9+ (Tiered Storage & Query Routing)
**Status**: Research complete, ready for implementation

---

## Executive Summary

Research into state-of-the-art HTAP systems (TiDB, CockroachDB, ClickHouse) and recent academic work (2024-2025) reveals three critical components for production HTAP:

1. **Raft-based Replication** - Row → Column sync with strong consistency (TiDB approach)
2. **Temperature-Aware Placement** - Learned models for hot/cold classification
3. **CDC Pipelines** - Debezium + Kafka for real-time OLTP → OLAP streaming

**Key Finding**: OmenDB's multi-engine architecture aligns with industry best practices. Phase 9 should implement WAL-based replication (simpler than full CDC initially) with learned temperature classification.

---

## 1. HTAP Replication Strategies (2024-2025)

### 1.1 TiDB 7.0 (Market Leader, $270M raised)

**Architecture**:
- **TiKV**: Row store (OLTP) - Raft-based consensus
- **TiFlash**: Column store (OLAP) - Raft learner replicas
- **Replication**: Asynchronous Raft log replication (~seconds latency)

**Key Innovations**:
```
TiKV (OLTP)  →  Raft Log  →  TiFlash (OLAP)
   ↓                            ↓
Row-based writes          Columnar analytics
<10ms latency            Vectorized execution
```

**Strengths**:
- ✅ Strong consistency (Raft consensus)
- ✅ Automatic failover and load balancing
- ✅ Real-time analytics (seconds delay)
- ✅ No ETL required

**Trade-offs**:
- ⚠️ Complex distributed coordination
- ⚠️ Higher operational overhead
- ⚠️ Raft replication adds ~2-5 seconds lag

**Reference**: [TiDB HTAP Architecture](https://www.pingcap.com/blog/how-an-htap-database-handles-oltp-and-olap-workloads-at-the-same-time/)

### 1.2 CockroachDB 24.1 ($5B valuation, ~$200M ARR)

**Architecture**:
- **MVCC** - Multi-version concurrency control
- **Follower Reads** - Serve analytics from local replicas
- **ColFlow Engine** - In-memory vectorized execution

**Key Innovations**:
```
Write → MVCC → Synchronous Replication → Follower Replicas
                                              ↓
                                         OLAP Queries
```

**Strengths**:
- ✅ No separate column store (simpler)
- ✅ Synchronous replication (zero data loss)
- ✅ Multi-GB scans without warehouse spin-up
- ✅ Strongly consistent reads across regions

**Trade-offs**:
- ⚠️ Row-based storage (less optimal for OLAP)
- ⚠️ Synchronous replication overhead

### 1.3 ClickHouse + CDC (Pure OLAP approach)

**Architecture**:
- **Source**: PostgreSQL/MySQL (OLTP)
- **Pipeline**: Debezium + Kafka (CDC)
- **Target**: ClickHouse (columnar OLAP)

**Key Innovations**:
```
OLTP DB → Debezium → Kafka → ClickHouse
(Postgres)  (CDC)   (Buffer)  (Analytics)
```

**Strengths**:
- ✅ Best-in-class OLAP performance
- ✅ Mature CDC ecosystem (Debezium)
- ✅ Decoupled: scale OLTP/OLAP independently

**Trade-offs**:
- ⚠️ Eventual consistency (~seconds to minutes)
- ⚠️ Complex pipeline (5+ components)
- ⚠️ Operational overhead (JVM tuning, monitoring)

**Reference**: [ClickHouse CDC Guide](https://clickhouse.com/blog/clickhouse-postgresql-change-data-capture-cdc-part-1)

---

## 2. Hot/Cold Data Placement (2024 Research)

### 2.1 Temperature Models (DEXA 2024)

**Paper**: "A Hierarchical Storage Mechanism for Hot and Cold Data Based on Temperature Model"

**Key Concept**: Quantify data "temperature" based on access patterns.

**Temperature Formula**:
```
T(data, t) = α × AccessFrequency(t) + β × Recency(t) + γ × Size(t)

where:
  α, β, γ = learned weights (tunable)
  AccessFrequency(t) = # accesses in time window
  Recency(t) = time since last access
  Size(t) = data size (larger = colder threshold)
```

**Placement Strategy**:
```
Temperature  | Storage Tier | Latency  | Cost
-------------|--------------|----------|-------
Hot   (>80%) | L1 (Memory)  | <1µs     | High
Warm  (30-80%)| L2 (SSD)    | <100µs   | Medium
Cold  (<30%) | L3 (HDD/S3)  | >1ms     | Low
```

**Implementation**:
1. Track access patterns per key/row
2. Calculate temperature periodically (e.g., every 5 minutes)
3. Migrate data between tiers based on thresholds
4. Use hysteresis to prevent thrashing

**OmenDB Application**:
- L1 (AlexStorage): Ultra-hot cache (<1µs reads)
- L2 (RocksDB): General OLTP (proven stability)
- L3 (Arrow/Parquet): Cold analytics (columnar)

### 2.2 Learned Query Optimization (Journal of Big Data, Dec 2024)

**Paper**: "A systematic review of deep learning applications in database query execution"

**Key Systems**:

#### WISK (Workload-aware Learned Index for Spatial Keyword queries)
- Self-adapts to query workload
- Optimizes index structure based on actual access patterns
- 2-4x speedup over static indexes

#### LSched (Workload-aware Query Scheduler)
- Uses ML to prioritize queries
- Optimizes operator execution and resource allocation
- Reduces p99 latency by 30-50%

**Relevance to OmenDB**:
- Learn from query patterns to route to optimal storage tier
- Predict temperature based on query workload (not just access count)
- Example: Frequent JOIN on table → promote to L1 (AlexStorage)

### 2.3 Seasonal Feature-Based Classification (ScienceDirect, May 2024)

**Paper**: "Cost-effective data classification storage through text seasonal features"

**Key Innovation**: Use temporal patterns for cold/hot classification.

**Strategy**:
```
Data Category        | Storage     | Migration Policy
---------------------|-------------|------------------
Current season       | NVM (hot)   | Keep in fast storage
Next season/seasonal | SSD (warm)  | Pre-fetch before season
Other seasons        | HDD (cold)  | Archive until needed
```

**OmenDB Application**:
- Time-series data: Recent → L1, Last month → L2, Archive → L3
- E-commerce: Q4 (holiday) → promote sales data to L1
- Financial: Tax season → promote tax records to L1

---

## 3. Change Data Capture (CDC) Mechanisms

### 3.1 Debezium + Kafka Architecture

**Components**:
```
┌─────────────┐     ┌──────────┐     ┌───────┐     ┌──────────┐
│ PostgreSQL  │ --> │ Debezium │ --> │ Kafka │ --> │ Consumer │
│ (OLTP)      │     │ Connector│     │ Topic │     │ (OLAP)   │
└─────────────┘     └──────────┘     └───────┘     └──────────┘
     WAL                CDC            Buffer         Sink
```

**How It Works**:
1. **Source**: Enable logical replication on PostgreSQL
2. **Debezium**: Reads WAL (Write-Ahead Log) in real-time
3. **Kafka**: Buffers change events (insert/update/delete)
4. **Consumer**: Applies changes to OLAP store (ClickHouse, Parquet, etc.)

**Latency**: ~100ms to 2 seconds end-to-end

### 3.2 Slack's CDC Transition (Confluent Current 2024)

**Case Study**: Slack migrated from batch → streaming CDC

**Before**:
- Batch pipeline: Every 15 minutes
- Lag: 15-60 minutes
- Cost: High (redundant full scans)

**After (CDC)**:
- Streaming: Debezium + Kafka + Vitess
- Lag: <5 seconds
- Cost: 60% reduction (incremental only)

**Lessons Learned**:
- ⚠️ Snapshot strategy critical (initial load)
- ⚠️ JVM tuning required (Debezium is Java)
- ⚠️ Monitoring essential (lag tracking)
- ✅ Worth complexity for real-time analytics

### 3.3 WAL-Based Replication (Simpler Alternative)

**Approach**: Direct WAL streaming (no Kafka overhead)

**Architecture**:
```
AlexStorage      →    WAL    →   Replication   →   ArrowStorage
(OLTP writes)         (Durability)  (Background)      (OLAP reads)
```

**Advantages**:
- ✅ Simpler: No Kafka/Debezium dependencies
- ✅ Lower latency: Direct streaming
- ✅ Atomic: WAL guarantees consistency

**Trade-offs**:
- ⚠️ Tightly coupled (harder to scale independently)
- ⚠️ Single point of failure (no Kafka buffering)

**OmenDB Phase 9 Recommendation**: Start with WAL-based replication, migrate to Debezium + Kafka when scale demands it (>100K ops/sec).

---

## 4. OmenDB Implementation Strategy

### Phase 9: WAL Replication (Weeks 1-2)

**Goal**: Real-time OLTP → OLAP sync

**Architecture**:
```rust
AlexStorage (L1/OLTP)
    ↓ insert(key, value)
    ↓ log to WAL
    ↓
WAL Replicator (background thread)
    ↓ read WAL entries
    ↓ convert row → columnar
    ↓
ArrowStorage (L3/OLAP)
    ↓ batch append to Parquet
```

**Implementation**:
1. Extend existing WAL (`src/wal.rs`)
2. Add `WalReplicator` component
3. Buffer changes, batch flush to Arrow (e.g., every 1000 rows or 5 seconds)
4. Use `RecordBatch` for efficient columnar writes

**Expected Performance**:
- Latency: <2 seconds (buffered batching)
- Throughput: 50K inserts/sec → 50K Arrow appends/sec
- Overhead: <5% CPU (background thread)

### Phase 10: Query Router (Weeks 3-4)

**Goal**: Intelligent query routing based on workload

**Components**:
```rust
QueryRouter {
    temperature_model: TemperatureModel,  // Tracks access patterns
    cost_estimator: CostEstimator,        // Estimates query cost per tier
    tier_selector: TierSelector,          // Chooses L1/L2/L3
}
```

**Routing Logic**:
```rust
fn route_query(&self, query: &Query) -> StorageTier {
    let temp = self.temperature_model.get_temperature(&query.keys);
    let cost_l1 = self.cost_estimator.estimate(query, Tier::L1);
    let cost_l2 = self.cost_estimator.estimate(query, Tier::L2);
    let cost_l3 = self.cost_estimator.estimate(query, Tier::L3);

    // Simple heuristic (can evolve to learned model)
    if temp > 0.8 && cost_l1 < cost_l2 {
        Tier::L1  // AlexStorage (hot, point queries)
    } else if query.is_range() || query.is_aggregation() {
        Tier::L3  // ArrowStorage (OLAP)
    } else {
        Tier::L2  // RocksDB (general OLTP)
    }
}
```

**Temperature Tracking**:
```rust
struct TemperatureModel {
    access_counts: HashMap<KeyRange, u64>,  // Access frequency
    last_access: HashMap<KeyRange, Instant>, // Recency
    window: Duration,                        // Time window (e.g., 5 min)
}

impl TemperatureModel {
    fn get_temperature(&self, keys: &[i64]) -> f64 {
        let freq = self.access_counts.get(keys).unwrap_or(&0);
        let recency = self.last_access.get(keys)
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(u64::MAX);

        // Simple formula (α=0.6, β=0.4)
        let freq_score = (*freq as f64 / 1000.0).min(1.0);
        let recency_score = (1.0 - (recency as f64 / 300.0)).max(0.0);

        0.6 * freq_score + 0.4 * recency_score
    }
}
```

### Phase 11: Learned Optimization (Weeks 5-6)

**Goal**: ML-based query routing

**Approach**:
1. Collect query logs (query, tier_chosen, latency, cost)
2. Train simple ML model (e.g., decision tree, gradient boosting)
3. Predict optimal tier based on query features
4. Online learning: update model as patterns change

**Features**:
- Query type (point/range/join/aggregation)
- Key range size
- Historical access frequency
- Time of day (seasonal patterns)
- Result set size

**Model**: XGBoost or LightGBM (fast inference <1ms)

**Expected Improvement**: 20-40% latency reduction vs rule-based routing

---

## 5. Research Gaps & Future Work

### 5.1 Immediate Needs (Phase 9)

✅ **Well-Researched**:
- WAL-based replication (existing work in PostgreSQL, TiDB)
- Temperature models (DEXA 2024 paper)
- CDC pipelines (Debezium + Kafka)

⚠️ **Needs More Investigation**:
- **Backpressure handling**: What if OLAP sink is slower than OLTP writes?
- **Schema evolution**: How to handle ALTER TABLE in replicated system?
- **Consistency guarantees**: Eventual vs strong consistency trade-offs

### 5.2 Medium-Term Research (Phases 10-11)

📚 **Papers to Read**:
1. **WISK** (Workload-aware learned indexes) - VLDB 2024
2. **LSched** (Learned query scheduling) - SIGMOD 2024
3. **LITune** (RL for hyperparameter tuning) - Feb 2025
4. **NFL** (Distribution normalization for Zipfian data) - VLDB 2022

🔬 **Experiments Needed**:
- Temperature model calibration on OmenDB workloads
- Learned routing vs rule-based routing A/B test
- Multi-tier query execution (scatter-gather across L1/L2/L3)

### 5.3 Long-Term Vision (6+ months)

🚀 **Advanced Features**:
- **Adaptive Partitioning**: Learn optimal data partitioning based on query patterns
- **Predictive Pre-fetching**: Migrate data to hot tier before query arrives
- **Federated Query**: Single query spans L1 (hot) + L3 (cold) transparently
- **Automatic Tiering**: ML model decides when to promote/demote data

---

## 6. Current Reality & Honest Positioning

### What OmenDB Actually Has (January 2025)

**Validated**:
- ✅ ALEX learned index implementation (4.81x faster point queries vs RocksDB on 10M sequential keys)
- ✅ CDFShop adaptive sampling (78-593x faster index building)
- ✅ All tests passing (63/63)
- ✅ Single-node OLTP performance

**Scope**: Single-node, in-memory/mmap, OLTP-focused prototype

### What We Don't Have

**Critical Gaps**:
- ❌ High Availability - No replication, no failover
- ❌ Distributed System - Single node only
- ❌ OLAP Engine - Arrow/Parquet integration not implemented yet
- ❌ Production Testing - No chaos engineering, no customer deployments
- ❌ Enterprise Features - No security, monitoring, compliance tooling

### Fair Comparison to Production Systems

**vs TiDB ($270M funding, market leader)**:
- ✅ OmenDB: 4.81x faster point queries (learned-friendly workloads)
- ❌ TiDB: Distributed, HA, geo-replication, mature OLAP (TiFlash), thousands of customers
- **Reality**: OmenDB is faster on specific workloads; TiDB is production-ready distributed system

**vs CockroachDB ($5B valuation, ~$200M ARR)**:
- ✅ OmenDB: Potential for specialized column store (not built yet)
- ❌ CockroachDB: Battle-tested distributed SQL, MVCC, follower reads, compliance features
- **Reality**: OmenDB is prototype; CockroachDB is enterprise-grade

**vs ClickHouse + CDC (100+ PB deployments)**:
- ✅ OmenDB: Simpler unified vision (when fully implemented)
- ❌ ClickHouse: Best-in-class OLAP, mature CDC ecosystem, proven at scale
- **Reality**: OmenDB is concept; ClickHouse is proven at massive scale

### Research Positioning

**What We Are**:
- Research-driven prototype exploring learned index optimization for HTAP workloads
- Single validated advantage: 4.81x faster point queries on learned-friendly distributions
- Building toward simpler HTAP architecture than multi-component pipelines

**What We're Not**:
- Not production-ready (no HA, single-node, prototype quality)
- Not enterprise-grade (missing security, compliance, support)
- Not proven at scale (zero customer deployments)

**Target Market** (when production-ready):
- Developers building real-time analytics apps
- Teams prioritizing simplicity over maximum scale
- Workloads with learned-friendly data distributions
- NOT for: Enterprise workloads needing HA, compliance, 99.99% uptime

### Research Roadmap to Parity

**Phase 9-11** (Next 6 weeks):
- WAL replication (OLTP → OLAP sync)
- Temperature-based query routing
- Arrow/Parquet integration

**Phase 12+** (Months 3-6):
- High availability (Raft consensus)
- Distributed execution
- Production hardening

**Honest Timeline**: 12+ months to approach feature parity with TiDB/CockroachDB

---

## 7. Architecture Update (January 2025)

**⚠️ IMPORTANT**: After analyzing the actual codebase, the architecture assumptions in this research document were **incorrect**.

### What We Actually Have

OmenDB **already has a unified HTAP architecture** via the Table system:
- ✅ Arrow/Parquet columnar storage (OLAP-ready)
- ✅ ALEX learned index (OLTP-optimized)
- ✅ Both in the same table (no replication needed)

### Corrected Phase 9 Implementation

See: `internal/PHASE_9_HTAP_ARCHITECTURE.md` for the actual implementation plan.

**What We DON'T Need** (from this research):
- ❌ WAL replication between separate systems (already unified)
- ❌ Schema conversion (already columnar)
- ❌ CDC pipeline (no separate OLTP/OLAP)

**What We DO Need** (revised):
- ✅ DataFusion integration for SQL on Arrow tables
- ✅ Query router (point queries → ALEX, analytics → DataFusion)
- ✅ Temperature tracking for hot/cold data

### Research Value

This document remains valuable for:
1. Understanding industry HTAP approaches (TiDB, CockroachDB)
2. Temperature model concepts (frequency + recency)
3. Learned optimization strategies (WISK, LSched)

But the implementation plan has been **revised** based on actual architecture.

## 8. Next Steps (Revised)

### Immediate (This Week)

1. ✅ **Architecture Discovery** - Identified unified HTAP in Table system
2. 🔨 **DataFusion Integration** - TableProvider for Table
3. 📐 **Query Router Design** - Route by workload type

### Short-Term (Weeks 1-2)

1. **Implement WalReplicator** (`src/replication/wal_replicator.rs`)
2. **Add Arrow Sink** (batch append to Parquet)
3. **Benchmark Replication** (latency, throughput)

### Medium-Term (Weeks 3-6)

1. **Temperature Model** (basic frequency + recency)
2. **Query Router** (rule-based, then learned)
3. **Multi-Tier Queries** (federation across L1/L2/L3)

---

## 8. References

### Industry (2024-2025)

1. **TiDB HTAP Architecture** - https://www.pingcap.com/blog/how-an-htap-database-handles-oltp-and-olap-workloads-at-the-same-time/
2. **CockroachDB HTAP** - https://www.getgalaxy.io/learn/data-tools/best-htap-databases-2025
3. **ClickHouse CDC** - https://clickhouse.com/blog/clickhouse-postgresql-change-data-capture-cdc-part-1
4. **Slack CDC Migration** - Confluent Current 2024

### Academic (2024)

1. **Temperature Model** - DEXA 2024: "A Hierarchical Storage Mechanism for Hot and Cold Data Based on Temperature Model"
2. **Learned Query Execution** - Journal of Big Data, Dec 2024: "A systematic review of deep learning applications in database query execution"
3. **Seasonal Classification** - ScienceDirect, May 2024: "Cost-effective data classification storage through text seasonal features"
4. **Data Partitioning Survey** - JCST, March 2024: "Enhancing Storage Efficiency and Performance"

### Tools & Frameworks

1. **Debezium** - https://debezium.io/
2. **Apache Kafka** - https://kafka.apache.org/
3. **Arrow & Parquet** - https://arrow.apache.org/

---

**Document Status**: Complete
**Next Action**: Design Phase 9 WAL Replication implementation
**Est. Implementation**: 2 weeks (Phase 9), 4 weeks (Phases 9-10)
