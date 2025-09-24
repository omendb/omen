# 📁 Repository Cleanup Plan

## Current State: Absolute Mess
- Documentation scattered across multiple dirs
- Redundant files everywhere
- No clear hierarchy
- Mix of old experiments and current work

## Target Structure

```
omendb/core/
├── CLAUDE.md                     # Primary AI context (keep, update)
├── README.md                     # Public-facing docs
├── LICENSE                       # Apache 2.0
│
├── internal/                     # Private working docs
│   ├── STATUS.md                # Current metrics (keep)
│   ├── TODO.md                  # Active tasks (keep)
│   ├── DECISIONS.md             # Architecture log (keep)
│   │
│   ├── strategy/                # Strategic documents
│   │   ├── STARTUP_MASTER_PLAN.md
│   │   ├── ARCHITECTURE_COMPARISON.md
│   │   ├── COMPETITIVE_ANALYSIS_2025.md
│   │   └── OMENDB_NEXT_GEN_PLAN.md
│   │
│   ├── architecture/            # Technical architecture
│   │   ├── HNSW_CORRECTNESS_RULES.md
│   │   ├── HNSW_DEVELOPMENT_GUIDE.md
│   │   ├── REFACTOR_RECOMMENDATION.md
│   │   └── STATE_OF_THE_ART_ARCHITECTURE.md
│   │
│   ├── research/                # Research & analysis
│   │   ├── mojo_vector_db_design_enterprise.md
│   │   ├── LANCEDB_ANALYSIS.md
│   │   ├── PERFORMANCE_ANALYSIS.md
│   │   └── COMPETITIVE_ARCHITECTURE_ANALYSIS.md
│   │
│   ├── prototypes/              # Code prototypes
│   │   └── HYBRID_IMPLEMENTATION_PROTOTYPE.mojo
│   │
│   └── archive/                 # Historical (existing)
│
├── omendb/engine/               # Mojo engine (main code)
│   ├── omendb/
│   │   ├── algorithms/          # HNSW, segmented, etc
│   │   ├── native.mojo         # Main DB implementation
│   │   └── ...
│   ├── python/                  # Python bindings
│   ├── benchmarks/             # Performance tests
│   └── tests/                  # Unit tests
│
├── docs/                        # Public documentation
│   ├── getting-started.md
│   ├── api-reference.md
│   └── architecture.md
│
├── examples/                    # Code examples
│   ├── basic_usage.py
│   ├── langchain_integration.py
│   └── production_deployment.py
│
└── scripts/                     # Build & test scripts
    ├── build.sh
    ├── test.sh
    └── benchmark.sh
```

## Files to Delete/Move

### Delete (Redundant/Outdated)
```bash
# Old server stuff we're not using
rm -rf omendb/server/

# Web UI we're not building yet
rm -rf omendb/web/

# Rust hybrid DB - separate project
rm -rf zendb/

# Redundant architecture files
rm internal/AI_AGENT_CONTEXT.md  # Duplicate of CLAUDE.md
rm internal/DOCUMENTATION_STRUCTURE.md  # Not needed
rm internal/README.md  # Redundant

# Old analysis files that are now incorporated
rm internal/BREAKTHROUGH_SEPT_20_2025.md  # Incorporated
rm internal/HONEST_REALITY_CHECK.md  # Incorporated
rm internal/MEMORY_STABILITY_ANALYSIS.md  # Old
rm internal/SEGMENTED_HNSW_RESULTS.md  # Old test
rm internal/THRESHOLD_UPDATE_RESULTS.md  # Old test

# Root level clutter
rm ARCHITECTURE.md  # Move content to docs/
```

