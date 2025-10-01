# OmenDB Architecture Limitations

**Context**: Analysis of current architecture to understand constraints for future features

---

## 🏗️ Current Architecture

### Storage Layer:
- **Columnar storage** (Apache Arrow/Parquet)
- **Append-only** design
- Optimized for fast inserts and scans

### Index Layer:
- **Learned index** (RMI) over primary key
- **Position-based** lookups (returns row position in storage)
- Optimized for point queries and range scans

### This Design is Excellent For:
- ✅ High-throughput inserts
- ✅ Fast point queries by primary key
- ✅ Fast range queries
- ✅ Time-series data (append-only)
- ✅ Analytics workloads (scans)

---

## ⚠️ Architectural Constraints for UPDATE/DELETE

### The Fundamental Problem:
**Learned indexes store positions, columnar storage is append-only.**

### Why UPDATE is Hard:
1. **Primary key changes break the index**
   - Index maps: `key → position`
   - If key changes, index needs complete rebuild

2. **In-place updates don't exist in columnar format**
   - Arrow RecordBatches are immutable
   - Would need to reconstruct entire batch

3. **Cascade effect**
   - Updating one column requires reconstructing the entire row
   - Then reconstructing the RecordBatch
   - Then updating the Parquet file

### Why DELETE is Even Harder:
1. **Position invalidation**
   - Learned index stores: `key[i] → position[i]`
   - Deleting row at position `p` makes all positions `> p` invalid
   - Would require reindexing 50M+ keys

2. **No tombstone support**
   - Columnar format doesn't have row-level tombstones
   - Would need to add a "deleted" column (changes schema)
   - Wastes storage space

3. **Compaction complexity**
   - Eventually need to reclaim space from deleted rows
   - Requires rewriting Parquet files
   - Invalidates all learned index positions

---

## 🔧 Possible Solutions (Increasing Complexity)

### Option 1: Hybrid Approach with Delta Storage (RECOMMENDED)
**Concept**: Keep main storage append-only, use separate delta storage for updates/deletes

```
Main Storage (Columnar):
  - Original data (immutable)
  - Learned index works perfectly

Delta Storage (Row-oriented):
  - Recent updates (key -> new_values)
  - Recent deletes (key -> tombstone)
  - Small, fast, in-memory

Query Path:
  1. Check delta storage first
  2. If not found or not deleted, query main storage
  3. Periodic merge: Apply deltas to main storage (background)
```

**Pros**:
- Doesn't break existing architecture
- Fast reads (learned index still works)
- Fast writes (delta is in-memory)
- Periodic compaction manageable

**Cons**:
- Query complexity (check two stores)
- Need background compaction job
- Some queries slower (delta lookup overhead)

### Option 2: Copy-on-Write with Versioning
**Concept**: Never modify data, create new versions

```
Table v1: rows 0-100K
Table v2: rows 0-100K (with updates applied)
  - Learned index rebuilt for v2
  - Old version garbage collected
```

**Pros**:
- Supports time-travel queries
- Simple conceptually
- No index invalidation within version

**Cons**:
- Massive storage overhead
- Index rebuild on every update
- Complex garbage collection

### Option 3: LSM-Tree Style Architecture
**Concept**: Multiple levels, each immutable, periodic compaction

```
L0: Recent writes (small, in-memory)
L1: Recent compactions (larger)
L2: Older data (largest)

Query: Check L0 → L1 → L2
Compaction: Merge L0+L1 → new L1
```

**Pros**:
- Industry-proven (RocksDB, LevelDB)
- Good write throughput
- Efficient compaction

**Cons**:
- Learned index per level
- Query amplification (check multiple levels)
- Complex compaction logic

### Option 4: Rebuild Index on Changes (SIMPLE BUT SLOW)
**Concept**: Just rebuild everything on UPDATE/DELETE

```rust
fn delete(&mut self, key: Value) -> Result<()> {
    // 1. Scan all rows, filter out deleted key
    let remaining = self.scan_all()?
        .into_iter()
        .filter(|row| row.pk() != key)
        .collect();

    // 2. Rebuild storage
    self.storage = TableStorage::new(...)?;
    for row in remaining {
        self.storage.insert(row)?;
    }

    // 3. Rebuild index
    self.index = TableIndex::new(remaining.len());
    for (i, row) in remaining.iter().enumerate() {
        self.index.insert(row.pk(), i)?;
    }
}
```

