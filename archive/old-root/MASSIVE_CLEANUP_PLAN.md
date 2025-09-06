# 🧹 Massive Documentation Cleanup Plan

## The Problem: 847 MD Files!
```
568 external/  (submodules - keep)
195 omendb/    (NEEDS MAJOR CLEANUP)
65  internal/  (needs consolidation)
8   zendb/     (archive)
11  root/      (move to proper locations)
```

## The Goal: ~50 Actionable Files
```
3   root/      (README, CLAUDE, LICENSE)
10  omendb/    (architecture, API, benchmarks)
20  internal/  (decisions, patterns, operations)
∞   external/  (submodules - don't touch)
```

## Immediate Actions

### Step 1: Nuclear Cleanup (30 min)
```bash
# Delete all duplicate agent-contexts (155 files!)
rm -rf omendb/engine/docs/agent-contexts/

# Archive the old server docs (40+ strategy files)
mkdir -p archive/old-server-docs
mv omendb/server/docs/* archive/old-server-docs/

# Archive ZenDB
mv zendb/ archive/

# Archive old web/server
mv omendb/server/ archive/
mv omendb/web/ archive/
```

### Step 2: Create Clean Structure (20 min)
```
/
├── README.md                    # Product overview
├── CLAUDE.md                   # AI agent instructions
├── LICENSE                     # Apache 2.0
│
├── omendb/
│   ├── src/                    # RENAME from engine/
│   │   ├── algorithms/
│   │   │   └── hnsw.mojo      # NEW: HNSW+ implementation
│   │   ├── bindings/
│   │   │   ├── python.mojo
│   │   │   └── c_api.mojo
│   │   └── native.mojo
│   │
│   └── docs/                   # Public documentation
│       ├── README.md          # Getting started
│       ├── ARCHITECTURE.md    # HNSW+ design
│       ├── API.md            # Python/C/Rust APIs
│       └── BENCHMARKS.md     # Performance vs pgvector
│
├── internal/                   # Private dev docs
│   ├── PLAYBOOK.md           # How to manage docs (for Claude)
│   ├── CURRENT.md            # Active sprint/tasks
│   ├── DECISIONS.md          # Key decisions log
│   └── patterns/             # Extracted patterns (keep good ones)
│
├── external/                  # Keep as-is
│   └── agent-contexts/       # Submodule (don't duplicate!)
│
└── archive/                   # Old stuff
    ├── zendb/
    ├── old-server/
    └── research/             # Old algorithm analyses
```

### Step 3: Documentation Rules for Claude

## 📝 CLAUDE Documentation Playbook

### File Management Rules

1. **NEVER create analysis files** - Update existing docs
2. **Three doc types only**:
   - `CURRENT.md` - What we're doing NOW
   - `DECISIONS.md` - Why we chose X over Y
   - `PLAYBOOK.md` - How to do things

3. **File locations**:
   - User-facing → `omendb/docs/`
   - Dev notes → `internal/`
   - Old stuff → `archive/`

4. **Update, don't create**:
   ```
   ❌ Creating "HNSW_ANALYSIS.md"
   ✅ Adding section to DECISIONS.md
   ```

5. **Keep it actionable**:
   ```
   ❌ "HNSW might be better than DiskANN"
   ✅ "Decision: Use HNSW. Run: touch omendb/src/algorithms/hnsw.mojo"
   ```

### Documentation Anti-patterns
- ❌ Analysis paralysis files
- ❌ Duplicate information
- ❌ "CRITICAL_PIVOT_URGENT.md" style names
- ❌ Theory without action items

### Documentation Patterns
- ✅ Clear decisions with rationale
- ✅ Runnable code examples
- ✅ File paths and line numbers
- ✅ Next steps clearly defined

## Multimodal vs Pure Vector Decision

### Pure Vector DB (Current Plan)
**Pros:**
- Focused, simpler to build
- Clear market (pgvector replacement)
- 4 weeks to MVP

**Cons:**
- Commoditized market
- Limited differentiation

### Multimodal DB with HNSW+
**Pros:**
- **Huge differentiation** - Text + vectors + structured
- **Perfect for AI apps** - Store prompts, embeddings, metadata together
- **Higher value** - Can charge more
- **Less competition** - MongoDB Atlas closest competitor

**Cons:**
- 8-12 weeks to MVP
- More complex

### My Recommendation: Start Pure, Add Multimodal

**Phase 1 (Month 1)**: Pure HNSW+ vector search
```python
index.add(vectors, ids)
index.search(query_vector)
```

**Phase 2 (Month 2)**: Add metadata filtering
```python
index.add(vectors, ids, metadata={"type": "product"})
index.search(query_vector, filter={"type": "product"})
```

**Phase 3 (Month 3)**: Full multimodal
```python
# This is the killer feature
db.add(
    text="iPhone 15 Pro",
    vector=embedding,
    metadata={"price": 999, "category": "phone"},
    image_url="https://..."
)

# Hybrid search!
results = db.search(
    text_query="latest iPhone",        # BM25
    vector_query=embedding,             # HNSW+
    filters={"price": {"$lt": 1000}}   # Structured
)
```

This gives you:
1. Quick MVP (pure vectors)
2. Clear upgrade path
3. Unique positioning ("MongoDB for AI")
4. Higher pricing power

## Next Steps

1. **Execute nuclear cleanup** (30 min)
2. **Create new structure** (20 min)
3. **Consolidate docs** into CURRENT.md (1 hour)
4. **Start HNSW+ implementation** 

## The 847 → 50 File Reduction

| Directory | Before | After | Action |
|-----------|--------|-------|---------|
| root/ | 11 | 3 | Move to internal/ |
| omendb/engine/docs/ | 155 | 0 | Delete duplicates |
| omendb/server/docs/ | 40 | 0 | Archive |
| internal/ | 65 | 20 | Consolidate |
| zendb/ | 8 | 0 | Archive |
| **TOTAL** | 279 | 23 | **-91% reduction** |

Ready to execute this cleanup?