# OmenDB Core Development Monorepo

**Private development repository for OmenDB database engines**

This monorepo contains the core development for multiple database engines designed for AI-native applications.

## Structure

```
├── engines/           # Database engine implementations  
│   ├── omendb/       # Vector database (Mojo + DiskANN)
│   └── zendb/        # Hybrid database (Rust + SQL + Vectors)
├── docs/             # Internal documentation & research
├── agent-contexts/   # AI agent configuration patterns (submodule)
├── shared/           # Shared components between engines
│   ├── benchmarks/   # Cross-engine performance testing
│   ├── vector-formats/ # Common vector serialization
│   └── agent-patterns/ # Shared AI agent patterns
└── experiments/      # R&D and prototypes
```

## Database Engines

### OmenDB (Vector Database)
- **Language**: Mojo
- **Algorithm**: DiskANN/Vamana  
- **Focus**: High-performance vector search
- **Status**: PQ compression working, 288 bytes/vector achieved

### ZenDB (Hybrid Database) 
- **Language**: Rust
- **Features**: SQL + Vectors + Time-travel + Multi-writer ACID
- **Focus**: PostgreSQL-compatible hybrid database
- **Status**: Storage engine complete, adding vector support

## Development Workflow

This monorepo enables AI agent coordination between both database projects:
- Shared benchmarking and performance validation
- Cross-pollination of algorithms and optimizations  
- Unified documentation and agent contexts
- Parallel development with shared components

## Getting Started

Each engine has its own build system and requirements:

```bash
# OmenDB (Mojo)
cd engines/omendb
pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib

# ZenDB (Rust)
cd engines/zendb  
cargo test
cargo run --example basic_usage
```

## Documentation

- `docs/` - Internal technical documentation
- `engines/*/README.md` - Engine-specific documentation
- `agent-contexts/` - AI agent configuration patterns
- `docs/technical/` - Architecture and design decisions

---

*This is a private development repository. Public releases will be extracted to separate repositories when ready.*