**Pros**:
- Simple to implement
- No architectural changes
- Guaranteed consistent

**Cons**:
- **EXTREMELY SLOW** (O(n) for every delete/update!)
- Not viable for production
- Defeats purpose of learned index

---

## 💡 Recommended Approach for Production

### Short-term (Next 2 weeks):
**Implement Option 4 (Rebuild on Change) with clear documentation:**

```sql
-- UPDATE/DELETE supported but slow (rebuilds index)
-- Not recommended for high-frequency updates
-- Use INSERT-only pattern for best performance

UPDATE users SET age = 30 WHERE id = 1;  -- Works but rebuilds entire table
DELETE FROM users WHERE id = 1;          -- Works but rebuilds entire table
```

**Rationale**:
- Provides SQL completeness for demos
- Simple to implement (200 lines)
- Clear performance characteristics
- Sets expectations correctly

### Medium-term (1-2 months):
**Implement Option 1 (Hybrid Delta Storage):**

```
Main Table (Append-only):
  - 99% of data
  - Learned index (fast)

Delta Table (Row-based):
  - 1% of data (recent changes)
  - B-tree index (flexible)

Background Job:
  - Merge deltas into main table nightly
  - Rebuild learned index
  - Clear delta table
```

**Performance Profile**:
- INSERT: Fast (append-only)
- SELECT: Fast (learned index + small delta check)
- UPDATE: Fast (writes to delta)
- DELETE: Fast (writes to delta)
- Background merge: Acceptable (off-peak hours)

### Long-term (3-6 months):
**Implement Option 3 (LSM-Tree with Learned Indexes):**
- Production-grade write performance
- Predictable read latency
- Industry-proven architecture
- Allows scaling to 100M+ rows

---

## 📊 Performance Implications

### Current (INSERT + SELECT only):
```
INSERT:       242,989 ops/sec  (excellent)
SELECT point: 354.8μs          (excellent)
SELECT range: 29.9μs           (excellent)
UPDATE:       N/A              (not implemented)
DELETE:       N/A              (not implemented)
```

### With Option 4 (Rebuild on Change):
```
INSERT:       242,989 ops/sec  (unchanged)
SELECT point: 354.8μs          (unchanged)
SELECT range: 29.9μs           (unchanged)
UPDATE:       ~5 seconds/op    (for 100K row table - SLOW!)
DELETE:       ~5 seconds/op    (for 100K row table - SLOW!)
```

### With Option 1 (Hybrid Delta):
```
INSERT:       242,989 ops/sec  (unchanged)
SELECT point: 400μs            (slight overhead from delta check)
SELECT range: 50μs             (slight overhead from delta merge)
UPDATE:       50,000 ops/sec   (in-memory delta)
DELETE:       50,000 ops/sec   (in-memory delta)
```

---

## 🎯 Recommendation for Open Source Release

**Document this honestly in README:**

```markdown
## Supported SQL Operations

### Fully Optimized (Production Ready):
- ✅ INSERT - 242K ops/sec
- ✅ SELECT with WHERE - 9-116x faster than full scan
- ✅ Range queries - Sub-millisecond performance

### Supported but Limited (v0.1.0):
- ⚠️ UPDATE - Functional but slow (rebuilds table)
- ⚠️ DELETE - Functional but slow (rebuilds table)

**For high-update workloads, see roadmap for v0.2.0 (hybrid delta storage).**
```

**Benefits**:
1. Honest about trade-offs
2. Sets clear expectations
3. Shows path forward (roadmap)
4. Learned index value prop still clear

---

## 📝 Implementation Priority

1. **This Week**: Implement Option 4 (200 lines, clear docs)
2. **This Month**: Write detailed design doc for Option 1
3. **Next Month**: Implement Option 1 with benchmarks
4. **Q1 2026**: Production-ready UPDATE/DELETE at scale

---

**Conclusion**: Current architecture is excellent for its target use case (time-series, analytics, append-only). UPDATE/DELETE support requires careful design but is achievable with hybrid storage approach.

*Last updated: September 29, 2025*