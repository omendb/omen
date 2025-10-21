# Phase 3 Week 2: JOIN Implementation Plan

**Date**: October 21, 2025
**Goal**: Implement INNER JOIN and LEFT JOIN support
**Target**: 40+ comprehensive tests, 2-3x performance vs SQLite maintained

---

## Current State

**SQL Engine Status**:
- ✅ Single table SELECT (with WHERE, ORDER BY, LIMIT)
- ✅ Aggregates (COUNT, SUM, AVG, MIN, MAX)
- ✅ GROUP BY support
- ❌ No JOIN support (line 642: "Only single table SELECT supported")

**Architecture**:
- Uses sqlparser-rs for SQL parsing
- Arrow-based row format
- ALEX index for fast lookups
- RocksDB for storage

---

## JOIN Implementation Strategy

### Phase 1: INNER JOIN (Days 1-2)

**Goal**: Implement basic INNER JOIN with equi-join conditions

**Algorithm**: Nested Loop Join (simple, correct, extensible)
```
for each row r1 in left_table:
    for each row r2 in right_table:
        if join_condition(r1, r2):
            emit combined_row(r1, r2)
```

**Optimization**: Use ALEX index for right table lookups when possible
- If join condition is `left.col = right.primary_key`, use index lookup
- Otherwise, fall back to full scan

**SQL Support**:
```sql
-- Basic equi-join
SELECT * FROM users INNER JOIN orders ON users.id = orders.user_id

-- Multiple columns
SELECT users.name, orders.total
FROM users
INNER JOIN orders ON users.id = orders.user_id

-- WHERE clause + JOIN
SELECT users.name, orders.total
FROM users
INNER JOIN orders ON users.id = orders.user_id
WHERE orders.total > 100
```

**Implementation Steps**:
1. Parse JOIN clause from `select.from[0].joins`
2. Extract join tables (left = from[0].relation, right = joins[0].relation)
3. Extract join condition from ON clause
4. Implement nested loop join
5. Combine row schemas (prefix columns with table names)
6. Project requested columns

### Phase 2: LEFT JOIN (Days 3)

**Goal**: Implement LEFT JOIN (all left rows + matching right rows)

**Algorithm**: Modified nested loop join
```
for each row r1 in left_table:
    found_match = false
    for each row r2 in right_table:
        if join_condition(r1, r2):
            emit combined_row(r1, r2)
            found_match = true
    if not found_match:
        emit combined_row(r1, null_row)
```

**SQL Support**:
```sql
-- Basic LEFT JOIN
SELECT users.name, orders.total
FROM users
LEFT JOIN orders ON users.id = orders.user_id

-- With WHERE clause
SELECT users.name, orders.total
FROM users
LEFT JOIN orders ON users.id = orders.user_id
WHERE users.age > 30
```

### Phase 3: Tests (Throughout)

**INNER JOIN Tests (20+)**:
- Basic equi-join (2 tables)
- Multiple join conditions
- WHERE clause + JOIN
- Column name conflicts (table.column qualification)
- No matching rows (empty result)
- All rows match
- LEFT.pk = RIGHT.fk (index-optimized path)
- LEFT.col = RIGHT.col (non-indexed path)
- Multiple SELECT columns
- Wildcard SELECT (*)
- Aggregates with JOIN
- ORDER BY with JOIN

**LEFT JOIN Tests (15+)**:
- Basic LEFT JOIN
- Left rows with no matches (NULL right columns)
- All left rows match
- WHERE clause + LEFT JOIN
- Aggregates with LEFT JOIN
- ORDER BY with LEFT JOIN

**Error Cases (5+)**:
- Ambiguous column names
- Invalid join condition
- Non-existent tables
- Non-existent columns in ON clause

---

## Implementation Details

### Parsing JOIN from sqlparser AST

```rust
// select.from: Vec<TableWithJoins>
// TableWithJoins {
//     relation: TableFactor,
//     joins: Vec<Join>
// }
// Join {
//     relation: TableFactor,
//     join_operator: JoinOperator
// }
// JoinOperator::Inner(JoinConstraint::On(Expr))
// JoinOperator::LeftOuter(JoinConstraint::On(Expr))
```

### Row Schema Combination

```rust
// Left table schema: [id, name, age]
// Right table schema: [id, user_id, total]
// Combined schema: [users.id, users.name, users.age, orders.id, orders.user_id, orders.total]
```

### Column Name Resolution

```rust
// SELECT users.name, orders.total FROM users JOIN orders ...
//
// Parse: Expr::CompoundIdentifier([users, name])
// Resolve: column index in combined schema
```

---

## Performance Expectations

**Target**: Maintain 1.5-3x speedup vs SQLite for JOINs

**Optimization Strategies**:
1. **Index-optimized joins**: Use ALEX index when joining on primary key
2. **Early filtering**: Apply WHERE clause before JOIN when possible
3. **Column projection**: Only materialize requested columns

**Benchmarks to Run**:
- INNER JOIN: 1M left rows, 1M right rows
- LEFT JOIN: 1M left rows, 500K right rows (50% match rate)
- Compare with SQLite on same queries

---

## Current Limitations (Acceptable for Week 2)

1. **Two tables only**: No multi-way joins yet (users JOIN orders JOIN products)
2. **Equi-join only**: No non-equality conditions (>, <, !=)
3. **ON clause only**: No USING or NATURAL joins
4. **No RIGHT JOIN**: Can be rewritten as LEFT JOIN
5. **No CROSS JOIN**: Can be added later
6. **No join reordering**: Execute joins in query order

---

## Files to Create/Modify

### Modify:
1. `src/sql_engine.rs`:
   - `execute_select()`: Detect and handle JOINs
   - `execute_join()`: New method for JOIN execution
   - `parse_join_condition()`: Extract join columns from ON clause
   - `combine_schemas()`: Merge left + right schemas
   - `resolve_column()`: Handle table.column references

### Create:
1. `tests/join_tests.rs`: INNER JOIN tests (20+)
2. `tests/left_join_tests.rs`: LEFT JOIN tests (15+)
3. `internal/PHASE_3_WEEK_2_JOIN_IMPLEMENTATION.md`: Detailed notes

---

## Success Criteria

- [ ] INNER JOIN implementation (equi-join on any columns)
- [ ] LEFT JOIN implementation
- [ ] 40+ comprehensive tests (all passing)
- [ ] Performance: 1.5-3x vs SQLite maintained
- [ ] Column name qualification (table.column)
- [ ] WHERE clause works with JOINs
- [ ] ORDER BY works with JOINs
- [ ] Documentation complete

---

## Next Steps

1. Implement `execute_join()` method in SqlEngine
2. Parse JOIN clause from sqlparser AST
3. Implement nested loop join algorithm
4. Handle schema combination
5. Add comprehensive tests
6. Benchmark and optimize

---

**Status**: Planning complete, ready to implement
**Next**: Implement INNER JOIN in `src/sql_engine.rs`
