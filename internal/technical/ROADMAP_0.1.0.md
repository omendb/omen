# OmenDB 0.1.0 Release Roadmap

**Version**: 0.1.0 (Production-ready foundation)
**Target**: Enterprise-grade SOTA database for time-series/HTAP workloads
**Timeline**: 10-12 weeks to 0.1.0 release
**Date**: October 21, 2025
**Philosophy**: Technical excellence over speed, quality over features

---

## Executive Summary

**Current State (Oct 21, 2025)**:
- ✅ Strong foundations: Multi-level ALEX (1.2x-2.5x vs SQLite), PostgreSQL protocol, RocksDB storage
- ✅ Performance validated: 1.12x faster at 10M scale (honest benchmarks, 3 runs)
- ✅ Basic transactions: BEGIN/COMMIT/ROLLBACK working
- ✅ Crash recovery: 100% success rate at 1M scale
- ❌ **1 failing test** (constraint extraction bug)
- ❌ **37 compiler warnings** (cleanup needed)
- ❌ **~15% SQL coverage** (need 40-50% for production)
- ❌ **No MVCC** (concurrent transactions unsafe)
- ❌ **No authentication/SSL** (cannot deploy securely)

**0.1.0 Goals**:
- ✅ 100% tests passing, zero warnings
- ✅ 40-50% SQL coverage (time-series/HTAP workloads)
- ✅ MVCC with snapshot isolation
- ✅ Authentication + SSL
- ✅ Production observability (EXPLAIN, metrics, logging)
- ✅ Backup/restore capability
- ✅ 24-hour stress test passing
- ✅ Complete documentation

**Why 0.1.0 (not v1.0)**:
- v1.0 comes AFTER proven production deployments
- 0.1.0 = Foundation complete, ready for early adopters
- Focus: Quality core features, not breadth

---

## Phase 0: Foundation Cleanup (THIS WEEK) 🚨

**Duration**: 3-5 days
**Status**: NOT STARTED
**Priority**: HIGHEST - Must complete before any other work

**Goal**: Clean codebase, 100% tests passing, zero warnings

### Critical Fixes

**Day 1 (Today): Fix Failing Test**
- [ ] Fix `test_extract_primary_key_table_level` in src/constraints.rs:305
  - **Issue**: Extracts "name" instead of "id" from `PRIMARY KEY (id)`
  - **Root cause**: Inline pattern checked before table-level pattern
  - **Fix**: Reorder pattern matching in lines 115-133
  - **Estimated**: 30 minutes
  - **Test**: `cargo test test_extract_primary_key_table_level`

**Day 1: Clean Compiler Warnings**
- [ ] Run `cargo clippy --fix --allow-dirty`
  - 37 unused variable/import warnings
  - **Estimated**: 1 hour
  - **Verify**: `cargo clippy -- -D warnings` (should pass)

**Day 1-2: Validate Clean Build**
- [ ] Run full test suite: `cargo test`
  - **Target**: 357/357 tests passing (currently 356/357)
- [ ] Run benchmarks to ensure no regression
  - Validate 10M scale: 1.12x speedup maintained
- [ ] Document baseline performance (for future comparison)

**Day 3-5: MVCC Research & Design**
- [ ] Study reference implementations:
  - ToyDB (well-documented, timestamp-based)
  - TiKV (production-grade, percolator model)
  - SkipDB (snapshot isolation)
- [ ] Design OmenDB MVCC architecture:
  - Timestamp oracle (monotonic transaction IDs)
  - Version chains in RocksDB: `(key, txn_id)` → value
  - Snapshot isolation semantics
  - Garbage collection strategy
  - Integration with ALEX index
- [ ] Create detailed design document
- [ ] Prototype key data structures

**Deliverable**: Clean codebase (100% tests, zero warnings), MVCC design doc

---

## Phase 1: MVCC Implementation (Weeks 1-3)

**Duration**: 2-3 weeks
**Status**: Design phase (Phase 0)
**Priority**: CRITICAL - Required for production concurrent safety

**Goal**: Snapshot isolation for concurrent transactions

### Week 1: Transaction Manager Foundation

**Days 1-3: Timestamp Oracle**
- [ ] Monotonic transaction ID allocation
- [ ] Transaction metadata tracking (start_ts, commit_ts)
- [ ] Active transaction registry
- [ ] Read timestamp assignment (snapshot creation)

**Days 4-5: Versioned Storage Layer**
- [ ] Extend RocksDB key format: `(key, txn_id)` → value
- [ ] Version chain implementation (append-only writes)
- [ ] Tombstone handling for deletes

