# 📋 OmenDB Task Tracker
*Auto-updated by AI agents - Single source of truth*

## 🎯 Current Sprint (Week of 2025-02-04)

### 🔥 Critical Path
1. **Zero-copy FFI** [🔄 In Progress]
   - File: `omendb/engine/omendb/native.mojo:2000-2100`
   - Implement `__array_interface__` for numpy
   - Expected impact: 10x overhead reduction
   - Assignee: AI Agent
   - Due: 2025-02-05

2. **Async Buffer Manager** [📋 Ready]
   - File: `omendb/engine/omendb/native.mojo:1850-2000`
   - Pattern: `internal/patterns/CONCURRENCY_PATTERNS.md#L214`
   - Expected impact: Fix 25K bottleneck
   - Assignee: AI Agent
   - Due: 2025-02-06

3. **Batch API Default** [📋 Ready]
   - File: `omendb/engine/python/omendb/api.py`
   - Make batch operations primary interface
   - Expected impact: 5x FFI improvement
   - Due: 2025-02-05

## 📊 Task Board

### ✅ Done (Last 7 Days)
- [2025-02-04] Researched IP-DiskANN (state-of-art 2025)
- [2025-02-04] Analyzed Mojo vs Rust (decision: stay with Mojo)
- [2025-02-04] Found zero-copy FFI solution
- [2025-02-04] Added reference repos (DiskANN, Chroma, Lance)
- [2025-02-04] Extracted ZenDB patterns to internal/patterns/
- [2025-02-04] Created comprehensive action plan

### 🔄 In Progress
```yaml
zero_copy_ffi:
  status: implementing
  branch: fix/zero-copy-ffi
  progress: 20%
  blockers: none
  
benchmark_script:
  status: testing
  issue: Timeout on 10K vectors
  next: Fix batch insertion
```

### 📋 Backlog (Prioritized)

#### High Priority
- [ ] **Eliminate Global Singleton**
  - Impact: Fix test isolation issues
  - Effort: Medium (4-6 hrs)
  - Pattern: Use UniquePtr[VectorStore]

- [ ] **Compact Data Structures**
  - Impact: 100x memory reduction
  - Effort: High (8+ hrs)
  - Replace: Dict[String, Int] → CompactMap

- [ ] **Memory-Mapped Storage**
  - Impact: True zero-copy
  - Effort: Medium (4 hrs)
  - Pattern: ZenDB mmap implementation

#### Medium Priority  
- [ ] **Streaming Merge** (FreshDiskANN)
  - Impact: Continuous index updates
  - Effort: High (2 days)
  - Reference: `external/research/COMPETITOR_ANALYSIS.md`

- [ ] **In-Place Updates** (IP-DiskANN)
  - Impact: No buffer needed
  - Effort: Very High (1 week)
  - Paper: arXiv:2502.13826

- [ ] **GPU Kernels**
  - Impact: 10x compute speedup
  - Effort: High (3 days)
  - Mojo GPU package available

#### Low Priority
- [ ] Update server to latest patterns
- [ ] Refresh web content
- [ ] Add telemetry/monitoring
- [ ] Create Docker images
- [ ] Write user documentation

## 🐛 Bug Tracker

### 🚨 Critical
- **BUG-001**: Segfault on duplicate IDs
  - Workaround: Always call `db.clear()` 
  - Fix: Check ID existence before insert
  - File: `native.mojo:813`

### ⚠️ Major  
- **BUG-002**: 25K vector performance cliff
  - Status: Fix in progress (async buffer)
  - File: `native.mojo:1850-2000`

- **BUG-003**: Memory leak in buffer
  - Status: Investigation needed
  - Suspected: Buffer not cleared on flush

### 📝 Minor
- **BUG-004**: Incorrect error messages
- **BUG-005**: Missing type hints in Python API

## 💡 Ideas & Research

### Investigate
- [ ] FAISS GPU kernels integration
- [ ] Arrow format for vectors
- [ ] Rust PyO3 for better FFI
- [ ] WebAssembly compilation

### Research Papers
- [ ] IP-DiskANN (2025) - In-place updates
- [ ] StreamingDiskANN (2023) - Continuous merge
- [ ] FreshDiskANN (2021) - Async WAL

## 📈 Performance Targets

| Milestone | Target | Current | Gap |
|-----------|--------|---------|-----|
| **M1: Fix Bottleneck** | 100K vectors | 25K | 75K |
| **M2: Production** | 1M vectors | 25K | 975K |
| **M3: Scale** | 1B vectors | 25K | ~1B |

## 🔗 Quick Links

### Code Locations
- **Core Engine**: `omendb/engine/omendb/native.mojo`
- **Python API**: `omendb/engine/python/omendb/api.py`
- **Benchmarks**: `omendb/engine/benchmarks/`
- **Tests**: `omendb/engine/tests/`

### Documentation
- **Architecture**: `internal/ARCHITECTURE.md`
- **Patterns**: `internal/patterns/`
- **Decisions**: `internal/DECISIONS.md`
- **Error Fixes**: `ERROR_FIXES.md`

### External References
- **DiskANN**: `external/diskann/`
- **Competitors**: `external/competitors/`
- **Agent Patterns**: `external/agent-contexts/`

## 📝 Git Workflow

```bash
# Start new task
git checkout -b fix/task-name

# Regular commits
git add -A
git commit -m "fix: implement zero-copy FFI for numpy arrays"

# Push when ready
git push -u origin fix/task-name

# Create PR with template
gh pr create --title "Fix: Zero-copy FFI" --body "..."
```

## 🤖 AI Agent Instructions

### When Starting Work
1. Check this file for current sprint tasks
2. Review `ACTION_PLAN.md` for implementation details
3. Look for `[🔄 In Progress]` tasks to continue
4. Check `ERROR_FIXES.md` for known issues

### When Completing Tasks
1. Update status in this file
2. Move task to "Done" section with date
3. Update progress percentages
4. Document any new patterns in `internal/patterns/`

### When Finding Bugs
1. Add to Bug Tracker with BUG-XXX ID
2. Document workaround if available
3. Link to relevant code location
4. Update `ERROR_FIXES.md` if common

---
*Last AI Update: 2025-02-04 by Claude*
*Next Sprint Planning: 2025-02-11*