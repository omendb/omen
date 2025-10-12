# Internal Documentation Best Practices

**Created**: October 11, 2025
**Purpose**: Guidelines for maintaining internal docs for AI-assisted development

---

## Current Status Assessment

**Total Documents**: 90 markdown files
- **Active**: 55 files
- **Archived**: 35 files
- **Organization**: Well-structured with clear directories

**Strengths** ✅:
- Clear directory structure (business, research, technical, phases)
- Comprehensive README.md index
- Regular status reports
- Archive system for historical docs
- Good separation of concerns

**Areas for Improvement** ⚠️:
- Some docs have outdated dates (now fixed)
- No explicit "last validated" dates on all docs
- Could benefit from more consistent naming conventions
- Some overlap between historical and current docs

---

## Best Practices for AI-Assisted Development

### 1. Document Classification System

**Use clear prefixes for document types:**

```
STATUS_*        - Current status reports (dated)
PHASE_*         - Milestone completion summaries
GUIDE_*         - How-to documentation
ANALYSIS_*      - Research and competitive analysis
PLAN_*          - Future roadmaps and strategies
SUMMARY_*       - Session or project summaries
```

**Example naming:**
- `STATUS_REPORT_OCT_2025.md` ✅ (clear, dated)
- `GUIDE_DEPLOYMENT.md` ✅ (clear purpose)
- `misc_notes.md` ❌ (vague, will get lost)

### 2. Document Metadata Template

**Every document should start with:**

```markdown
# Document Title

**Created**: YYYY-MM-DD
**Last Updated**: YYYY-MM-DD
**Status**: Active | Historical | Superseded
**Superseded By**: [file if applicable]
**Purpose**: One-line description
**Audience**: Developers | Business | Claude | All

---
```

**Why this matters for AI:**
- AI agents can quickly assess document relevance
- Clear status prevents outdated info from being used
- Purpose/audience helps with context selection
- Dates enable time-based filtering

### 3. Single Source of Truth Principle

**One authoritative document per topic:**

❌ **Anti-pattern: Multiple competing documents**
```
internal/STATUS_OLD.md
internal/STATUS_NEW.md
internal/STATUS_LATEST.md
internal/CURRENT_STATUS.md
```

✅ **Better: Versioned with clear successor**
```
internal/STATUS_REPORT_OCT_2025.md          # Current
internal/STATUS_REPORT_JAN_2025.md          # Historical (marked as superseded)
internal/archive/STATUS_REPORT_SEP_2024.md  # Archived
```

**Implementation:**
- Keep ONE current status doc (dated)
- Historical versions clearly marked as "Superseded by X"
- Archive anything >3 months old

### 4. Directory Structure Best Practices

**Current OmenDB structure (Good!) ✅:**

```
internal/
├── README.md                          # Index - always start here
├── STATUS_REPORT_OCT_2025.md         # Current single source of truth
│
├── business/                          # Business strategy, funding
├── research/                          # Competitive analysis, validation
├── technical/                         # Architecture, implementation
├── phases/                            # Milestone completion docs
├── design/                            # Design decisions
│
└── archive/                           # Historical docs (>3 months)
    ├── alexstorage/                   # Superseded experiments
    ├── assessments/                   # Old status reports
    └── optimizations/                 # Historical optimization work
```

**Key principles:**
- Flat hierarchy at root level (max 2-3 levels deep)
- Clear purpose per directory
- Active docs at top level, historical in archive/
- README.md that acts as comprehensive index

### 5. AI Agent Context Management

**Make it easy for AI to find relevant info:**

**✅ Good: Explicit pointers in top-level docs**
```markdown
## Current Status
**Read:** `STATUS_REPORT_OCT_2025.md` ⭐
- Multi-level ALEX validated to 100M+ scale
- PostgreSQL wire protocol complete
```

**❌ Bad: Scattered info without index**
```markdown
We have several docs about status... check around...
```

**Best practices:**
- README.md is the entry point
- Use ⭐ to mark most important docs
- Include "Read if..." guidance
- Cross-link related documents

### 6. Session Documentation

**After significant work sessions, create summaries:**

**Template:**
```markdown
# Session Summary - [Topic]

**Date**: YYYY-MM-DD
**Duration**: X hours
**Focus**: Brief description

## What We Built
- Feature 1
- Feature 2

## Key Decisions
1. Decision + rationale
2. Decision + rationale

## Results
- Benchmark results
- Tests added
- Performance impact

## Next Steps
- [ ] Task 1
- [ ] Task 2

## Commits This Session
- abc1234 - Description
- def5678 - Description
```

