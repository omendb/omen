# Agent-Contexts Submodule Organization

**Date**: October 11, 2025
**Purpose**: Plan for integrating agent-contexts as submodule

---

## Proposed Structure

```
omendb/core/
├── agent-contexts/              # Submodule (generic patterns)
│   ├── AI_AGENT_INDEX.md       # Universal decision trees
│   ├── ERROR_PATTERNS.md       # Common error solutions
│   ├── AI_CODE_PATTERNS.md     # Code organization
│   ├── DOC_PATTERNS.md         # Documentation structure
│   └── languages/
│       └── rust/
│           └── RUST_PATTERNS.md
│
├── CLAUDE.md                    # OmenDB-specific context
├── ARCHITECTURE.md              # OmenDB architecture
├── CONTRIBUTING.md              # OmenDB contribution guide
│
└── internal/                    # OmenDB-specific strategy
    ├── STATUS_REPORT_OCT_2025.md
    ├── README.md
    ├── business/                # OmenDB business strategy
    ├── research/                # OmenDB benchmarks
    └── technical/               # OmenDB technical docs
```

---

## What Goes Where

### agent-contexts/ (Submodule - Universal)

**Content**: Patterns that work for ANY project

**Should contain**:
- Generic decision trees (IF task X THEN do Y)
- Common error patterns and solutions
- Language-specific patterns (Rust, Python, etc.)
- Universal code organization patterns
- Documentation structure templates

**Examples from OmenDB to extract**:
```
✅ Generic patterns to contribute upstream:
- "IF: Performance regression THEN: git bisect + profile"
- "IF: Test failure THEN: check --nocapture, git log, fix"
- "IF: Adding benchmark THEN: [template]"
- Rust-specific patterns (cargo commands, error handling)

❌ Don't put OmenDB-specific things:
- "Check internal/STATUS_REPORT_OCT_2025.md"
- "Multi-level ALEX architecture patterns"
- "OmenDB competitive analysis"
```

### CLAUDE.md (Project Root - Project Context)

**Content**: High-level project context for AI

**Should contain**:
- Current status (October 2025)
- Architecture overview
- Performance characteristics
- Quick reference commands
- Links to: internal/ and agent-contexts/

**Update to reference submodule**:
```markdown
## For AI Assistants

**Generic patterns**: See `agent-contexts/AI_AGENT_INDEX.md`
**Project-specific**: See `internal/STATUS_REPORT_OCT_2025.md`
```

### internal/ (Project-Specific Strategy)

**Content**: OmenDB business, research, technical docs

**Should contain**:
- STATUS_REPORT_OCT_2025.md (comprehensive status)
- Competitive analysis
- Benchmark results
- Business strategy (funding, customers)
- Technical decisions specific to OmenDB

**Does NOT contain**:
- Generic coding patterns (those go in agent-contexts/)
- Universal error solutions (those go in agent-contexts/)

---

## Setup Instructions

### 1. Add agent-contexts as Submodule

```bash
cd /home/nick/github/omendb/core

# Add submodule
git submodule add https://github.com/nijaru/agent-contexts.git agent-contexts

# Initialize and update
git submodule init
git submodule update

# Commit
git add .gitmodules agent-contexts
git commit -m "feat: Add agent-contexts submodule for universal AI patterns"
```

### 2. Update CLAUDE.md to Reference Submodule

Add to CLAUDE.md:
```markdown
## AI Assistant Guidelines

**Universal patterns**: `agent-contexts/AI_AGENT_INDEX.md`
- Common error patterns
- Generic decision trees
- Rust-specific patterns

**OmenDB-specific**: `internal/STATUS_REPORT_OCT_2025.md`
- Current project status
- Performance characteristics
- Competitive positioning
- Next priorities
```

### 3. Extract Generic Patterns to Contribute Upstream

From `internal/AI_AGENT_GUIDE.md`, extract universal patterns:

**Candidates for agent-contexts contribution**:

1. **Rust benchmark pattern** → `languages/rust/BENCHMARKING.md`
```markdown
## Running Benchmarks in Rust

IF: Need to measure performance
THEN:
  1. cargo build --release
  2. hyperfine './target/release/[binary]'
  3. For profiling: cargo flamegraph --bin [binary]
```

2. **Rust test debugging pattern** → `languages/rust/TESTING.md`
```markdown
## Debugging Test Failures

IF: Test fails
THEN:
  1. cargo test [test_name] -- --nocapture
  2. git log -- [test_file] (check recent changes)
  3. Fix code or update test
  4. cargo test (verify all pass)
```

3. **Performance regression pattern** → `ERROR_PATTERNS.md`
```markdown
## Performance Regression

SYMPTOMS: Benchmark slower than expected

DIAGNOSIS:
1. git log --grep="perf" (find last baseline)
2. git bisect start HEAD [last_good]
3. Profile to identify bottleneck

FIX:
1. Reference profiling tools (flamegraph, perf)
2. Check cache effects (sequential vs random data)
3. Validate with A/B benchmark
```

### 4. Remove Duplicate Content

After contributing to agent-contexts:

```bash
# Remove the generic parts from internal/AI_AGENT_GUIDE.md
# Keep only OmenDB-specific decision trees:
# - Multi-level ALEX patterns
# - OmenDB benchmark commands
# - OmenDB documentation update triggers

# Or rename to internal/OMENDB_AGENT_GUIDE.md
# to make it clear it's project-specific
```

---

## Migration Plan

