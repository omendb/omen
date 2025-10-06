# AlexStorage Development Roadmap

**Date:** October 6, 2025
**Current State:** Phase 4 complete (WAL durability)
**Strategy:** Specialize as high-performance read-optimized storage

---

## Current Status

### Completed (Phases 1-4)

✅ **Phase 1:** Foundation (ALEX + mmap)
- 3.49x faster queries than RocksDB at 100K scale

✅ **Phase 2:** Write optimization (deferred remapping)
- 7.02x faster mixed workload at 100K scale
- 31.73x faster mixed workload at 1M scale

✅ **Phase 3:** Read optimization (zero-copy)
- 4.23x faster queries at 1M scale
- 14% query improvement

✅ **Phase 4:** Durability (WAL)
- Crash recovery implemented
- 4.81x faster queries at 1M scale
- 28.59x faster mixed workload at 1M scale

### Missing (Critical for Production)

🔴 **P0 (Blocking):**
- Concurrency (single-threaded only)
- Delete operations (can't remove data)

🟡 **P1 (Important):**
- Compaction (space reclamation)
- Group commit (3.14x slower bulk inserts)

🟢 **P2 (Nice to have):**
- Range queries
- Compression
- Checksums
- Monitoring

---

## Strategic Direction

### Option Analysis

**Option 1: Specialize (CHOSEN)**
- Focus: Read-optimized storage for specific use cases
- Timeline: 2 weeks to production-ready
- Market: "Fastest read cache with durability"
- Features: Concurrency + deletes + basic compaction + group commit

**Option 2: Build to Parity**
- Focus: General-purpose RocksDB replacement
- Timeline: 2-3 months
- Risk: Won't beat RocksDB on writes
- Features: Everything RocksDB has

**Option 3: Hybrid**
- Focus: RocksDB writes + AlexStorage reads
- Timeline: 2-3 weeks
- Complexity: Syncing, consistency
- Features: Best of both worlds

**Decision: Option 1 (Specialize)**

**Rationale:**
1. We have 4.81x read advantage - exploit it
2. Write optimization is diminishing returns
3. Faster time to production (2 weeks vs 2-3 months)
4. Clear market positioning

---

## Roadmap (Next 2 Weeks)

### Phase 5: Concurrency (P0 - Days 1-3)

**Goal:** Support multi-threaded access

**Approach:** Read-write locks (simple, effective)

**Tasks:**
- [ ] Day 1: Add `RwLock<AlexStorage>` wrapper
- [ ] Day 1: Implement concurrent read API
- [ ] Day 2: Implement concurrent write API (serialized)
- [ ] Day 2: Add tests (concurrent reads, writes)
- [ ] Day 3: Benchmark (multi-threaded workload)
- [ ] Day 3: Document performance characteristics

**Expected Performance:**
- Concurrent reads: Near-linear scaling (4 threads → 4x throughput)
- Concurrent writes: No improvement (serialized by lock)
- Mixed (4 threads): 2-3x throughput improvement

**Success Criteria:**
- ✅ All tests passing with concurrent access
- ✅ No data corruption
- ✅ 2x+ throughput with 4 reader threads

---

### Phase 6: Delete Operations (P0 - Days 4-5)

**Goal:** Support delete operations

**Approach:** Tombstones (simple, deferred compaction)

**Tasks:**
- [ ] Day 4: Implement `delete(key)` method
- [ ] Day 4: Add tombstone support to ALEX
- [ ] Day 4: Update WAL for delete entries
- [ ] Day 5: Add delete tests
- [ ] Day 5: Benchmark delete performance

**Expected Performance:**
- Delete latency: ~2,000ns (similar to insert)
- Query after delete: ~900ns (check for tombstone)
- Space: Grows unbounded until compaction

**Success Criteria:**
- ✅ Delete operations work correctly
- ✅ Queries respect tombstones
- ✅ WAL replay handles deletes
- ✅ All tests passing

---

### Phase 7: Compaction (P1 - Days 6-8)

**Goal:** Reclaim space from deleted entries

**Approach:** Offline compaction (rebuild file)

**Tasks:**
- [ ] Day 6: Design compaction algorithm
- [ ] Day 6: Implement file rebuilding
- [ ] Day 7: Add mmap remapping after compaction
- [ ] Day 7: Test compaction correctness
- [ ] Day 8: Benchmark compaction time
- [ ] Day 8: Add compaction tests

**Expected Performance:**
- Compaction time: 1-5 seconds for 1M keys
- Space reclaimed: 50-90% (depends on delete rate)
- Downtime: Offline compaction (blocks all operations)

**Success Criteria:**
- ✅ Compaction removes tombstones
- ✅ File size reduced
- ✅ Queries work after compaction
- ✅ All tests passing

---

### Phase 8: Group Commit (P1 - Days 9-10)

**Goal:** Improve bulk insert performance

**Approach:** Batched WAL writes with timeout

**Tasks:**
- [ ] Day 9: Design group commit API
- [ ] Day 9: Implement batched WAL writes
- [ ] Day 9: Add timeout mechanism (10ms)
- [ ] Day 10: Test group commit correctness
- [ ] Day 10: Benchmark bulk insert performance

**Expected Performance:**
- Bulk insert: 500-1,000ns/key (2-5x improvement)
- Competitive with RocksDB (1,565ns/key)
- Latency trade-off: +10ms max for durability

**Success Criteria:**
- ✅ Bulk inserts 2x+ faster
- ✅ Durability maintained
- ✅ All tests passing

---

### Phase 9: Range Queries (P2 - Days 11-13)

**Goal:** Efficient range scans

**Approach:** ALEX range bounds + sequential mmap scan

**Tasks:**
- [ ] Day 11: Design range query API
- [ ] Day 11: Implement ALEX range bounds
- [ ] Day 12: Handle unsorted file (scan)
- [ ] Day 12: Optimize scan (prefetching)
- [ ] Day 13: Test range queries
- [ ] Day 13: Benchmark scan throughput

**Expected Performance:**
- Range scan: 100-500ns/key (sequential mmap)
- Better for sorted keys (ALEX bounds tight)
- Worse for random keys (ALEX bounds loose)

**Success Criteria:**
- ✅ Range queries work correctly
- ✅ Competitive performance for sorted keys
- ✅ All tests passing

---

### Phase 10: Production Hardening (P3 - Days 14+)

**Goal:** Production-ready features

**Tasks:**
- [ ] Compression (ZSTD)
- [ ] Checksums (CRC32)
- [ ] Metrics (Prometheus)
- [ ] Error handling
- [ ] Logging (tracing)
- [ ] Documentation (API docs)
- [ ] Examples (use cases)

**Timeline:** Ongoing, as needed

---

## Performance Targets

### Current (Phase 4)

| Metric | Current | vs RocksDB |
|--------|---------|------------|
| Point queries | 829 ns | 4.81x faster |
| Mixed (80/20) | 2,465 ns | 28.59x faster |
| Bulk insert | 4,915 ns | 3.14x slower |
| Concurrency | Single-thread | N/A |

### Target (After Phase 9)

| Metric | Target | vs RocksDB |
|--------|--------|------------|
| Point queries | <1,000 ns | 4x+ faster |
| Mixed (80/20) | <3,000 ns | 20x+ faster |
| Bulk insert | <1,500 ns | Competitive |
| Concurrency | 4 threads | 2-3x throughput |
| Delete | <2,500 ns | Competitive |
| Compaction | <5 sec/1M keys | Offline |
| Range scan | <500 ns/key | Competitive |

---

## Use Cases (After Completion)

### Target Use Cases

✅ **Read-heavy caching:**
- Redis alternative with durability
- Session stores
- Configuration databases
- 4x+ faster reads

✅ **Time-series data:**
- Metrics, logs, events
- Append-only workloads
- Efficient range scans

✅ **Real-time serving:**
- Low-latency reads (<1μs)
- High-throughput queries (>1M qps)
- CDN edge caching

✅ **Embedded databases:**
- Mobile apps
- Desktop apps
- Single-threaded workloads

---

## Non-Target Use Cases

❌ **Write-heavy workloads:**
- Social feeds
- Messaging systems
- Better: RocksDB, ScyllaDB

❌ **High-concurrency writes:**
- OLTP databases
- Multi-writer systems
- Better: PostgreSQL, MySQL

❌ **Distributed systems:**
- Sharding
- Replication
- Better: CockroachDB, TiDB

---

## Success Metrics

### Technical Metrics

**Performance:**
- ✅ 4x+ faster point queries than RocksDB
- ✅ 20x+ faster mixed workloads than RocksDB
- ✅ Competitive bulk insert (<1,500ns)
- ✅ Multi-threaded support (2-3x throughput)

**Features:**
- ✅ Concurrency (read-write locks)
- ✅ Delete operations (tombstones)
- ✅ Compaction (space reclamation)
- ✅ Group commit (write optimization)
- ✅ Range queries (scans)

**Quality:**
- ✅ All tests passing
- ✅ No data corruption
- ✅ Crash recovery works
- ✅ Documentation complete

### Business Metrics

**Positioning:**
- "Fastest read-optimized storage with durability"
- 4x faster than RocksDB for read-heavy workloads
- Redis alternative with persistence

**Timeline:**
- 2 weeks to production-ready
- Clear feature roadmap
- Validated performance claims

---

## Risk Assessment

### Technical Risks

**Low Risk:**
- ✅ Read performance (validated)
- ✅ WAL durability (tested)
- ✅ Zero-copy approach (proven)

**Medium Risk:**
- ⚠️ Concurrency (need careful testing)
- ⚠️ Compaction (potential for bugs)
- ⚠️ Group commit (durability trade-offs)

**High Risk:**
- 🔴 Production deployment (untested at scale)
- 🔴 Edge cases (corruption, crashes)
- 🔴 Performance regressions (optimization conflicts)

### Mitigation Strategies

**Testing:**
- Comprehensive unit tests
- Integration tests
- Stress tests (long-running)
- Fuzz testing (corruption detection)

**Validation:**
- Benchmark at scale (10M+ keys)
- Compare with RocksDB (fair comparison)
- Production pilot (small deployments)

**Monitoring:**
- Add metrics (latency, throughput)
- Add logging (errors, warnings)
- Add health checks (corruption detection)

---

## Decision Points

### End of Week 1 (After Phase 7)

**Evaluate:**
- Performance (competitive with RocksDB?)
- Stability (tests passing?)
- Complexity (maintainable?)

**Decide:**
- Continue to Phase 8-9? (specialize further)
- Stop and harden? (production-ready MVP)
- Pivot to hybrid? (RocksDB + AlexStorage)

### End of Week 2 (After Phase 9)

**Evaluate:**
- Performance vs targets
- Feature completeness
- Production readiness

**Decide:**
- Launch (production pilot)
- Defer (need more features)
- Redesign (fundamental issues)

---

## Conclusion

**AlexStorage has proven its read performance advantage:**
- 4.81x faster point queries than RocksDB
- 28.59x faster mixed workloads than RocksDB
- Simple, clean implementation

**Next 2 weeks: Build to production-ready:**
1. Add concurrency (P0)
2. Add deletes + compaction (P0-P1)
3. Add group commit (P1)
4. Add range queries (P2)
5. Harden for production (P3)

**Strategy: Specialize as read-optimized storage**
- Focus on read-heavy use cases
- Don't compete on writes
- Market: "Fastest read cache with durability"

**Timeline: 2 weeks to production-ready MVP**

---

**Last Updated:** October 6, 2025
**Next Phase:** Phase 5 (Concurrency)
**Target Completion:** October 20, 2025