**Why this matters:**
- Future Claude sessions can understand what was done
- Captures decision rationale (not just code changes)
- Links commits to high-level goals
- Enables continuity across sessions

### 7. Status Report Cadence

**Recommended schedule:**

- **Weekly**: Update if major milestone achieved
- **Monthly**: Comprehensive status report (like STATUS_REPORT_OCT_2025.md)
- **Quarterly**: Archive old docs, reorganize structure

**Monthly status report should include:**
1. Executive summary (what's working, what's next)
2. Technical milestones achieved
3. Competitive position update
4. Business metrics (if any)
5. Risk assessment
6. Next 30/60/90 day plans

### 8. Archival Policy

**When to archive a document:**

1. **Superseded**: New version exists
2. **Completed**: Milestone docs after completion
3. **Obsolete**: Plans that were changed
4. **Historical**: >3 months old and not referenced

**How to archive:**
```bash
git mv internal/OLD_DOC.md internal/archive/category/OLD_DOC.md
# Update any references to point to new doc
# Add "Superseded by X" note to archived doc header
```

**Keep in archive, don't delete:**
- Git history preservation isn't enough
- AI agents benefit from browsing old decisions
- "Why did we not do X?" questions need answers

### 9. Cross-Referencing Strategy

**Use relative links liberally:**

```markdown
## Related Docs
- Technical details: [Architecture](../technical/ARCHITECTURE.md)
- Performance: [Benchmarks](../research/ALEX_PERFORMANCE_VALIDATION.md)
- Next steps: [Roadmap](../business/YC_W25_ROADMAP.md)
```

**Benefits:**
- AI agents can follow relationships
- Human readers find context easily
- Git moves preserve links (if relative)

### 10. Code-to-Doc Linking

**Link from code to design docs:**

```rust
// Multi-level ALEX implementation
// Design: internal/design/MULTI_LEVEL_ALEX.md
// Benchmarks: internal/research/100M_SCALE_RESULTS.md
pub struct MultiLevelAlex {
    // ...
}
```

**Link from docs to code:**

```markdown
## Implementation
See: `src/alex/multi_level.rs:123` for routing logic
```

**Why this matters:**
- Future you needs to understand "why"
- Claude needs context to make good suggestions
- New contributors can trace decisions

---

## OmenDB-Specific Recommendations

### Current Structure Review

**What's working well:** ✅

1. **Clear README.md index**
   - Good categorization
   - Links to key docs
   - "Read if..." guidance

2. **Comprehensive STATUS_REPORT**
   - Covers all aspects (technical, business, competitive)
   - Clear milestones and next steps
   - Honest assessment of gaps

3. **Archive system**
   - Preserves historical context
   - Organized by category
   - Keeps active docs clean

4. **Phase completion docs**
   - Good milestone tracking
   - Clear before/after state
   - Useful for continuity

### Suggested Improvements

#### 1. Add "Last Validated" metadata

**Problem**: Some docs may have outdated benchmark numbers

**Solution**: Add to doc header
```markdown
**Last Validated**: 2025-10-11
**Validation Method**: Ran benchmark_vs_sqlite 100M
**Results**: 1.5-3x speedup confirmed
```

#### 2. Create a DECISIONS log

**Problem**: "Why did we choose X?" questions are scattered

**Solution**: `internal/DECISIONS.md`
```markdown
# Key Design Decisions

## 2025-10-05: Multi-level ALEX over single-level
**Context**: Single-level hit cache bottleneck at 50M+ scale
**Options Considered**:
1. Increase MAX_DENSITY
2. Reduce retrain frequency
3. Multi-level hierarchy ✅ chosen

**Rationale**: Multi-level is industry standard solution...
**Result**: Validated to 100M+, 1.5-3x speedup maintained
**Related**: internal/design/MULTI_LEVEL_ALEX.md
```

#### 3. Consolidate scattered "current status" info

**Current state**:
- `STATUS_REPORT_OCT_2025.md` (comprehensive)
- `STATUS_UPDATE.md` (brief)
- `CLAUDE.md` (for AI agent)
- `README.md` (project readme)
- `internal/README.md` (docs index)

**Recommendation**: This is actually fine! Different audiences need different detail levels.

**Just ensure consistency:**
- All should reference October 2025
- All should point to STATUS_REPORT_OCT_2025.md as source of truth
- All should have consistent "next steps"

