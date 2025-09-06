# Development Guide

This guide covers development workflows for the OmenDB Core monorepo.

## Repository Structure

```
omendb/           # OmenDB product suite
├── engine/       # Mojo vector database  
├── server/       # Rust HTTP/gRPC service
└── web/          # SolidJS frontend
zendb/            # Rust hybrid database
internal/         # Strategy & research docs
agent-contexts/   # AI configuration (submodule)
```

## Prerequisites

### For OmenDB Engine (Mojo)
- Pixi package manager
- Mojo toolchain (installed via Pixi)
- Python 3.11+ (for bindings)

### For OmenDB Server & ZenDB (Rust)  
- Rust stable toolchain
- Cargo

### For OmenDB Web (SolidJS)
- Node.js 18+
- npm or pnpm

## Quick Start

### OmenDB Engine
```bash
cd omendb/engine
pixi install                     # Install dependencies
pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib
pixi run benchmark-quick         # Test with 1K-10K vectors
```

### OmenDB Server
```bash
cd omendb/server
cargo build
cargo run -- --config config.toml
```

### OmenDB Web
```bash
cd omendb/web
npm install
npm run dev                      # Start dev server at localhost:3000
```

### ZenDB
```bash
cd zendb
cargo test                       # Run test suite (61/70 passing)
cargo run --example basic_usage  # Run example
```

## Development Workflows

### Running Tests

```bash
# OmenDB engine tests
cd omendb/engine
make test-core
make test-compression

# ZenDB tests
cd zendb
cargo test

# Web frontend tests
cd omendb/web
npm run test
npm run typecheck
```

### Building for Production

```bash
# OmenDB engine release build
cd omendb/engine
pixi run mojo build --release omendb/native.mojo

# Server release build
cd omendb/server
cargo build --release

# Web production build
cd omendb/web  
npm run build
```

### Performance Benchmarking

```bash
# OmenDB benchmarks
cd omendb/engine
pixi run benchmark-standard      # 1K-100K vectors

# ZenDB benchmarks
cd zendb
cargo bench
```

## Common Tasks

### Adding Dependencies

**Mojo (OmenDB Engine)**:
```bash
cd omendb/engine
# Edit pixi.toml to add conda/pypi dependencies
pixi install
```

**Rust (Server/ZenDB)**:
```bash
cd omendb/server  # or zendb
cargo add <crate_name>
```

**JavaScript (Web)**:
```bash
cd omendb/web
npm install <package_name>
```

### Code Formatting

```bash
# Rust formatting
cargo fmt

# JavaScript/TypeScript
npm run format

# Mojo (follows Python style)
# Use black or ruff for Python files
```

## Debugging

### OmenDB Engine (Mojo)
```bash
cd omendb/engine
mojo debug omendb/native.mojo
# Or debug Python bindings
python -m pdb python/test_script.py
```

### Rust Projects
```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Use rust-gdb or rust-lldb
rust-gdb target/debug/zendb
```

### Web Frontend
Browser DevTools with source maps enabled in development mode.

## Known Issues & Workarounds

### OmenDB Engine
- **Scale bottleneck**: Performance degrades at 25K+ vectors (under investigation)
- **Global singleton**: All DB() instances share the same VectorStore
- **FFI overhead**: Use batch operations to minimize FFI calls

### ZenDB  
- 9 tests still failing (mostly edge cases)
- WAL recovery needs optimization

## Git Workflow

### Commits
```bash
git add .
git commit -m "feat(component): description"
# Use conventional commits: feat, fix, docs, refactor, test, chore
```

### Component Prefixes
- `omendb/engine`: Vector database changes
- `omendb/server`: Server changes  
- `omendb/web`: Frontend changes
- `zendb`: Hybrid database changes
- `internal`: Documentation/strategy changes

## Deployment

### Local Development
All components can run locally for development.

### Production Deployment
(Coming soon - Docker/Kubernetes configurations)

## Getting Help

- Check `internal/` for architecture decisions and research
- Review `CLAUDE.md` for AI assistant context
- See component READMEs for specific details

## Contributing

1. Create feature branch
2. Make changes with tests
3. Ensure all builds pass
4. Submit PR with clear description

## License

This is a private repository. Components may have different licenses when released publicly.