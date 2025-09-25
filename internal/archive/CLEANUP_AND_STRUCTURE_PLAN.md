# Documentation Cleanup & Structure Plan

**Date**: September 25, 2025
**YC Deadline**: November 10, 2025 (45 days)

## 1. External/ Cleanup Plan ✅

### Remove (Not needed for learned DB):
```bash
git rm -rf external/diskann         # Vector algorithm, not relevant
git rm -rf external/modular         # Mojo reference, using Rust
git rm -rf external/competitors     # Vector DB competitors
git rm -rf external/benchmarks      # Vector search benchmarks
```

### Keep:
- `external/agent-contexts/` - Essential for AI automation

### Add for Learned DB:
```bash
# Learned index references
git submodule add https://github.com/learnedsystems/RMI external/learned-systems/rmi
git submodule add https://github.com/learnedsystems/SOSD external/learned-systems/sosd
git submodule add https://github.com/learnedsystems/RadixSpline external/learned-systems/radix-spline

# Benchmark frameworks
git submodule add https://github.com/learnedsystems/SOSD external/benchmarks/sosd
```

## 2. Internal/ Documentation Consolidation 📁

### Current Problems:
- Too many overlapping files (STRATEGIC_DECISION, PIVOT_DECISION, etc.)
- No clear separation between business/technical
- Outdated vector DB content mixed with new pivot

### Proposed Structure:
```
internal/
├── ARCHITECTURE.md        # Core technical design (single source of truth)
├── BUSINESS.md            # Strategy, market, investor pitch
├── STATUS.md              # Current metrics, blockers, progress
├── ROADMAP.md             # Timeline, milestones, deadlines
├── archive/               # All old vector DB docs
│   ├── vector-db/         # Previous work
│   └── research/          # Historical research
└── decisions/             # Important decisions with dates
    ├── 2025-09-25-pivot-to-learned.md
    └── 2025-09-XX-embedded-vs-server.md
```

### Files to Consolidate:
```bash
# Merge into ARCHITECTURE.md:
- LEARNED_DB_TECHNICAL_SPEC.md
- Parts of PIVOT_DECISION.md (technical sections)

# Merge into BUSINESS.md:
- STRATEGIC_DECISION.md
- RESEARCH_FINDINGS_AND_RECOMMENDATIONS.md
- Parts of PIVOT_DECISION.md (business sections)

# Merge into ROADMAP.md:
- LEARNEDDB_IMPLEMENTATION_ROADMAP.md
- IMMEDIATE_ACTION_PLAN.md
- TODO.md

# Archive (move to archive/):
- MOJO_25.6_FFI_LIMITATIONS.md
- MOJO_25.6_MIGRATION_PLAN.md
- BULK_CONSTRUCTION_BREAKTHROUGH.md
- Old RESEARCH.md
```

## 3. Optimal AI Agent Structure 🤖

### Core Files for AI Context:

```markdown
# CONTEXT.md (New - AI Agent Instructions)
This file tells AI agents exactly what we're building and how to help.
- Project: Learned database replacing B-trees with ML
- Language: Rust
- Architecture: PostgreSQL extension → Standalone → Cloud
- Timeline: YC Nov 10 deadline
- Current focus: RMI prototype

# ARCHITECTURE.md (Technical Truth)
- System design
- Data structures
- Algorithms
- Performance targets
- Technical decisions

# BUSINESS.md (Strategy & Pitch)
- Market analysis
- Competition
- Go-to-market
- Revenue model
- YC pitch

# STATUS.md (Current State)
- What works
- What's broken
- Blockers
- Metrics
- Daily progress

# ROADMAP.md (What's Next)
- Week-by-week plan
- Milestones
- Success metrics
- Go/no-go points
```

### Why This Structure Works:
1. **No Overlap**: Each file has clear purpose
2. **AI-Friendly**: Easy to load context
3. **Human-Friendly**: Know where to look
4. **Version Control**: Clean git history

## 4. Architecture Decisions 🏗️

### Deployment Modes (Like DuckDB)

**Yes, we should do both embedded and server mode:**

```rust
// Mode 1: Embedded (like SQLite/DuckDB)
use omendb::LearnedDB;
let db = LearnedDB::open("mydata.omen")?;
let result = db.query("SELECT * FROM users WHERE id = 123")?;

// Mode 2: PostgreSQL Extension (immediate path)
CREATE EXTENSION omendb_learned;
CREATE INDEX learned_idx ON users USING learned(id);

// Mode 3: Standalone Server (future)
omendb-server --port 5432 --data-dir /var/omendb
```

### Why Multiple Modes:

**Embedded Mode** (Priority 2):
- For data science workflows
- Python/R integration
- No server overhead
- DuckDB competitor

**PostgreSQL Extension** (Priority 1):
- Immediate adoption path
- Production trust
- Easy benchmarking
- Drop-in replacement

**Server Mode** (Priority 3):
- Cloud deployment
- Multi-tenant
- Distributed future

