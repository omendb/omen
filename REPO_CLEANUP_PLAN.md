# Repository Cleanup & Organization Plan

**Date:** October 1, 2025
**Status:** Analysis complete, ready for cleanup

---

## 📊 Current State Analysis

### Test Coverage: 45.62% (1495/3277 lines)

**Well-tested modules (>70%):**
- ✅ `mvcc.rs`: 100% (22/22)
- ✅ `metrics.rs`: 99% (120/121)
- ✅ `catalog.rs`: 89% (54/61)
- ✅ `table_wal.rs`: 84% (84/100)

**Under-tested modules (<50%):**
- ❌ `postgres/*`: 0% coverage (just implemented, no tests)
- ❌ `sql_engine.rs`: 34% (170/502)
- ❌ `integration_tests.rs`: 4% (12/301)
- ❌ `backup.rs`: 31% (96/310)
- ❌ `datafusion/redb_table.rs`: 36% (15/42)

**Test Results:**
- 182/195 tests passing
- 13 tests ignored
- 0 failures
- No flaky tests

---

## 📝 Documentation Issues

### Current State: 20 markdown files, 6728 total lines

**Problematic docs:**

1. **SESSION_SUMMARY*.md** (3 files, 825 lines)
   - SESSION_SUMMARY.md (312 lines) - Sept 30 session
   - SESSION_SUMMARY_OCT1.md (249 lines) - Oct 1 session
   - SESSION_SUMMARY_OCT1_DAY2.md (264 lines) - Oct 1 day 2
   - **Issue:** Temporary session notes, should be archived or deleted

2. **BUGS_FOUND.md** (233 lines)
   - **Status:** Historical record from verification phase
   - **Decision:** Move to docs/archive/ or delete (bugs fixed)

3. **TIER1_PROGRESS.md** (233 lines)
   - **Status:** Sept 30 progress tracking
   - **Decision:** Outdated, merge into PROJECT_STATUS.md or delete

4. **VERIFICATION_*.md** (3 files, 923 lines)
   - VERIFICATION_PLAN.md (258 lines)
   - VERIFICATION_COMPLETE.md (313 lines)
   - V0.2.0_VERIFICATION_REPORT.md (352 lines)
   - **Issue:** Pre-release verification docs, now complete
   - **Decision:** Archive or consolidate into single doc

5. **Duplicate status docs:**
   - PROJECT_STATUS.md (383 lines) - Main status
   - PRODUCTION_READY.md (377 lines) - Overlaps with PROJECT_STATUS
   - **Decision:** Merge into single source of truth

---

## 🎯 Cleanup Recommendations

### Phase 1: Archive Temporary Docs (SAFE)

Create `docs/archive/` and move:
```bash
mkdir -p docs/archive/verification docs/archive/sessions

# Session notes
mv SESSION_SUMMARY*.md docs/archive/sessions/

# Verification docs
mv BUGS_FOUND.md TIER1_PROGRESS.md docs/archive/verification/
mv VERIFICATION_*.md V0.2.0_VERIFICATION_REPORT.md docs/archive/verification/
```

### Phase 2: Consolidate Status Docs

**Merge into PROJECT_STATUS.md:**
- Current content from PROJECT_STATUS.md
- Production readiness section from PRODUCTION_READY.md
- Week 1 accomplishments from WEEK1_SUMMARY.md
- Add Week 2 Day 1 (PostgreSQL wire protocol)

**Delete after merge:**
- PRODUCTION_READY.md (redundant)

**Keep:**
- WEEK1_SUMMARY.md (historical record)
- PGWIRE_NOTES.md (technical deep-dive)

### Phase 3: Keep Core Docs

**Keep these (well-organized, valuable):**
- ✅ README.md (495 lines) - Main entry point
- ✅ QUICKSTART.md (466 lines) - Getting started guide
- ✅ PROJECT_STATUS.md (383 lines, after merge) - Current status
- ✅ PERFORMANCE.md (362 lines) - Benchmark results
- ✅ LIBRARY_DECISIONS.md (479 lines) - Architecture rationale
- ✅ DATAFUSION_MIGRATION.md (419 lines) - Migration guide
- ✅ ARCHITECTURE_LIMITATIONS.md (298 lines) - Known limitations
- ✅ PGWIRE_NOTES.md (268 lines) - PostgreSQL protocol details
- ✅ WEEK1_SUMMARY.md (462 lines) - Week 1 retrospective

**Move to docs/ subdirectory:**
- ERROR_HANDLING_AUDIT.md → docs/audits/
- STRUCTURED_LOGGING.md → docs/operations/

---

## 🧪 Testing Priorities

### Critical: Add PostgreSQL Wire Protocol Tests (0% → 80%)

**Priority 1: Integration tests**
```rust
#[tokio::test]
async fn test_psql_connection() {
    // Start server, connect with tokio-postgres, verify basic queries
}

#[tokio::test]
async fn test_query_execution() {
    // Test SELECT, INSERT, special commands
}

#[tokio::test]
async fn test_type_conversion() {
    // Test all Arrow → PostgreSQL type conversions
}
```

