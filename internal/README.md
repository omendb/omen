# Internal Documentation

**Last Updated:** October 2, 2025
**Status:** All docs current and consolidated

---

## 🎯 Quick Start

### New to the project?
**Read:** `business/EXECUTIVE_SUMMARY.md`
- Three viable business paths (Algorithm-first, Feature-first, Hybrid)
- System architectures for each path
- Decision framework: Run benchmarks Week 1, decide which path based on results

### Applying to YC W25 (deadline Nov 10)?
**Read:** `business/YC_W25_ROADMAP.md`
- 5-week sprint plan
- What YC actually wants (traction benchmarks from 2024-2025)
- Realistic monetization ($9-29/month, not $299)
- Week-by-week tasks and success criteria

### Need current technical status?
**Read:** `technical/CURRENT_STATUS.md`
- Latest development status
- What's working (218 tests passing, 2,862x speedup at 100K rows)
- What's missing (pgvector, scale validation, benchmarks)

---

## Directory Structure

```
internal/
├── README.md                     # This file
├── business/                     # Strategy, funding, market (4 docs)
│   ├── EXECUTIVE_SUMMARY.md     # ⭐ START HERE - Consolidated strategy
│   ├── YC_W25_ROADMAP.md        # 5-week plan for Nov 10 deadline
│   ├── VC_FUNDING_STRATEGY_OCT_2025.md
│   └── LICENSING_STRATEGY.md
├── technical/                    # Architecture, systems (10 docs)
│   ├── CURRENT_STATUS.md        # Latest project status
│   ├── COMPREHENSIVE_ARCHITECTURAL_REVIEW_OCT_2025.md
│   └── [8 other technical docs]
└── research/                     # Learned index validation (4 docs)
    ├── LEARNED_INDEX_VALIDATION.md
    └── [3 other research docs]
```

---

## Document Guide

### Business Strategy (internal/business/)

| Document | Purpose | Date | Read If... |
|----------|---------|------|-----------|
| **EXECUTIVE_SUMMARY.md** ⭐ | Consolidated strategy with 3 paths | Oct 2 | You need overview of viable strategies |
| **YC_W25_ROADMAP.md** | 5-week tactical plan | Oct 2 | Applying to YC by Nov 10 |
| **VC_FUNDING_STRATEGY_OCT_2025.md** | Detailed funding analysis | Oct 2 | Need deep dive on monetization/funding |
| **LICENSING_STRATEGY.md** | Open source licensing strategy | Sep 29 | Deciding on license (ELv2 vs AGPL vs Apache) |

**All 4 docs are current. No conflicts.**

### Technical Architecture (internal/technical/)

| Document | Purpose | Date | Read If... |
|----------|---------|------|-----------|
| **CURRENT_STATUS.md** | Latest development status | Oct 2 | You need current state |
| **COMPREHENSIVE_ARCHITECTURAL_REVIEW_OCT_2025.md** | Complete system analysis | Oct 2 | Need deep technical review |
| ARCHITECTURE_REFACTOR.md | Architecture planning | Sep 29 | Planning refactor |
| DATAFUSION_OPTIMIZATION_PLAN.md | Query optimization | Oct 2 | Working on DataFusion |
| PRODUCTION_READINESS_ASSESSMENT.md | Production checklist | Oct 1 | Preparing for production |
| TESTING_REQUIREMENTS.md | Test strategy | Oct 1 | Writing tests |
| OBSERVABILITY_GUIDE.md | Logging/monitoring | Oct 1 | Adding observability |
| OBSERVABILITY_PLAN.md | Observability roadmap | Oct 1 | Planning observability |
| ERROR_HANDLING_AUDIT.md | Error handling review | Oct 1 | Auditing error handling |
| TECH_STACK.md | Technology choices | Sep 30 | Understanding tech stack |

### Research & Validation (internal/research/)

| Document | Purpose | Date | Read If... |
|----------|---------|------|-----------|
| **LEARNED_INDEX_VALIDATION.md** | Learned index benchmarks | Oct 1 | Need learned index performance data |
| COMPREHENSIVE_ANALYSIS.md | Research analysis | Sep 26 | Historical research context |
| PROPER_TEST_PLAN.md | Testing methodology | Sep 26 | Planning scale tests |
| README.md | Research directory guide | Sep 26 | Overview of research docs |

---

## Key Decisions Made

### Strategic Direction (Oct 2, 2025)
- **Focus:** Embedded PostgreSQL for time-series + AI workloads
- **Three paths:** Algorithm-first (if 10-50x faster), Feature-first (if <5x faster), Hybrid (recommended)
- **Target market:** $5B+ TAM (time-series $1.45B + vectors $4B)

### YC W25 Application (Oct 2, 2025)
- **Deadline:** November 10, 2025 (5 weeks away)
- **Decision point:** Run benchmarks Week 1 - if 10-50x faster on time-series, apply to YC
- **Target traction:** 500-2K GitHub stars, 50-100 active users, $1-5K MRR (optional)

### Monetization (Oct 2, 2025)
- **Revised pricing:** $9/month (Starter), $29/month (Pro), $299-999/month (Enterprise)
- **Model:** Open Core (free embedded database + paid cloud sync/enterprise)
- **Based on:** Turso pricing research (YC W23, embedded SQLite)

### Licensing (Sep 29, 2025)
- **Recommendation:** Elastic License v2 (ELv2)
- **Rationale:** Protects against cloud providers, less friction than AGPL
- **See:** `business/LICENSING_STRATEGY.md`

---

## What's NOT Here

### Removed (in git history if needed):
- ❌ Old MVP ideas (pgAnalytics, etc.) - Archived, then deleted
- ❌ Superseded strategy docs (SOLO_DEV_STRATEGY.md, DECISION_SUMMARY.md, etc.)
- ❌ Historical cleanup docs (REPOSITORY_CLEANUP_SUMMARY.md, etc.)

### Use git history if you need old docs:
```bash
# Find deleted files
git log --all --full-history -- "internal/business/SOLO_DEV_STRATEGY.md"

# Recover if needed
git checkout <commit> -- path/to/old/file
```

---

## Contributing

When adding new internal docs:

1. **Choose the right directory:**
   - `business/` - Strategy, funding, market, monetization
   - `technical/` - Architecture, systems, code design
   - `research/` - Experiments, benchmarks, validations

2. **Name with dates for major docs:**
   - Format: `TOPIC_NAME_OCT_2025.md`
   - Makes it clear which is latest

3. **Update this README:**
   - Add to relevant table above
   - Update "Last Updated" date

4. **Delete superseded docs:**
   - Don't accumulate old versions
   - Use git history if you need them later

---

## Quick Reference

**Need to understand the project in 5 minutes?**
→ Read `business/EXECUTIVE_SUMMARY.md`

**Need to decide on YC application?**
→ Read `business/YC_W25_ROADMAP.md` (realistic 5-week plan)

**Need current project status?**
→ Read `technical/CURRENT_STATUS.md`

**Need learned index performance data?**
→ Read `research/LEARNED_INDEX_VALIDATION.md`

**Everything else:** Browse the tables above for specific topics
