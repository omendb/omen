# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# OmenDB Core Monorepo - AI Agent Context
*Token-efficient navigation - include first*

## Current Focus: OmenDB Vector Database 🎯
**Primary Project**: OmenDB - High-performance vector database in Mojo
**Status**: PQ compression working (288 bytes/vector), debugging 25K+ vector bottleneck
**Architecture**: DiskANN/Vamana algorithm, memory-mapped storage, Python/C bindings
**Secondary Project**: ZenDB - Experimental multimodal database (on hold)

## Quick Facts
- **OmenDB**: High-performance vector engine (Mojo, DiskANN algorithm)
- **ZenDB**: Hybrid database with SQL + Vectors + Time-travel (Rust, MVCC/WAL)
- **Server**: Rust HTTP/gRPC server (⚠️ potentially outdated)
- **Web**: SolidJS frontend (⚠️ content outdated but functional)
- **Shared**: Benchmarks, vector formats, agent patterns
- **Status**: OmenDB needs scale fixes, ZenDB ready for optimization

## Repository Structure (Updated)
```
/omendb/             # Main vector database project
├── engine/          # Mojo vector engine (focus here)
├── server/          # Rust HTTP/gRPC wrapper (may be outdated)
└── web/             # Marketing site (needs content update)

/zendb/              # Experimental multimodal DB (on hold)

/internal/           # Internal knowledge base
├── patterns/        # Extracted patterns (STORAGE, CONCURRENCY)
├── research/        # Technical research
├── archived/        # Historical investigations
└── strategy/        # Business planning

/external/           # External references
└── agent-contexts/  # AI patterns submodule
```

## Development Commands

### OmenDB Engine (Mojo)
```bash
cd omendb/engine
pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib
pixi run benchmark-quick    # 1K-10K vectors
pixi run benchmark-standard # 1K-100K vectors
make test-core             # Core functionality
make test-compression      # PQ compression tests
```

### OmenDB Server (Rust HTTP/gRPC)
```bash
cd omendb/server
cargo build                # Build server
cargo test                 # Run server tests
cargo run -- --config config.toml  # Run server
```

### OmenDB Web (SolidJS)
```bash
cd omendb/web
npm install                # Install dependencies
npm run dev               # Development server
npm run build             # Production build
npm run typecheck         # Type checking
```

### ZenDB (Rust) 
```bash
cd zendb
cargo test                 # 61/70 tests passing
cargo bench               # Performance benchmarks
cargo run --example basic_usage
```

### Cross-Engine Operations
```bash
# From repository root
make benchmark-both        # Compare both engines
make test-shared          # Shared component tests
```

## Error → Fix Mappings

| Error/Issue | Fix | Location |
|------------|-----|----------|
| 25K vector bottleneck | Check buffer flush, increase memory pool | omendb/engine/omendb/native.mojo:1850 |
| Global singleton crash | Clear DB between tests, unique IDs | omendb/engine/omendb/native.mojo:78 |
| Dict overhead (8KB/entry) | Use SparseMap instead | See patterns/STORAGE_PATTERNS.md |
| FFI overhead | Batch operations, not individual | Use add_batch() not add() |

## Decision Trees

```
IF debugging_bottleneck:
    → Check internal/patterns/CONCURRENCY_PATTERNS.md
    → Focus on buffer flush mechanism
ELIF adding_feature:
    → Check internal/patterns/STORAGE_PATTERNS.md
    → Follow existing patterns in engine/
ELIF fixing_error:
    → Check error table above
    → Run specific test: pixi run test-{component}
```

### Critical Pattern - Global Singleton
```python
# ⚠️ All DB() instances share same VectorStore
db1 = DB()
db1.add_batch(vectors, ids=["vec_0", ...])

db2 = DB()  # Same database, not new instance!
# ❌ WRONG: Reusing IDs causes segfault
db2.add_batch(vectors, ids=["vec_0", ...])  

# ✅ CORRECT: Clear between tests
db2.clear()
db2.add_batch(vectors, ids=["vec_1", ...])
```

### Mojo Stdlib Memory Issues
```mojo
# ❌ NEVER use these - massive overhead
Dict[String, Int]  # 8KB per entry!
List[String]       # 5KB per item!

# ✅ Use custom implementations  
SparseMap         # 180x better than Dict
Fixed arrays      # Predictable memory
```

### FFI Overhead Pattern
```python
db.add_batch(vectors)       # ✅ Single FFI: 1.5KB/vector  
for v in vectors: db.add(v) # ❌ Many FFI: 8.3KB/vector
```

## Key Architecture Files

### OmenDB Core
```
omendb/engine/omendb/native.mojo:1850-2000   # VectorStore core
omendb/engine/omendb/native.mojo:500-700     # Buffer management
omendb/engine/omendb/diskann.mojo:200-300    # Search hot path
omendb/engine/pixi.toml                      # Mojo environment
omendb/engine/Makefile                       # 300+ line build system
omendb/server/Cargo.toml                     # Server dependencies
omendb/web/package.json                      # Web dependencies
```

### ZenDB Core  
```
zendb/Cargo.toml                             # Rust dependencies
zendb/src/storage/                           # MVCC, WAL, compression
zendb/src/sql/                               # SQL parser & execution
zendb/tests/                                 # 61/70 tests passing
```

### Internal & Shared Systems
```
internal/INDEX.md                            # Documentation navigation
internal/WORKFLOW.md                         # Development processes
internal/strategy/                           # Business planning
internal/research/                           # Technical research
shared/benchmarks/                           # Cross-engine testing
agent-contexts/                              # Git submodule (AI contexts)
```

## Development Workflow

### Key Principles
1. **AI Agent Coordination**: Use `/agent-contexts/` for shared patterns
2. **Cross-Engine Validation**: Benchmark both engines for algorithm validation  
3. **Monorepo Benefits**: Shared components, unified documentation
4. **Scale-First**: OmenDB targeting 1M+ vectors, ZenDB for SQL+vector workloads

### Common Tasks
- **OmenDB scale issues**: Review buffer flush in `native.mojo:1850-2000`
- **ZenDB test failures**: Check failing tests with `cargo test -- --nocapture`
- **Cross-engine benchmarks**: Use `shared/benchmarks/` for comparisons
- **Mojo debugging**: `mojo debug native.mojo` + check `docs/MOJO_PATTERNS.md`
- **Documentation updates**: Follow `/docs/INDEX.md` hierarchy

### Build System Notes
- **OmenDB**: Pixi environment (conda-based) with comprehensive Makefile
- **ZenDB**: Standard Cargo with extensive test suite
- **Dependencies**: Mojo toolchain for OmenDB, Rust stable for ZenDB

## Scale & Performance Context

### Current Metrics
| Engine | Scale | Memory/Vector | Status |
|--------|-------|---------------|---------|
| OmenDB | 25K vectors | 288 bytes (PQ fixed) | Performance bottleneck |  
| ZenDB | SQL+Vector hybrid | Optimized storage | 61/70 tests passing |
| Industry Standard | 1B+ vectors | 30-100 bytes | Target benchmark |

### Integration Points
- Shared vector serialization formats in `/shared/vector-formats/`
- Cross-engine benchmarking for algorithm validation
- Unified AI agent development patterns

---
*Optimized for dual-engine development - see `/docs/` for technical deep-dives*