# SIMD & GPU Cleanup Status (October 2025)

## What Changed This Session
- Removed the fictional GPU stack (`omendb/engine/omendb/gpu/` plus validation scripts) and replaced it with documented CPU stubs in `omendb/engine/omendb/core/gpu_context.mojo`.
- Replaced the broken `advanced_simd.mojo` with lean, compiler-friendly helpers that defer to the specialized kernels, and updated `hnsw.mojo` to describe the actual CPU path we rely on.
- Normalized contributor docs (`AGENTS.md`, `CLAUDE.md`) so guidance reflects the CPU-first strategy and no longer references the old advanced_simd disablement flow.

## Current Performance Signals
- `pixi run python test_binary_quantization_quick.py`
  - Insert throughput: **~766 vec/s** for 2K×768D batch (binary quant enabled)
  - Search latency: **~0.74 ms** (10 results) with ~148K estimated distances/s
  - Accuracy check passes using dict-based result distance fields.
- `pixi run python test_simd_performance.py`
  - Confirms SIMD kernels beat the generic path for 128–768D (≈1.7× uplift).
  - Completes the 25K vector phase via the sequential fallback at ~292 vec/s; still need a faster batched path for competitive throughput.
- `pixi run mojo build ...` now compiles cleanly with only expected warnings about disabled debug flags.

## Outstanding Issues / Next Targets
1. **Bulk Graph Throughput** – Sequential fallback keeps the 25K batch stable (~292 vec/s) but far below targets; need a safe yet faster batched construction strategy.
2. **Throughput Gap** – Current CPU throughput (≈743 vec/s) is still well below the 2,143 vec/s measured in September; focus next on batching distance evaluations and pruning logging overhead in the hot paths.
3. **Doc Updates** – Once we validate the fixes above, refresh the broader status docs (`internal/CURRENT_CAPABILITIES*.md`) so they no longer claim `advanced_simd.mojo` is uncompileable or that GPU pathways exist.

Use this note as the seed for CLAUDE/tech-spec updates so future agents start with the real CPU-first baseline.
