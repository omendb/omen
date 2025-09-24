# OmenDB Documentation Index

## Quick Start
- **[README.md](README.md)** – Repository overview and quick start
- **[CLAUDE.md](CLAUDE.md)** – Guidelines for AI agents and contributors

## Core Documentation

### Root Level (Summaries)
- **[ARCHITECTURE.md](ARCHITECTURE.md)** – High-level CPU-first design (2 min read)
- **[RESEARCH.md](RESEARCH.md)** – Competitive landscape overview (2 min read)
- **[STATUS.md](STATUS.md)** – Current metrics and next steps (1 min read)

### Internal (Full Details)
- **[internal/ARCHITECTURE.md](internal/ARCHITECTURE.md)** – Complete technical specification
- **[internal/RESEARCH.md](internal/RESEARCH.md)** – SOTA techniques and competitor analysis
- **[internal/STATUS.md](internal/STATUS.md)** – Detailed metrics and 3-week roadmap

## Current State (October 2025)

### Performance
- **Current**: 670-1,052 vec/s (varies by batch size)
- **Target**: 25,000+ vec/s
- **Gap**: 25-35x performance needed

### Critical Path
1. **SoA distance kernels** – Migrate all distance ops to Structure of Arrays
2. **Zero-copy FFI** – NumPy buffer protocol to eliminate Python overhead
3. **Chunked builder** – DiskANN-style batch processing with workspaces
4. **Parallel execution** – Multi-core after sequential stabilizes

### Known Issues
- SIMD compilation broken (`advanced_simd.mojo`)
- FFI overhead consuming 50% of execution time
- Bulk insertion crashes at 25K vectors

## Navigation Guide

### For Quick Context
Start with root-level summaries (ARCHITECTURE, RESEARCH, STATUS)

### For Implementation
Dive into `internal/` for full technical details

### For History
Check `internal/archive/` for superseded documentation

## Maintenance
- Update STATUS.md after benchmark runs
- Keep root summaries under 100 lines
- Full details go in internal/
- Archive old docs with date stamps

---
*Last updated: October 2025*