# TODO

_Last Updated: 2025-10-22_

## High Priority

### Next Phase Decision (6 weeks to 0.1.0)
**Current Status**: 45% SQL coverage (target: 50%), 557 tests (target: 500+)

**Option 1: Continue Phase 3 SQL Features** (3-5 days)
- [ ] Subqueries (WHERE EXISTS, scalar subqueries) - 2-3 days
- [ ] Window functions (ROW_NUMBER, RANK) - 2-3 days
- **Pros**: Reach 50%+ SQL coverage, more feature-complete
- **Cons**: Delays production readiness, complex features

**Option 2: Move to Phase 4 Observability** (production-first)
- [ ] EXPLAIN QUERY PLAN command
- [ ] Query performance metrics
- [ ] Slow query logging
- [ ] Prometheus metrics endpoint
- **Pros**: Production readiness, user debugging tools
- **Cons**: 45% SQL coverage (slightly below 50% target)

### Performance Validation (Deferred)
- [ ] Validate cache effectiveness at 10M scale
- [ ] RocksDB tuning (reduce 77% overhead to <30%)
- [ ] Measure MVCC overhead (target: <20%)
- [ ] Re-run honest benchmarks vs SQLite

## Recently Completed

- [x] **Phase 2 Security (Days 1-10) COMPLETE** ✅
  - [x] Days 1-5: Auth + User Management (40 tests)
  - [x] Days 6-7: SSL/TLS Implementation
  - [x] Day 8: Security integration tests (17 tests)
  - [x] Day 9: Security documentation (SECURITY.md)
  - [x] Day 10: Security audit & validation
  - **Total**: 57 security tests, 10 days on schedule

## Backlog

### Phase 3: SQL Features (Remaining)
- [x] Aggregations (COUNT, SUM, AVG, MIN, MAX, GROUP BY) ✅
- [x] HAVING clause ✅
- [x] CROSS JOIN ✅
- [ ] Subqueries (WHERE EXISTS, scalar subqueries)
- [ ] Window functions (ROW_NUMBER, RANK)
- [ ] Advanced JOIN types (FULL OUTER, RIGHT)

### Phase 4: Observability
- [ ] EXPLAIN QUERY PLAN command
- [ ] Query performance metrics
- [ ] Slow query logging
- [ ] Prometheus metrics endpoint

### Phase 5: Backup & Recovery
- [ ] pg_dump/pg_restore compatibility
- [ ] Point-in-time recovery
- [ ] Incremental backups
- [ ] Backup verification tools

### Phase 6: Production Hardening
- [ ] Connection pooling improvements
- [ ] Query timeout enforcement
- [ ] Resource limits per query
- [ ] Prepared statement caching
- [ ] Health check endpoints

## Completed Recently

- [x] **Phase 3 Quick Wins** (Oct 22, 1 session) ✅
  - [x] Aggregations: COUNT, SUM, AVG, MIN, MAX, GROUP BY (22 tests)
  - [x] HAVING clause: Full filtering support (7 tests)
  - [x] CROSS JOIN: Cartesian product (3 tests)
  - **Result**: SQL coverage 35% → 45%, 557 total tests
- [x] Phase 2 Days 6-7: SSL/TLS for PostgreSQL wire protocol ✅
- [x] Phase 2 Days 1-5: Auth + User Management (40 tests)
- [x] Cache Layer Days 1-10: LRU cache (1-10GB), 2-3x speedup validated
- [x] Phase 3 Week 2: INNER JOIN + LEFT JOIN (14 tests)
- [x] Phase 3 Week 1: UPDATE/DELETE support (30 tests)
- [x] Phase 1: MVCC snapshot isolation (85 tests)
- [x] Multi-level ALEX index (1.5-3x faster than SQLite)
