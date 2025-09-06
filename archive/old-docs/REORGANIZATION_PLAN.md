# Repository Reorganization Plan

## Current Issues
1. **Root directory clutter**: 11 MD files in root (should be 2-3 max)
2. **ZenDB question**: Experimental DB taking space, not aligned with HNSW+ focus
3. **Redundant research**: 8 algorithm analysis files saying similar things
4. **Internal has 64 files**: Many outdated or redundant

## Proposed Structure

```
/
├── README.md                    # Keep (entry point)
├── CLAUDE.md                   # Keep (AI instructions)
├── LICENSE                     # Add if missing
│
├── /omendb/                    # Main product
│   ├── engine/                 # Mojo HNSW+ implementation
│   ├── bindings/              # NEW: Python/C/Rust bindings
│   └── docs/                  # NEW: Consolidate documentation
│       ├── architecture.md
│       ├── benchmarks.md
│       └── api.md
│
├── /internal/                  # Private knowledge
│   ├── decisions/             # Key decisions only
│   │   ├── 2025-02-HNSW.md  # Algorithm choice
│   │   └── archive/          # Old decisions
│   ├── research/              # Consolidate to 2-3 files
│   │   ├── algorithm_comparison.md
│   │   └── archive/
│   └── operations/            # Day-to-day
│       ├── action_plan.md    # Current sprint
│       ├── session_log.md    # Work history
│       └── patterns/          # Extracted patterns
│
├── /external/                  # Keep as-is
│   ├── agent-contexts/        # Submodule
│   ├── diskann/              # Reference (may remove)
│   └── competitors/          # Keep for benchmarking
│
└── /archive/                   # NEW: Move old projects
    ├── zendb/                 # Move here
    ├── omendb/server/        # Old Rust server
    └── omendb/web/           # Marketing site
```

## Files to Move/Consolidate

### From Root → internal/operations/
- ACTION_PLAN.md → internal/operations/action_plan.md
- SESSION_LOG.md → internal/operations/session_log.md
- TASKS.md → internal/operations/tasks.md
- ERROR_FIXES.md → internal/operations/troubleshooting.md

### From Root → internal/decisions/
- DECISIONS.md → internal/decisions/index.md
- DISCOVERIES.md → internal/decisions/learnings.md

### From Root → Delete (redundant with CLAUDE.md)
- AI_AGENT_PLAYBOOK.md (merge useful parts into CLAUDE.md)
- QUICK_REFERENCE.md (merge into CLAUDE.md)
- DEVELOPMENT.md (outdated, covered in CLAUDE.md)

### Research Consolidation
Keep only:
1. `internal/research/FINAL_ALGORITHM_DECISION.md` → Rename to `hnsw_decision.md`
2. `internal/research/COMPETITOR_ANALYSIS.md` → Keep as reference

Archive others to `internal/research/archive/`

## ZenDB Decision

**Recommendation: Archive ZenDB**
- Not aligned with HNSW+ focus
- Adds confusion to repo
- 30% failing tests
- Can resurrect later if needed

Move to `/archive/zendb/` with README explaining it's experimental multimodal work.

## Implementation Order

1. **Create new directories** (5 min)
2. **Move root files** (10 min)
3. **Archive ZenDB** (5 min)
4. **Consolidate research** (20 min)
5. **Update CLAUDE.md** with new structure (10 min)
6. **Update README.md** to be cleaner (10 min)

## Benefits

- **Cleaner root**: Just README, CLAUDE, LICENSE
- **Focused**: OmenDB with HNSW+ is the only active project
- **Better organization**: Clear separation of public/internal/archived
- **AI-friendly**: Simpler navigation for agents

## Questions Before Proceeding

1. Keep any GPU experiments in `omendb/engine/gpu/`?
2. Should bindings be separate or in engine?
3. Any files you want to explicitly preserve?