### Week 2: Snapshot Isolation

**Days 1-3: Read Path**
- [ ] Snapshot read at timestamp
- [ ] Visibility rules implementation:
  - Transaction sees own uncommitted writes
  - Transaction sees committed writes before snapshot_ts
  - Transaction doesn't see writes after snapshot_ts
- [ ] Handle uncommitted versions (skip)
- [ ] Handle deleted versions (tombstones)

**Days 4-5: Write Conflict Detection**
- [ ] Write-write conflict detection
- [ ] First-committer-wins rule
- [ ] Serialization failure errors
- [ ] Retry logic for applications

### Week 3: Testing & Validation

**Days 1-2: Concurrent Transaction Support**
- [ ] Multiple active transactions
- [ ] Lock-free reads (MVCC benefit)
- [ ] Optimistic concurrency control
- [ ] Read-only transaction optimization

**Days 3-5: Comprehensive MVCC Testing**
- [ ] Unit tests: visibility rules, version chains
- [ ] Integration tests: concurrent transactions
- [ ] Stress tests: 100+ concurrent writers
- [ ] Anomaly tests: lost updates, write skew, phantom reads
- [ ] Performance validation: ensure no >20% regression

**Success Criteria**:
- [ ] Snapshot isolation working correctly
- [ ] 100+ MVCC tests passing
- [ ] Concurrent transactions safe
- [ ] Performance: <20% overhead vs non-MVCC

**Deliverable**: Production-ready MVCC with snapshot isolation

---

## Phase 2: Security & Authentication (Weeks 4-5)

**Duration**: 1-2 weeks
**Status**: Not started
**Priority**: HIGH - Required for secure deployment

**Goal**: Secure connections and authenticated access

### Week 4: Authentication

**Days 1-3: Password Authentication**
- [ ] User management (CREATE USER, DROP USER, ALTER USER)
- [ ] Password storage (argon2 hashing)
- [ ] Authentication on connection
- [ ] PostgreSQL auth protocol (SCRAM-SHA-256)
- [ ] Default admin user creation

**Days 4-5: SSL/TLS**
- [ ] TLS support for PostgreSQL protocol
- [ ] Certificate management (load from file)
- [ ] Require SSL mode (configurable)
- [ ] Self-signed cert generation for development

### Week 5: Authorization (Optional)

**If time permits:**
- [ ] Basic table-level permissions (GRANT/REVOKE)
- [ ] Permission checking in query execution
- [ ] Role-based access control (basic)

**Success Criteria**:
- [ ] Connections require authentication
- [ ] SSL/TLS working
- [ ] Passwords securely hashed
- [ ] Auth integrated with PostgreSQL protocol

**Deliverable**: Secure, authenticated database

---

## Phase 3: SQL Completeness (Weeks 6-9)

**Duration**: 3-4 weeks
**Status**: Not started
**Priority**: HIGH - Required for usability

**Goal**: 40-50% SQL coverage (focused on time-series/HTAP use cases)

### Week 6: Essential Constraints

**Days 1-2: NOT NULL Enforcement**
- [ ] Parse NOT NULL from CREATE TABLE
- [ ] Validate on INSERT/UPDATE
- [ ] Clear error messages

**Days 3-5: FOREIGN KEY Constraints**
- [ ] Parse FOREIGN KEY syntax
- [ ] Reference validation (INSERT/UPDATE)
- [ ] CASCADE/RESTRICT/SET NULL actions
- [ ] ON DELETE/ON UPDATE actions
- [ ] Circular reference detection

### Week 7: Uniqueness & Advanced Queries

**Days 1-2: UNIQUE Constraints**
- [ ] Parse UNIQUE syntax
- [ ] Index-based uniqueness checking
- [ ] Transaction-aware validation
- [ ] Composite UNIQUE constraints

**Days 3-5: Subqueries**
- [ ] Subquery in WHERE clause
- [ ] Subquery in SELECT clause
- [ ] Scalar subqueries
- [ ] EXISTS/NOT EXISTS
- [ ] IN with subquery

### Week 8: Advanced SQL Features

**Days 1-3: CTEs (WITH clause)**
- [ ] Parse WITH syntax
- [ ] Materialize CTE
- [ ] Reference CTE in main query
- [ ] Multiple CTEs
- [ ] Recursive CTEs (optional, if time)

