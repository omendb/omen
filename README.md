# omen

**Embedded PostgreSQL-compatible vector database**

> ⚠️ **Active Development - Week 7**
>
> This project is under active development. We're building a vector database from first principles with learned data structures and modern algorithms.
>
> **Current Status**: 142 tests passing, HNSW + Binary Quantization working, implementing custom HNSW for performance
>
> **Not Production-Ready**: API will change, performance is being optimized, documentation incomplete
>
> See [ai/STATUS.md](ai/STATUS.md) for current progress and [PRODUCT_ROADMAP.md](PRODUCT_ROADMAP.md) for future plans.
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

**What Works** (Week 7):
- ✅ HNSW index with 97-100% recall
- ✅ Binary Quantization (19.9x memory reduction)
- ✅ Graph serialization (4175x faster than rebuild at 1M vectors)
- ✅ Parallel building (16x speedup)
- ✅ 142 tests passing
- ✅ ASAN validated (zero memory safety issues)

**What We're Working On**:
- 🔨 Custom HNSW implementation (replacing library for better performance)
- 🔨 SIMD optimizations
- 🔨 Performance benchmarking vs alternatives
- 🔨 PostgreSQL protocol integration
- 🔨 Production hardening

**What's Not Ready**:
- ❌ No public release yet (Week 7 of development)
- ❌ Performance claims not finalized (still optimizing)
- ❌ API may change
- ❌ Documentation incomplete
- ❌ No migration tools yet

## Development Roadmap

**Current Phase (Weeks 7-10)**: Core engine optimization
- Custom HNSW implementation
- Performance profiling and optimization
- Scale testing (1M+ vectors)

**Next Phase (Weeks 11-15)**: Advanced features
- Extended RaBitQ quantization (SIGMOD 2025 paper)
- HNSW-IF for billion-scale support
- Time-series module integration

**Future**: Production release when ready (no ETA yet)

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

**Current** (Week 7, with library HNSW):
- Build speed: 16x faster with parallel building
- Query latency: <15ms p95 at 1M vectors
- Memory: ~7GB for 1M vectors @ 1536D
- Recall: 97-100% on standard benchmarks

**Goal** (After custom HNSW + optimizations):
- Significantly faster queries (profiling in progress)
- Better memory efficiency
- Billion-scale support

*Note: We're not making specific competitive claims until implementation is complete and properly benchmarked.*

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

## Development Philosophy

**Research-Driven**: Build from first principles using proven algorithms from recent papers

**Measured Performance**: Validate all performance claims with rigorous benchmarking

**Production-Quality**: Comprehensive testing (142 tests, ASAN validated, crash recovery)

**Honest Communication**: Share progress openly, don't overpromise

## Installation (When Ready)

Not yet available. Project is in active development.

When released, installation will be:
```bash
cargo install omen
omen server --port 5433
```

## Contributing

We welcome contributions! Areas that need help:
- Performance optimization
- Testing and validation
- Documentation
- Research paper implementation

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines (coming soon).

## License

**Elastic License 2.0** (source-available)

This means:
- ✅ Free to use, modify, and self-host
- ✅ Source code publicly available
- ✅ Community can contribute
- ❌ Cannot resell as a managed cloud service

Full text: [LICENSE](LICENSE)

## Related Projects

- **[seerdb](https://github.com/omendb/seerdb)** - Research-grade storage engine (foundation for omen)
- **omen-server** (private) - Future managed service
- **omen-queue** (private) - Future job queue (paused, will use seerdb)

## Contact

- GitHub Issues: [Report bugs or request features](https://github.com/omendb/omen/issues)
- Development updates: Watch this repo for progress

---

**Remember**: This is a work in progress. Use at your own risk. API will change. Performance numbers are preliminary.

See [ai/STATUS.md](ai/STATUS.md) for detailed current status and [PRODUCT_ROADMAP.md](PRODUCT_ROADMAP.md) for long-term vision.
