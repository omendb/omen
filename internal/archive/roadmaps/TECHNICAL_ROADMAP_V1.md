# OmenDB v1.0 Technical Roadmap - Production-Ready Database

**Goal**: Enterprise-grade SOTA database for time-series/HTAP workloads
**Timeline**: 10-12 weeks to production-ready v1.0
**Philosophy**: Quality over speed, features over marketing

---

## Executive Summary

**Current State:**
- ✅ Fast (1.2x-2.5x vs SQLite)
- ✅ Scalable (100M validated)
- ✅ Basic transactions (BEGIN/COMMIT/ROLLBACK)
- ❌ **~15% SQL coverage** (unusable for most apps)
- ❌ **No concurrent transaction safety** (no MVCC)
- ❌ **No security** (no auth, no SSL)
- ❌ **1 failing test**

**Target State (v1.0):**
- ✅ 40-50% SQL coverage (enough for time-series/HTAP apps)
- ✅ MVCC with snapshot isolation (safe concurrent transactions)
- ✅ Authentication + SSL (deployable to production)
- ✅ Observability (EXPLAIN, logging, metrics)
- ✅ Backup/restore (data safety)
- ✅ 100% tests passing
- ✅ Production hardened (stress tested, documented)

**Differentiation:**
- **NOT** a PostgreSQL replacement
- **NOT** a general-purpose database
- **YES** to time-series/IoT/metrics workloads
- **YES** to HTAP (no ETL, single system)
- **YES** to operational simplicity

---

## Phase 1: Foundation & Stability (2 weeks)

**Goal**: Fix critical bugs, establish stable baseline

### Week 1: Critical Fixes
**Days 1-2: Fix Failing Test + Clean Build**
- [ ] Fix `test_extract_primary_key_table_level` (src/constraints.rs:305)
  - Issue: Incorrect PRIMARY KEY extraction for table-level constraints
  - Fix: Check table-level pattern BEFORE inline pattern
  - Estimated: 30 minutes
- [ ] Run `cargo fix --allow-dirty` to fix compiler warnings
  - 37 unused variable/import warnings
  - Estimated: 1 hour
- [ ] Run all tests, ensure 100% pass rate
  - Current: 356/357 passing (99.7%)
  - Target: 357/357 passing (100%)

**Days 3-5: MVCC Research & Design**
- [ ] Study ToyDB MVCC implementation (reference from research)
- [ ] Study TiKV timestamp-based approach
- [ ] Design OmenDB MVCC architecture:
  - Timestamp-based versioning
  - Version chains in RocksDB
  - Transaction ID allocation
  - Snapshot isolation semantics
  - Garbage collection strategy
- [ ] Document design decisions
- [ ] Create implementation plan
- [ ] Prototype key data structures

**Deliverable**: Clean codebase (100% tests), MVCC design doc

### Week 2: MVCC Foundation
**Days 1-3: Transaction Manager**
- [ ] Implement timestamp oracle (monotonic transaction IDs)
- [ ] Transaction metadata tracking
- [ ] Active transaction registry
- [ ] Commit timestamp assignment

**Days 4-5: Versioned Storage**
- [ ] Extend RocksDB key format: `(key, txn_id)` → value
- [ ] Version chain implementation
- [ ] Write new versions (append-only)
- [ ] Read at snapshot (find visible version)

**Deliverable**: MVCC prototype (read-your-own-writes working)

---

## Phase 2: MVCC Implementation (2-3 weeks)

**Goal**: Full MVCC with snapshot isolation

### Week 3: Snapshot Isolation
**Days 1-3: Read Logic**
- [ ] Snapshot read at timestamp
- [ ] Visibility rules (txn_id vs snapshot_ts)
- [ ] Handle uncommitted versions
- [ ] Handle deleted versions

**Days 4-5: Write Conflict Detection**
- [ ] Write-write conflict detection
- [ ] First-committer-wins rule
- [ ] Retry logic for conflicts
- [ ] Error handling (serialization failures)

### Week 4: Concurrency & Testing
**Days 1-2: Concurrent Transaction Support**
- [ ] Multiple active transactions
- [ ] Lock-free reads
- [ ] Optimistic concurrency control
- [ ] Read-only transaction optimization

