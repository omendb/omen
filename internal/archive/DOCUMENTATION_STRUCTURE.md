# OmenDB Documentation Structure

## Overview
This document defines the organization of internal documentation for optimal AI agent comprehension and developer workflow.

## Core Documentation Files (Top Level)

### 1. ARCHITECTURE.md
**Purpose**: Master technical specification and design decisions
**Contents**:
- Core architecture (HNSW+, Mojo engine, Python bindings)
- Algorithm selection rationale (BruteForce < 5K, HNSW+ >= 5K)
- Memory layout and SIMD strategies
- Module boundaries and responsibilities
- Performance requirements and constraints

### 2. RESEARCH.md
**Purpose**: Consolidated SOTA research and competitive analysis
**Contents**:
- Latest HNSW+ optimizations (2025 papers)
- Competitor benchmarks and analysis
- Binary quantization techniques
- Hub Highway Architecture findings
- Implementation priorities based on research

### 3. STATUS.md
**Purpose**: Current performance metrics and development roadmap
**Contents**:
- Latest benchmark results (updated from each sprint)
- Known issues and blockers
- 3-week rolling roadmap
- Performance gap analysis
- Active development branches

## Directory Structure

```
internal/
├── ARCHITECTURE.md      # Master technical spec
├── RESEARCH.md          # SOTA techniques & competitor analysis
├── STATUS.md            # Current metrics & roadmap
├── research/            # Detailed research papers/analysis
│   └── [specific deep dives]
├── archive/             # Historical documentation
│   └── [dated snapshots]
└── workarounds/         # Mojo-specific fixes
    └── [language limitations]
```

## Documentation Guidelines

### For AI Agents
1. **Single source of truth**: Each concept documented in exactly one place
2. **Cross-references**: Use relative links between documents
3. **Actionable format**: Include code examples and command snippets
4. **Status indicators**: Mark sections as VALIDATED, IN_PROGRESS, or PLANNED

### For Developers
1. **Update STATUS.md**: After each performance test or major change
2. **Archive old docs**: Move to `internal/archive/YYYY-MM-DD/` when superseded
3. **Research updates**: Add to RESEARCH.md with citation and relevance score

## Migration Plan
1. Consolidate scattered docs into core 3 files
2. Archive outdated/duplicate content
3. Update CLAUDE.md with new structure
4. Validate all cross-references work

## Best Practices
- Keep each file under 500 lines for quick agent loading
- Use consistent markdown formatting
- Include "Last Updated" timestamps
- Provide command examples for all claims