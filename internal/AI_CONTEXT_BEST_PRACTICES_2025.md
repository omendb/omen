# AI Context Management Best Practices (2025)

**Date**: October 11, 2025
**Based on**: Latest Claude Code, Cursor, Copilot best practices
**Research**: Anthropic, GitHub, industry leaders

---

## Executive Summary

**2025 State of the Art:**
- **Context Engineering** is the new paradigm (not just "prompt engineering")
- **Cascading context files** enable team-wide + personal configurations
- **Model Context Protocol (MCP)** allows dynamic tool integration
- **Sub-agents** manage specialized tasks with focused contexts
- **200k token context** is standard (Claude Code advantage)

**OmenDB Current Status**: ✅ Already following many best practices

---

## Key Findings from Research

### 1. CLAUDE.md is Critical (Highest Priority)

**What it is:**
- Single source of truth for project context
- Automatically ingested by Claude Code at session start
- Persists across sessions (project memory)

**What to put in it:**
```markdown
# What CLAUDE.md Should Contain (2025 Best Practice)

1. **Project overview** (2-3 sentences)
2. **Current status** (version, phase, key metrics)
3. **Architecture overview** (high-level only)
4. **Common commands** (build, test, benchmark)
5. **Code style guidelines** (conventions, patterns)
6. **Key files/directories** (where things are)
7. **Testing instructions** (how to validate changes)
8. **Pointers to deeper docs** (internal/, external/)
```

**OmenDB Status**: ✅ You have CLAUDE.md - needs updating to 2025 standard

### 2. Cascading Context Files (NEW for 2025)

**Hierarchy:**
```
/CLAUDE.md                      # Project-wide (committed)
/CLAUDE.local.md                # Personal overrides (gitignored)
/src/alex/claude.md             # Subdirectory-specific (committed)
/.claude/commands/              # Custom commands (committed)
```

**Benefits:**
- **Team-wide** consistency via committed CLAUDE.md
- **Personal** preferences via .local.md (not pushed to repo)
- **Context-specific** guidance in subdirectories
- **Reusable** commands shared across team

**OmenDB Opportunity**: Add subdirectory claude.md files for:
- `src/alex/claude.md` - Multi-level ALEX specific guidance
- `src/postgres/claude.md` - PostgreSQL protocol specifics
- `tests/claude.md` - Testing conventions

### 3. Custom Commands (.claude/commands/)

**What they are:**
- Reusable prompt templates
- Automatically available to all team members
- Stored in `.claude/commands/*.md`

**Examples for OmenDB:**

**`.claude/commands/benchmark.md`:**
```markdown
Run competitive benchmark against SQLite:

1. Build release: cargo build --release
2. Run benchmark: ./target/release/benchmark_vs_sqlite 10000000
3. Compare results to internal/research/ALEX_SQLITE_BENCHMARK_RESULTS.md
4. If significant difference:
   - Profile: ./target/release/profile_query_path
   - Investigate regression or improvement
   - Update STATUS_REPORT if major
```

**`.claude/commands/release-checklist.md`:**
```markdown
Pre-release checklist:

- [ ] All tests pass: cargo test --release
- [ ] Benchmarks at baseline: ./target/release/benchmark_vs_sqlite 10000000
- [ ] No clippy warnings: cargo clippy
- [ ] Documentation updated: ARCHITECTURE.md, CLAUDE.md
- [ ] CHANGELOG.md updated
- [ ] internal/STATUS_REPORT_OCT_2025.md reflects current state
```

**OmenDB Opportunity**: Create `.claude/commands/` for common workflows

### 4. Model Context Protocol (MCP)

**What it is:**
- Native Claude Code feature
- Connects to external tools/APIs
- Dynamic context injection

**Examples:**
```
Claude Code + MCP can access:
- Databases (PostgreSQL, SQLite)
- APIs (GitHub, Linear, Notion)
- Dashboards (Grafana, Datadog)
- Context7, Fetch servers
```

**OmenDB Opportunity:**
- MCP server for OmenDB benchmarks (query live performance data)
- MCP server for GitHub issues (track customer feedback)
- MCP server for competitive intel (auto-update from competitor releases)

**Future**: Not urgent, but powerful for automation

### 5. Sub-Agents (Advanced)

