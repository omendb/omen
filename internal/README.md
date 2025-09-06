# Internal Documentation Index

## Quick Navigation for AI Agents

### Current Work
- **[NOW.md](NOW.md)** - What we're working on this sprint
- **[DOC_ORGANIZATION.md](DOC_ORGANIZATION.md)** - How to organize documentation

### Core Documentation
- **[DECISIONS.md](DECISIONS.md)** - Major decisions and rationale (append-only)
- **[KNOWLEDGE.md](KNOWLEDGE.md)** - Patterns, gotchas, and learnings

### Architecture
- **[architecture/MULTIMODAL.md](architecture/MULTIMODAL.md)** - Multimodal database design
- **[architecture/](architecture/)** - System design documents

### Archive
- **[archive/](archive/)** - Historical documents and old research

## Current Project Status

### What We're Building
**OmenDB**: Open source multimodal database for AI applications
- Vectors + text + metadata in unified system
- 10x faster than MongoDB Atlas
- GPU acceleration path via Mojo

### Key Decisions
1. **Multimodal from start** (not pure vector first)
2. **HNSW+ algorithm** (not DiskANN)
3. **Mojo core** + Rust server + Python bindings
4. **SQL interface** with vector extensions

### Why These Choices
- **Multimodal**: 10x pricing power, less competition
- **HNSW+**: Industry standard, streaming updates
- **Mojo**: GPU compilation, Python-native, SIMD
- **SQL**: Everyone knows it, better query planning

### Development Timeline
- **Month 1**: HNSW+ core with metadata filtering
- **Month 2**: Add BM25 text search, query planner
- **Month 3**: Production features, cloud deployment
- **Month 4**: GPU acceleration, enterprise features

## For New AI Agents

### Starting a Session
1. Read `/CLAUDE.md` for project overview
2. Read `NOW.md` for current tasks
3. Check `DECISIONS.md` for context
4. Reference `KNOWLEDGE.md` for patterns

### Making Changes
1. Update `NOW.md` with progress
2. Append decisions to `DECISIONS.md`
3. Update patterns in `KNOWLEDGE.md`
4. Never create analysis files

### Documentation Philosophy
- **Actionable over analytical**
- **Single source of truth**
- **Append-only for decisions**
- **AI-agent optimized**

## Technical Context

### Current Bottlenecks
- Mojo missing async/await (use thread pools)
- Limited Mojo stdlib (implement what we need)
- No mature HTTP server (use Python wrapper)

### Performance Targets
- 100K vectors/sec insertion
- <10ms hybrid query latency
- 2-4 bytes per vector memory
- 10K QPS single node

### Competitive Advantages
- GPU compilation (Mojo exclusive)
- Python-native (zero FFI overhead)
- Open source full functionality
- Unified multimodal queries

---
*Last updated: Feb 5, 2025 - See NOW.md for current sprint*