**Priority 2: Unit tests for encoding.rs**
- Test arrow_to_pg_type() for all types
- Test null handling
- Test DataRow encoding

**Priority 3: Handler tests**
- Test special command detection
- Test error mapping
- Test response creation

### Medium Priority: Increase SQL Engine Coverage (34% → 60%)

**Focus areas:**
- JOIN execution paths
- Aggregate functions (SUM, COUNT, AVG)
- ORDER BY + LIMIT combinations
- Complex WHERE clauses

### Low Priority: Integration Tests (4% → 20%)

Most integration_tests.rs code is test setup, not critical for coverage.

---

## 🌐 REST API Decision

### Question: Do we need REST API alongside PostgreSQL wire protocol?

**Arguments FOR REST API:**
1. **Operational endpoints:**
   - `/health` - Kubernetes liveness/readiness
   - `/metrics` - Prometheus scraping
   - `/api/v1/query` - HTTP query interface (for web apps)
   - `/api/v1/admin/*` - Admin operations

2. **Web/mobile compatibility:**
   - No need for PostgreSQL driver in browser
   - Simple curl/fetch API
   - WebSocket potential for streaming

3. **Observability:**
   - Easy integration with monitoring tools
   - JSON response format
   - Standard HTTP status codes

**Arguments AGAINST:**
1. **PostgreSQL wire protocol is sufficient** for:
   - All database operations (queries, inserts, admin)
   - Any programming language (pg drivers exist for all)
   - BI tools, data pipelines, etc.

2. **Overhead:**
   - Additional code to maintain
   - Another attack surface
   - Duplicate functionality

**Recommendation: YES, but minimal**

Implement a **lightweight REST API** focused on:
- ✅ Health checks (`/health`, `/ready`)
- ✅ Metrics export (`/metrics` - Prometheus format)
- ✅ Simple query endpoint (`POST /query` - returns JSON)
- ❌ Skip full CRUD operations (use PostgreSQL protocol)

**Estimated effort:** 2-3 hours with axum (already in dependencies)

**Value:** High for Kubernetes deployments, monitoring, and web dashboards.

---

## 📦 Cross-Platform Testing (Linux/Fedora)

### Prerequisites for Fedora PC

**System packages:**
```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# PostgreSQL client (for testing)
sudo dnf install postgresql

# Build dependencies
sudo dnf install gcc make openssl-devel

# Optional: NVIDIA CUDA (for GPU learned index acceleration)
# Only if testing GPU optimization
sudo dnf install cuda-toolkit
```

**Cargo tools:**
```bash
cargo install cargo-tarpaulin  # Coverage
cargo install cargo-watch      # Hot reload
cargo install cargo-nextest    # Better test runner
```

### Testing on Linux

**Key differences to test:**
- ✅ File paths (no Mac-specific assumptions)
- ✅ Network binding (0.0.0.0 vs 127.0.0.1)
- ✅ PostgreSQL client compatibility
- ✅ Performance characteristics (Intel vs Apple Silicon)
- ✅ NVIDIA GPU acceleration (if implementing learned index on GPU)

**Recommended test sequence:**
```bash
# 1. Clone and build
git clone <repo> && cd omendb-rust
cargo build --release

# 2. Run tests
cargo test --release

# 3. Start PostgreSQL server
cargo run --release --bin postgres_server

# 4. Test with psql (from another terminal)
psql -h 127.0.0.1 -p 5432 -c "SELECT * FROM users"

# 5. Run benchmarks
cargo bench

# 6. Coverage
cargo tarpaulin --lib
```

---

## 🚀 Action Items

### Immediate (This Session)

1. ✅ Archive session notes to docs/archive/
2. ✅ Move verification docs to docs/archive/
3. ✅ Update PROJECT_STATUS.md with pgwire completion
4. ✅ Commit cleanup
5. ✅ Push all changes to remote

### Week 2 Day 2 (Next Session)

1. ⏳ Add PostgreSQL wire protocol tests (4-6 hours)
   - Integration tests with tokio-postgres
   - Type conversion tests
   - Handler unit tests

2. ⏳ Implement minimal REST API (2-3 hours)
   - Health endpoints
   - Metrics endpoint
   - Simple query endpoint

3. ⏳ Test on Fedora/Linux (1-2 hours)
   - Cross-platform compatibility
   - Performance validation

### Week 2 Day 3-4 (After testing)

1. Documentation update
2. Coverage improvement to 60%+
3. Performance benchmarking on Linux
4. Prepare for open source release

---

## 📊 Summary

**Current State:**
- ✅ Clean code (no .sh/.py files in root)
- ✅ 182/195 tests passing
- ⚠️ Too many docs (20 files, need consolidation)
- ⚠️ Low coverage in new modules (postgres: 0%, sql_engine: 34%)
- ✅ All changes committed

**Next Steps:**
1. Clean up documentation (archive temporary files)
2. Add PostgreSQL protocol tests
3. Implement minimal REST API (health/metrics)
4. Test on Linux/Fedora
5. Push all changes

**Estimated Time:** 3-4 hours for cleanup + testing + REST API
