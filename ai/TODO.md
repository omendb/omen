# TODO

_Last Updated: 2025-10-21_

## High Priority

### Phase 2: Security (Days 6-10 remaining)
- [ ] **Days 6-7**: SSL/TLS for PostgreSQL wire protocol
  - [ ] Configure TLS for pgwire connections
  - [ ] Load certificates from disk
  - [ ] Add --tls flag to postgres_server
  - [ ] Test TLS connections with psql
- [ ] **Day 8**: Security integration tests
  - [ ] End-to-end auth + TLS tests
  - [ ] Multi-user concurrent access tests
  - [ ] Permission boundary tests
  - Target: 50+ total security tests
- [ ] **Day 9**: Security documentation
  - [ ] Write SECURITY.md with deployment guide
  - [ ] Document TLS setup procedures
  - [ ] Add security examples
- [ ] **Day 10**: Final validation & security audit
  - [ ] Review all security code paths
  - [ ] Test default admin password warning
  - [ ] Verify no hardcoded credentials

### Performance Validation (Post Phase 2)
- [ ] Validate cache effectiveness at 10M scale
- [ ] RocksDB tuning (reduce 77% overhead to <30%)
- [ ] Measure MVCC overhead (target: <20%)
- [ ] Re-run honest benchmarks vs SQLite

## In Progress

- [x] Phase 2 Days 1-5: Auth + User Management (40/40 tests passing) ✅
  - [x] Day 1: UserStore with RocksDB persistence
  - [x] Day 2: OmenDbAuthSource integration
  - [x] Day 3-4: SQL user management (CREATE/DROP/ALTER USER)
  - [x] Day 5: Catalog integration with default admin user

## Backlog

### Phase 3: SQL Features (Weeks 3-4)
- [ ] Aggregations (COUNT, SUM, AVG, MIN, MAX, GROUP BY)
- [ ] Subqueries (WHERE EXISTS, scalar subqueries)
- [ ] Window functions (ROW_NUMBER, RANK)
- [ ] Advanced JOIN types (FULL OUTER, CROSS)

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

- [x] Cache Layer Days 1-10: LRU cache (1-10GB), 2-3x speedup validated
- [x] Phase 3 Week 2: INNER JOIN + LEFT JOIN (14 tests)
- [x] Phase 3 Week 1: UPDATE/DELETE support (30 tests)
- [x] Phase 1: MVCC snapshot isolation (85 tests)
- [x] Multi-level ALEX index (1.5-3x faster than SQLite)
