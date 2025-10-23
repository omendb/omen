# Hybrid Search Design

**Status**: Design Phase
**Date**: October 23, 2025
**Objective**: Combine vector similarity search with SQL predicates for HTAP queries

---

## Overview

Hybrid search enables queries that combine:
1. **Vector similarity search** (ORDER BY embedding <-> query LIMIT k)
2. **SQL predicates** (WHERE category = 'X' AND price < 100)

This is OmenDB's unique differentiator vs pure vector DBs (Pinecone, Weaviate) that lack SQL capabilities.

---

## Example Queries

```sql
-- Use case 1: E-commerce product search
SELECT * FROM products
WHERE category = 'electronics' AND price < 100
ORDER BY embedding <-> '[...]'::vector
LIMIT 10;

-- Use case 2: Document search with filters
SELECT * FROM documents
WHERE author = 'John Doe' AND created_at > '2024-01-01'
ORDER BY embedding <-> '[...]'::vector
LIMIT 20;

-- Use case 3: User similarity with constraints
SELECT * FROM users
WHERE age BETWEEN 25 AND 35 AND city = 'San Francisco'
ORDER BY profile_embedding <-> '[...]'::vector
LIMIT 50;
```

---

## Architecture Components

### 1. Hybrid Query Detection

**Input**: Parsed SQL query (sqlparser AST)

