# 📁 Repository Cleanup Plan (REVISED)

**Change**: Keep server/ and web/ - they're part of the strategy!

## Target Structure

```
omendb/core/
├── CLAUDE.md                     # Primary AI context
├── README.md                     # Public-facing docs
├── LICENSE                       # Apache 2.0
│
├── internal/                     # Private working docs
│   ├── STATUS.md                # Current metrics
│   ├── TODO.md                  # Active tasks
│   ├── DECISIONS.md             # Architecture log
│   │
│   ├── strategy/                # Strategic documents
│   │   ├── STARTUP_MASTER_PLAN.md
│   │   ├── RUST_VS_MOJO_DECISION.md ✨
│   │   ├── MOJO_REALITY_CHECK.md ✨
│   │   └── COMPETITIVE_ANALYSIS_2025.md
│   │
│   ├── architecture/            # Technical architecture
│   │   ├── UNIFIED_ARCHITECTURE_FINAL.md ✨
│   │   ├── HNSW_CORRECTNESS_RULES.md
│   │   ├── REFACTOR_RECOMMENDATION.md
│   │   └── STATE_OF_THE_ART_ARCHITECTURE.md
│   │
│   ├── research/
│   │   ├── mojo_vector_db_design_enterprise.md
│   │   ├── ARCHITECTURE_COMPARISON.md
│   │   └── PERFORMANCE_ANALYSIS.md
│   │
│   ├── prototypes/
│   │   └── HYBRID_IMPLEMENTATION_PROTOTYPE.mojo
│   │
│   └── archive/                 # Historical
│
├── omendb/
│   ├── engine/                 # Mojo engine (KEEP) ✅
│   │   ├── omendb/
│   │   │   ├── algorithms/    # HNSW implementations
│   │   │   └── native.mojo    # Core DB
│   │   ├── python/             # Python bindings
│   │   └── benchmarks/         # Performance tests
│   │
│   ├── server/                 # Rust server (KEEP) ✅
│   │   ├── src/                # HTTP/gRPC server
│   │   ├── k8s/                # Kubernetes configs
│   │   └── Cargo.toml          # Rust dependencies
│   │
│   └── web/                    # Marketing site (KEEP) ✅
│       ├── src/                # SolidJS app
│       └── package.json        # Node dependencies
│
├── zendb/                       # DELETE - separate project ❌
│
├── docs/                        # Public documentation
├── examples/                    # Code examples
└── scripts/                     # Build & utilities
```

## What to Keep vs Delete

### ✅ KEEP (Valuable)
```
omendb/engine/    # Mojo computational engine
omendb/server/    # Rust async server wrapper
omendb/web/       # Marketing website
internal/         # Working documentation
external/         # Agent contexts
```

### ❌ DELETE (Not needed)
```
zendb/            # Separate Rust project, not part of OmenDB
internal/AI_AGENT_CONTEXT.md     # Duplicate of CLAUDE.md
internal/DOCUMENTATION_STRUCTURE.md
internal/README.md
internal/BREAKTHROUGH_SEPT_20_2025.md
internal/HONEST_REALITY_CHECK.md
internal/MEMORY_STABILITY_ANALYSIS.md
internal/SEGMENTED_HNSW_RESULTS.md
internal/THRESHOLD_UPDATE_RESULTS.md
ARCHITECTURE.md   # Old, move content to docs/
```

## Cleanup Commands

