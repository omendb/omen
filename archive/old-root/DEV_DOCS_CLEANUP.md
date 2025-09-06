# Development Documentation Cleanup

## Current Chaos: 847 MD files
- Most are duplicates or outdated
- No clear system for Claude to follow
- Information scattered across 10+ directories

## New Dev Doc System (Max 10 files)

```
/
├── README.md              # Keep minimal
├── CLAUDE.md             # Master instruction file for AI agents
│
├── internal/
│   ├── NOW.md           # Current sprint & immediate tasks
│   ├── DECISIONS.md     # Why we chose X (append-only log)
│   └── KNOWLEDGE.md     # Patterns, gotchas, learnings
│
└── omendb/
    ├── HNSW.md          # Implementation notes
    └── FFI.md           # Bindings strategy
```

## Claude Rules for Dev Docs

```python
def manage_docs(task):
    if task == "current_work":
        update("internal/NOW.md")  # What I'm doing
    
    elif task == "architecture_decision":  
        append("internal/DECISIONS.md")  # Why we chose it
    
    elif task == "learned_something":
        update("internal/KNOWLEDGE.md")  # Gotchas, patterns
    
    elif task == "implementation_detail":
        update("omendb/HNSW.md")  # Code-specific notes
    
    # NEVER create new files like:
    # ❌ CRITICAL_ARCHITECTURE_PIVOT.md
    # ❌ HNSW_VS_IPDISKANN_ANALYSIS.md  
    # ❌ FINAL_FINAL_DECISION_v3.md
```

## Immediate Cleanup Actions

### 1. Archive Everything Old
```bash
mkdir -p archive/2025-02-cleanup
mv internal/research/*.md archive/2025-02-cleanup/
mv omendb/server/docs/* archive/2025-02-cleanup/
mv omendb/engine/docs/* archive/2025-02-cleanup/
mv *.md archive/2025-02-cleanup/  # except README, CLAUDE
```

### 2. Create Consolidated Dev Docs

#### internal/NOW.md
```markdown
# Current Sprint (Feb 2025)

## This Week: HNSW+ Implementation
- [ ] Create omendb/src/algorithms/hnsw.mojo
- [ ] Implement hierarchical layers
- [ ] Add SIMD distance calculations
- [ ] Python bindings with zero-copy

## Blockers
- None

## Next Week
- Benchmarking against pgvector
```

#### internal/DECISIONS.md  
```markdown
# Decision Log

## 2025-02-05: HNSW+ over IP-DiskANN
**Choice**: HNSW+ 
**Why**: Market proven, GPU-ready, Mojo strengths apply
**Rejected**: IP-DiskANN (unproven), DiskANN (broken)

## 2025-02-05: Multimodal Strategy
**Choice**: Start pure vector, add multimodal in Phase 2
**Why**: Faster MVP, clear upgrade path
```

#### internal/KNOWLEDGE.md
```markdown  
# Learned Patterns & Gotchas

## Mojo FFI
- Python: Zero overhead via PythonObject
- C: Export as shared library, ~100ns overhead
- Gotcha: Dict[String,Int] uses 8KB per entry!

## HNSW Implementation
- Use lock-free atomics for parallel insert
- SIMD all distance calculations
- Memory: 2 bytes/vector expected
```

### 3. Delete Redundant Research
All these can be consolidated into one decision:
- CRITICAL_ARCHITECTURE_PIVOT.md
- ALGORITHM_DECISION_2025.md  
- HNSW_VS_IPDISKANN_IMPLEMENTATION.md
- PURE_ALGORITHM_COMPARISON.md
- SCALE_ARCHITECTURE_DECISION.md
- FINAL_ALGORITHM_DECISION.md

→ Becomes one paragraph in `internal/DECISIONS.md`

## Multimodal Answer

**Yes, multimodal is better long-term**, but:

### Pure Vector First (Month 1)
- Get to market fast
- Prove HNSW+ performance  
- Build community

### Add Multimodal (Month 2-3)
```python
# The killer feature - unified AI data
doc = {
    "text": "iPhone 15 review",
    "embedding": vector,
    "metadata": {"rating": 4.5},
    "image": binary_data
}

# Hybrid search across all modalities
results = db.search(
    text="iPhone",           # Text search
    vector=query_embedding,  # Semantic search  
    filters={"rating": {">": 4}}  # Structured
)
```

**This positions OmenDB as "MongoDB for AI apps"** rather than "another pgvector".

## Implementation Order

1. **Now**: Clean up docs (1 hour)
2. **Today**: Start HNSW+ in `omendb/src/algorithms/hnsw.mojo`
3. **Week 1**: Pure vector HNSW+ working
4. **Week 2**: Benchmarks beating pgvector
5. **Month 2**: Add metadata filtering
6. **Month 3**: Full multimodal

## Questions

1. Should we rename `omendb/engine/` → `omendb/src/`?
2. Archive or delete the 8 redundant algorithm files?
3. Keep ZenDB around for future SQL layer?

Ready to execute this focused cleanup?