**Days 4-5: Window Functions**
- [ ] OVER clause parsing
- [ ] PARTITION BY
- [ ] ORDER BY within window
- [ ] ROW_NUMBER(), RANK(), DENSE_RANK()
- [ ] LAG(), LEAD() (for time-series)

### Week 9: Data Types & Functions

**Days 1-2: Essential Data Types**
- [ ] AUTO_INCREMENT / SERIAL
- [ ] TIMESTAMP / DATE / TIME
- [ ] BOOLEAN
- [ ] JSON/JSONB (basic support)

**Days 3-5: SQL Functions**
- [ ] String functions: CONCAT, SUBSTRING, UPPER, LOWER, LENGTH
- [ ] Math functions: ABS, ROUND, CEIL, FLOOR, MOD
- [ ] Date functions: NOW(), CURRENT_TIMESTAMP, DATE_ADD, DATE_SUB
- [ ] Aggregate functions: STDDEV, VARIANCE, COUNT(DISTINCT)
- [ ] Utility: COALESCE, NULLIF, CASE/WHEN

**Success Criteria**:
- [ ] 40-50% SQL coverage achieved
- [ ] Common time-series queries work
- [ ] HTAP workloads supported
- [ ] 200+ SQL tests passing

**Deliverable**: Usable SQL database for target workloads

---

## Phase 4: Observability (Week 10)

**Duration**: 1 week
**Status**: Not started
**Priority**: MEDIUM - Needed for production debugging

**Goal**: Debug and optimize production workloads

### Days 1-2: EXPLAIN Implementation

- [ ] EXPLAIN output for SELECT
- [ ] Show query plan (DataFusion integration)
- [ ] Index usage information
- [ ] Estimated rows and cost
- [ ] EXPLAIN ANALYZE (actual execution stats)

### Days 3-5: Logging & Metrics

- [ ] Query logging (configurable level)
- [ ] Slow query log (threshold-based)
- [ ] Query statistics tracking:
  - Query count by type
  - Average/p50/p95/p99 latency
  - Cache hit rates
- [ ] Transaction metrics:
  - Commit/abort counts
  - Conflict rates
  - Active transaction count
- [ ] Connection metrics:
  - Active connections
  - Connection pool utilization
  - Connection errors

**Success Criteria**:
- [ ] EXPLAIN shows useful plans
- [ ] Slow queries identified
- [ ] Metrics exportable (Prometheus format)
- [ ] Production debugging possible

**Deliverable**: Production observability

---

## Phase 5: Backup & Recovery (Week 11)

**Duration**: 1 week
**Status**: Not started
**Priority**: MEDIUM - Required for data safety

**Goal**: Reliable backup and restore

### Days 1-2: Online Backup

- [ ] Snapshot-based backup (MVCC snapshots)
- [ ] Backup to file/directory
- [ ] Incremental backup
- [ ] Backup metadata (timestamp, size, version)

### Days 3-4: Restore

- [ ] Restore from backup
- [ ] Data validation after restore
- [ ] Progress reporting
- [ ] Restore to different directory

### Day 5: Point-in-Time Recovery (PITR)

- [ ] WAL archiving
- [ ] Replay WAL to specific timestamp
- [ ] Recovery target specification
- [ ] Validation after recovery

**Success Criteria**:
- [ ] Backup completes without blocking writes
- [ ] Restore reconstructs exact database state
- [ ] PITR accurate to second granularity
- [ ] Validated at 10M+ rows

**Deliverable**: Backup/restore capability

---

## Phase 6: Production Hardening (Weeks 12-13)

**Duration**: 2 weeks
**Status**: Not started
**Priority**: HIGH - Required for reliability

**Goal**: Battle-tested, stress-validated database

### Week 12: Stress & Chaos Testing

**Days 1-3: Sustained Load Testing**
- [ ] 24-hour continuous operation
  - Mixed workload: 80% reads, 20% writes
  - 100+ concurrent connections
  - Monitor: memory usage, latency, throughput
- [ ] Memory leak detection (valgrind/heaptrack)
- [ ] Connection churn test (rapid connect/disconnect)
- [ ] Large dataset test (100M+ rows)

**Days 4-5: Chaos Testing**
- [ ] Kill -9 during writes (crash recovery validation)
- [ ] Disk full scenarios (graceful handling)
- [ ] Out of memory scenarios
- [ ] Network interruption simulation
- [ ] Corruption injection (detect and report)

### Week 13: Documentation & Release Prep

