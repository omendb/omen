# ⚠️ DEPRECATED ALGORITHMS

## Status: These algorithms are being replaced

**All DiskANN implementations in this directory are DEPRECATED as of Feb 5, 2025.**

### Why Deprecated
- DiskANN is fundamentally batch-oriented, incompatible with streaming updates
- Switching to HNSW+ algorithm (industry standard, better fit)
- See `/internal/DECISIONS.md` for full rationale

### Deprecated Files
- `diskann.mojo` - Original DiskANN implementation
- `diskann_integrated.mojo` - Attempted streaming version
- `heap_based_diskann.mojo` - Heap optimization attempt
- `optimized_diskann.mojo` - Performance optimization attempt
- `proper_diskann.mojo` - "Correct" implementation attempt

### Still Active
- `bruteforce.mojo` - Keep for testing/comparison
- `priority_queue.mojo` - Utility, may be reused

### New Implementation Location
**HNSW+ implementation will be at:**
```
omendb/engine/omendb/algorithms/hnsw.mojo  # TO BE CREATED
```

## For AI Agents

**DO NOT use any DiskANN code for new development.**

When implementing new features:
1. Use HNSW+ algorithm patterns
2. Reference `/internal/architecture/MULTIMODAL.md`
3. Check `/internal/MOJO_WORKAROUNDS.md` for language limitations

---
*This directory will be cleaned up after HNSW+ implementation is complete*