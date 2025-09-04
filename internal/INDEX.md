# Internal Documentation Index

## Core Documentation

| File | Purpose | Status |
|------|---------|--------|
| [../CLAUDE.md](../CLAUDE.md) | AI agent context for monorepo | ✅ Current |
| [../DEVELOPMENT.md](../DEVELOPMENT.md) | Development workflows | ✅ Current |
| [WORKFLOW.md](WORKFLOW.md) | Development processes | ⚠️ Needs update |
| [DECISIONS.md](DECISIONS.md) | Architecture decisions | ⚠️ Needs review |

## Technical Documentation

| Directory/File | Purpose | Status |
|----------------|---------|--------|
| [technical/](technical/) | Architecture specs | ⚠️ Mixed |
| [patterns/](patterns/) | Code patterns and fixes | ⚠️ Review needed |
| [research/](research/) | Performance research | 📚 Historical |
| [decisions/](decisions/) | Architecture decisions | 📚 Reference |

## Current Status

### OmenDB Engine
- **Scale**: Bottleneck at 25K+ vectors
- **Memory**: 288 bytes/vector (PQ compression working)
- **Issues**: Global singleton, FFI overhead
- **Focus**: Debug buffer flush performance

### ZenDB
- **Tests**: 61/70 passing (87%)
- **Features**: ACID, MVCC, WAL, compression complete
- **Issues**: Cache eviction, 2PC test timing
- **Focus**: Fix remaining tests, add SQL layer

## Directory Structure

```
internal/
├── archive/         # Historical investigations and old docs
├── decisions/       # Architecture decision records
├── patterns/        # Code patterns and error fixes
├── private/         # Business strategy (confidential)
├── research/        # Performance and optimization research
├── status/          # Project status tracking
└── technical/       # Technical specifications
```

## Key Files by Purpose

### Need Architecture Info?
- `decisions/` - Why we chose specific approaches
- `technical/` - How systems are designed

### Debugging Issues?
- `patterns/ERROR_FIXES.md` - Common error solutions
- `patterns/MOJO_PATTERNS.md` - Mojo-specific patterns
- `research/` - Performance investigations

### Planning Work?
- `WORKFLOW.md` - Development processes
- `GITHUB_ISSUES.md` - Issue tracking

### Historical Context?
- `archive/` - Past investigations and decisions
- `CHANGELOG.md` - Project history

---
*Last updated: January 2025*