**Days 3-5: MVCC Testing**
- [ ] Unit tests (visibility, conflicts)
- [ ] Integration tests (concurrent txns)
- [ ] Stress tests (100+ concurrent writers)
- [ ] Anomaly tests (lost updates, write skew)
- [ ] Performance validation (ensure no regression)

**Deliverable**: Production-ready MVCC implementation

---

## Phase 3: Security & Authentication (1-2 weeks)

**Goal**: Make database deployable to production

### Week 5: Authentication
**Days 1-3: Password Authentication**
- [ ] User management (CREATE USER, DROP USER)
- [ ] Password storage (bcrypt/argon2)
- [ ] Authentication on connection
- [ ] PostgreSQL auth protocol (md5/scram-sha-256)
- [ ] Default admin user

**Days 4-5: SSL/TLS**
- [ ] TLS support for PostgreSQL protocol
- [ ] Certificate management
- [ ] Enforce encryption option
- [ ] Self-signed cert generation for dev

**Optional (if time): Basic Authorization**
- [ ] GRANT/REVOKE on tables
- [ ] Role-based access control
- [ ] Permission checking in query execution

**Deliverable**: Secure connections, authenticated access

---

## Phase 4: SQL Completeness (3-4 weeks)

**Goal**: 40-50% SQL coverage (time-series/HTAP use cases)

### Week 6: Constraints
**Days 1-2: NOT NULL Enforcement**
- [ ] Parse NOT NULL from CREATE TABLE
- [ ] Validate on INSERT
- [ ] Validate on UPDATE
- [ ] Error messages

**Days 3-5: FOREIGN KEY Constraints**
- [ ] Parse FOREIGN KEY syntax
- [ ] Reference validation on INSERT
- [ ] Reference validation on UPDATE
- [ ] CASCADE/RESTRICT actions
- [ ] ON DELETE/ON UPDATE actions

### Week 7: Essential SQL Features
**Days 1-2: UNIQUE Constraints**
- [ ] Parse UNIQUE syntax
- [ ] Index for uniqueness checking
- [ ] Validate on INSERT/UPDATE
- [ ] Transaction-aware checking

**Days 3-5: Subqueries**
- [ ] Subquery in WHERE clause
- [ ] Subquery in SELECT clause
- [ ] Scalar subqueries
- [ ] Correlated subqueries
- [ ] EXISTS/NOT EXISTS

### Week 8: Advanced SQL
**Days 1-3: CTEs (WITH clause)**
- [ ] Parse WITH syntax
- [ ] Materialize CTE
- [ ] Reference CTE in main query
- [ ] Multiple CTEs
- [ ] Recursive CTEs (optional)

**Days 4-5: Window Functions**
- [ ] OVER clause parsing
- [ ] PARTITION BY
- [ ] ORDER BY within window
- [ ] ROW_NUMBER(), RANK(), DENSE_RANK()
- [ ] LAG(), LEAD()

### Week 9: SQL Polish
**Days 1-2: Data Types**
- [ ] AUTO_INCREMENT / SERIAL
- [ ] TIMESTAMP / DATE / TIME
- [ ] BOOLEAN
- [ ] JSON/JSONB (basic support)

**Days 3-5: SQL Functions**
- [ ] String functions (CONCAT, SUBSTRING, UPPER, LOWER)
- [ ] Math functions (ABS, ROUND, CEIL, FLOOR)
- [ ] Date functions (NOW(), DATE_ADD, DATE_SUB)
- [ ] Aggregate functions (STDDEV, VAR, COUNT DISTINCT)
- [ ] COALESCE, NULLIF, CASE

**Deliverable**: 40-50% SQL coverage, usable for time-series apps

---

## Phase 5: Observability (1-2 weeks)

**Goal**: Debug and optimize production workloads

### Week 10: Query Introspection
**Days 1-2: EXPLAIN Implementation**
- [ ] EXPLAIN output for SELECT
- [ ] Show query plan
- [ ] Index usage
- [ ] Estimated rows
- [ ] Cost estimates

