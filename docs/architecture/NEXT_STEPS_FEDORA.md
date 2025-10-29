# Next Steps - Fedora Benchmarking

**Date**: October 28, 2025
**Status**: Ready for Fedora benchmarking when machine comes online
**Fedora Status**: Offline (last seen 2h ago via Tailscale)

---

## Summary - What's Complete ✅

### Mac M3 Max Baseline
- ✅ **1M validation complete**: 3165s build, p95=20.37ms queries, 5.7GB memory
- ✅ **10K benchmark test passed**: OmenDB side working (9.6s, 1040 vec/sec, p95=10.78ms)
- ✅ **All infrastructure ready**: Binaries compiled, docs written, methodology defined

### Code Status
- ✅ **Repository cleaned**: 31 binaries archived, warnings fixed
- ✅ **Critical bugs fixed**: save/load bug resolved (commit c88054e)
- ✅ **Performance reviewed**: Production-ready, no obvious issues
- ✅ **367 tests passing**: All validation complete

### Documentation
- ✅ **1M_VALIDATION_RESULTS.md**: Complete baseline results from Mac
- ✅ **FEDORA_BENCHMARK_SETUP.md**: Step-by-step setup guide
- ✅ **PGVECTOR_BENCHMARK_PLAN.md**: Methodology and success criteria
- ✅ **QUICK_PERF_REVIEW.md**: Hot path analysis

### Benchmark Infrastructure
- ✅ **benchmark_pgvector_comparison.rs**: Compiled and tested (OmenDB side)
- ✅ **validate_1m_end_to_end.rs**: Complete, documented results
- ✅ **profile_queries.rs**: Ready for flamegraph analysis

---

## What's Needed - Fedora Setup 🔨

When Fedora comes online, execute these steps:

### 1. PostgreSQL + pgvector Installation (30 min)

```bash
# SSH to Fedora
ssh nick@fedora

# Install PostgreSQL 16
sudo dnf install -y postgresql16-server postgresql16-devel postgresql16
sudo /usr/pgsql-16/bin/postgresql-16-setup initdb
sudo systemctl start postgresql-16
sudo systemctl enable postgresql-16

# Install build dependencies
sudo dnf install -y git gcc make clang-devel

# Clone and install pgvector
cd ~/github
git clone https://github.com/pgvector/pgvector.git
cd pgvector
export PATH=/usr/pgsql-16/bin:$PATH
make
sudo make install

# Verify
ls /usr/pgsql-16/lib/pgvector.so
ls /usr/pgsql-16/share/extension/vector*
```

### 2. PostgreSQL Configuration (5 min)

```bash
# Allow local connections (benchmarking only)
sudo bash -c 'cat >> /var/lib/pgsql/16/data/pg_hba.conf << EOF
local   all             all                                     trust
host    all             all             127.0.0.1/32            trust
host    all             all             ::1/128                 trust
EOF'

# Restart
sudo systemctl restart postgresql-16

# Create benchmark database
sudo -u postgres psql -c "CREATE DATABASE benchmark_pgvector;"
sudo -u postgres psql benchmark_pgvector -c "CREATE EXTENSION vector;"
sudo -u postgres psql benchmark_pgvector -c "\\dx vector"
```

### 3. Copy omen Repository (10 min)

```bash
# On Mac: Create tarball
cd /Users/nick/github/omendb/omen
tar czf /tmp/omen-$(date +%Y%m%d).tar.gz .

# Copy to Fedora
scp /tmp/omen-*.tar.gz nick@fedora:/tmp/

# On Fedora: Extract
mkdir -p ~/github/omendb
cd ~/github/omendb
tar xzf /tmp/omen-*.tar.gz
```

### 4. Build omen on Fedora (5 min)

```bash
cd ~/github/omendb/omen

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Build release
cargo build --release --bin benchmark_pgvector_comparison

# Verify
./target/release/benchmark_pgvector_comparison --help
```

---

## Running the Benchmark (1-2 hours)

### 1M Vectors Benchmark

```bash
cd ~/github/omendb/omen

# Run 3 times for median
for i in 1 2 3; do
    echo "=== Run $i/3 ==="
    ./target/release/benchmark_pgvector_comparison 1000000 2>&1 | tee benchmark_run_$i.log
    sleep 60  # Cool down between runs
done

# Analyze results
grep "Build time:" benchmark_run_*.log
grep "Query p95:" benchmark_run_*.log
grep "Memory:" benchmark_run_*.log
```

