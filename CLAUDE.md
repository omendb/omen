# Repository Guidelines

## Documentation Organization
**Start here for context**:
1. **Root summaries**: `ARCHITECTURE.md`, `RESEARCH.md`, `STATUS.md` - quick overviews
2. **Detailed specs**: `internal/ARCHITECTURE.md`, `internal/RESEARCH.md`, `internal/STATUS.md` - full technical details
3. **Single source of truth**: Each concept documented in exactly one place
4. **Archive old docs**: Move to `internal/archive/YYYY-MM-DD/` when superseded
5. **Update STATUS.md**: After each performance test or major change

## Project Structure
- `omendb/engine` - Mojo vector core with Python bindings under `python/omendb/`
- `zendb/` - Rust hybrid database (`src/`, `tests/`, `docs/`)
- `internal/` - Living documentation (ARCHITECTURE, RESEARCH, STATUS) plus dated archives
- `external/agent-contexts/` - Decision trees for assistant runs
- `omendb/server` and `omendb/web` - Secondary, update only when engine APIs change

## Current Performance (October 2025)
- **Throughput**: 670-1,052 vec/s (varies by batch size)
- **Target**: 25,000+ vec/s (after SoA migration + zero-copy FFI)
- **Bottlenecks**: SIMD compilation broken, FFI overhead 50%, bulk insertion crashes at 25K
- **Next steps**: SoA distance kernels → zero-copy ingestion → chunked builder

## Build & Test Commands
```bash
# Build engine
cd omendb/engine
pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib -I omendb

# Quick tests (current: 670-1,052 vec/s)
pixi run python test_binary_quantization_quick.py
pixi run python test_simd_performance.py

# Full benchmarks
pixi run benchmark-quick
pixi run python benchmark_competitive.py

# ZenDB tests
cd zendb && cargo test
```

## Coding Standards
- **Mojo**: 4-space indent, `snake_case` functions, `UpperCamelCase` types, SoA-friendly bulk ops
- **Python**: PEP 8 with type hints, benchmarks as `benchmark_*.py`
- **Rust**: Follow `rustfmt`, run `cargo fmt --check`
- **No GPU code**: Mojo has no GPU support - delete any GPU code found

## Testing Requirements
- Record before/after performance metrics
- Use deterministic seeds for reproducibility
- Update STATUS.md with results
- Include command lines used

## Commit Guidelines
- Format: `type: concise summary` (feat, fix, docs, BREAKTHROUGH)
- Keep under 72 chars
- Branch names: `fix/soa-distance-kernels` style
- PRs must include benchmark deltas

## Key Technical Decisions
1. **CPU-first strategy** - Mojo has no GPU support (discovered Oct 2025)
2. **HNSW+ algorithm** - M=16, ef_construction=200, ef_search=50
3. **Binary quantization** - 32x compression, minimal recall loss
4. **SoA storage** - Structure of Arrays for SIMD efficiency

## Immediate Priorities
1. **Fix SIMD** - Migrate from broken `advanced_simd.mojo` to `specialized_kernels.mojo`
2. **Zero-copy FFI** - Implement NumPy buffer protocol
3. **Chunked builder** - DiskANN-style batch processing
4. **Parallel chunks** - Multi-core speedup after sequential stabilizes

---
*For detailed technical specs see `internal/ARCHITECTURE.md`*
*For SOTA research see `internal/RESEARCH.md`*
*For current metrics see `internal/STATUS.md`*