**What they are:**
- Specialized AI assistants for specific tasks
- Own context windows and tool permissions
- Prevent context overflow

**Example use cases:**
```
Main Claude session:
├── Sub-agent: Code reviewer (focused on CONTRIBUTING.md)
├── Sub-agent: Benchmark runner (focused on performance)
├── Sub-agent: Documentation writer (focused on user docs)
└── Sub-agent: Test writer (focused on test patterns)
```

**OmenDB Status**: Not currently used, not needed yet (200k context sufficient)

**When to adopt**: If context window consistently maxed out (not the case currently)

---

## Recommended Structure for OmenDB (2025 Best Practice)

### Current Structure (Good!) ✅

```
omendb/core/
├── CLAUDE.md                   # ✅ Exists, needs update
├── ARCHITECTURE.md             # ✅ Good
├── CONTRIBUTING.md             # ✅ Good
├── internal/                   # ✅ Well-organized
│   ├── STATUS_REPORT_OCT_2025.md
│   ├── business/
│   ├── research/
│   ├── technical/
│   └── phases/
└── external/
    └── agent-contexts/         # ⚠️ Empty, needs init
```

### Recommended Additions (2025 Standard)

```
omendb/core/
├── CLAUDE.md                   # UPDATE to 2025 standard
├── CLAUDE.local.md             # ADD (gitignored, personal)
├── .gitignore                  # ADD: CLAUDE.local.md
│
├── .claude/                    # NEW directory
│   ├── commands/               # Custom commands
│   │   ├── benchmark.md
│   │   ├── release-checklist.md
│   │   ├── update-status.md
│   │   └── competitive-analysis.md
│   └── settings.json           # Claude Code settings (optional)
│
├── src/
│   ├── alex/
│   │   └── claude.md           # NEW: ALEX-specific context
│   ├── postgres/
│   │   └── claude.md           # NEW: PostgreSQL-specific
│   └── datafusion/
│       └── claude.md           # NEW: DataFusion-specific
│
├── tests/
│   └── claude.md               # NEW: Testing conventions
│
├── internal/                   # KEEP as-is (working well)
│   └── [existing structure]
│
└── external/
    └── agent-contexts/         # INITIALIZE submodule
        └── [universal patterns]
```

### Key Files Purpose

| File | Purpose | Audience | Update Frequency |
|------|---------|----------|------------------|
| `CLAUDE.md` | Project context | All AI agents | Major milestones |
| `CLAUDE.local.md` | Personal prefs | Individual dev | As needed |
| `.claude/commands/*.md` | Reusable workflows | Team | When patterns emerge |
| `src/*/claude.md` | Module-specific | AI working on module | When module changes |
| `internal/STATUS_REPORT_*.md` | Comprehensive status | Deep context | Monthly |
| `external/agent-contexts/` | Universal patterns | Any project | Community driven |

---

## Updated CLAUDE.md Template (2025 Standard)

