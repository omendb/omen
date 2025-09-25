# OmenDB Internal Documentation

**Last Updated**: September 25, 2025

## Core Documents (5 Files Only)

| File | Purpose | When to Read |
|------|---------|--------------|
| **ARCHITECTURE.md** | Technical design, algorithms, implementation | Building features |
| **BUSINESS.md** | Market analysis, monetization, YC pitch | Strategic decisions |
| **ROADMAP.md** | Timeline, milestones, daily tasks | Planning work |
| **STATUS.md** | Current progress, blockers, metrics | Daily standup |
| **CONTEXT.md** | AI agent instructions (minimal) | Loading AI context |

## Quick Summary

### What We're Building
PostgreSQL extension with learned indexes (ML replaces B-trees) for 10x faster lookups.

### Why
- Zero competition (vs 30+ vector DB competitors)
- 10-100x performance improvement possible
- Clear monetization (enterprise features)

### How
1. Linear RMI in Rust
2. PostgreSQL extension via pgrx
3. Ship by Oct 7 or pivot

### Focus
**PostgreSQL extension ONLY**. No standalone DB, no embedded mode, no server mode.

### Success Metric
Lookup latency <40ns (10x faster than 200ns B-tree).

## External Resources

```
external/
├── papers/          # Research papers (RMI, ALEX, etc.)
├── learned-systems/ # Reference implementations
└── agent-contexts/  # AI automation patterns
```

## Archive

Old vector DB docs and outdated decisions moved to `internal/archive/`.

---

*Keep it simple. Ship fast. Focus on PostgreSQL extension only.*