### Technical Architecture:
```
┌─────────────────────────────────────┐
│         Client APIs                 │
│  (Python, Rust, Node.js, SQL)       │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│      Query Processor                │
│  (SQL Parser, Optimizer, Executor)  │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│     Learned Index Layer             │
│  (RMI, RadixSpline, ALEX)          │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│      Storage Engine                 │
│  (Columnar, Memory-mapped, S3)      │
└─────────────────────────────────────┘
```

## 5. Critical Research Needed 🔍

### Before Starting Implementation:

**1. Index Update Strategy**
```bash
# Research question: How to handle inserts?
- Delta buffer approach (like LSM tree)
- Periodic retraining
- Hybrid learned + B-tree
```

**2. Query Optimization**
```bash
# Can we learn query patterns too?
- Learned cardinality estimation
- Learned join order
- Learned cost model
```

**3. Storage Format**
```bash
# Columnar or row-based?
- Apache Arrow for interop?
- Custom format optimized for learned indexes?
- Parquet compatibility?
```

**4. Concurrency Model**
```bash
# MVCC or locking?
- Read-heavy optimization
- Write amplification concerns
- Transaction isolation levels
```

## 6. Business Documentation Needed 📊

### For YC Application:

```markdown
# internal/business/YC_APPLICATION.md
- Problem statement
- Solution
- Market size
- Competition analysis
- Team
- Traction plan

# internal/business/INVESTOR_DECK.md
- 10 slides
- Demo video script
- FAQ preparation

# internal/business/PRICING_MODEL.md
- Open source core
- Cloud pricing
- Enterprise features
```

## 7. Immediate Actions (Today)

```bash
# 1. Clean up external/
git rm -rf external/diskann external/modular external/competitors
git add external/learned-systems

# 2. Archive old docs
mkdir -p internal/archive/vector-db
mv internal/MOJO*.md internal/archive/vector-db/
mv internal/BULK*.md internal/archive/vector-db/

# 3. Consolidate docs
# Create new clean structure
cat internal/LEARNED_DB_TECHNICAL_SPEC.md >> internal/ARCHITECTURE.md
cat internal/STRATEGIC_DECISION.md >> internal/BUSINESS.md
cat internal/LEARNEDDB_IMPLEMENTATION_ROADMAP.md >> internal/ROADMAP.md

# 4. Create AI context file
echo "# OmenDB AI Context" > internal/CONTEXT.md
echo "Building learned database in Rust..." >> internal/CONTEXT.md

# 5. Research critical unknowns
# Read papers on update handling
# Study DuckDB embedded architecture
# Analyze PostgreSQL extension framework
```

## 8. What Else to Decide

### Technical Decisions:
- [ ] **Storage format**: Arrow vs custom
- [ ] **Retraining strategy**: Online vs batch
- [ ] **Model types**: Linear only or neural nets too?
- [ ] **GPU support**: Worth it for inference?
- [ ] **Distribution**: Single-node first or plan for sharding?

### Business Decisions:
- [ ] **Open source license**: MIT vs Apache vs BSL
- [ ] **Company structure**: Delaware C-corp now or wait?
- [ ] **Co-founder equity**: 50/50 or vesting?
- [ ] **Initial customers**: Who to target first?

### Product Decisions:
- [ ] **SQL compatibility**: Full SQL or subset?
- [ ] **Wire protocol**: PostgreSQL compatible?
- [ ] **First benchmark**: TPC-H or custom?
- [ ] **Language bindings**: Which first after Rust?

## 9. Research Papers to Read

### Essential (Read Today):
1. "The Case for Learned Index Structures" (Google, 2018)
2. "SOSD: A Benchmark for Learned Indexes" (2021)
3. "RadixSpline: A Single-Pass Learned Index" (2020)

### Important (This Week):
4. "ALEX: An Updatable Learned Index" (2020)
5. "The PGM-index" (2020)
6. "CDFShop: Exploring Learned Index Structures" (2020)

### Reference (As Needed):
7. "Learned Cardinalities" (2019)
8. "Neo: A Learned Query Optimizer" (2019)
9. "FITing-Tree: A Data-aware Index" (2020)

## 10. Timeline Reality Check

### We Have 45 Days Until YC (Nov 10)

**Week 1-2**: Prototype + Research
**Week 3-4**: PostgreSQL Extension
**Week 5-6**: Benchmarks + Demo
**Week 7**: YC Application + Video

### Daily Time Allocation:
- 50% coding
- 20% research
- 20% benchmarking
- 10% documentation

### Success Metrics by Oct 10:
- [ ] RMI working with 10x performance
- [ ] PostgreSQL extension functional
- [ ] TPC-H benchmark results
- [ ] Demo video recorded
- [ ] 100+ GitHub stars

## Conclusion

The path is clear but we need to:
1. **Clean up immediately** - Remove vector DB artifacts
2. **Consolidate docs** - 4 core files max
3. **Decide on architecture** - Embedded + Extension + Server
4. **Research unknowns** - Update handling critical
5. **Start coding** - 45 days is tight

---

*"Clarity is speed. Clean structure enables fast execution."*