```markdown
# OmenDB - PostgreSQL-Compatible HTAP Database

**Version**: 2.0 (Multi-level ALEX production ready)
**Status**: October 2025 - Seeking customers
**Performance**: 1.5-3x faster than SQLite, scales to 100M+ rows
**Stack**: Rust, Multi-level ALEX, DataFusion, PostgreSQL wire protocol

---

## Quick Start

\`\`\`bash
# Build
cargo build --release

# Test
cargo test

# Benchmark
./target/release/benchmark_vs_sqlite 10000000

# Servers
./target/release/postgres_server    # Port 5433
\`\`\`

---

## Architecture

\`\`\`
Multi-level ALEX Index → PostgreSQL Protocol → DataFusion Query Engine
\`\`\`

**Key Components**:
- `src/alex/` - Multi-level learned index (production ready)
- `src/postgres/` - PostgreSQL wire protocol
- `src/datafusion/` - SQL query engine
- `src/table.rs` - Unified storage

**Deep dive**: See `ARCHITECTURE.md`

---

## Code Conventions

### Style
- **Tests**: Every new feature requires tests in `tests/`
- **Benchmarks**: Performance-critical changes need benchmarks
- **Docs**: Update docs alongside code changes

### Patterns
- **Error handling**: Use `Result<T>` with context via `anyhow`
- **Concurrency**: Prefer `RwLock` over `Mutex` for read-heavy
- **Memory**: Use `memory_pool.rs` for large allocations

### Testing
\`\`\`bash
cargo test                        # Unit tests
cargo test --release              # Integration tests
cargo clippy                      # Lints
\`\`\`

---

## Common Workflows

**For benchmarking**: `/benchmark` command or `./target/release/benchmark_vs_sqlite`
**For profiling**: `/profile` command or `./target/release/profile_query_path`
**For release**: `/release-checklist` command

**Custom commands**: See `.claude/commands/` directory

---

## Context Sources

**Quick reference**: This file
**Current status**: `internal/STATUS_REPORT_OCT_2025.md`
**Universal patterns**: `external/agent-contexts/`
**Module-specific**: `src/[module]/claude.md`

---

## Performance Characteristics

**Validated (October 2025)**:
- 1M-100M scale: 1.5-3x faster than SQLite ✅
- Memory: 1.50 bytes/key (28x better than PostgreSQL) ✅
- Latency: 628ns at 10M, 1.24μs at 100M ✅
- Durability: 100% crash recovery success ✅

**Source**: `internal/research/100M_SCALE_RESULTS.md`

---

## Current Focus (October 2025)

**Priority 1**: Customer acquisition (3-5 LOIs target)
**Priority 2**: Competitive benchmarks (CockroachDB, DuckDB)
**Priority 3**: Fundraising prep ($1-3M seed)

**Details**: `internal/STATUS_REPORT_OCT_2025.md`

---

## Team

**Solo developer**: Nick (AI-assisted via Claude Code)
**Hardware**: Fedora PC (i9-13900KF, 32GB, RTX 4090), Mac M3 Max
**Development approach**: Context engineering + test-driven

---

**Last Updated**: October 11, 2025
**Next Review**: After customer LOI acquisition (6 weeks)
```

---

## .gitignore Updates

Add to `.gitignore`:
```gitignore
# Personal Claude Code overrides
CLAUDE.local.md
**/claude.local.md

# Claude Code local settings
.claude/settings.local.json
```

---

## Custom Commands to Create

### 1. `.claude/commands/benchmark.md`

```markdown
Run full benchmark suite:

1. **Build release**:
   \`\`\`bash
   cargo build --release
   \`\`\`

2. **SQLite comparison** (10M scale):
   \`\`\`bash
   ./target/release/benchmark_vs_sqlite 10000000
   \`\`\`

3. **Check baseline**:
   - Compare to: `internal/research/ALEX_SQLITE_BENCHMARK_RESULTS.md`
   - Expected: 1.5-3x speedup
   - If <1.2x: Investigate regression

4. **Profile if needed**:
   \`\`\`bash
   ./target/release/profile_query_path
   \`\`\`

5. **Document results**:
   - If major change: Update STATUS_REPORT
   - If regression: Create issue
   - If improvement: Celebrate!
```

### 2. `.claude/commands/update-status.md`

```markdown
Update project status after milestone:

1. **Gather data**:
   - Recent commits: `git log --oneline -20`
   - Test status: `cargo test 2>&1 | tail -5`
   - Benchmark results: Latest run

2. **Update STATUS_REPORT**:
   - Open: `internal/STATUS_REPORT_OCT_2025.md`
   - Update "Recent Achievements" section
   - Update "What's Working" if applicable
   - Update "Next Steps" priorities

3. **Update CLAUDE.md**:
   - If major milestone: Update "Current Focus"
   - If performance changed: Update "Performance Characteristics"
   - Update "Last Updated" date

4. **Update internal/README.md**:
   - If phase completed: Add to "Key Milestones"
   - Update "Last Updated" date

5. **Commit all docs**:
   \`\`\`bash
   git add CLAUDE.md internal/
   git commit -m "docs: Update status after [milestone]"
   \`\`\`
```

### 3. `.claude/commands/competitive-analysis.md`

```markdown
Analyze competitor for positioning:

1. **Research**:
   - Check competitor website for latest features
   - Check GitHub for recent commits (if open source)
   - Check blog/changelog for announcements

2. **Benchmark** (if possible):
   - Docker setup for competitor
   - Equivalent workload (same features)
   - Document methodology clearly

3. **Document findings**:
   - Create: `internal/research/[COMPETITOR]_ANALYSIS_[DATE].md`
   - Include: Features, performance, positioning
   - Update: `internal/STATUS_REPORT_OCT_2025.md` competitive section

4. **Identify gaps**:
   - What do they have that we don't?
   - What do we have that they don't?
   - Where should we focus?
```

