# Repository Restructure Review

**Date:** October 1, 2025
**Purpose:** Comprehensive review before flattening omendb-rust/ to root

---

## 🔍 Current State Analysis

### Directory Structure

```
/Users/nick/github/omendb/core/  (git repo)
├── src/                          ⚠️ OLD experimental code (14 files)
│   ├── lib.rs, rmi.rs, linear.rs, error.rs
│   ├── Last commit: Sep 26 (5ad8c03)
│   └── Simple RMI prototype
│
├── omendb-rust/                  ✅ PRODUCTION codebase (48 files)
│   ├── src/                      🚀 Active development (Oct 1)
│   │   ├── postgres/ (559 lines, NEW)
│   │   ├── catalog.rs, sql_engine.rs, etc.
│   │   └── 182/195 tests passing
│   ├── Cargo.toml (169 lines)    ✅ Full dependencies
│   ├── README.md (495 lines)     ✅ Complete documentation
│   └── All markdown docs (20 files)
│
├── Cargo.toml (36 lines)         ⚠️ Minimal, experimental
├── README.md (242 lines)         ⚠️ Monorepo overview
├── python/                       ⚠️ Python experiments
├── learneddb/                    ⚠️ Another Rust experiment
├── mvp/                          ⚠️ Flask app prototype
└── Other files (scripts, configs)
```

### File Counts

| Location | Rust Files | Status | Last Update |
|----------|-----------|---------|-------------|
| `src/` | 14 | Experimental | Sep 26 |
| `omendb-rust/src/` | 48 | Production | Oct 1 (today) |

### Key Differences

**Cargo.toml:**
- Root: 36 lines, minimal deps (ndarray, serde, rayon)
- omendb-rust/: 169 lines, full stack (datafusion, pgwire, axum, redb)

**README.md:**
- Root: 242 lines, monorepo overview
- omendb-rust/: 495 lines, complete database documentation with benchmarks

**Source Code:**
- Root src/: Simple RMI prototype, no tests, no PostgreSQL support
- omendb-rust/src/: Full database with WAL, catalog, SQL, PostgreSQL, 182 tests

---

## ⚠️ Conflicts to Resolve

### File Conflicts (will overwrite if not careful)

