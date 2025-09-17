# Internal Documentation

## Core Files (Read in Order)
1. **ARCHITECTURE.md** – Current system design and roadmap (SoA storage, zero-copy ingestion, chunked builder, parallel plan).
2. **RESEARCH.md** – SOTA techniques, competitor metrics, and implementation priorities.
3. **STATUS.md** – Latest benchmarks (per batch size) and next-action callouts.

## Directory Overview
```
internal/
├── ARCHITECTURE.md          # System blueprint & roadmap
├── RESEARCH.md              # SOTA reference and priorities
├── STATUS.md                # Benchmarks & blockers
├── DOCUMENTATION_STRUCTURE.md
├── SIMD_CPU_CLEANUP_STATUS.md  # Historical notes (Oct 2025 cleanup)
├── architecture/            # Supplemental specs (multimodal, storage)
├── current/                 # WIP scratch docs for active sprints
├── research/                # Deep-dive experiments/papers
└── archive/2025-10-pre-reorg/ # Previous documentation set
```

## Usage Guidelines
### AI Agents / Contributors
- Load the three core docs first; they contain the SoA/zero-copy/chunk plan and current metrics.
- When implementing or benchmarking, update `STATUS.md` with the command, dataset, and results.
- Major architectural changes (memory layout, ingestion, batching) belong in `ARCHITECTURE.md` with rationale.
- Capture new research findings or competitor learnings in `RESEARCH.md` with citations or links.
- Move outdated material into `archive/` instead of deleting, so history remains available.

### Maintenance Tips
- Keep each document concise (< ~500 lines) to ease consumption by future agents.
- Add “Last Updated” timestamps when making significant edits.
- Include specific test commands and targets when describing goals or metrics.
- Reflect actual implementation state—avoid speculative language without an accompanying plan.

## Current Snapshot (Oct 2025)
- **Insertion throughput:** 1K batch ~1,052 vec/s; 25K batch ~294 vec/s (sequential fallback).
- **Search latency:** ~0.73 ms (binary quant on).
- **Next milestones:** SoA distance kernels → zero-copy ingestion → chunked/parallel builder.

Keep this README synchronized with the other docs so new sessions follow the correct path immediately.

## Caveats & Mitigations
Refer to `internal/ARCHITECTURE.md` for detailed risk notes (SoA kernels, zero-copy safeguards, chunked builder plan).
