# Archived Algorithms

## RoarGraph

**Archived Date**: 2025-07-28
**Reason**: Performance benchmarks showed RoarGraph was 0.4-0.6x slower than HNSW for pure vector search tasks.

### Background
RoarGraph was the original algorithm used in OmenDB but benchmarking revealed:
- Construction speed: 15-20K vec/s (vs HNSW 50K+ vec/s)
- Query latency: 0.14ms @50K vectors (vs HNSW <0.03ms)
- Not competitive with industry-standard HNSW implementations

### Future Use
RoarGraph may still be valuable for:
- Cross-modal search (its original design goal)
- Research into alternative graph structures
- Specialized use cases where its unique properties provide advantages

### Migration
OmenDB has switched to HNSW as the default algorithm for competitive performance with Faiss and other vector databases.