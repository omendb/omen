# Repository Guidelines

## Documentation Organization (AI-Optimized)
**🚨 MANDATORY: Load these first**:
1. **`internal/HNSW_INVARIANTS.md`** - ⛔ What MUST NEVER be violated
2. **`internal/STATUS.md`** - 📊 Current performance (867 vec/s, 95.5% recall)
3. **`internal/COMPETITIVE_ANALYSIS_2025.md`** - 🎯 Market targets (20K+ vec/s needed)
4. **`internal/AI_AGENT_CONTEXT.md`** - 🤖 Complete development guide

**Reference docs** (load when needed):
- `internal/ARCHITECTURE.md`, `internal/HNSW_DEVELOPMENT_GUIDE.md`, `internal/RESEARCH.md`
- **Single source of truth**: Each concept documented exactly once
- **Update STATUS.md**: After every performance test or optimization attempt

## Project Structure
- `omendb/engine` - Mojo vector core with Python bindings under `python/omendb/`
- `zendb/` - Rust hybrid database (`src/`, `tests/`, `docs/`)
- `internal/` - Living documentation (ARCHITECTURE, RESEARCH, STATUS) plus dated archives
- `external/agent-contexts/` - Decision trees for assistant runs
- `omendb/server` and `omendb/web` - Secondary, update only when engine APIs change

## Current Performance (September 2025)
- **Insertion**: 867 vec/s with 95.5% recall@10 (stable, quality excellent)
- **Proven Capability**: 27,604 vec/s achieved (but 1% recall due to broken navigation)
- **Competitive Target**: 20,000+ vec/s with 95% recall (matches Qdrant/Weaviate)
- **Core Challenge**: Maintain HNSW invariants while optimizing performance
- **Next steps**: Fix bulk construction → segment parallelism → SIMD → zero-copy FFI

## CRITICAL: HNSW Invariants (NEVER Violate)
- **Hierarchical navigation**: MUST navigate from entry_point down through layers
- **Bidirectional connections**: MUST maintain A↔B connections
- **Progressive construction**: Graph MUST be valid after each insertion
- **Quality requirement**: Recall@10 MUST be ≥95%

## Build & Test Commands
```bash
# Build engine
cd omendb/engine
pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib -I omendb

# Current performance validation (867 vec/s, 95.5% recall)
pixi run python benchmarks/final_validation.py

# Quick tests
pixi run python test_binary_quantization_quick.py

# Debug broken SIMD (compiler issues)
pixi run python test_simd_performance.py

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