### Phase 1: Setup (Today)
- [x] Document current state (this file)
- [ ] Add agent-contexts as submodule
- [ ] Update CLAUDE.md to reference both
- [ ] Test that Claude can access submodule

### Phase 2: Extract Generic Patterns (Next Session)
- [ ] Review internal/AI_AGENT_GUIDE.md
- [ ] Identify universal patterns
- [ ] Fork nijaru/agent-contexts
- [ ] Contribute Rust patterns upstream
- [ ] Submit PR

### Phase 3: Cleanup (After PR merged)
- [ ] Update submodule to include your contributions
- [ ] Remove duplicate content from internal/
- [ ] Keep only OmenDB-specific patterns
- [ ] Update all docs to reference new structure

---

## Benefits

### For OmenDB
1. **Cleaner separation**: Generic vs project-specific
2. **Smaller context**: AI reads less redundant info
3. **Community patterns**: Benefit from others' contributions
4. **Consistent updates**: Upstream improvements flow down

### For agent-contexts Community
1. **Rust patterns**: Few Rust examples currently
2. **Benchmark patterns**: Performance-focused patterns
3. **Database patterns**: Learned index architecture patterns
4. **Real-world validation**: Patterns from production project

---

## Recommended Contributions to agent-contexts

### 1. Rust Language Patterns

**File**: `languages/rust/RUST_PATTERNS.md`

```markdown
## Rust Development Patterns

### Building
IF: Development iteration
THEN: cargo build (fast, unoptimized)

IF: Benchmarking or profiling
THEN: cargo build --release (slow, optimized)

### Testing
IF: All tests
THEN: cargo test

IF: Specific test with output
THEN: cargo test [name] -- --nocapture

IF: Integration tests only
THEN: cargo test --release

### Profiling
IF: CPU profiling
THEN: cargo flamegraph --bin [binary]

IF: Memory profiling
THEN: cargo valgrind --bin [binary]

IF: Detailed timing
THEN: cargo build --release && perf record ./target/release/[binary]
```

### 2. Performance Regression Pattern

**File**: `ERROR_PATTERNS.md` (add section)

```markdown
## Performance Regression

SYMPTOMS:
- Benchmark slower than baseline
- Tests pass but throughput decreased
- Memory usage increased

DIAGNOSIS:
1. Identify baseline:
   - git log --grep="perf"
   - git log --grep="benchmark"

2. Bisect:
   - git bisect start HEAD [last_good_commit]
   - git bisect run ./benchmark.sh

3. Profile:
   - Language-specific profiling (see languages/)
   - Compare hot paths: baseline vs current

FIX:
1. Identify hot path change
2. Optimize or revert
3. Add regression test to CI

PREVENT:
- Benchmark in CI on every commit
- Set performance budgets (fail if >10% regression)
- Profile before merging large changes
```

### 3. Database Development Patterns

**New file**: `domains/DATABASE_PATTERNS.md`

```markdown
## Database Development Patterns

### Benchmarking
IF: Comparing against competitor
THEN:
  1. Ensure same features (durability, ACID, etc.)
  2. Same workload (not best-case vs worst-case)
  3. Same data distribution (random/sequential/zipfian)
  4. Document what IS and ISN'T tested
  5. Report median of 3+ runs

### Scaling Tests
IF: Validating scale (1M → 10M → 100M)
THEN:
  1. Test at geometric intervals (1M, 10M, 100M, 1B)
  2. Check memory growth (should be O(n))
  3. Check latency growth (should be O(log n) for indexes)
  4. Test with realistic data (not all sequential)

### Durability Testing
IF: Need crash recovery validation
THEN:
  1. Insert data + flush
  2. Kill process (SIGKILL, not graceful)
  3. Restart + validate all data present
  4. Repeat 100+ times for confidence
  5. Test concurrent crashes
```

---

## Example: Updated CLAUDE.md Structure

```markdown
# OmenDB Development Context

## For AI Assistants

### Start Here
1. **Universal patterns**: `agent-contexts/AI_AGENT_INDEX.md`
   - Common errors and solutions
   - Rust development patterns
   - Generic decision trees

2. **OmenDB-specific**: `internal/STATUS_REPORT_OCT_2025.md`
   - Current project status (Oct 2025)
   - Multi-level ALEX architecture
   - Performance: 1.5-3x vs SQLite, 100M scale
   - Next: Customer acquisition, funding

### Quick Reference

**Generic Rust commands**: See `agent-contexts/languages/rust/`
**OmenDB benchmarks**: See internal doc or run:
```bash
./target/release/benchmark_vs_sqlite 10000000
./target/release/postgres_server  # Port 5433
```

**Documentation structure**:
- agent-contexts/ = Universal patterns (any project)
- CLAUDE.md = Project context (this file)
- internal/ = Strategy & status (OmenDB-specific)
- ARCHITECTURE.md = Technical architecture
- CONTRIBUTING.md = Code guidelines
```

---

## Next Steps

1. **Immediate**: Add submodule
   ```bash
   git submodule add https://github.com/nijaru/agent-contexts.git
   ```

2. **Update**: CLAUDE.md to reference submodule

3. **Test**: Claude can read agent-contexts/ files

4. **Extract**: Generic patterns from internal/AI_AGENT_GUIDE.md

5. **Contribute**: Fork agent-contexts, add Rust patterns, PR

6. **Cleanup**: Remove duplicates after PR merged

---

**Status**: Ready to execute
**Owner**: Nick
**Timeline**: Phase 1 today, Phase 2-3 over next week
**Benefit**: Cleaner docs, community contribution, better AI context