---

## Subdirectory Claude Files

### `src/alex/claude.md`

```markdown
# Multi-Level ALEX Context

**Purpose**: Hierarchical learned index, production ready
**Status**: Scales to 100M+ rows, 1.5-3x faster than SQLite

## Key Principles

1. **Fixed fanout**: 64 keys/leaf (cache-line optimized)
2. **Adaptive retraining**: Only when `needs_retrain()` returns true
3. **Height 2-3**: Optimal for 100M scale
4. **Gapped arrays**: O(1) inserts, no rebuilds

## Architecture

\`\`\`
Root (learned model)
  ↓
Inner nodes (routing, height 1-2)
  ↓
Leaf nodes (64 keys each, gapped arrays)
\`\`\`

## Common Operations

**Insert**: O(1) amortized via gapped arrays
**Query**: O(log n) via hierarchical routing
**Range**: O(k) where k = result size

## Implementation Files

- `multi_level.rs` - Main multi-level structure
- `alex_tree.rs` - Single-level base (legacy)
- `gapped_node.rs` - Gapped array implementation
- `linear_model.rs` - Learned model training
- `simd_search.rs` - SIMD-accelerated search (future)

## Performance Expectations

| Scale | Query | Memory | Build Time |
|-------|-------|--------|------------|
| 1M    | 628ns | 14MB   | 1.2s       |
| 10M   | 628ns | 14MB   | 12s        |
| 100M  | 1.24μs| 143MB  | 180s       |

## Testing

\`\`\`bash
cargo test alex                          # Unit tests
cargo test --release multi_level        # Integration
./target/release/benchmark_multi_level_alex  # Performance
\`\`\`

## References

- Design doc: `internal/design/MULTI_LEVEL_ALEX.md`
- Benchmarks: `internal/research/100M_SCALE_RESULTS.md`
- Paper: ALEX (Ding et al., 2020)
```

### `src/postgres/claude.md`

```markdown
# PostgreSQL Wire Protocol Context

**Purpose**: PostgreSQL compatibility layer
**Status**: Production ready, port 5433

## Protocol Flow

1. **Startup**: Client sends startup message
2. **Auth**: Simple/SASL authentication
3. **Query**: Simple or extended query protocol
4. **Response**: Return rows, error, or status
5. **Termination**: Clean connection close

## Implementation

- `server.rs` - TCP server, connection handling
- `handlers.rs` - Message parsing and routing
- `encoding.rs` - Type encoding/decoding
- `tests.rs` - Protocol compliance tests

## Key Types

\`\`\`rust
// Frontend messages (client → server)
StartupMessage, Query, Parse, Bind, Execute, Close, Terminate

// Backend messages (server → client)
AuthenticationOk, ReadyForQuery, RowDescription, DataRow, CommandComplete
\`\`\`

## Testing

\`\`\`bash
# Start server
./target/release/postgres_server

# Test with psql
psql -h localhost -p 5433 -d omendb

# Run protocol tests
cargo test postgres_integration
\`\`\`

## References

- PostgreSQL protocol spec: https://www.postgresql.org/docs/current/protocol.html
- Tests: `tests/postgres_integration_tests.rs`
```

### `tests/claude.md`

```markdown
# Testing Conventions

## Test Organization

\`\`\`
tests/
├── concurrency_tests.rs        # Multi-threaded tests
├── persistence_tests.rs        # WAL, crash recovery
├── learned_index_*_tests.rs    # ALEX validation
├── postgres_integration_tests.rs  # PostgreSQL compliance
└── rest_api_tests.rs           # REST API
\`\`\`

## Test Principles

1. **Isolation**: Each test is independent
2. **Cleanup**: Tests clean up after themselves
3. **Determinism**: No timing dependencies
4. **Coverage**: Aim for 80%+ on critical paths

## Common Patterns

### Setup/Teardown
\`\`\`rust
#[test]
fn test_name() {
    // Setup
    let table = create_test_table();

    // Test
    let result = table.operation();

    // Assert
    assert!(result.is_ok());

    // Cleanup (via Drop)
}
\`\`\`

### Concurrency Testing
\`\`\`rust
#[test]
fn test_concurrent() {
    let table = Arc::new(create_test_table());
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let table = Arc::clone(&table);
            thread::spawn(move || {
                // Concurrent operations
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}
\`\`\`

## Running Tests

\`\`\`bash
cargo test                      # All tests
cargo test --release            # Integration (faster)
cargo test [name]               # Specific test
cargo test -- --nocapture       # See println! output
cargo test -- --test-threads=1  # Serial execution
\`\`\`

## When Tests Fail

1. **Read output**: `cargo test [name] -- --nocapture`
2. **Check recent changes**: `git log -- [test_file]`
3. **Run in isolation**: `cargo test [name] -- --test-threads=1`
4. **Profile if slow**: `cargo flamegraph --test [name]`
```