**Days 3-5: Logging & Metrics**
- [ ] Query logging (configurable)
- [ ] Slow query log (threshold-based)
- [ ] Query statistics (count, avg time, p99)
- [ ] Connection metrics
- [ ] Cache hit rate tracking
- [ ] Transaction metrics (commits, aborts, conflicts)

**Deliverable**: Production observability

---

## Phase 6: Backup & Recovery (1 week)

**Goal**: Data safety guarantees

### Week 11: Backup/Restore
**Days 1-2: Online Backup**
- [ ] Snapshot-based backup (using MVCC snapshots)
- [ ] Incremental backup
- [ ] Backup to file/directory
- [ ] Backup metadata (timestamp, size)

**Days 3-4: Restore**
- [ ] Restore from backup
- [ ] Validation after restore
- [ ] Progress reporting

**Day 5: PITR (Point-in-Time Recovery)**
- [ ] WAL archiving
- [ ] Replay WAL to timestamp
- [ ] Recovery target specification

**Deliverable**: Backup/restore capability

---

## Phase 7: Production Hardening (2 weeks)

**Goal**: Battle-tested, documented, deployable

### Week 12: Testing & Validation
**Days 1-3: Stress Testing**
- [ ] 24-hour stability test (writes + reads)
- [ ] Concurrent transaction stress (1000+ txns)
- [ ] Large dataset test (100M+ rows)
- [ ] Memory leak detection
- [ ] Resource exhaustion tests

**Days 4-5: Chaos Testing**
- [ ] Kill -9 during writes (crash recovery)
- [ ] Disk full scenarios
- [ ] Network interruption (for future replication)
- [ ] Corruption injection (detect + recover)

### Week 13: Documentation & Polish
**Days 1-2: User Documentation**
- [ ] Getting started guide
- [ ] SQL reference (supported features)
- [ ] Configuration reference
- [ ] Deployment guide
- [ ] Troubleshooting guide

**Days 3-4: Developer Documentation**
- [ ] Architecture overview
- [ ] MVCC internals
- [ ] Contributing guide
- [ ] Code conventions

**Day 5: Release Preparation**
- [ ] Final bug fixes
- [ ] Performance validation
- [ ] Create v1.0 release notes
- [ ] Tag v1.0 release

**Deliverable**: Production-ready OmenDB v1.0

---

## Success Criteria (v1.0)

### Functional Requirements ✅
- [ ] **SQL Coverage**: 40-50% of PostgreSQL features
  - Core: SELECT, INSERT, UPDATE, DELETE
  - JOINs: INNER, LEFT, RIGHT
  - Aggregates: COUNT, SUM, AVG, MIN, MAX, GROUP BY, HAVING
  - Constraints: PRIMARY KEY, FOREIGN KEY, UNIQUE, NOT NULL
  - Advanced: Subqueries, CTEs, Window functions
- [ ] **MVCC**: Snapshot isolation with concurrent transactions
- [ ] **Security**: Authentication + SSL
- [ ] **Observability**: EXPLAIN, logging, metrics
- [ ] **Backup**: Online backup + restore + PITR

### Performance Requirements ✅
- [ ] **Faster than SQLite**: 1.2x-2.5x speedup maintained
- [ ] **Memory efficiency**: <2 bytes/key
- [ ] **Scalability**: 100M rows without degradation
- [ ] **No regression**: All benchmarks ≥ current performance

### Quality Requirements ✅
- [ ] **Tests**: 100% passing, >500 unit + integration tests
- [ ] **Stability**: 24-hour stress test without failures
- [ ] **Crash recovery**: 100% success rate
- [ ] **Concurrency**: 1000+ concurrent transactions without deadlocks
- [ ] **Documentation**: Complete user + developer docs

### Production Readiness ✅
- [ ] **No security vulnerabilities**: Auth + SSL working
- [ ] **No data loss scenarios**: Backup/restore validated
- [ ] **No critical bugs**: All P0 issues resolved
- [ ] **Monitored**: Metrics exportable (Prometheus format)
- [ ] **Documented**: Deployment + troubleshooting guides

---

## Non-Goals (v1.0)

