# OmenDB Core Development Monorepo

**Private development repository for OmenDB database engines**

This monorepo contains the core development for multiple database engines designed for AI-native applications.

## Structure

```
├── omendb/           # OmenDB product suite
│   ├── engine/       # Vector database (Mojo + DiskANN)
│   ├── server/       # HTTP/gRPC service (Rust)
│   └── web/          # Marketing site & docs portal (SolidJS)
├── zendb/            # Hybrid database (Rust + SQL + Vectors)
├── internal/         # Internal documentation & research
│   ├── research/     # Performance & architecture research
│   ├── strategy/     # Business & product strategy
│   └── archive/      # Historical investigations
├── shared/           # Cross-product components
│   └── benchmarks/   # Cross-engine performance testing
└── agent-contexts/   # AI agent configuration patterns (submodule)
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
# OmenDB Engine (Mojo)
cd omendb/engine
pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib

# OmenDB Server (Rust)
cd omendb/server
cargo build
cargo run -- --config config.toml

# OmenDB Web (SolidJS)
cd omendb/web
npm install && npm run dev

# ZenDB (Rust)
cd zendb
cargo test
cargo run --example basic_usage
```

## Documentation

- `internal/` - Internal strategy, research, and architecture decisions
- `omendb/*/README.md` - OmenDB component documentation
- `zendb/README.md` - ZenDB documentation
- `agent-contexts/` - AI agent configuration patterns

---

*This is a private development repository. Public releases will be extracted to separate repositories when ready.*