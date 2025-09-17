# Research & Competitive Landscape (October 2025)

## Focus
Build a Mojo-based CPU engine that matches FAISS/HNSWlib insertion/search throughput. The path: SoA storage → zero-copy ingestion → chunked/parallel construction → persistence & compression.

## Active Initiatives
- **SoA distance kernels** – Immediate refactor so all distance operations consume column-major buffers.
- **Zero-copy ingestion** – Buffer-protocol feed into Mojo; fall back to AoS copy only when necessary.
- **Chunked bulk builder** – DiskANN/Vamana-inspired batching with reusable workspaces, limited hill climbs, and deterministic graph merges.
- **Parallel chunk execution** – Mojo `parallelize` once sequential chunking is verified; use thread-local workspaces to avoid contention.
- **Compression strategy** – Binary quant active; PQ reranking staged after throughput goals are met.

## Benchmark Targets (Oct 2025)
| Engine | Insert vec/s | Notes |
|--------|--------------|-------|
| FAISS (CPU) | 50K+ | SoA + blocking + PQ |
| HNSWlib | ~20K | Baseline to match/beat |
| Qdrant | ~20K | Rust, binary quant |
| Pinecone | ~15K | Managed cloud |
| Weaviate | ~15K | Go |
| **OmenDB** | 0.3–1.0K | Stable but far from target; chunked builder + SoA kernels are next |

## Priority Queue
1. SoA-aware distance helpers (`distance_node_to_query`, `distance_between_nodes`).
2. Zero-copy ingestion to eliminate Python serialization.
3. Chunked builder with `BulkWorkspace` and limited hill climbs.
4. Parallel chunk processing for multi-core speedup.
5. PQ reranking + persistence once CPU path is competitive.

## References & Benchmarks
- HNSWlib / DiskANN publications for batched construction patterns.
- FAISS CPU roadmap for SoA best practices.
- Qdrant blog posts on binary quantization.
- ANN-Benchmarks / VectorDBBench for cross-engine validation.

See `internal/RESEARCH.md` for detailed tactics and citations.
