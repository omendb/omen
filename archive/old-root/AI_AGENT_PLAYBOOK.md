# 🤖 AI Agent Playbook for OmenDB
*Optimized workflow for Claude and other AI agents*

## 🚀 Session Start Checklist

```bash
# 1. Orient yourself (30 seconds)
cat CLAUDE.md           # Navigation & current state
tail -50 SESSION_LOG.md # What happened last session
cat ACTION_PLAN.md      # Current priorities

# 2. Check status (30 seconds)  
grep "In Progress" TASKS.md     # Active work
grep "🚨 Critical" TASKS.md     # Urgent bugs
git status                       # Uncommitted changes

# 3. Pick up where we left off
# The last SESSION_LOG entry tells you exactly what to do next
```

## 📖 Document Navigation Map

```mermaid
CLAUDE.md (START HERE)
    ├── ACTION_PLAN.md ← "What should I work on?"
    ├── TASKS.md ← "What's the full backlog?"
    ├── SESSION_LOG.md ← "What was done before?"
    ├── DISCOVERIES.md ← "What did we learn?"
    ├── ERROR_FIXES.md ← "How do I fix X?"
    └── internal/
        ├── patterns/ ← "Proven solutions"
        └── ARCHITECTURE.md ← "How does it work?"
```

## 🎯 Execution Strategy

### Phase 1: Performance Crisis (THIS WEEK)
**Goal**: Fix the 25K bottleneck that's blocking everything

#### Task 1: Zero-Copy FFI (2 hours)
```bash
# Start
echo "$(date) | Starting zero-copy FFI implementation" >> SESSION_LOG.md

# Code location
vim omendb/engine/omendb/native.mojo +2000

# Implementation (already researched)
# See: DISCOVERIES.md entry from 2025-02-04
# Pattern in: external/modular/mojo/docs/manual/python/types.mdx

# Test
cd omendb/engine
pixi run benchmark-quick

# Complete
# Update TASKS.md: Move to Done
# Update SESSION_LOG.md: Record completion
```

#### Task 2: Async Buffer Manager (4 hours)
```bash
# Location
vim omendb/engine/omendb/native.mojo +1850

# Pattern from Chroma WAL v3
# See: internal/patterns/CONCURRENCY_PATTERNS.md:214

# Implementation steps:
1. Create AsyncBufferManager struct
2. Add double buffering
3. Use Thread.spawn for async flush
4. Test with 100K vectors
```

#### Task 3: Batch API (1 hour)
```bash
# Make batch operations the default
vim omendb/engine/python/omendb/api.py

# Change single add() to accumulate
# Flush batch at size threshold
```

### Phase 2: Architecture Cleanup (NEXT WEEK)
- [ ] Eliminate global singleton → UniquePtr[VectorStore]
- [ ] Replace Dict/List with compact structures
- [ ] Implement memory-mapped storage

### Phase 3: State-of-Art Features (WEEK 3)
- [ ] FreshDiskANN streaming merge
- [ ] IP-DiskANN in-place updates (if time)
- [ ] GPU acceleration with Mojo's GPU package

## 🔧 Common Workflows

### Starting a New Feature
```bash
# 1. Create branch
git checkout -b feature/async-buffer

# 2. Update TASKS.md
# Move task to "In Progress"

# 3. Write failing test first
vim tests/test_async_buffer.mojo

# 4. Implement feature
vim omendb/engine/omendb/native.mojo

# 5. Test locally
pixi run test-core

# 6. Benchmark if performance related
pixi run benchmark-standard

# 7. Update docs
echo "Discovery about X" >> DISCOVERIES.md
```

### Debugging a Crash
```bash
# 1. Check known issues
grep -A5 "Segfault" ERROR_FIXES.md

# 2. Enable stack traces
export MOJO_ENABLE_STACK_TRACE_ON_ERROR=1
mojo build -debug-level=line-tables

# 3. Run with debugging
mojo debug native.mojo

# 4. Document the fix
echo "## $(date) | Fixed segfault in X" >> DISCOVERIES.md
```

### End of Session
```bash
# 1. Update SESSION_LOG.md
cat >> SESSION_LOG.md << EOF

## $(date) | Claude | <Session Title>

### Completed
- ✅ Task 1
- ✅ Task 2

### Discovered
- Important finding

### Blocked On
- Any blockers

### Next Session Should
- Continue with X
- Start Y

---
EOF

# 2. Update TASKS.md statuses

# 3. Commit everything
git add -A
git commit -m "session: implement zero-copy FFI and async buffer

- Reduced FFI overhead from 8.3KB to 50 bytes per vector
- Implemented async buffer flush to fix 25K bottleneck
- Updated documentation with discoveries"

# 4. Push if on feature branch
git push -u origin feature/current-branch
```

## 🚨 Critical Rules

### DO
- ✅ Always read CLAUDE.md first
- ✅ Check SESSION_LOG.md for context
- ✅ Update TASKS.md in real-time
- ✅ Document discoveries immediately
- ✅ Test after every change
- ✅ Commit frequently with clear messages

### DON'T
- ❌ Start coding without checking ACTION_PLAN.md
- ❌ Create new files without updating docs
- ❌ Leave sessions without updating SESSION_LOG.md
- ❌ Ignore ERROR_FIXES.md when debugging
- ❌ Make large changes without tests

## 📊 Success Metrics

Track these in SESSION_LOG.md:

- **Vectors handled**: Current vs target (25K → 1M)
- **FFI overhead**: Bytes per vector (8.3KB → 50 bytes)
- **Build rate**: Vectors/second (10K → 40K)
- **Query QPS**: Queries/second (unknown → 8000)
- **Memory usage**: Bytes per vector (288 → maintain)

## 🔗 Quick Command Reference

```bash
# Build
cd omendb/engine
pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib

# Test
pixi run test-core
pixi run benchmark-quick    # 1K-10K vectors
pixi run benchmark-standard # 1K-100K vectors

# Debug
export MOJO_ENABLE_STACK_TRACE_ON_ERROR=1
mojo debug native.mojo

# Format
mojo format ./

# Search patterns
grep -r "pattern" internal/patterns/
rg "TODO|FIXME|XXX" omendb/engine/

# View recent changes
git log --oneline -10
git diff HEAD~1
```

## 🎬 Ready to Start?

1. You've read this playbook ✓
2. You know the priority (fix 25K bottleneck) ✓  
3. You know where to start (zero-copy FFI) ✓
4. You have the pattern (in DISCOVERIES.md) ✓

**Let's begin!**

---
*This playbook is optimized for AI agents. Update it when you discover better workflows.*