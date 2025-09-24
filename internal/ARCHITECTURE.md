# OmenDB Architecture

**Status**: In Development
**Current Reality**: 3.3K vec/s, 100% recall (individual insertion)
**Problem**: Bulk insertion gives 30K+ vec/s but 0% recall (broken)

## Design

**Language**: Pure Mojo (no FFI overhead)
**Algorithm**: HNSW for CPU (proven, reliable)
**Mode**: Embedded database (like SQLite)
**GPU**: Not viable yet (Apple Silicon support added Sept 21, experimental)

## Current Performance
- **Working**: 3,300 vec/s with 100% recall (individual insertion)
- **Broken**: 30,000+ vec/s with 0% recall (bulk insertion skips navigation)
- **Target**: 20,000+ vec/s with 95% recall (competitors)

## Critical HNSW Rules (Why Bulk Is Broken)

### 1. Never Skip Layer Navigation
```mojo
// ✅ CORRECT - Always navigate top-down
var curr = entry_point
for layer in range(entry_level, target_layer, -1):
    curr = search_layer(query, curr, 1, layer)

// ❌ BROKEN - Skipping creates disconnected graph (0% recall)
var neighbors = search_at_layer(query, target_layer)
```

### 2. Maintain Bidirectional Connections
Every A→B connection needs B→A connection.

### 3. Use Proper Distance Functions
SIMD `_fast_distance_between_nodes()`, not approximations.

## Next Steps
1. Fix bulk construction while preserving navigation
2. Implement segment parallelism (10K vectors per segment)
3. True parallel segment building

---
*Reality check: We have fast OR correct, need both.*