# Documentation Organization Guide for AI Agents

## Quick Reference for @mentions
**When to use this doc**: Reference this file when organizing docs or unsure where information belongs.

## Core Principles
1. **Actionable over analytical** - Docs should guide action, not just describe
2. **Single source of truth** - One location per topic, referenced elsewhere
3. **AI-agent optimized** - Clear structure for context windows
4. **Append-only for decisions** - Never delete history, only add

## Directory Structure

```
/internal/
├── NOW.md                   # Current sprint & immediate tasks
├── DECISIONS.md            # Major decisions log (append-only)
├── KNOWLEDGE.md            # Patterns, gotchas, solutions
├── DOC_ORGANIZATION.md     # This file (how to organize)
├── /architecture/          # System design docs
│   ├── MULTIMODAL.md      # Multimodal architecture
│   ├── HNSW_DESIGN.md     # HNSW+ implementation plan
│   └── STORAGE.md         # Storage layer design
├── /research/             # Research findings (read-only)
│   └── *.md              # Completed research docs
└── /archive/             # Old/outdated docs

/CLAUDE.md                  # AI agent instructions (root)
/README.md                  # Public-facing docs
```

## Where Things Go

### NOW.md - Current Work
```markdown
# What's happening right now
- Current sprint tasks
- Active blockers
- This week's priorities
- Who's working on what (for multi-agent)
```

### DECISIONS.md - Why We Chose X
```markdown
# Date: Decision Name
## Context
## Options Considered  
## Decision & Rationale
## Consequences
```
**Rules**: Append-only, never delete, include dates

### KNOWLEDGE.md - How To Do X
```markdown
# Language Patterns
# Common Errors & Fixes
# Performance Optimizations
# Integration Patterns
```
**Rules**: Update in place, keep actionable

### Architecture Docs
- Technical designs that guide implementation
- Should be stable once decided
- Update rarely, reference often

## Doc Writing Guidelines

### For AI Agents
1. **Front-load critical info** - Most important in first 50 lines
2. **Use clear headers** - AI can navigate by structure
3. **Include examples** - Code > descriptions
4. **Cross-reference** - Link related docs

### Actionable Format
```markdown
## Problem/Goal
What we're trying to achieve

## Solution/Approach  
How to do it (with code)

## Example
```mojo
# Actual working code
```

## Common Pitfalls
What to avoid and why
```

### Avoid These
- ❌ Long analytical essays
- ❌ Duplicate information
- ❌ Vague descriptions
- ❌ Outdated examples

## Consolidation Rules

### When to Consolidate
- Multiple files covering same topic
- Scattered decisions across files
- Research complete, needs integration

### How to Consolidate
1. Identify canonical location
2. Move actionable parts to KNOWLEDGE.md
3. Move decisions to DECISIONS.md
4. Archive analysis files
5. Update cross-references

## File Lifecycle

```
Research → Architecture → Implementation → Knowledge
   ↓           ↓              ↓              ↓
research/  architecture/    NOW.md      KNOWLEDGE.md
                           
Outdated → Archive
   ↓         ↓
archive/  (with date)
```

## Common Scenarios

### "Where do I put algorithm analysis?"
- Decision (why HNSW+) → DECISIONS.md
- Implementation plan → architecture/HNSW_DESIGN.md
- Code patterns → KNOWLEDGE.md

### "Where do I put competitive analysis?"
- Internal strategy → architecture/COMPETITIVE.md
- Market decisions → DECISIONS.md
- Never in public docs (no competitor names)

### "Where do I track bugs?"
- Active bugs → NOW.md
- Bug patterns → KNOWLEDGE.md
- Post-mortem → DECISIONS.md

## AI Agent Workflow

### Starting New Session
1. Read `/CLAUDE.md` for instructions
2. Read `/internal/NOW.md` for current work
3. Check `/internal/DECISIONS.md` for context
4. Reference `/internal/KNOWLEDGE.md` for patterns

### Updating Docs
1. Check this guide for where info belongs
2. Update appropriate file (don't create new)
3. Archive if replacing existing doc
4. Update NOW.md with session summary

### Research Tasks
1. Create research/ file during research
2. Extract decisions → DECISIONS.md
3. Extract patterns → KNOWLEDGE.md
4. Archive research file when complete

## Anti-Patterns to Avoid

### Creating Analysis Files
```bash
# ❌ WRONG
write("HNSW_ANALYSIS_FINAL_V3.md")

# ✅ RIGHT  
append("DECISIONS.md", "## Why HNSW+...")
update("KNOWLEDGE.md", "## HNSW Patterns...")
```

### Duplicate Information
```bash
# ❌ WRONG
"Mojo patterns" in 5 different files

# ✅ RIGHT
Single source: KNOWLEDGE.md#mojo-patterns
Others reference: "See KNOWLEDGE.md#mojo-patterns"
```

### Vague Locations
```bash
# ❌ WRONG
"See other doc for details"

# ✅ RIGHT
"See architecture/STORAGE.md#tiered-storage"
```

## Maintenance Schedule

### Every Session
- Update NOW.md with progress
- Add decisions to DECISIONS.md
- Update KNOWLEDGE.md with learnings

### Weekly
- Archive completed research
- Consolidate scattered info
- Update architecture docs if needed

### Monthly
- Review and clean archive/
- Update CLAUDE.md if needed
- Consolidate KNOWLEDGE.md sections

---
*Reference this guide with @DOC_ORGANIZATION when organizing documentation*