**Detection criteria**:
- Has ORDER BY with vector distance operator (<->, <#>, <=>)
- Has LIMIT clause
- Has WHERE clause with SQL predicates

**Output**: `HybridQueryPattern`

```rust
pub struct HybridQueryPattern {
    /// Vector similarity search pattern
    pub vector_pattern: VectorQueryPattern,

    /// SQL filter predicates from WHERE clause
    pub sql_predicates: Expr,

    /// Table name
    pub table_name: String,
}
```

### 2. Query Planning Strategies

**Strategy 1: Filter-First** (SQL predicates → Vector search)
- **When**: SQL predicates are highly selective (< 10% of rows)
- **How**:
  1. Execute WHERE clause using ALEX index (if available)
  2. Get filtered row IDs
  3. Run HNSW search only on filtered vectors
- **Best for**: Primary key equality, narrow ranges, rare categories

**Strategy 2: Vector-First** (Vector search → SQL filter)
- **When**: SQL predicates are not selective (> 50% of rows)
- **How**:
  1. Run HNSW+BQ vector search to get top-k candidates
  2. Apply SQL predicates to filter results
  3. Return filtered results (may be < k if filters are strict)
- **Best for**: Broad ranges, common categories, complex predicates

**Strategy 3: Dual-Scan** (Parallel execution → Merge)
- **When**: Medium selectivity (10-50% of rows)
- **How**:
  1. Execute both SQL predicates and vector search in parallel
  2. Intersect results by row IDs
  3. Rerank intersection by exact distances
- **Best for**: Medium selectivity, balanced workloads

### 3. Selectivity Estimation

**Heuristics**:
```rust
fn estimate_selectivity(predicate: &Expr, table_size: usize) -> f64 {
    match predicate {
        // Primary key equality: 1 row (very selective)
        Expr::BinaryOp { op: Eq, left: PK, ... } => 1.0 / table_size as f64,

        // Primary key range: estimate based on range width
        Expr::BinaryOp { op: And, ... } => estimate_range_selectivity(...),

        // Non-indexed equality: assume 1% selectivity
        Expr::BinaryOp { op: Eq, ... } => 0.01,

        // General predicates: assume 10% selectivity
        _ => 0.10,
    }
}
```

**Thresholds**:
- High selectivity: `selectivity < 0.10` → Filter-First
- Medium selectivity: `0.10 <= selectivity <= 0.50` → Dual-Scan
- Low selectivity: `selectivity > 0.50` → Vector-First

### 4. Execution Engine Changes

**Current flow** (sql_engine.rs):
```
execute_select():
  1. execute_where_clause() → rows
  2. apply_order_by() → sorted rows
  3. project columns → result
```

**New hybrid flow**:
```
execute_select():
  1. Detect if query is hybrid (has WHERE + vector ORDER BY)
  2. If hybrid:
     a. Estimate selectivity of WHERE clause
     b. Choose strategy (Filter-First / Vector-First / Dual-Scan)
     c. Execute hybrid query
  3. Else: existing flow
```

**New method**:
```rust
fn execute_hybrid_query(
    &self,
    pattern: HybridQueryPattern,
    strategy: HybridQueryStrategy,
    table: &Table,
) -> Result<Vec<Row>> {
    match strategy {
        HybridQueryStrategy::FilterFirst => {
            // 1. Execute WHERE clause using ALEX
            let filtered_rows = self.execute_where_clause(table, &pattern.sql_predicates)?;
            let filtered_ids: Vec<u64> = filtered_rows.iter().map(|r| r.id()).collect();

            // 2. Run HNSW search only on filtered vectors
            table.vector_search_filtered(
                &pattern.vector_pattern.column_name,
                &pattern.vector_pattern.query_vector,
                pattern.vector_pattern.k,
                &filtered_ids,
            )
        }

        HybridQueryStrategy::VectorFirst => {
            // 1. Run HNSW+BQ search to get top-k*3 candidates (over-fetch)
            let candidates = table.vector_search(
                &pattern.vector_pattern.column_name,
                &pattern.vector_pattern.query_vector,
                pattern.vector_pattern.k * 3,
            )?;

            // 2. Apply SQL predicates to filter results
            let filtered = self.execute_where_clause_with_schema(
                &table.schema(),
                candidates,
                &pattern.sql_predicates,
            )?;

            // 3. Take top-k after filtering
            Ok(filtered.into_iter().take(pattern.vector_pattern.k).collect())
        }

        HybridQueryStrategy::DualScan => {
            // TODO: Parallel execution + merge (Phase 2)
            // For now, fall back to Filter-First
            self.execute_hybrid_query(
                pattern,
                HybridQueryStrategy::FilterFirst,
                table,
            )
        }
    }
}
```

---

## Implementation Plan

### Phase 1: Filter-First Strategy (Week 5 Days 1-3)

**Priority**: Highest (covers 70% of use cases)

**Tasks**:
1. Add `HybridQueryPattern` struct to `vector_query_planner.rs`
2. Implement hybrid query detection in `VectorQueryPattern::detect()`
3. Add selectivity estimation to `VectorQueryPlanner`
4. Implement `execute_hybrid_query()` with Filter-First strategy
5. Add `Table::vector_search_filtered()` method (HNSW search on subset)
6. Write 10-15 tests for Filter-First strategy

**Test cases**:
- Primary key equality + vector search
- Primary key range + vector search
- Non-indexed equality + vector search
- Multiple predicates (AND) + vector search
- Edge case: Filter returns 0 rows
- Edge case: Filter returns > k rows

### Phase 2: Vector-First Strategy (Week 5 Days 4-5)

**Priority**: High (covers remaining 30% of use cases)

**Tasks**:
1. Implement Vector-First in `execute_hybrid_query()`
2. Add over-fetch parameter (k * expansion_factor)
3. Handle case where filtering reduces results below k
4. Write 8-10 tests for Vector-First strategy

**Test cases**:
- Broad filter (category IN [many values]) + vector search
- No WHERE clause (pure vector search)
- Filter that eliminates 50% of candidates
- Filter that eliminates 90% of candidates

### Phase 3: Optimization & Benchmarks (Week 6)

**Tasks**:
1. Implement Dual-Scan strategy (parallel execution)
2. Refine selectivity estimation with statistics
3. Add query plan explanation (EXPLAIN output)
4. Benchmark hybrid queries vs separate queries
5. Document hybrid search in user guide

**Benchmarks**:
- Compare Filter-First vs Vector-First for different selectivities
- Measure speedup vs "separate queries then merge"
- Test with 1M vectors, various filter selectivities (1%, 10%, 50%)

---

## Cost Estimation

### Filter-First Cost

```
Cost = SQL_Filter_Cost + Vector_Search_Filtered_Cost
     = O(log N) ALEX lookup + O(k * log M) HNSW search
     where M = filtered set size
```

**Example**: 1M vectors, WHERE id = 5 (1 row), k=10
- ALEX: 20 comparisons (~0.02ms)
- HNSW on 1 vector: 10ms
- **Total: ~10ms**

### Vector-First Cost

```
Cost = Vector_Search_Cost + SQL_Filter_Cost
     = O(k * log N) HNSW + O(k * 3) filter evaluation
```

**Example**: 1M vectors, WHERE category = 'X' (50% selectivity), k=10
- HNSW search for k*3=30: 7ms
- Filter 30 rows: 0.03ms
- **Total: ~7ms**

### Baseline (Separate Queries)

```
Cost = SQL_Filter_Cost + Full_Rerank_Cost
     = O(log N) ALEX + O(M * D) L2 distances
     where M = filtered set size, D = vector dimensions
```

**Example**: 1M vectors, WHERE category = 'X' (500K rows), k=10
- ALEX range: 0.5ms
- L2 distances on 500K vectors (1536D): 50,000ms (50 seconds!)
- **Total: ~50 seconds** (unacceptable)

**Conclusion**: Hybrid search is 5000x faster than naive approach.

---

## API Impact

### No Breaking Changes

Existing queries work as-is. Hybrid search is automatic when both WHERE and vector ORDER BY are present.

### New Index Type (Future)

```sql
-- Composite index for hybrid search (Phase 3)
CREATE INDEX ON products USING hybrid (category, embedding);
```

This would build both ALEX (for category) and HNSW (for embedding) in a single index structure.

---

## Testing Strategy

### Unit Tests (20-25 tests)

1. **Hybrid pattern detection** (5 tests)
   - Detect vector + WHERE
   - Detect vector-only (not hybrid)
   - Detect WHERE-only (not hybrid)
   - Detect invalid patterns

2. **Selectivity estimation** (5 tests)
   - Primary key equality (very selective)
   - Primary key range (medium)
   - Non-indexed equality (low)
   - Complex predicates (default)

3. **Filter-First execution** (8 tests)
   - PK equality + vector
   - PK range + vector
   - Non-indexed predicate + vector
   - Multiple predicates + vector
   - Edge cases (0 rows, > k rows)

4. **Vector-First execution** (7 tests)
   - Broad filter + vector
   - Over-fetch and trim
   - Filter eliminates most candidates
   - Pure vector search (no WHERE)

### Integration Tests (5-8 tests)

1. **E-commerce scenario**: Products table, category + price filters
2. **Document search**: Documents table, author + date filters
3. **User similarity**: Users table, age + city filters
4. **Large-scale**: 10K vectors, various filter selectivities
5. **Concurrent hybrid queries**: Multiple clients, MVCC isolation

### Benchmark Tests (3-5 tests)

1. **Filter-First vs Vector-First**: Compare strategies across selectivities
2. **Hybrid vs Baseline**: Show speedup vs naive approach
3. **Scaling**: 10K, 100K, 1M vectors with hybrid queries

---

## Success Criteria

1. ✅ **Correctness**: Hybrid queries return same results as baseline (separate queries + merge)
2. ✅ **Performance**: Filter-First is >100x faster than baseline for selective filters
3. ✅ **Performance**: Vector-First is >10x faster than baseline for broad filters
4. ✅ **Robustness**: Handles edge cases (0 results, all results, etc.)
5. ✅ **Testing**: >90% code coverage, all tests passing

---

## Future Enhancements (Post-MVP)

### 1. Statistics-Based Planning (Phase 4)

Replace heuristic selectivity estimation with real statistics:
- Track cardinality for each column
- Build histograms for range queries
- Collect query execution statistics
- Adaptive query planning based on history

### 2. Composite Indexes (Phase 5)

Co-locate SQL index and vector index:
```rust
struct HybridIndex {
    alex: AlexIndex,      // For SQL predicates
    hnsw: HNSWIndex,      // For vector search
    mapping: Vec<u64>,    // Map SQL row ID → HNSW node ID
}
```

### 3. Multi-Vector Queries (Phase 6)

Support queries with multiple vector columns:
```sql
SELECT * FROM products
WHERE category = 'X'
ORDER BY
  0.7 * (image_embedding <-> query_image) +
  0.3 * (text_embedding <-> query_text)
LIMIT 10;
```

---

## References

- **ALEX Paper**: "ALEX: An Updatable Adaptive Learned Index" (SIGMOD 2020)
- **HNSW Paper**: "Efficient and Robust Approximate Nearest Neighbor Search" (TPAMI 2018)
- **RaBitQ Paper**: "Faster and Better Approximate Nearest Neighbor Search" (SIGMOD 2024)
- **Hybrid Search Examples**: Qdrant, Weaviate, Elasticsearch documentation

---

**Next Steps**: Implement Phase 1 (Filter-First strategy) in Week 5 Days 1-3.
