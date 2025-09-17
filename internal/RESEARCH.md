# Research & Competitive Analysis (October 2025)

## Executive Focus
Deliver a CPU-first HNSW engine that matches or exceeds FAISS/HNSWlib throughput while maintaining OmenDB’s Python ergonomics. The path relies on SoA data layout, zero-copy ingestion, chunked/parallel graph construction, and lightweight compression.

## Active Techniques
| Area | Status | Notes |
|------|--------|-------|
| SoA vector storage | **In place** | Vectors mirrored in column-major layout; distance kernels migrating next. |
| SIMD kernels | **Requires update** | Specialized kernels exist; they must read SoA buffers directly to reduce cache misses. |
| Zero-copy ingestion | **Queued** | Buffer-protocol feed into Mojo after SoA kernels are live. |
| Chunked batch builder | **Designing** | DiskANN/Vamana-inspired chunk processing with reusable workspaces. |
| Parallel chunks | **Phase 2** | Use Mojo `parallelize` once chunked sequential path is verified. |
| Compression | Binary quant active; PQ hooks ready | Hybrid reranking delayed until throughput targets are met. |
| Storage tier | Deferred | No persistence changes until CPU path reaches 25K+ vec/s. |

## Competitive Landscape (Oct 2025)
| Engine | Insert vec/s | Notes |
|--------|--------------|-------|
| FAISS (CPU) | 50K+ | AVX512 SoA, batch builders, PQ. |
| HNSWlib | ~20K | C++ baseline; our initial target to match/beat. |
| Qdrant | ~20K | Rust, binary quant, production hardened. |
| Pinecone | ~15K | Cloud managed; GPU offload optional. |
| Weaviate | ~15K | Go core. |
| **OmenDB** | 0.3–1.0K (see STATUS) | Stable but far from target; SoA+zero-copy must move us into the 5–10× territory before final tuning. |

## Research Priorities
1. **SoA Distance Pipeline (Immediate)** – Load directly from column-major buffers; enable wide-SIMD loads. Expected uplift: 2–3× per distance call.
2. **Zero-Copy Ingestion** – Drop Python serialization and write straight into SoA buffers. Expected uplift: 5–10× for large batches.
3. **Chunked Graph Construction** – DiskANN-style range search with reusable workspaces and limited beams. Goal: ≥1.5K vec/s at 25K.
4. **Parallel Chunk Execution** – After sequential chunking is stable, share `BulkWorkspace` across threads with lock-free merge points. Target: closing the gap to 25K+ vec/s.
5. **Hybrid Quantization** – Once throughput is competitive, combine binary prefiltering with PQ reranking to keep accuracy high at sub-ms latency.
6. **Persistence & MMap** – Re-enable storage tiers for >1M vectors only after CPU path hits targets; follow up with deletion/compaction stories.

## References & Inspirations
- HNSW – Malkov & Yashunin (2016).
- DiskANN/Vamana – Microsoft Research (chunked batched insertion, PQ integration).
- FAISS CPU roadmap – SoA + blocking strategies (2024+ releases).
- Qdrant whitepapers – Binary quantization w/ in-memory workspaces.
- Mojo roadmap – SIMD alignment guidelines (internal MAX docs).

## Benchmarks to Track
- `test_binary_quantization_quick.py` – sanity check.
- `test_simd_performance.py` – throughput vs. distance pipeline improvements.
- `benchmark_competitive.py` (WIP) – cross-engine comparisons once SoA/zero-copy landing.
- External: SIFT1M / DEEP1B subsets for public comparisons.

## Documentation Discipline
- Keep `internal/ARCHITECTURE.md` aligned with these milestones.
- Record every measurable improvement in `internal/STATUS.md` with the exact test command and vector size.
- Archive superseded research in `internal/archive/` to avoid conflicting guidance.
