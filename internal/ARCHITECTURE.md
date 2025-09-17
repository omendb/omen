# OmenDB Technical Architecture (CPU-First)
*Last Updated: October 2025*

## 1. Platform Assumptions
- **Language stack:** Mojo core in `omendb/engine`, thin Python bindings (`python/omendb/native.so`).
- **Execution model:** Single-process HNSW graph with optional binary/PQ compression. Mojo currently provides *CPU* SIMD only—GPU features remain out of scope until the toolchain exposes them.
- **Design principle:** Keep correctness first, then layer performance features (SoA layout → zero-copy ingestion → batched/parallel construction → persistence).

## 2. Storage Layout
| Layer | Purpose | Current Status | Next Steps |
|-------|---------|----------------|------------|
| AoS buffer (`self.vectors`) | Compatibility staging (single inserts, legacy paths) | Allocated but no longer the hot path | Retain until all APIs speak SoA directly |
| SoA buffer (`self.vectors_soa`, stride = capacity) | SIMD-friendly column-major storage | **Active.** Every insert writes here; distance kernels will read from it | Extend distance helpers to load directly from SoA (in-flight) |
| Workspace scratch (`BulkWorkspace`) | Chunk-local buffers, candidate heaps, visited arrays | Prototype using shared heaps | Finalize chunk pipeline + per-thread workspaces |

## 3. Ingestion Pipeline
1. **Python binding** receives NumPy arrays or Python lists.
2. **Zero-copy target (planned):** If the input exposes a buffer interface matching `(n_vectors, dimension)`, hand Mojo an `UnsafePointer[Float32]` and copy straight into SoA storage. Fallback to AoS copy when necessary.
3. **Quantization hooks** (binary/PQ) operate on the SoA data, storing compressed artifacts alongside the raw floats.

## 4. Graph Construction
- **Node pool:** Pre-allocates nodes with fixed-size connection arrays (`NodePool`).
- **Insert path:**
  1. Allocate IDs & vectors (AoS + SoA).
  2. Precompute binary codes once; pass `Optional[BinaryQuantizedVector]` into `_insert_node`.
  3. `_insert_node` walks the graph using `_search_layer_for_M_neighbors`, now backed by reusable heap workspaces.
- **Bulk insertion:**
  - Current implementation reuses the single-node path for correctness. Throughput at 25 K vectors is ~294 vec/s.
  - Planned upgrade: chunk-based builder with `BulkWorkspace` (chunk queries, SoA distances, candidate/cache reuse) followed by optional parallel chunk execution.

## 5. Distance Pipeline
- **Current implementation:** `_simple_euclidean_distance(node_idx, query_ptr)` still reloads AoS data. The SoA helpers (`write_vector_soa`, `_load_vector_component`) are in place—the next sprint moves the kernels to consume SoA directly.
- **Binary distance:** `BinaryQuantizedVector` stores bit-packed codes; `binary_distance` handles fast XOR+popcount. Keep SoA-aligned binary buffers to avoid cache misses.
- **Adaptive search (`ef`, multi-starts):** Scales `ef` and seeding based on index size (capped at 208 for large graphs). Workspace heaps (`FastMinHeap/MaxHeap`) reset per query to avoid reallocation.

## 6. Query Execution
1. Select search backend (flat buffer for small sets, HNSW for larger ones).
2. Use the shared heap workspace to evaluate distances (SoA once kernels are migrated).
3. Return string IDs by mapping numeric IDs via sparse id maps.
4. Roadmap: multi-query batching (matrix–matrix distance via SoA) and optional path caching for clustered workloads.

## 7. Compression & Reranking
- **Binary quantization:** Enabled by default; ~32× memory reduction and ~0.74 ms search in quick tests.
- **Product Quantization:** Training hooks exist (`PQCompressor`), but reranking is disabled until throughput milestones are met.
- **Integration plan:** Binary filter → PQ rerank → optional float rerank (top-K).

## 8. Parallelism Strategy (Future)
1. Solidify chunked sequential builder (SoA distances, workspace reuse).
2. Use Mojo’s `parallelize` to process independent chunks with thread-local `BulkWorkspace` instances.
3. Merge graph updates via lock-free mechanisms or coarse-grained locks on disjoint node ranges.
4. Provide thread-local entry points to reduce contention.

## 9. Persistence & Storage Tiers (Deferred)
- MMAP-backed SoA files and log-structured metadata layers remain on hold until in-memory throughput hits target (~25 K vec/s inserts).
- Plan includes: periodic chunk flush, write-ahead logging, and background compaction of quantized vectors.

## 10. Current Metrics (October 2025)
| Batch Size | Dimension | Throughput | Notes |
|------------|-----------|------------|-------|
| 2,000 | 768D | **~763 vec/s** | `test_binary_quantization_quick.py` (binary quant on) |
| 1,000 | 768D | **~1,052 vec/s** | `test_simd_performance.py` |
| 5,000 | 768D | **~450 vec/s** | — |
| 10,000 | 768D | **~338 vec/s** | — |
| 25,000 | 768D | **~294 vec/s** | Sequential fallback; stable but not yet competitive |

## 11. Roadmap Snapshot
1. **SoA distance kernels:** Replace AoS reads with SoA loads, add node↔node helper. Validate quick + SIMD tests.
2. **Zero-copy ingestion:** Buffer protocol → Mojo pointers → direct SoA writes. Measure end-to-end ingestion speed.
3. **Chunked builder:** Introduce `BulkWorkspace`, chunk-level distance evaluation, and workspace reuse. Target ≥1.5 K vec/s at 25 K.
4. **Parallel chunk execution:** Spread chunks across workers using thread-local workspaces; ensure deterministic graph structure.
5. **Persistence + PQ:** Reintroduce storage/mmap layers and PQ reranking once CPU path is within striking distance of 25 K vec/s.

Keep this document aligned with the actual implementation plan so future agents and contributors share the same roadmap.

## Caveats & Mitigations
- **SoA distance kernels pending**: verify functional parity between AoS and SoA (unit tests comparing outputs) before switching production paths.
- **Binary quantization parity**: once SoA distances land, cross-validate binary distance outputs to ensure compression flow stays correct.
- **Chunked builder design**: finalize heuristics for neighbor selection, pruning, and merging before coding.
- **Zero-copy ingestion safeguards**: enforce buffer shape/stride checks and fallback paths when inputs are not C-contiguous Float32.
- **Parallel chunk execution**: revisit WIP implementation with a detailed concurrency plan prior to re-enabling.
