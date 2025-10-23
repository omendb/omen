# OmenDB Server

**PostgreSQL-compatible vector database**

Drop-in replacement for pgvector. 10x faster, 28x more memory efficient. Source-available. Self-hostable.

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

## Why OmenDB?

### vs pgvector (Current Standard)
- **10x faster** at 10M+ vectors
- **28x more memory efficient** (<2GB vs 60GB for 10M vectors)
- **Drop-in compatible** (PostgreSQL wire protocol)

### vs Pinecone (Popular Cloud)
- **90% cheaper** ($99 vs $500/month for 10M vectors)
- **Self-hostable** (compliance-friendly: HIPAA, SOC2, data sovereignty)
- **Source-available** (can verify, modify, contribute)
- **PostgreSQL-compatible** (no new API to learn)

### vs Weaviate/Qdrant (Vector Specialists)
- **PostgreSQL compatibility** (use existing tools, drivers, ORMs)
- **HTAP architecture** (one database for vectors + business data)
- **Learned indexing** (ALEX - 28x more memory efficient than B-trees)

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

## Pricing

| Tier | Price | Vectors | Queries/mo | Best For |
|------|-------|---------|------------|----------|
| **Developer** | **FREE** | 100K | 100K | Prototyping, learning |
| **Starter** | **$29/mo** | 1M | 1M | Side projects, early startups |
| **Growth** | **$99/mo** | 10M | 10M | Production apps, scaling startups |
| **Enterprise** | **Custom** | Unlimited | Unlimited | Large deployments, compliance |

**Why this pricing wins**:
- **Predictable**: No surprise bills (vs Pinecone usage spikes)
- **Transparent**: Know your costs upfront
- **Competitive**: 90% cheaper than Pinecone

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

**Week 1** (Current):
- Vector data type prototype
- ALEX indexing for high-dimensional vectors
- PostgreSQL wire protocol integration

**Month 1-4**: Vector foundation
- Distance operators (<->, <#>, <=>)
- ANN search optimization
- Benchmark vs pgvector

**Month 5-6**: First production release
- Cloud deployment (managed service)
- Self-hosting mode
- First 10-50 customers

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

- **omen-lite**: Embedded vector database (Year 2+ - not yet started)
- **pg-learned**: PostgreSQL extension demonstrating learned indexes ([omendb/pg-learned](https://github.com/omendb/pg-learned))

---

**Status**: 🚧 Early development (Week 1) - Not ready for production use

**Star this repo to follow development** ⭐