```bash
# Phase 1: Backup
cp -r /Users/nick/github/omendb/core /Users/nick/github/omendb/core.backup

# Phase 2: Remove zendb (separate project)
rm -rf zendb/

# Phase 3: Remove redundant internal docs
cd internal/
rm AI_AGENT_CONTEXT.md
rm DOCUMENTATION_STRUCTURE.md
rm README.md
rm BREAKTHROUGH_SEPT_20_2025.md
rm HONEST_REALITY_CHECK.md
rm MEMORY_STABILITY_ANALYSIS.md
rm SEGMENTED_HNSW_RESULTS.md
rm THRESHOLD_UPDATE_RESULTS.md
cd ..

# Phase 4: Create new structure
mkdir -p internal/strategy
mkdir -p internal/architecture
mkdir -p internal/prototypes
mkdir -p docs
mkdir -p examples
mkdir -p scripts

# Phase 5: Move files to proper locations
# Strategy docs
mv internal/STARTUP_MASTER_PLAN.md internal/strategy/
mv internal/RUST_VS_MOJO_DECISION.md internal/strategy/
mv internal/MOJO_REALITY_CHECK.md internal/strategy/
mv internal/COMPETITIVE_ANALYSIS_2025.md internal/strategy/
mv internal/OMENDB_NEXT_GEN_PLAN.md internal/strategy/

# Architecture docs
mv internal/UNIFIED_ARCHITECTURE_FINAL.md internal/architecture/
mv internal/HNSW_CORRECTNESS_RULES.md internal/architecture/
mv internal/HNSW_DEVELOPMENT_GUIDE.md internal/architecture/
mv internal/HNSW_OPTIMIZATION_FINDINGS.md internal/architecture/
mv internal/REFACTOR_RECOMMENDATION.md internal/architecture/
mv internal/STATE_OF_THE_ART_ARCHITECTURE.md internal/architecture/

# Research docs
mv internal/ARCHITECTURE_COMPARISON.md internal/research/
mv internal/LANCEDB_ANALYSIS.md internal/research/
mv internal/PERFORMANCE_ANALYSIS.md internal/research/
mv internal/COMPETITIVE_ARCHITECTURE_ANALYSIS.md internal/research/

# Prototypes
mv internal/HYBRID_IMPLEMENTATION_PROTOTYPE.mojo internal/prototypes/

# Archive old root docs
mv ARCHITECTURE.md internal/archive/
```

## New Files to Create

### 1. Updated CLAUDE.md
```markdown
# OmenDB Development Context

## Architecture: Rust + Mojo Hybrid
- **Rust Server**: Async, networking, state (omendb/server/)
- **Mojo Engine**: Computation, SIMD, GPU (omendb/engine/)
- **Web**: Marketing site (omendb/web/)

## Key Documents
1. Strategy: internal/strategy/RUST_VS_MOJO_DECISION.md
2. Architecture: internal/architecture/UNIFIED_ARCHITECTURE_FINAL.md
3. Status: internal/STATUS.md
```

### 2. Professional README.md
```markdown
# OmenDB

The developer's favorite vector database. Blazing fast, embedded-first, scales to cloud.

## Architecture
- 🦀 Rust server for async operations
- 🔥 Mojo engine for computation
- 🚀 100K+ vec/s insertion
- 💫 <3ms search with 95% recall

## Quick Start
pip install omendb
```

## Why Keep Server and Web?

### Server (Rust)
- **Handles async** that Mojo can't do yet
- **Production ready** with HTTP/gRPC
- **Already implemented** - don't throw away work
- **Enables background indexing** via threading

### Web (SolidJS)
- **Marketing presence** needed for launch
- **Documentation site** for developers
- **Benchmarks page** to show performance
- **Already built** - just needs content update

### Engine (Mojo)
- **Core differentiator** - "Powered by Mojo"
- **Performance advantage** - SIMD native
- **GPU future** - Metal/CUDA path

## The Hybrid Advantage

```
User's View:
Python API → Simple & clean

Our Implementation:
Python → Rust Server → Mojo Engine

Why It Works:
- Ships TODAY (Mojo can't do async until 2026+)
- Best performance (Rust I/O + Mojo compute)
- Future-proof (can evolve as Mojo matures)
```

## Summary

**Don't delete server/ and web/** - they're essential parts of the hybrid architecture that works TODAY with Mojo's current limitations.

The Rust+Mojo hybrid is not a compromise, it's the optimal architecture.