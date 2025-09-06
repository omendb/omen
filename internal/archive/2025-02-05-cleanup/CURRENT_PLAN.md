# Current Documentation & Development Plan
*As of Feb 2025 - Recording before context limit*

## Documentation Structure (Middle Ground: ~15 Files)

### From 847 → 15 Files
We reduced from 847 MD files to 4, but that's too consolidated. Optimal structure:

```
/
├── CLAUDE.md              # Global AI instructions
├── README.md             # Project overview
│
├── internal/
│   ├── current/          # Active work (3 files)
│   │   ├── sprint.md     # This week's tasks
│   │   ├── blockers.md   # Current issues
│   │   └── session.md    # Last session summary
│   │
│   ├── decisions/        # Architecture log (3 files)
│   │   ├── algorithms.md # HNSW vs DiskANN decision
│   │   ├── stack.md      # Mojo, Python, FFI choices
│   │   └── business.md   # Open source + cloud model
│   │
│   ├── knowledge/        # Technical reference (4 files)
│   │   ├── mojo.md       # Language patterns
│   │   ├── hnsw.md       # Algorithm specifics
│   │   ├── performance.md # Benchmarks, targets
│   │   └── gotchas.md    # Common errors, fixes
│   │
│   ├── ACTION_PLAN.md    # Roadmap (keep as-is)
│   └── README.md         # Index to all docs
│
├── omendb/
│   ├── CLAUDE.md         # Mojo-specific instructions
│   └── engine/
│       ├── CLAUDE.md     # Engine context
│       └── algorithms/
│           └── CLAUDE.md # HNSW implementation notes
│
└── archive/              # 800+ old files (hidden)
```

### Why This Structure Works

1. **Not hidden directories** - Avoiding `.claude/` etc that I might miss
2. **Logical grouping** - Current work, decisions, knowledge
3. **Small focused files** - Each under 200 lines
4. **Context-aware** - CLAUDE.md files near relevant code
5. **Session continuity** - `session.md` for quick catch-up

## Technical Decisions Made

### Algorithm: HNSW+ (Not DiskANN)
- **Why**: Industry standard, proven, GPU-ready
- **Timeline**: 4 weeks to MVP
- **Targets**: 100K vectors/sec build, 10K QPS

### Stack: Mojo Core
- **Python**: Zero FFI overhead (native)
- **C/Rust**: Via shared library (~100ns overhead)
- **GPU**: Future - same code compiles to both

### Business Model: Dual
- **Open Source**: CPU-only, broad adoption
- **Cloud**: GPU-accelerated, premium pricing

### Roadmap: Phased
1. **Month 1**: Pure vector HNSW+
2. **Month 2**: Add metadata filtering
3. **Month 3**: Full multimodal (vector + text + structured)

## Implementation Status

### Completed ✅
- Researched algorithms (IP-DiskANN, HNSW, CAGRA)
- Decided on HNSW+ over DiskANN
- Cleaned 847 → 4 files (too much)
- Defined business model

### Current Sprint (Week of Feb 5)
```bash
# Today's tasks
1. Create middle-ground doc structure (15 files)
2. Start HNSW+ implementation:
   touch omendb/engine/algorithms/hnsw.mojo
3. Define core structures:
   - HNSWIndex with hierarchical layers
   - M=16 connections, ef=200 search param
```

### Next Week
- Implement insert() and search() functions
- Add SIMD distance calculations
- Create Python bindings

## Key Commands

```bash
# Build
cd omendb/engine
pixi run mojo build omendb/native.mojo -o python/omendb/native.so

# Test
pixi run benchmark-quick

# Reference
https://github.com/nmslib/hnswlib
```

## Critical Context for Next Session

1. **We pivoted from DiskANN to HNSW+** - DiskANN fundamentally incompatible with streaming
2. **Mojo has bidirectional edges already** - But we're not using them since switching to HNSW
3. **Business focus**: Beat pgvector first (10x performance), then add multimodal
4. **Doc structure**: Moving from 4 files to ~15 for better organization
5. **No hidden dirs**: Use `internal/` not `.claude/` or `.ai/`

## Questions Resolved

- **HNSW vs IP-DiskANN?** → HNSW (proven, simpler, GPU-ready)
- **Pure vector vs multimodal?** → Start pure, add multimodal phase 2
- **Mojo vs Rust?** → Keep Mojo (Python native, future GPU)
- **847 files vs 4 files?** → Middle ground: ~15 files

---
*This captures our current state before potential context reset*