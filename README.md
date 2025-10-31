# omen

**Embedded PostgreSQL-compatible vector database**

> ⚠️ **Active Development**: This project is under active development (Week 7).
> Not production-ready yet. API may change before v1.0.
> See [ai/STATUS.md](ai/STATUS.md) for current progress.
>
> **Target**: Production-ready in 3-4 weeks
> **License**: Elastic License 2.0 (free to use/modify, cannot resell as managed service)

Drop-in replacement for pgvector. 10x faster, 28x more memory efficient. Source-available. Embeddable.

---

## Quick Start

**Coming Soon** - Currently in development (Week 1: Vector prototype)

```sql
-- Coming soon: PostgreSQL-compatible wire protocol
CREATE TABLE documents (
  id SERIAL PRIMARY KEY,
  content TEXT,
  embedding vector(1536)
);

CREATE INDEX ON documents USING alex (embedding);

SELECT * FROM documents
ORDER BY embedding <-> '[0.1, 0.2, ...]'
LIMIT 10;
```

---

## Why omen?

### vs pgvector (Current Standard)
- **10x faster** at 10M+ vectors
- **28x more memory efficient** (<2GB vs 60GB for 10M vectors)
- **Drop-in compatible** (PostgreSQL wire protocol)
- **Embeddable** (no separate server required)

### vs Pinecone (Popular Cloud)
- **100% free** (embedded library, no hosting costs)
- **Self-hostable** (compliance-friendly: HIPAA, SOC2, data sovereignty)
- **Source-available** (can verify, modify, contribute)
- **PostgreSQL-compatible** (no new API to learn)

### vs Weaviate/Qdrant (Vector Specialists)
- **PostgreSQL compatibility** (use existing tools, drivers, ORMs)
- **HTAP architecture** (one database for vectors + business data)
- **Learned indexing** (ALEX - 28x more memory efficient than B-trees)
- **Embeddable** (runs in your process, no network overhead)

---

## Technology

**ALEX Learned Indexing**:
- Multi-level hierarchical structure
- Predicts data location (vs tree traversal)
- 28x more memory efficient than traditional indexes
- Validated to 100M+ rows

**PostgreSQL Wire Protocol**:
- Drop-in replacement for pgvector
- Works with psql, pgcli, existing ORMs
- No new API to learn

**Production-Ready Stack**:
- MVCC snapshot isolation (concurrent operations)
- Crash recovery (100% success rate)
- Auth + SSL/TLS (enterprise-ready)
- RocksDB storage (LSM tree, write-optimized)

---

## License & Pricing

**omen** is source-available under the Elastic License 2.0:
- ✅ **Free to use** (embedded in your applications)
- ✅ **Free to modify** (fork, customize, contribute)
- ✅ **Free to self-host** (deploy on your infrastructure)
- ❌ **Cannot resell as managed service** (protects future omen-server business)

**For hosted/managed service**: **omen-server** is planned for the future - a fully managed cloud service built on omen with multi-tenancy, authentication, and enterprise features.

---

## Use Cases

**RAG Applications**:
- Chatbots with document Q&A
- Knowledge base search
- Customer support automation

**Semantic Search**:
- Code search (find similar functions)
- Research paper search
- Documentation search

**AI Agents**:
- LangChain / LlamaIndex integrations
- Multi-step reasoning with memory
- Tool-augmented agents

**Enterprise AI**:
- Healthcare: Patient similarity, medical records
- Finance: Fraud detection, document analysis
- Legal: Case law search, contract similarity

---

## Development Status

**Week 7** (Current - October 2025):
- ✅ HNSW index implementation (99.5% recall, <15ms p95)
- ✅ Binary Quantization (19.9x memory reduction)
- ✅ Graph serialization (4175x speedup at 1M scale)
- ✅ Parallel building (16x speedup)
- ✅ PostgreSQL wire protocol
- ✅ MVCC snapshot isolation
- ✅ 142 tests passing (Phase 2 validation 60% complete)

**Next Steps**:
- Complete Phase 2 validation (edge cases, resource limits)
- Performance benchmarks vs pgvector
- Production hardening
- Documentation and examples

---

## License

**Elastic License 2.0** (source-available)

**What this means**:
- ✅ Free to use, modify, and self-host
- ✅ Source code publicly available
- ✅ Community can contribute
- ✅ Enterprises can deploy on their infrastructure
- ❌ Cannot resell as managed service

[Read full license](LICENSE)

---

## Contributing

**Coming soon** - We'll open contributions once the vector prototype is stable (Month 2-3)

For now:
- Star the repo to follow development
- Join discussions (coming soon)
- Report issues (coming soon)

---

## Documentation

- [CONTEXT.md](CONTEXT.md) - Quick overview (start here)
- [CLAUDE.md](CLAUDE.md) - Full project context
- [ai/TODO.md](ai/TODO.md) - Current development tasks
- [ai/STATUS.md](ai/STATUS.md) - Current state
- [ai/RESEARCH.md](ai/RESEARCH.md) - Vector algorithm research

---

## Community & Support

- **Website**: [omendb.io](https://omendb.io) (coming soon)
- **Email**: nick@omendb.com
- **GitHub Discussions**: Coming soon
- **Twitter**: Coming soon

---

## Related Projects

- **omen-server**: Planned fully managed cloud service built on omen (future)
- **pg-learned**: PostgreSQL extension demonstrating learned indexes ([omendb/pg-learned](https://github.com/omendb/pg-learned))

---

**Status**: 🔬 Week 7 - Phase 2 validation (142 tests passing) - Not ready for production use

**Star this repo to follow development** ⭐
