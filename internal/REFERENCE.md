# OmenDB Documentation Reference Map
*AI Agent Navigation Guide*

## Core Files (Always Current)

### Primary Context (Include in every conversation)
```bash
@CLAUDE.md                    # Entry point & quick context
```

### Task-Specific Includes
```bash
# For status/progress queries:
@docs/STATUS.md               # Current metrics, issues, state

# For technical work:
@docs/TECH_SPEC.md           # Architecture, algorithms, design

# For code standards:
@docs/CODE_STANDARDS.md      # Naming, patterns, AI agent rules

# For task planning:
@docs/TODO.md                # Priorities, blockers, roadmap

# For understanding decisions:
@docs/DECISIONS.md           # Append-only decision log

# For development history:
@docs/CHANGELOG.md           # What changed when
```

## Information Locations (Single Source of Truth)

### Performance & Metrics
- **Current Performance**: `docs/STATUS.md#performance-metrics`
- **Benchmark Results**: `docs/STATUS.md#competitive-comparison`
- **Optimization Targets**: `docs/TODO.md#next-sprint`

### Architecture & Design
- **System Architecture**: `docs/TECH_SPEC.md#architecture`
- **Algorithm Details**: `docs/TECH_SPEC.md#algorithm-diskann`
- **Storage Design**: `docs/TECH_SPEC.md#storage-architecture`
- **API Design**: `docs/TECH_SPEC.md#api-design`

### Known Issues & Bugs
- **Current Issues**: `docs/STATUS.md#known-issues`
- **Blockers**: `docs/TODO.md#blocked-by-mojo`
- **Testing Gaps**: `docs/TODO.md#testing-gaps`

### Implementation Details
- **Code Locations**: `docs/TECH_SPEC.md#file-structure`
- **Build Commands**: `docs/STATUS.md#key-commands`
- **FFI Details**: `docs/TECH_SPEC.md#mojo-specific-considerations`

### Implementation & Code
- **FFI Zero-Copy**: `docs/TECH_SPEC.md#ffi-zero-copy-implementation`
- **Bug Discoveries**: `docs/DECISIONS.md` (dated entries)
- **Algorithm Params**: `docs/TECH_SPEC.md#algorithm-tuning-parameters`
- **Testing Methods**: `docs/TECH_SPEC.md#testing-benchmarking`

### Business & Strategy
- **Market Position**: `docs/TECH_SPEC.md#competitive-analysis`
- **Future Plans**: `docs/TODO.md#future-roadmap`
- **Private Docs**: `docs/private/business/` (not for public repo)

## File System Map

```
/Users/nick/github/omendb/omendb-cloud/    # Private repo (internal docs)
├── CLAUDE.md                              # Entry point
├── docs/
│   ├── STATUS.md                         # Current state (single source)
│   ├── TECH_SPEC.md                      # Technical specification
│   ├── TODO.md                           # Task list & priorities
│   ├── DECISIONS.md                      # Decision log with lessons
│   ├── CHANGELOG.md                      # Development history
│   ├── REFERENCE.md                      # This file - navigation
│   ├── CODE_STANDARDS.md                 # Coding standards & AI rules
│   ├── DOC_STANDARDS.md                  # Documentation guidelines
│   ├── ADMIN_DASHBOARD_SPEC.md           # Web interface spec
│   ├── private/                          # Business docs (keep private)
│   │   └── business/                     # Investor, YC, strategy
│   └── archive/                          # All outdated docs
│       └── 2025-08-23-cleanup/           # Today's archived files

/Users/nick/github/omendb/omendb/          # Public repo (code only)
├── README.md                              # Public introduction
├── omendb/                                # Mojo source code
│   ├── native.mojo                       # Entry point
│   ├── algorithms/diskann.mojo           # Core algorithm
│   └── core/                             # Core components
└── python/                                # Python API
    └── omendb/api.py                      # User interface
```

## Quick Reference

### Performance Numbers (as of Aug 23, 2025)
- **Checkpoint**: 739,310 vec/s (needs validation)
- **Batch Insert**: 85K vec/s (verified)
- **Search**: 0.62ms @ 128D (verified)
- **Memory**: 40MB/1M vectors (needs optimization)

### Key Technical Facts
- **Language**: Mojo (Python syntax, C++ performance)
- **Algorithm**: DiskANN only (no switching)
- **Storage**: Memory-mapped with double-buffering
- **Platform**: macOS/Linux (no Windows yet)

### Common Tasks & Required Files

| Task | Include These Files |
|------|-------------------|
| Performance Work | `@docs/STATUS.md` + `@docs/TECH_SPEC.md#performance-characteristics` |
| Bug Fixes | `@docs/STATUS.md#known-issues` + `@docs/DECISIONS.md` |
| Architecture Changes | `@docs/TECH_SPEC.md` + `@docs/DECISIONS.md` |
| Sprint Planning | `@docs/TODO.md` + `@docs/STATUS.md` |
| Benchmarking | `@docs/TECH_SPEC.md#testing-benchmarking` |
| Code Implementation | `@docs/TECH_SPEC.md` (has all code examples) |

## Navigation Tips for AI Agents

1. **Start with CLAUDE.md** - Always included, provides context
2. **Check STATUS.md** - For current state and metrics
3. **Reference TECH_SPEC.md** - For how things work
4. **Use TODO.md** - For what needs doing
5. **Append to DECISIONS.md** - For recording new decisions

## Maintenance Rules

1. **Single Source**: Each fact lives in ONE place only
2. **Update Immediately**: Change docs when code changes
3. **Archive Don't Delete**: Move old docs to archive/
4. **Date Everything**: Use `date +"%Y-%m-%d"` format
5. **Reference Don't Duplicate**: Link to info, don't copy it

---
*This reference map is the authoritative guide to OmenDB documentation structure.*