---

## Implementation Plan

### Phase 1: Core Files (Today)

```bash
# 1. Update CLAUDE.md to 2025 standard
# (Use template above)

# 2. Add CLAUDE.local.md to .gitignore
echo "CLAUDE.local.md" >> .gitignore
echo "**/claude.local.md" >> .gitignore

# 3. Create .claude/commands/ directory
mkdir -p .claude/commands

# 4. Commit
git add CLAUDE.md .gitignore .claude/
git commit -m "feat: Update to 2025 AI context best practices"
```

### Phase 2: Custom Commands (This Week)

```bash
# Create command files
touch .claude/commands/benchmark.md
touch .claude/commands/update-status.md
touch .claude/commands/competitive-analysis.md
touch .claude/commands/release-checklist.md

# Populate with content from above
# Commit
git add .claude/commands/
git commit -m "feat: Add custom Claude commands for common workflows"
```

### Phase 3: Subdirectory Context (Next Week)

```bash
# Create subdirectory claude.md files
touch src/alex/claude.md
touch src/postgres/claude.md
touch tests/claude.md

# Populate with content from above
# Commit
git add src/*/claude.md tests/claude.md
git commit -m "feat: Add subdirectory-specific Claude context files"
```

### Phase 4: External Agent-Contexts (Ongoing)

```bash
# Initialize submodule
cd external/agent-contexts
git submodule update --init --recursive
cd ../..

# Or if URL needs updating
git submodule set-url external/agent-contexts https://github.com/nijaru/agent-contexts.git
git submodule update --init --recursive

# Commit
git add external/agent-contexts
git commit -m "feat: Initialize agent-contexts submodule"
```

---

## Summary: What Makes OmenDB Docs Great (2025 Standard)

### ✅ Already Doing Well

1. **internal/ organization** - Well-structured, topic-based
2. **STATUS_REPORT** - Comprehensive monthly updates
3. **CLAUDE.md exists** - Just needs modernization
4. **Clear separation** - Strategy vs code vs user docs

### 🔧 Recommended Additions (2025 Best Practice)

1. **Modernize CLAUDE.md** - Use 2025 template
2. **Add .claude/commands/** - Reusable workflows
3. **Add subdirectory claude.md** - Context-specific guidance
4. **Initialize agent-contexts** - Universal patterns
5. **Add CLAUDE.local.md** to .gitignore - Personal overrides

### 📊 Comparison to Industry Leaders

| Practice | Industry Standard | OmenDB Status |
|----------|-------------------|---------------|
| CLAUDE.md | Required | ✅ Have (needs update) |
| internal/ organization | Recommended | ✅ Excellent |
| Cascading contexts | Best practice | ⚠️ Not yet implemented |
| Custom commands | Best practice | ⚠️ Not yet implemented |
| MCP integration | Advanced | ⚠️ Not needed yet |
| Sub-agents | Advanced | ⚠️ Not needed yet |

### 🎯 Bottom Line

**Your current structure is excellent!** The main improvements are:

1. Modernize CLAUDE.md to 2025 standard (highest impact)
2. Add .claude/commands/ for team workflows (high impact)
3. Add subdirectory claude.md for focused context (medium impact)
4. Initialize agent-contexts submodule (nice to have)

**Priority**: Focus on #1 and #2, they'll have immediate impact.

---

**Last Updated**: October 11, 2025
**Sources**: Anthropic, GitHub, industry research
**Status**: Ready to implement
