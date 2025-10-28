# Fedora Benchmark Setup Guide

**Machine**: Fedora i9-13900KF (24-core, 32GB DDR5)
**Purpose**: Primary benchmarking platform for pgvector comparison
**Rationale**: 16x parallel building speedup vs 4.6x on Mac

---

## System Preparation

### 1. Install PostgreSQL 16

```bash
# Install PostgreSQL 16 (latest stable)
sudo dnf install -y postgresql16-server postgresql16-devel postgresql16

# Initialize database
sudo /usr/pgsql-16/bin/postgresql-16-setup initdb

# Start and enable service
sudo systemctl start postgresql-16
sudo systemctl enable postgresql-16

# Verify installation
psql --version  # Should show: psql (PostgreSQL) 16.x
```

### 2. Install pgvector

```bash
# Install build dependencies
sudo dnf install -y git gcc make clang-devel

# Clone pgvector
cd ~/github
git clone https://github.com/pgvector/pgvector.git
cd pgvector

# Build and install (for PostgreSQL 16)
export PATH=/usr/pgsql-16/bin:$PATH
make
sudo make install

# Verify installation
ls /usr/pgsql-16/lib/pgvector.so  # Should exist
ls /usr/pgsql-16/share/extension/vector*  # Should show vector.control, vector--*.sql
```

### 3. Configure PostgreSQL

```bash
# Allow local connections without password (for benchmarking)
sudo bash -c 'cat >> /var/lib/pgsql/16/data/pg_hba.conf << EOF
# Benchmark access (local only)
local   all             all                                     trust
host    all             all             127.0.0.1/32            trust
host    all             all             ::1/128                 trust
EOF'

# Restart PostgreSQL
sudo systemctl restart postgresql-16
```

### 4. Create benchmark database

```bash
# Connect as postgres user
sudo -u postgres psql

# Create database and enable extension
CREATE DATABASE benchmark_pgvector;
\c benchmark_pgvector
CREATE EXTENSION vector;

# Verify extension
\dx vector  # Should show: vector | 0.8.1 | ...

# Create test table
CREATE TABLE embeddings (
  id SERIAL PRIMARY KEY,
  embedding vector(1536)
);

# Exit
\q
```

---

## Rust Environment Setup

### 1. Install Rust (if not already installed)

```bash
# Install mise (modern version manager)
curl -fsSL https://mise.run/install.sh | sh

# Add to shell
echo 'eval "$(~/.local/bin/mise activate bash)"' >> ~/.bashrc
source ~/.bashrc

# Install latest Rust
mise install rust@latest
mise use rust@latest

# Verify
rustc --version
cargo --version
```

### 2. Clone omen repository

```bash
# Clone from local Mac (via Tailscale)
# On Mac: cd /Users/nick/github/omendb/omen && tar czf omen.tar.gz .
# On Fedora:
cd ~/github/omendb
mkdir -p omen
cd omen
scp nick@apple:/Users/nick/github/omendb/omen/omen.tar.gz .
tar xzf omen.tar.gz

# Or use git (if pushed to remote)
# git clone git@github.com:omendb/omen.git
# cd omen
```

### 3. Build omen

```bash
cd ~/github/omendb/omen

# Install build dependencies
sudo dnf install -y clang-devel

# Build release version
cargo build --release

# Verify
./target/release/validate_1m_end_to_end --help  # Should show usage or run
```

---

## Benchmark Execution

### 1. Run OmenDB 1M baseline

```bash
cd ~/github/omendb/omen

# Run 1M validation (parallel building)
time ./target/release/validate_1m_end_to_end

# Expected results:
# - Build: ~25 minutes (16x speedup, 643 vec/sec)
# - Save: ~4-5 seconds
# - Load: ~6 seconds
# - Query p95: <15ms
# - Disk: ~7.3 GB
```

### 2. Run pgvector 1M baseline

```bash
# TODO: Create benchmark_pgvector_comparison.rs
# This will:
# 1. Generate 1M realistic embeddings (1536D)
# 2. Insert into PostgreSQL via pgvector
# 3. Build HNSW index (M=48, ef_construction=200)
# 4. Run 100 queries (k=10)
# 5. Measure: build time, memory, query latency, recall
```

### 3. Compare results

```bash
# Output format:
# OmenDB vs pgvector - 1M Vectors (1536D)
#
# Build Time:
#   OmenDB (parallel):  25.9 min
#   pgvector:           TBD
#   Speedup:            TBD
#
# Memory Usage:
#   OmenDB (no BQ):     7.3 GB
#   OmenDB (with BQ):   365 MB (estimated)
#   pgvector:           TBD
#
# Query Latency (p95):
#   OmenDB:             14.23 ms
#   pgvector:           TBD
```

---

## Performance Tuning

### PostgreSQL Configuration

For optimal pgvector performance:

```bash
sudo -u postgres psql

# Increase shared buffers (25% of RAM = 8GB)
ALTER SYSTEM SET shared_buffers = '8GB';

# Increase work_mem (for index building)
ALTER SYSTEM SET work_mem = '512MB';

# Increase maintenance_work_mem (for CREATE INDEX)
ALTER SYSTEM SET maintenance_work_mem = '2GB';

# Disable synchronous commit (for benchmarking only)
ALTER SYSTEM SET synchronous_commit = 'off';

# Restart to apply
sudo systemctl restart postgresql-16
```

### OmenDB Configuration

No special tuning needed - uses all available cores automatically via Rayon.

---

## Monitoring

### System Resources

```bash
# CPU usage
htop  # Press F5 for tree view, watch parallel building

# Memory usage
watch -n 1 'free -h'

# Disk I/O
iostat -x 1

# PostgreSQL activity
sudo -u postgres psql -c "SELECT * FROM pg_stat_activity WHERE state = 'active';"
```

### Benchmark Progress

```bash
# OmenDB - watch stdout for progress
# pgvector - query pg_stat_progress_create_index
sudo -u postgres psql benchmark_pgvector -c "SELECT * FROM pg_stat_progress_create_index;"
```

---

## Troubleshooting

### PostgreSQL connection issues

```bash
# Check if PostgreSQL is running
sudo systemctl status postgresql-16

# Check logs
sudo tail -f /var/lib/pgsql/16/data/log/postgresql-*.log

# Test connection
psql -h localhost -U postgres -d benchmark_pgvector -c "SELECT version();"
```

### Build failures

```bash
# Missing clang-devel
sudo dnf install -y clang-devel

# RocksDB compilation issues
cargo clean
cargo build --release
```

### Out of memory

```bash
# Check available memory
free -h

# For 1M vectors: ~7-8GB needed
# For 10M vectors: ~70-80GB needed (exceeds 32GB - use Mac)

# Reduce batch size if needed (edit validate_1m_end_to_end.rs)
```

---

## Next Steps

1. **Complete setup**: Run all commands above
2. **Validate OmenDB**: Run 1M validation on Fedora
3. **Implement pgvector benchmark**: Create benchmark_pgvector_comparison.rs
4. **Run comparison**: Execute both benchmarks, collect data
5. **Document results**: Update PGVECTOR_BENCHMARK_RESULTS.md

---

**Last Updated**: October 28, 2025
**Status**: Setup guide created, ready for execution
**Owner**: Nick (Fedora i9-13900KF)
**Next**: SSH to Fedora and execute setup