### Move/Reorganize
```bash
# Create new directories
mkdir -p internal/strategy
mkdir -p internal/architecture
mkdir -p internal/prototypes
mkdir -p docs
mkdir -p examples
mkdir -p scripts

# Move strategy docs
mv internal/STARTUP_MASTER_PLAN.md internal/strategy/
mv internal/COMPETITIVE_ANALYSIS_2025.md internal/strategy/
mv internal/OMENDB_NEXT_GEN_PLAN.md internal/strategy/
mv internal/ARCHITECTURE_COMPARISON.md internal/strategy/

# Move architecture docs
mv internal/HNSW_CORRECTNESS_RULES.md internal/architecture/
mv internal/HNSW_DEVELOPMENT_GUIDE.md internal/architecture/
mv internal/HNSW_OPTIMIZATION_FINDINGS.md internal/architecture/
mv internal/REFACTOR_RECOMMENDATION.md internal/architecture/
mv internal/STATE_OF_THE_ART_ARCHITECTURE.md internal/architecture/

# Move research docs
mv internal/LANCEDB_ANALYSIS.md internal/research/
mv internal/PERFORMANCE_ANALYSIS.md internal/research/
mv internal/COMPETITIVE_ARCHITECTURE_ANALYSIS.md internal/research/

# Move prototypes
mv internal/HYBRID_IMPLEMENTATION_PROTOTYPE.mojo internal/prototypes/

# Keep in place
# internal/STATUS.md
# internal/TODO.md
# internal/DECISIONS.md
# internal/RESEARCH.md
```

## New Files to Create

### 1. Updated CLAUDE.md
- Incorporate latest learnings
- Clear hierarchy: overview → key docs → details
- Add startup context

### 2. Professional README.md
```markdown
# OmenDB - Vector Database for the AI Era

Fast, embedded vector database that scales from laptop to cloud.

## Features
- 100K+ vectors/sec insertion
- <3ms search latency
- 95%+ recall at scale
- Zero dependencies
- SQLite-like embedded mode

## Quick Start
pip install omendb
```

### 3. Public docs/
- getting-started.md
- api-reference.md
- architecture.md
- benchmarks.md

### 4. Examples directory
- Basic usage
- LangChain integration
- Production deployment
- Migration from Pinecone/Weaviate

## Cleanup Commands

```bash
# Phase 1: Backup everything
cp -r /Users/nick/github/omendb/core /Users/nick/github/omendb/core.backup

# Phase 2: Delete unnecessary files
cd /Users/nick/github/omendb/core

# Remove unused subdirectories
rm -rf omendb/server/ omendb/web/ zendb/

# Remove redundant docs
rm internal/AI_AGENT_CONTEXT.md
rm internal/DOCUMENTATION_STRUCTURE.md
rm internal/README.md
rm internal/BREAKTHROUGH_SEPT_20_2025.md
rm internal/HONEST_REALITY_CHECK.md
rm internal/MEMORY_STABILITY_ANALYSIS.md
rm internal/SEGMENTED_HNSW_RESULTS.md
rm internal/THRESHOLD_UPDATE_RESULTS.md
rm ARCHITECTURE.md

# Phase 3: Create new structure
mkdir -p internal/strategy
mkdir -p internal/architecture
mkdir -p internal/prototypes
mkdir -p docs
mkdir -p examples
mkdir -p scripts

# Phase 4: Move files to proper locations
# (commands listed above)

# Phase 5: Create new essential files
# - Updated CLAUDE.md
# - Professional README.md
# - Public documentation
```

## Benefits After Cleanup

1. **Clear Hierarchy**: CLAUDE.md → category dirs → specific docs
2. **No Redundancy**: Each doc has unique purpose
3. **Professional**: Ready for public/investors
4. **Maintainable**: Easy to find and update
5. **AI-Friendly**: Clear structure for Claude/Cursor

## Priority Order

1. **First**: Update CLAUDE.md with new structure
2. **Second**: Clean up internal/ directory
3. **Third**: Create public README and docs/
4. **Fourth**: Remove server/web/zendb cruft
5. **Fifth**: Add examples and scripts

This will transform the repo from "absolute mess" to "professional startup codebase".