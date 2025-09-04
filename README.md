# OmenDB Core Monorepo

**Private monorepo for dual database engine development**

This repository contains two complementary database engines designed for AI-native and cloud-native applications.

## Repository Structure

```
omendb/               # OmenDB product suite
├── engine/           # Mojo vector database (DiskANN algorithm)
├── server/           # Rust HTTP/gRPC service (potentially outdated)
└── web/              # SolidJS frontend (needs content updates)

zendb/                # Rust hybrid database
├── src/              # Core database implementation
├── tests/            # 61/70 tests passing
└── docs/             # Architecture & development docs

internal/             # Internal documentation
├── research/         # Performance & architecture research
├── strategy/         # Business planning (private)
├── decisions/        # Architecture decisions
└── archive/          # Historical investigations

agent-contexts/       # AI assistant patterns (git submodule)
```

## Database Engines

### OmenDB Engine (Vector Database)
- **Language**: Mojo with Python bindings
- **Algorithm**: DiskANN/Vamana for billion-scale vectors
- **Memory**: 288 bytes/vector (PQ compression fixed)
- **Status**: ⚠️ Performance bottleneck at 25K+ vectors
- **Known Issues**: Global singleton VectorStore, FFI overhead

### ZenDB (Hybrid SQL Database)
- **Language**: Rust
- **Features**: ACID transactions, MVCC, WAL, compression, multi-writer
- **Architecture**: B+Tree with page-level locking
- **Status**: ✅ 87% tests passing (61/70)
- **Next Steps**: Fix cache eviction, add SQL layer

## Quick Start

```bash
# Clone with submodules
git clone --recursive git@github.com:omendb/core.git
cd core

# OmenDB Engine (requires Pixi)
cd omendb/engine
pixi install
pixi run benchmark-quick         # Test with 1K-10K vectors

# ZenDB (requires Rust)
cd zendb
cargo test                        # 61/70 tests passing
cargo run --example basic_usage

# OmenDB Server (optional, may be outdated)
cd omendb/server
cargo build --release

# Web Interface (optional)
cd omendb/web
npm install && npm run dev
```

## Development Priorities

### Immediate Focus
1. **OmenDB**: Debug 25K+ vector bottleneck in buffer flush
2. **ZenDB**: Fix remaining 9 test failures
3. **Shared**: Build unified vector benchmarks

### Strategic Goals
- **OmenDB**: Scale to 1M+ vectors with memory-mapped storage
- **ZenDB**: Add SQL layer for PostgreSQL compatibility
- **Integration**: Cross-engine vector format standardization

## Documentation

- **[CLAUDE.md](CLAUDE.md)** - AI assistant context and patterns
- **[DEVELOPMENT.md](DEVELOPMENT.md)** - Development workflows
- **[internal/](internal/)** - Architecture decisions and research
- **[agent-contexts/](agent-contexts/)** - Shared AI patterns (submodule)

## Contributing

This is a private repository. Development is coordinated through:
- AI agent workflows (see agent-contexts/)
- Cross-engine benchmarking
- Shared optimization patterns

---

*Private development repository - January 2025*