1. **src/** (14 files OLD vs 48 files NEW)
   - ❌ CANNOT merge - completely different codebases
   - ✅ Solution: Rename root src/ → archive/experiments/rust_prototype/

2. **Cargo.toml** (36 lines vs 169 lines)
   - ❌ Root version is outdated
   - ✅ Solution: Keep omendb-rust/Cargo.toml, archive root version

3. **README.md** (242 lines vs 495 lines)
   - ❌ Root version is monorepo overview
   - ✅ Solution: Keep omendb-rust/README.md, archive root version

4. **target/** (build artifacts)
   - Both exist, but can be regenerated
   - ✅ Solution: Delete both, rebuild after restructure

---

## 📋 Safe Restructure Plan

### Phase 1: Create Archive (SAFE - no overwrites)

```bash
cd /Users/nick/github/omendb/core

# Archive old experimental code
mkdir -p archive/experiments/{rust_prototype,python_experiments,learneddb_experiment,mvp_prototype}

git mv src archive/experiments/rust_prototype/
git mv python archive/experiments/python_experiments/
git mv learneddb archive/experiments/learneddb_experiment/
git mv mvp archive/experiments/mvp_prototype/

# Archive old root configs
mkdir -p archive/old_root/
git mv Cargo.toml archive/old_root/Cargo.toml.old
git mv README.md archive/old_root/README.md.old
git mv pyproject.toml archive/old_root/
git mv proof_of_concept.py archive/old_root/
git mv test_hot_cold_architecture.py archive/old_root/
git mv *.sh archive/old_root/  # setup_monorepo.sh, start_standalone.sh

# Archive other experimental files
git mv mojo_learned_index.mojo archive/experiments/
git mv README_PYTHON.md archive/experiments/
git mv FFI_BRIDGE_DESIGN.md archive/old_root/
git mv MOJO_ARCHITECTURE_ANALYSIS.md archive/old_root/
git mv PRODUCTION_READINESS_PLAN.md archive/old_root/
```

### Phase 2: Flatten omendb-rust/ (after Phase 1 complete)

```bash
# Now safe to move - no conflicts
cd /Users/nick/github/omendb/core

# Move all omendb-rust contents to root
git mv omendb-rust/src .
git mv omendb-rust/Cargo.toml .
git mv omendb-rust/Cargo.lock .

# Move all markdown docs
git mv omendb-rust/*.md .

# Move other important directories
git mv omendb-rust/docs .
git mv omendb-rust/k8s .

# Clean up target (build artifacts, regenerate later)
rm -rf omendb-rust/target
rm -rf target

# Remove empty omendb-rust directory
rmdir omendb-rust
```

### Phase 3: Organize Documentation

```bash
# Archive temporary session docs
mkdir -p archive/docs/{sessions,verification}
git mv SESSION_SUMMARY*.md archive/docs/sessions/
git mv VERIFICATION*.md BUGS_FOUND.md TIER1_PROGRESS.md V0.2.0_VERIFICATION_REPORT.md archive/docs/verification/

# Move specialized docs to proper locations
mkdir -p docs/audits docs/operations
git mv ERROR_HANDLING_AUDIT.md docs/audits/
git mv STRUCTURED_LOGGING.md docs/operations/
```

### Phase 4: Update Remaining References

After flattening, update these files:
- `.github/workflows/*` - Update paths if any reference omendb-rust/
- `CLAUDE.md` - Update any path references
- `docs/` - Update any cross-references

---

## ✅ Safety Checks

### Before Starting

1. ✅ All changes committed (git status clean)
2. ✅ Create git branch for safety: `git checkout -b restructure-flatten`
3. ✅ Confirm omendb-rust/ is the active codebase (verified above)
4. ✅ Verify backup exists: `git log` shows all commits preserved

### During Execution

1. ✅ Use `git mv` (not `mv`) - preserves history
2. ✅ Archive first (Phase 1) - no overwrites possible
3. ✅ Flatten second (Phase 2) - only after archive complete
4. ✅ Test build after each phase: `cargo check`

### After Completion

1. ✅ Verify all tests still pass: `cargo test`
2. ✅ Verify postgres_server builds: `cargo build --bin postgres_server`
3. ✅ Check git status: No untracked deletions
4. ✅ Review with: `git diff --stat main restructure-flatten`

---

## 🎯 Expected Final Structure

```
/Users/nick/github/omendb/core/  (can rename repo to "omendb")
├── src/                         ✅ Production code (from omendb-rust/src)
│   ├── postgres/
│   ├── catalog.rs
│   ├── sql_engine.rs
│   └── ... (all 48 files)
├── Cargo.toml                   ✅ Production config (169 lines)
├── README.md                    ✅ Full documentation (495 lines)
├── docs/
│   ├── audits/
│   ├── operations/
│   └── runbooks/
├── archive/
│   ├── experiments/             📦 Old code
│   │   ├── rust_prototype/      (old src/)
│   │   ├── python_experiments/
│   │   ├── learneddb_experiment/
│   │   └── mvp_prototype/
│   ├── docs/                    📦 Temporary docs
│   │   ├── sessions/
│   │   └── verification/
│   └── old_root/                📦 Old configs
├── benchmarks/                  ✅ Keep (important)
├── external/                    ✅ Keep (dependencies)
├── internal/                    ✅ Keep (strategy docs)
├── k8s/                         ✅ Keep (deployment)
└── CLAUDE.md                    ✅ Keep (project context)
```

### What Gets Archived (~15 directories/files)

**Experimental code:**
- src/ (old prototype)
- python/, learneddb/, mvp/
- mojo_learned_index.mojo
- proof_of_concept.py, test_hot_cold_architecture.py

**Old configs:**
- Root Cargo.toml, README.md
- pyproject.toml, *.sh scripts
- FFI_BRIDGE_DESIGN.md, MOJO_ARCHITECTURE_ANALYSIS.md

**Temporary docs:**
- SESSION_SUMMARY*.md (3 files)
- VERIFICATION*.md (4 files)
- BUGS_FOUND.md, TIER1_PROGRESS.md

**Total archived:** ~2,500+ lines of old/temporary content

### What Stays Active (~8,000 lines)

**Core docs (keep in root):**
- README.md (495 lines)
- QUICKSTART.md (466 lines)
- PROJECT_STATUS.md (383 lines)
- PERFORMANCE.md (362 lines)
- LIBRARY_DECISIONS.md (479 lines)
- DATAFUSION_MIGRATION.md (419 lines)
- ARCHITECTURE_LIMITATIONS.md (298 lines)
- PGWIRE_NOTES.md (268 lines)
- WEEK1_SUMMARY.md (462 lines)
- REPO_CLEANUP_PLAN.md (new)

**Source code:**
- src/ (48 files, ~8,000+ lines)
- 182/195 tests
- Full PostgreSQL wire protocol
- DataFusion SQL engine

---

## 🚀 Execution Plan

### Step 1: Create Safety Branch
```bash
cd /Users/nick/github/omendb/core
git checkout -b restructure-flatten
git status  # Verify clean
```

### Step 2: Execute Phase 1 (Archive)
Run all Phase 1 commands, then:
```bash
git status  # Review changes
git add -A
git commit -m "chore: Archive experimental code and old configs"
cargo check  # Verify omendb-rust/ still builds
```

### Step 3: Execute Phase 2 (Flatten)
Run all Phase 2 commands, then:
```bash
git status  # Review changes
git add -A
git commit -m "refactor: Flatten omendb-rust/ to root directory"
cargo check  # Verify builds with new structure
```

### Step 4: Execute Phase 3 (Organize)
Run all Phase 3 commands, then:
```bash
git status  # Review changes
git add -A
git commit -m "docs: Organize documentation into archive/"
```

### Step 5: Validate
```bash
cargo test --lib  # All tests pass
cargo build --bin postgres_server  # Binary builds
git log --oneline -5  # Review commits
git diff --stat main restructure-flatten  # Review changes
```

### Step 6: Merge to Main (if validation passes)
```bash
git checkout main
git merge restructure-flatten
git branch -d restructure-flatten
git push origin main
```

---

## ⚠️ Rollback Plan

If anything goes wrong:

```bash
# Return to main branch
git checkout main

# Delete restructure branch
git branch -D restructure-flatten

# Start over with review
```

All changes are on a branch - main branch is untouched until final merge.

---

## 🎯 Benefits After Restructure

1. **Simpler structure:** One src/, one Cargo.toml, one README
2. **Cleaner root:** No experimental code mixed with production
3. **Clear history:** Archived code preserved with git history
4. **Less confusion:** No "omendb-rust" subdirectory (it's all Rust!)
5. **Standard layout:** Matches typical Rust project structure
6. **Easier to navigate:** All active code in obvious locations

---

## ✅ Ready to Proceed?

**All checks passed:**
- ✅ Identified active vs experimental code
- ✅ No data loss (everything archived or kept)
- ✅ Safety branch strategy
- ✅ Rollback plan
- ✅ Detailed validation steps
- ✅ Preserves git history

**Estimated time:** 15-20 minutes

**Risk:** Low (on separate branch, easy rollback)

**Recommendation:** Proceed with restructure.
