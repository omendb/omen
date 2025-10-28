# Archived Binaries

## Date: October 27, 2025
## Reason: Repository reorganization (omendb-server → omen)

As part of the pivot from a general-purpose embedded database to a specialized vector database, we archived 26 pre-pivot benchmark binaries that were no longer relevant to the vector database focus.

## Pre-Vector Pivot Benchmarks (26 binaries)

### ALEX vs B-tree Comparisons (Obsolete after vector pivot)
- `benchmark_vs_btree.rs` - ALEX vs B-tree comparison
- `benchmark_alex_storage.rs` - ALEX storage performance
- `benchmark_alex_improvements.rs` - ALEX optimization benchmarks
- `test_1m_alex.rs` - 1M row ALEX test
- `profile_alex_detailed.rs` - Detailed ALEX profiling

### SQLite Comparisons (Obsolete after vector pivot)
- `benchmark_vs_sqlite.rs` - Direct SQLite comparison
- `benchmark_table_vs_sqlite.rs` - Table-level SQLite comparison
- `benchmark_multilevel_vs_sqlite.rs` - Multi-level ALEX vs SQLite

### Multi-Level ALEX Experiments (Superseded by vector implementation)
- `benchmark_multi_level_alex.rs` - Multi-level ALEX benchmarks
- `test_multi_level.rs` - Multi-level ALEX tests
- `debug_multi_level.rs` - Multi-level ALEX debugging

### HTAP Demos (Not vector-focused)
- `benchmark_htap.rs` - HTAP workload benchmarks
- `htap_demo.rs` - HTAP demonstration
- `benchmark_temperature.rs` - Temperature-based workload tests

### YCSB Benchmarks (Not vector-focused)
- `ycsb_benchmark.rs` - YCSB workload benchmarks
- `ycsb_quick_test.rs` - Quick YCSB validation
- `ycsb_subset.rs` - YCSB subset testing

### Storage Engine Experiments (Old storage investigations)
- `benchmark_redb_learned.rs` - redb storage experiments
- `benchmark_mmap_validation.rs` - Memory-mapped I/O validation

### General Benchmarks (Not vector-specific)
- `benchmark_honest_comparison.rs` - General comparison benchmark
- `benchmark_full_system.rs` - Full system benchmark
- `benchmark_comprehensive.rs` - Comprehensive benchmark suite
- `benchmark_fair_comparison.rs` - Fair comparison benchmark
- `benchmark_datafusion_sql.rs` - DataFusion SQL benchmark
- `benchmark_simd_search.rs` - SIMD search benchmark
- `benchmark_memory_optimization.rs` - Memory optimization benchmark

## Why Archived vs Deleted?

These binaries represent significant development history and experimental work. While no longer relevant to the vector database focus, they provide:

1. **Historical context** - Shows what was tried during development
2. **Lessons learned** - Performance characteristics of different approaches
3. **Recovery option** - Can be restored if needed for reference
4. **Git history preservation** - Maintains complete development timeline

## Current Focus (Post-Archival)

The repository now focuses exclusively on vector database capabilities:

**Vector-Related Binaries (Kept):**
- HNSW benchmarks
- Binary Quantization benchmarks
- PCA benchmarks
- Vector serialization tests
- Parallel building tests

**Infrastructure Binaries (Kept):**
- PostgreSQL server
- REST server
- Integration tests
- Backup tools
- Crash recovery tests
- Metrics server

## Restoration Instructions

If you need to restore any of these binaries:

```bash
# Copy binary back to src/bin/
cp archive/old-binaries/BINARY_NAME.rs src/bin/

# Add [[bin]] entry back to Cargo.toml
[[bin]]
name = "BINARY_NAME"
path = "src/bin/BINARY_NAME.rs"

# Rebuild
cargo build --release --bin BINARY_NAME
```

---

*Archived during: OmenDB reorganization (omendb-server → omen)*
*Pivot: General embedded database → PostgreSQL-compatible vector database*
*Technology preserved: HNSW, Binary Quantization, MVCC, PostgreSQL protocol*