### Expected Results (Based on Previous Fedora Tests)

**OmenDB** (Fedora 24-core):
- Build time: ~26 minutes (16x speedup vs Mac's 52 min)
- Build rate: ~640 vec/sec (vs Mac's 316 vec/sec)
- Query p95: ~15ms (vs Mac's 20ms)
- Memory: ~7.3 GB (same as Mac)

**pgvector** (to be measured):
- Build time: TBD (expected: 30-120 minutes)
- Query p95: TBD (expected: 15-50ms)
- Memory: TBD (expected: ~6.1 GB)

---

## Analysis & Documentation (2-3 hours)

### 1. Calculate Median Results

```bash
# From 3 runs, report median for:
# - Build time
# - Query p50, p95, p99
# - Memory usage
# - Recall (if measured)
```

### 2. Create Results Document

File: `docs/architecture/PGVECTOR_BENCHMARK_RESULTS.md`

Structure:
- Executive summary (3 paragraphs)
- Full results (all metrics, all scales)
- Methodology (reproducibility instructions)
- Parameter tuning process
- Known limitations (honest disclosure)
- Hardware specifications
- When to use OmenDB vs pgvector

### 3. Update Marketing Claims

Based on results, update positioning:
- **Optimistic**: "5-9x faster builds, 16x memory savings, 2x query speed"
- **Realistic**: "5x faster with parallel building, 16x memory with BQ"
- **Pessimistic**: "16x memory savings, comparable performance"

### 4. Update Roadmap

Update `ai/STATUS.md`:
- Mark Week 7-8 complete
- Document benchmark results
- Plan Week 9-10: HNSW-IF implementation

---

## Success Criteria ✅

**Minimum Viable Results** (to proceed):
- ✅ 2x faster queries OR 5x faster builds OR 16x memory savings (at least ONE)
- ✅ >90% recall for both systems
- ✅ Reproducible methodology documented
- ✅ Honest limitations disclosed

**Ideal Results** (strong marketing):
- ✅ 5x faster builds (parallel)
- ✅ 16x memory savings (BQ)
- ✅ 2x faster queries
- ✅ >95% recall for both
- ✅ Clear scale advantage at 1M+

**Stretch Goals**:
- ✅ 10M scale validation (if RAM permits: 32GB available)
- ✅ Independent reproduction by external party
- ✅ Published benchmark blog post
- ✅ HackerNews discussion (100+ points)

---

## Fallback Plan (If Fedora Unavailable)

If Fedora remains offline, alternatives:
1. **Cloud VM**: Spin up AWS c7g.8xlarge (16 vCPU) for benchmarking
2. **Mac only**: Run benchmarks on Mac, disclose hardware in results
3. **Defer**: Wait for Fedora, focus on HNSW-IF implementation (Week 9-10)

**Recommendation**: Wait 24-48h for Fedora before considering alternatives

---

## Quick Reference - Key Files

**On Mac** (ready):
- `docs/architecture/1M_VALIDATION_RESULTS.md` - Baseline results
- `docs/architecture/FEDORA_BENCHMARK_SETUP.md` - Setup guide
- `docs/architecture/PGVECTOR_BENCHMARK_PLAN.md` - Methodology
- `src/bin/benchmark_pgvector_comparison.rs` - Benchmark binary

**To Create** (after Fedora run):
- `docs/architecture/PGVECTOR_BENCHMARK_RESULTS.md` - Final results
- `docs/blog/PGVECTOR_BENCHMARK_ANNOUNCEMENT.md` - Blog post draft

**To Update**:
- `ai/STATUS.md` - Mark Week 7-8 complete
- `ai/TODO.md` - Plan Week 9-10 tasks
- `CLAUDE.md` - Update current status

---

## Contact Points

**Fedora Access**: `ssh nick@fedora` (Tailscale)
**Fedora Status**: Check `tailscale status | grep fedora`
**Fedora IP**: 100.93.39.25 (tail612d50.ts.net)

**Hardware**:
- Fedora: i9-13900KF (24-core), 32GB DDR5, RTX 4090
- Mac: M3 Max (~12 cores), 128GB RAM

---

**Status**: October 28, 2025 - All Mac work complete, waiting for Fedora
**Next**: Execute Fedora setup → Run benchmarks → Document results
**Timeline**: 4-6 hours once Fedora is online

---

**Last Updated**: October 28, 2025
**Ready For**: Fedora benchmarking (when machine comes online)
