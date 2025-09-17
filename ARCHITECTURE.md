# OmenDB Architecture (CPU-First, October 2025)

## Overview
- **Mojo core + Python bindings**: all hot paths run in Mojo with SIMD; Python only marshals batched requests.
- **Execution model**: HNSW graph with binary quantization; CPU-only until Mojo exposes GPU APIs.
- **Design mantra**: Stabilise correctness → adopt SoA storage → remove FFI/heap overhead → add chunked & parallel builders → reintroduce persistence.

## Current Layout
| Component | Description | Status |
|-----------|-------------|--------|
| `omendb/engine/omendb/algorithms/hnsw.mojo` | HNSW implementation, bulk ingest path, SoA buffers, heap workspaces | Active |
| `omendb/engine/omendb/utils/specialized_kernels.mojo` | Dimension-specific SIMD kernels | Needs SoA-aware loads |
| `omendb/engine/omendb/compression/binary.mojo` | Binary quantization (32× compression) | Active |
| `omendb/engine/omendb/core/gpu_context.mojo` | CPU stub (no GPU) | Active |

## Storage & Ingestion
- **AoS buffer (`self.vectors`)**: staging for legacy APIs; no longer the fast path.
- **SoA buffer (`self.vectors_soa`)**: column-major storage sized by `vector_stride`. Every insert writes here.
- **Zero-copy plan**: After SoA kernels land, the Python binding will accept NumPy buffers and copy directly to SoA, bypassing Python lists.

## Graph Construction Roadmap
1. **SoA distance kernels** – Refactor `_simple_euclidean_distance`, `_search_layer_for_M_neighbors`, and related helpers to read SoA data directly, eliminating AoS reloads.
2. **Zero-copy ingestion** – Buffer-protocol path from Python → Mojo (`UnsafePointer[Float32]`), writing straight into SoA storage.
3. **Chunked builder** – Introduce `BulkWorkspace`, process vectors in SoA-backed chunks (1–4K) with limited hill climbs and reusable heaps.
4. **Parallel chunks** – Use Mojo `parallelize` to run independent chunks with thread-local workspaces; merge graph updates safely.
5. **Persistence & PQ** – Once throughput targets (~25K vec/s insertion) are hit, re-enable storage tiers and PQ reranking.

## Current Metrics (Oct 2025)
- Binary quick test (2K × 768D): ~763 vec/s, 0.74 ms search.
- SIMD suite: ~1 052 / 450 / 338 / 294 vec/s at 1K / 5K / 10K / 25K batches.
- Stability: No crashes, but 25K throughput still limited by sequential fallback.

## Next Actions (Summary)
1. Implement SoA-aware distance helpers and update all call sites.
2. Add zero-copy ingestion hook in the Python binding.
3. Build the chunked builder and evaluate throughput at 25K vectors.
4. Extend the plan to parallel chunks, persistence, and PQ once above foundations are solid.

Refer to `internal/ARCHITECTURE.md` for the detailed blueprint and daily progress.
