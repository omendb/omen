# STATUS

**Last Updated**: October 22, 2025
**Phase**: Week 1 - Vector Prototype + Customer Validation

---

## Current State

**Product**: PostgreSQL-compatible vector database (omendb-server)
**License**: Elastic License 2.0
**Status**: Early development (Week 1)

---

## What's Working ✅

- Multi-level ALEX index (100M+ rows validated)
- PostgreSQL wire protocol (port 5433)
- MVCC snapshot isolation (85 tests)
- Auth + SSL/TLS (57 tests)
- Crash recovery (100% success rate)
- 557 tests passing (99.8%)

---

## Current Focus (Week 1)

**ALEX Vector Prototype**:
- Research pgvector implementation
- Design vector(N) data type
- Prototype ALEX for 1536-dim vectors
- Benchmark: memory, latency, index build time

**Customer Validation**:
- Find 50 pgvector users
- Cold outreach emails
- Schedule 3-5 discovery calls

---

## Strategic Decisions (Finalized)

1. **License**: Elastic License 2.0 for both omendb-server + omen-lite
2. **Pricing**: Hybrid (Free, $29, $99/month + Enterprise)
3. **Year 1 Focus**: omendb-server ONLY (omen-lite Year 2+)
4. **Customers**: AI startups (70%), Enterprise (30%)
5. **Positioning**: "PostgreSQL-compatible vector database"

---

## Next Milestones

- Week 1: ALEX vector prototype + 5 customer calls
- Month 2-4: Vector foundation (data types, operators, indexing)
- Month 5-6: Production release (cloud + self-hosting, first 10 customers)
- Year 1 Goal: $10K MRR (50 customers × $200 avg)

---

## Blockers

None currently