#### 4. Create templates directory

**Add**: `internal/templates/`

```
internal/templates/
├── SESSION_SUMMARY_TEMPLATE.md
├── STATUS_REPORT_TEMPLATE.md
├── BENCHMARK_RESULTS_TEMPLATE.md
├── PHASE_COMPLETION_TEMPLATE.md
└── COMPETITIVE_ANALYSIS_TEMPLATE.md
```

**Benefits:**
- Consistency across docs
- Faster doc creation
- Claude can use templates to generate new docs

#### 5. Add a "For Claude" section to README

**Add to `internal/README.md`:**

```markdown
## For AI Assistants (Claude Code)

When starting a new session:
1. **Always** read `STATUS_REPORT_OCT_2025.md` first
2. Check `../CLAUDE.md` for high-level context
3. Review relevant subdirectory based on task:
   - Technical work → `technical/`
   - Benchmarks → `research/`
   - Strategy → `business/`

**Quick Context Checklist:**
- [ ] Current status understood
- [ ] Recent milestones reviewed
- [ ] Next priorities identified
- [ ] Technical constraints noted
```

---

## Comparison to Other Projects

### What "agent-contexts" Repos Typically Do

Many projects create `external/agent-contexts/` or `.agent/` directories for:

1. **Project context** - What the AI needs to know
2. **Coding guidelines** - How to write code
3. **Architecture docs** - How system is structured
4. **Common tasks** - Frequent operations

**OmenDB equivalent:**
- `CLAUDE.md` = Project context ✅
- `CONTRIBUTING.md` = Coding guidelines ✅
- `ARCHITECTURE.md` = Architecture docs ✅
- `internal/` = Detailed strategy/status ✅

**You're already following best practices!**

### Industry Examples

**Anthropic Claude Code Best Practices:**
- Single CLAUDE.md at root (you have this ✅)
- Internal strategy docs separate from code (you have this ✅)
- Regular status updates (you have this ✅)

**GitHub Copilot Best Practices:**
- Clear README.md (you have this ✅)
- Architecture docs (you have this ✅)
- Inline code comments for "why" not "what"

**Cursor AI Best Practices:**
- Project-specific guidelines in root (you have this ✅)
- Separation of user docs vs internal docs (you have this ✅)

---

## Action Items for OmenDB

### Immediate (Done ✅)
- [x] Update all dates to October 2025
- [x] Create STATUS_REPORT_OCT_2025.md
- [x] Update CLAUDE.md with current status
- [x] Update internal/README.md

### Next Session
- [ ] Create `internal/DECISIONS.md` log
- [ ] Create `internal/templates/` directory
- [ ] Add "Last Validated" dates to key benchmark docs
- [ ] Add "For AI Assistants" section to internal/README.md

### Ongoing
- [ ] Monthly status report (next: November 2025)
- [ ] Session summaries after major work
- [ ] Archive docs >3 months old
- [ ] Update DECISIONS.md when making major choices

---

## Quick Reference Card

**Starting a new session as Claude:**
1. Read: `CLAUDE.md` (2 min context)
2. Read: `internal/STATUS_REPORT_OCT_2025.md` (comprehensive status)
3. Check: Git recent commits
4. Identify: Current phase and next priorities

**After completing work:**
1. Update relevant STATUS or PHASE doc if major milestone
2. Create SESSION_SUMMARY if >2 hours of work
3. Update README.md if structure changed
4. Commit docs alongside code changes

**When unsure:**
- "What's our current status?" → `internal/STATUS_REPORT_OCT_2025.md`
- "What's the architecture?" → `ARCHITECTURE.md` + `internal/technical/`
- "Why did we do X?" → `internal/DECISIONS.md` (to be created) or search internal/
- "What's next?" → Check STATUS_REPORT "Next Steps" section

---

## Summary

**Your current documentation is excellent for AI-assisted development!**

**Strong points:**
- Clear structure with purpose-driven directories
- Comprehensive status reporting
- Good archival system
- Regular updates

**Minor improvements:**
- Add "Last Validated" metadata
- Create DECISIONS log
- Add templates for consistency
- Add "For AI" guidance in README

**Overall grade: A- (Very good!)**

The main "fix" needed was updating dates to October 2025, which is now complete.

---

**Last Updated**: October 11, 2025
**Author**: Claude (Code assistant)
**Status**: Living document - update as practices evolve