**Explicitly NOT in scope for v1.0:**
- ❌ Replication (defer to v2.0)
- ❌ Distributed transactions (single-node only)
- ❌ Full PostgreSQL compatibility (target 40-50%)
- ❌ Stored procedures/triggers
- ❌ Full-text search
- ❌ Geographic data (PostGIS)
- ❌ Advanced indexing (GIN, GiST, BRIN)
- ❌ Table partitioning
- ❌ Parallel query execution
- ❌ Query caching
- ❌ Materialized views
- ❌ Client libraries (use psql/existing Postgres libs)

**Why defer:** Focus on core database quality, not breadth of features

---

## Risk Mitigation

### Risk: MVCC too complex (2-4 weeks estimate)
**Mitigation:**
- Start with simple timestamp-based approach
- Reference ToyDB implementation (well-documented)
- Prototype early, validate before full implementation
- Fallback: Single-writer model with READ COMMITTED

### Risk: SQL completeness takes too long
**Mitigation:**
- Prioritize features by usage frequency
- Focus on time-series use cases (our target)
- Leverage DataFusion for OLAP queries
- Document unsupported features clearly

### Risk: Performance regression with MVCC
**Mitigation:**
- Benchmark after each change
- Read-only transaction optimization
- Garbage collection tuning
- Maintain performance test suite

### Risk: Timeline slip
**Mitigation:**
- Weekly progress reviews
- Cut features if needed (maintain quality)
- Focus on P0/P1 features only
- Defer nice-to-haves to v1.1

---

## Weekly Milestones

| Week | Milestone | Deliverable |
|------|-----------|-------------|
| 1 | Foundation | 100% tests, MVCC design |
| 2 | MVCC Prototype | Read-your-own-writes |
| 3-4 | MVCC Complete | Snapshot isolation working |
| 5 | Security | Auth + SSL |
| 6-9 | SQL Features | 40-50% coverage |
| 10 | Observability | EXPLAIN + metrics |
| 11 | Backup | Online backup + PITR |
| 12-13 | Hardening | Stress tested + documented |

**Go/No-Go Decision Points:**
- **Week 2**: MVCC prototype working? (If no: fallback to single-writer)
- **Week 5**: Security working? (If no: cannot ship)
- **Week 10**: Performance acceptable? (If no: optimize before features)
- **Week 12**: Stress tests passing? (If no: fix before release)

---

## Post-v1.0 Roadmap (v1.1 - v2.0)

### v1.1 (Maintenance - 1-2 months)
- Bug fixes from production feedback
- Performance tuning
- Additional SQL features (based on user requests)
- Client library (Rust, Python)
- Migration tool (PostgreSQL → OmenDB)

### v2.0 (Scale-out - 4-6 months)
- Replication (streaming, async)
- Read replicas
- Automatic failover
- Multi-region support
- Query caching
- Parallel query execution
- Materialized views

---

## Immediate Next Steps (This Week)

1. **Fix failing test** (today, 30 min)
2. **Clean compiler warnings** (today, 1 hour)
3. **MVCC design document** (Days 1-2, 2 days)
4. **MVCC prototype** (Days 3-5, 3 days)

**First commit:**
```bash
# Fix the failing test
cargo test --lib constraints::tests::test_extract_primary_key_table_level
```

---

## Questions to Validate

Before starting implementation:

1. **Target market confirmed?**
   - Time-series/IoT/metrics workloads
   - HTAP (eliminate ETL)
   - NOT general-purpose PostgreSQL replacement

2. **SQL coverage acceptable?**
   - 40-50% (vs 100% PostgreSQL)
   - Focus on common queries
   - Document unsupported features

3. **Timeline realistic?**
   - 10-12 weeks aggressive but achievable
   - Built-in buffer weeks for unknowns
   - Can cut features if needed

4. **MVCC approach agreed?**
   - Timestamp-based (like TiKV, ToyDB)
   - Snapshot isolation (not serializable for v1)
   - Garbage collection strategy

5. **Performance targets?**
   - Maintain 1.2x-2.5x vs SQLite
   - Accept 10-20% overhead for MVCC
   - Optimize hot paths

---

**Status**: Roadmap complete, ready for execution
**Next**: Fix failing test, start MVCC design
**Timeline**: 10-12 weeks to v1.0
**Focus**: Quality > speed, production-ready > feature-rich