**Days 1-2: User Documentation**
- [ ] Getting started guide
- [ ] SQL reference (supported features)
- [ ] Configuration reference
- [ ] Deployment guide (single-node)
- [ ] Troubleshooting guide

**Days 3-4: Developer Documentation**
- [ ] Architecture overview (ALEX, MVCC, storage)
- [ ] Contributing guide
- [ ] Code conventions
- [ ] Testing guidelines

**Day 5: Release Preparation**
- [ ] Final performance validation
- [ ] Security audit (clippy --pedantic, cargo audit)
- [ ] Create 0.1.0 release notes
- [ ] Tag 0.1.0 release
- [ ] Build release binaries

**Success Criteria**:
- [ ] 24-hour stress test: zero crashes
- [ ] 200+ concurrent connections: stable
- [ ] Memory: no leaks detected
- [ ] Documentation: complete and accurate
- [ ] All tests passing (500+ tests)

**Deliverable**: Production-ready OmenDB 0.1.0

---

## Success Criteria for 0.1.0 Release

### Functional Requirements ✅
- [ ] **SQL Coverage**: 40-50% of PostgreSQL (focused on time-series/HTAP)
  - Core: SELECT, INSERT, UPDATE, DELETE
  - JOINs: INNER, LEFT, RIGHT
  - Aggregates: COUNT, SUM, AVG, MIN, MAX, GROUP BY, HAVING
  - Constraints: PRIMARY KEY, FOREIGN KEY, UNIQUE, NOT NULL
  - Advanced: Subqueries, CTEs, Window functions (basic)
- [ ] **MVCC**: Snapshot isolation for concurrent transactions
- [ ] **Security**: Authentication + SSL/TLS
- [ ] **Observability**: EXPLAIN, logging, metrics (Prometheus)
- [ ] **Backup**: Online backup + restore + PITR

### Performance Requirements ✅
- [ ] **Maintain speedup**: 1.2x-2.5x faster than SQLite (no regression)
- [ ] **Memory efficiency**: <2 bytes/key maintained
- [ ] **Scalability**: 100M rows without degradation
- [ ] **MVCC overhead**: <20% latency increase vs non-MVCC

### Quality Requirements ✅
- [ ] **Tests**: 100% passing, 500+ unit + integration tests
- [ ] **Stability**: 24-hour stress test without failures
- [ ] **Crash recovery**: 100% success at 10M+ scale
- [ ] **Concurrency**: 200+ concurrent connections stable
- [ ] **Code quality**: Zero clippy warnings, cargo audit clean

### Production Readiness ✅
- [ ] **Security**: Authentication, SSL, no known vulnerabilities
- [ ] **Data safety**: Backup/restore validated
- [ ] **Monitoring**: Metrics exportable, slow queries logged
- [ ] **Documentation**: Complete deployment + troubleshooting guides
- [ ] **No critical bugs**: All P0 issues resolved

---

## Explicit Non-Goals for 0.1.0

**Deferred to 0.2.0 or later:**
- ❌ Replication (single-node only for 0.1.0)
- ❌ Distributed transactions
- ❌ Full PostgreSQL compatibility (targeting 40-50%)
- ❌ Stored procedures/triggers
- ❌ Full-text search
- ❌ Geographic data (PostGIS)
- ❌ Advanced indexing (GIN, GiST, BRIN)
- ❌ Table partitioning
- ❌ Parallel query execution
- ❌ Query result caching
- ❌ Materialized views (basic views only)
- ❌ Client libraries (use standard PostgreSQL libraries)

**Why defer**: 0.1.0 focuses on correctness and core database quality. Additional features come after proven deployment.

---

## Timeline Summary

| Week | Phase | Deliverable |
|------|-------|-------------|
| 0 (This week) | Foundation Cleanup | 100% tests, MVCC design |
| 1-3 | MVCC Implementation | Snapshot isolation working |
| 4-5 | Security | Auth + SSL |
| 6-9 | SQL Features | 40-50% coverage |
| 10 | Observability | EXPLAIN + metrics |
| 11 | Backup | Online backup + PITR |
| 12-13 | Hardening | Stress tested + documented |

**Total**: 10-12 weeks to 0.1.0 release

**Go/No-Go Decision Points**:
- **Week 1**: MVCC prototype working? (If no: redesign)
- **Week 3**: MVCC complete and tested? (If no: delay SQL features)
- **Week 5**: Security working? (If no: cannot release)
- **Week 10**: Performance maintained? (If no: optimize before release)
- **Week 12**: Stress tests passing? (If no: fix before release)

---

