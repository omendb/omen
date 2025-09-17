# OmenDB Core Monorepo

Private workspace for the OmenDB vector engine (Mojo + Python) and the ZenDB hybrid database (Rust).

## Repository Layout
```
omendb/
├── engine/        # Mojo vector engine + Python bindings + tests
├── server/        # Rust API surface (maintained only when APIs shift)
└── web/           # SolidJS UI shell (update alongside server changes)

zendb/             # Rust hybrid database (src/, tests/, docs/)

internal/          # Living documentation (ARCHITECTURE, RESEARCH, STATUS)
external/agent-contexts/
                    # AI assistant decision trees required for automation
```

## OmenDB Engine (Mojo)
- **Design**: CPU-first HNSW with binary quantization, SoA storage, reusable workspaces.
- **Current throughput** (768D): ~1,052 vec/s (1K batch), ~763 vec/s (2K), ~294 vec/s (25K sequential fallback).
- **Active roadmap**: migrate distance helpers to SoA, add zero-copy ingestion from NumPy buffers, introduce chunked bulk builder, then parallelize chunk execution.
- **Verification**: `pixi run mojo build omendb/native.mojo ...`, `pixi run python test_binary_quantization_quick.py`, `pixi run python test_simd_performance.py`, `pixi run benchmark-quick`.

## ZenDB (Rust)
- **Focus**: Hybrid row+column store with ACID guarantees.
- **Status**: Requires fresh test run (`cargo test`); treat docs/tests in `zendb/` as the source of truth.
- **Next steps**: Address outstanding test failures, continue API surface cleanup as part of the broader roadmap.

## Quick Start
```bash
# Clone with submodules
git clone --recursive git@github.com:omendb/core.git
cd core

# Build & smoke-test the Mojo engine
cd omendb/engine
pixi install
pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib -I omendb
PYTHONPATH=python pixi run python -c "import omendb"
pixi run python test_binary_quantization_quick.py

# Run the Rust suite
cd ../../zendb
cargo test
```

## Current Engineering Focus
1. **SoA distance kernels** – make every distance helper load directly from column-major storage and validate parity with existing AoS paths.
2. **Zero-copy ingestion** – accept NumPy buffer protocol inputs and write straight to SoA buffers with robust fallbacks.
3. **Chunked bulk builder** – design chunked ingestion with reusable workspaces before enabling parallel execution.
4. **Parallel chunk processing** – once sequential chunking is solid, introduce thread-local workspaces and deterministic merges.

## Documentation
- `AGENTS.md` – contributor and agent quick-start guide.
- `internal/ARCHITECTURE.md`, `internal/RESEARCH.md`, `internal/STATUS.md` – living design, research, and status references (read these before major changes).
- `CLAUDE.md` – auxiliary agent instructions.
- Archive legacy docs under `internal/archive/` when superseded, keeping the top-level index aligned with the CPU-first plan.

_Last updated: October 2025_
