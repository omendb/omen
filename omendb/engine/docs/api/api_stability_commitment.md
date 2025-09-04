# OmenDB API Stability Commitment

## Core Philosophy
Following the SQLite model, OmenDB commits to API stability as a core value. Code written for v0.0.1 will work with v1.0 and beyond.

## What This Means

### For Users
- **Never break existing code** - Your integration is safe
- **Internal optimizations transparent** - Performance improves without changes
- **One way to do things** - Simple, obvious API

### Current API (Stable)
```python
# Single vector - won't change
db.add(id: str, vector: array-like, metadata: dict)

# Batch operation - won't change  
db.add_batch(vectors: array-like, ids: List[str], metadata: List[dict])

# Search - won't change
db.search(query: array-like, limit: int)
```

## Internal Optimizations (Can Change)

### What We Optimize (Invisible to Users)
- Buffer management strategies
- Graph construction algorithms
- Memory allocation patterns
- Flush operations
- Quantization techniques

### Recent Example: Batch Flush
- **Problem**: Slow flush at buffer boundaries
- **Solution**: Added internal batch operations
- **User Impact**: None - same API, better performance
- **This is the ideal approach**

## Design Principles

### 1. Stability Over Features
Better to have fewer stable features than many unstable ones.

### 2. Internal Freedom
We can completely rewrite internals as long as API remains stable.

### 3. Performance by Default
Optimizations should be automatic, not require API changes.

### 4. Semantic Versioning
- v0.x.x: API may have minor changes (currently here)
- v1.0.0: API frozen, only additions allowed
- v2.0.0: Only if absolutely necessary (avoid forever)

## What We DON'T Do

### No API Proliferation
❌ `add_batch_optimized()`
❌ `add_batch_fast()`
❌ `add_batch_v2()`

✅ Just `add_batch()` that gets faster internally

### No Breaking Changes
❌ Changing parameter order
❌ Removing parameters
❌ Changing return types

✅ Adding optional parameters with defaults is OK

### No Configuration Complexity
❌ Dozens of tuning parameters
❌ Complex optimization flags
❌ Version-specific options

✅ Smart defaults that work well

## Commitment to Enterprise Users

For enterprise deployments, API stability means:
- **No surprise breaking changes** in updates
- **Confident dependency** on OmenDB
- **Long-term support** for your code
- **Investment protection** in integrations

## Timeline

### Current (v0.0.8-dev)
- Core API stabilizing
- Minor adjustments possible
- Focus on internal optimization

### v1.0 Release
- API frozen
- Full backward compatibility commitment
- Enterprise-ready stability

### Long-term (v1.x)
- Only additive changes
- Internal optimizations continue
- Performance improvements transparent

## Summary

The recent `add_batch` optimization exemplifies our approach:
- **Internal change**: Added batch method to DiskANN
- **User experience**: Unchanged API, better performance
- **Philosophy**: This is how all optimizations should work

OmenDB aims to be the SQLite of vector databases - not just in being embedded, but in being a stable foundation you can build on for decades.