## Risk Mitigation

### Risk: MVCC complexity (2-4 weeks estimate)
**Mitigation**:
- Start with simple timestamp-based approach (ToyDB model)
- Prototype early (Phase 0)
- Reference well-documented implementations
- Fallback: Single-writer with READ COMMITTED

### Risk: SQL features take too long
**Mitigation**:
- Prioritize by usage frequency in time-series workloads
- Leverage DataFusion for complex queries
- Cut nice-to-haves if needed
- Document unsupported features clearly

### Risk: Performance regression with MVCC
**Mitigation**:
- Benchmark after each change
- Read-only transaction optimization (no version overhead)
- Garbage collection tuning
- Accept 10-20% overhead (typical for MVCC)

### Risk: Timeline slip
**Mitigation**:
- Weekly progress reviews
- Cut features if needed (quality > breadth)
- Focus on P0 features only
- Move nice-to-haves to 0.2.0

---

## Immediate Next Steps (Today)

### Priority 1: Fix Failing Test (30 min)
```bash
# Test the failing test
cargo test test_extract_primary_key_table_level

# Fix src/constraints.rs lines 115-133
# Reorder: check table-level PRIMARY KEY before inline

# Verify fix
cargo test test_extract_primary_key_table_level
```

### Priority 2: Clean Warnings (1 hour)
```bash
# Auto-fix warnings
cargo clippy --fix --allow-dirty

# Verify clean
cargo clippy -- -D warnings
```

### Priority 3: Validate Clean Build (2 hours)
```bash
# Run all tests
cargo test

# Run benchmarks (ensure no regression)
cargo build --release
./target/release/benchmark_honest_comparison

# Expected: 10M scale at 1.12x speedup maintained
```

### Priority 4: Start MVCC Design (2-3 days)
- Study ToyDB, TiKV, SkipDB implementations
- Design OmenDB MVCC architecture
- Document design decisions
- Create implementation plan

---

## Post-0.1.0 Roadmap Preview

### 0.2.0 (Enhancements - 4-6 weeks)
- Additional SQL features (based on user feedback)
- Performance optimizations
- Client libraries (Rust, Python)
- Migration tools (PostgreSQL → OmenDB)

### 0.3.0 (Scale-out - 8-12 weeks)
- Read replicas
- Async replication
- Automatic failover
- Query result caching
- Parallel query execution

### 1.0.0 (Production-Proven)
- Released AFTER proven production deployments
- Multiple customers in production
- 6-12 months of operational stability
- Full enterprise feature set validated

---

## Current Status (October 21, 2025)

**Completed This Week**:
- ✅ Cache metrics implementation (1M entry LRU)
- ✅ Performance variance analysis (3 benchmark runs)
- ✅ Identified Run 1 as outlier (cold cache)
- ✅ Established stable baseline: 1.12x at 10M scale
- ✅ Comprehensive gap analysis (this document)

**Current State**:
- 356/357 tests passing (99.7%)
- 37 compiler warnings
- 1.12x faster than SQLite at 10M (validated)
- ~15% SQL coverage
- No MVCC (concurrent transactions unsafe)
- No authentication/SSL

**Next Actions** (This Week):
1. Fix failing test (today)
2. Clean compiler warnings (today)
3. MVCC design (Days 2-5)
4. Begin Phase 1 implementation (next week)

---

## Questions Answered

### Q: Why 0.1.0 instead of v1.0?
**A**: v1.0 signals production-proven stability. 0.1.0 = foundation complete, ready for early adopters. We'll reach v1.0 after 6-12 months of proven deployments.

### Q: Why focus on 40-50% SQL coverage?
**A**: Target market is time-series/HTAP, not general-purpose. 40-50% covers common workloads. Full compatibility comes later if needed.

### Q: Why MVCC so early in roadmap?
**A**: Without MVCC, concurrent transactions are unsafe. Critical for any multi-user production deployment. Must have for 0.1.0.

### Q: Can we ship without backup/restore?
**A**: No. Data safety is non-negotiable for production. Backup/restore required for 0.1.0.

### Q: What about replication?
**A**: Deferred to 0.2.0+. Single-node performance and correctness first. Replication adds complexity that can wait.

---

**Status**: Roadmap complete, ready for execution
**Version**: 0.1.0 (NOT v1.0)
**Next**: Fix failing test, clean warnings, start MVCC design
**Timeline**: 10-12 weeks to 0.1.0 release
**Focus**: Enterprise-grade SOTA database, technical excellence over speed
