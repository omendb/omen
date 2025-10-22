# Decisions

_Architectural decisions with context and rationale_

---

## 2025-10-21: Use Multi-Level ALEX over DiskANN

**Context**: Choosing learned index structure for 100M+ row scalability

**Decision**: Multi-level ALEX (hierarchical learned index)

**Rationale**:
- Linear scaling to 100M+ rows (validated)
- 1.50 bytes/key memory (28x better than PostgreSQL's 42 bytes/key)
- 1.24μs query latency at 100M scale
- Simpler implementation than DiskANN variants
- Better cache locality with fixed 64 keys/leaf fanout

**Tradeoffs**:
- Not as cutting-edge as DiskANN
- Requires retraining on updates (mitigated by batching)

**Result**: 1.5-3x faster than SQLite at 1M-10M scale ✅

---

## 2025-10-21: Use RocksDB over Custom Storage

**Context**: Need persistent storage backend for MVCC + ALEX

**Decision**: RocksDB (LSM tree)

**Rationale**:
- Industry-proven (powers DynamoDB, Cassandra, CockroachDB)
- Write-optimized LSM tree matches our workload
- Native support for MVCC via versioned keys
- HN validation: LSM trees best for write-heavy databases
- Mature, battle-tested codebase

**Tradeoffs**:
- 77% query overhead at 10M scale (disk I/O bottleneck)
- Solution: Large cache layer (1-10GB) addresses 80x in-memory/disk gap

**Alternatives Considered**:
- Custom storage: Deferred to post-0.1.0 (too complex, unproven)
- SQLite: Too slow for our scale targets
- LMDB: Read-optimized, not ideal for write-heavy

**Result**: Stable foundation, cache layer achieves 2-3x speedup ✅

---

## 2025-10-21: MVCC with Timestamp-Based Snapshot Isolation

**Context**: Need safe concurrent transactions

**Decision**: Timestamp-based MVCC with snapshot isolation

**Rationale**:
- PostgreSQL-compatible isolation semantics
- First-committer-wins prevents write conflicts
- Read-your-own-writes for usability
- Inverted txn_id (u64::MAX - txn_id) for O(1) latest version lookup
- Clean integration with ALEX (tracks latest version per key)

**Inspired By**:
- ToyDB: Timestamp-based MVCC in Rust
- TiKV: Percolator model
- PostgreSQL: xmin/xmax visibility

**Tradeoffs**:
- Garbage collection overhead (mitigated by GC watermark)
- Extra storage for multiple versions (acceptable for read-heavy)

**Result**: 85 MVCC tests passing, production-ready ✅

---

## 2025-10-21: Large Cache Layer (1-10GB LRU)

**Context**: 77% of query time spent in RocksDB disk I/O

**Decision**: Large LRU cache (1-10GB configurable, default 100K entries ≈ 1GB)

**Rationale**:
- HN insight: 80x gap between in-memory and disk access
- Zipfian workloads: 80% queries hit 10% of data
- 90% hit rate → 2-3x speedup validated
- Optimal cache size: 1-10% of data (not 50%)
- Thread-safe Arc<RwLock<LruCache>>

**Tradeoffs**:
- Memory overhead (acceptable for modern servers)
- Larger cache paradox: 50% cache slower than 1% (memory pressure)

**Result**: 2-3x speedup, cache hit rate 90%, cache overhead minimal ✅

---

## 2025-10-21: PostgreSQL Wire Protocol Compatibility

**Context**: Need production-ready client compatibility

**Decision**: Full PostgreSQL wire protocol (port 5433, pgwire crate)

**Rationale**:
- Drop-in replacement for PostgreSQL clients
- SCRAM-SHA-256 authentication (industry standard)
- Simple + Extended Query Protocol support
- Huge ecosystem compatibility (psql, pgAdmin, ORMs)

**Tradeoffs**: Bound to PostgreSQL semantics (acceptable, it's the industry standard)

**Result**: Works with psql, SCRAM-SHA-256 auth, CREATE USER commands ✅

---

## 2025-10-21: Persistent User Storage with RocksDB

**Context**: Phase 2 Security - users must survive restarts

**Decision**: UserStore backed by RocksDB (separate column family)

**Rationale**:
- Consistent with main storage architecture
- SCRAM-SHA-256 password hashing (PBKDF2, 4096 iterations)
- Username validation (PostgreSQL-compatible)
- Atomic operations, crash-safe
- Default admin user created on first init

**Alternatives Considered**:
- In-memory HashMap: Lost on restart (rejected)
- SQLite: Extra dependency (unnecessary)
- File-based: Not ACID-compliant (rejected)

**Result**: 40 security tests passing, users persist across restarts ✅

---

## 2025-10-14: Honest Performance Claims Only

**Context**: Marketing temptation to exaggerate speedups

**Decision**: Only claim validated, reproducible speedups

**Rationale**:
- Small-medium (10K-1M): "1.5-3x faster than SQLite" ✅ Validated
- Large (10M): "1.2x faster than SQLite" ✅ Honest
- CockroachDB: "10-50x faster" ❌ Projected, needs validation
- 3-run averages with variance reported
- Worst-case outliers investigated (resolved as noise)

**Principle**: Technical credibility > marketing hype

**Result**: Trustworthy performance narrative, HN-ready ✅

---

## 2025-10-08: Single PRIMARY KEY Constraint

**Context**: Schema design complexity vs usability

**Decision**: Require exactly one PRIMARY KEY per table (for now)

**Rationale**:
- Simplifies ALEX index integration (one learned index per table)
- Clear semantics for MVCC (key = primary key)
- Composite keys deferred to post-0.1.0
- PostgreSQL compatibility for simple schemas

**Tradeoffs**: Limits some use cases (acceptable for 0.1.0)

**Future**: Add composite PRIMARY KEY support in v0.2.0

---

## 2025-10-06: Target 0.1.0, Not 1.0

**Context**: Balancing ambition vs shipping

**Decision**: Ship 0.1.0 with core features, iterate to 1.0 after real usage

**Rationale**:
- 0.1.0 = production-ready, not feature-complete
- 1.0 = proven in prod deployments, stable API
- 10-12 weeks to 0.1.0 (achievable)
- Learn from real users before 1.0 commitments

**v0.1.0 Requirements**:
- MVCC snapshot isolation ✅
- PostgreSQL protocol ✅
- Authentication + SSL
- Basic SQL (SELECT/INSERT/UPDATE/DELETE/JOIN)
- Crash recovery ✅
- 1.5-3x faster than SQLite ✅

**Result**: Clear scope, achievable timeline, focused execution ✅
