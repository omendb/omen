# omen

**Embedded PostgreSQL-compatible vector database**

> ⚠️ **Active Development**
>
> This project is under active development. We're building a vector database from first principles with learned data structures and modern algorithms.
>
> **Not Production-Ready**: API will change, performance is being optimized, documentation incomplete.
>
> See [ai/STATUS.md](ai/STATUS.md) for current progress.
>
> **License**: Elastic License 2.0 (free to use/modify, cannot resell as managed service)

---

## What We're Building

**Vision**: PostgreSQL-compatible vector database that's fast, memory-efficient, and embeddable.

**Technical Approach**:
- **HNSW index** for approximate nearest neighbor search
- **Binary Quantization** for memory efficiency
- **RocksDB storage** for persistence
- **PostgreSQL wire protocol** for compatibility
- **MVCC transactions** for concurrency

**Why This Matters**:
- pgvector doesn't scale beyond ~10M vectors
- Cloud vector DBs (Pinecone, etc.) are expensive and not self-hostable
- Most vector DBs have custom APIs (we use PostgreSQL protocol)

## Current Implementation

**What Works**:
- ✅ HNSW index implementation
- ✅ Binary Quantization for memory efficiency
- ✅ Graph serialization and persistence
- ✅ Parallel building
- ✅ Comprehensive test suite
- ✅ Memory safety validated

**What We're Working On**:
- 🔨 Custom HNSW implementation (replacing library for better performance)
- 🔨 SIMD optimizations
- 🔨 Performance benchmarking vs alternatives
- 🔨 PostgreSQL protocol integration
- 🔨 Production hardening

**What's Not Ready**:
- ❌ No public release yet
- ❌ Performance optimization in progress
- ❌ API may change
- ❌ Documentation incomplete
- ❌ No migration tools yet

See [ai/STATUS.md](ai/STATUS.md) for detailed current status and roadmap.

## Technical Details

### Architecture

```
omen/
├── vector/           # HNSW + Binary Quantization
├── storage/          # RocksDB persistence
├── sql_engine/       # SQL query execution
├── catalog/          # Table management
├── mvcc/             # MVCC transactions
└── rocks_storage/    # Storage backend
```

### Performance Characteristics

See [ai/STATUS.md](ai/STATUS.md) for current performance metrics and optimization progress.

## Research Foundation

This project implements ideas from recent research:

**Vector Indexing**:
- HNSW (Hierarchical Navigable Small World graphs)
- Binary Quantization for memory efficiency
- RaBitQ (SIGMOD 2024) quantization

**Storage**:
- LSM trees (RocksDB)
- Learned data structures (future: seerdb)
- MVCC snapshot isolation

See [ai/research/](ai/research/) for detailed paper summaries and implementation notes.


## License

Elastic License 2.0 - Free to use, modify, and self-host. Cannot resell as managed service. See [LICENSE](LICENSE).

## Related Projects

- **[seerdb](https://github.com/omendb/seerdb)** - Research-grade storage engine (foundation for omen)

---

**Note**: This is a work in progress. API will change. See [ai/STATUS.md](ai/STATUS.md) for current status.
