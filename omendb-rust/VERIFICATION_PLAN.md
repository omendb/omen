# OmenDB Comprehensive Verification Plan

**Goal**: Find every bug, edge case, and correctness issue before open sourcing

---

## 🔍 Critical Code Paths to Verify

### 1. Insert Path
- [ ] Duplicate primary keys (should error or update?)
- [ ] NULL values in primary key
- [ ] NULL values in other columns
- [ ] Empty strings
- [ ] Very large strings (>1GB)
- [ ] Integer overflow (i64::MAX + 1)
- [ ] Float special values (NaN, Infinity, -Infinity)
- [ ] Insert to non-existent table
- [ ] Insert with wrong column count
- [ ] Insert with wrong types
- [ ] Concurrent inserts to same table
- [ ] Insert while reading
- [ ] Insert with disk full

### 2. Query Path (SELECT)
- [ ] Query empty table
- [ ] Query non-existent table
- [ ] Query with non-existent column
- [ ] WHERE on non-indexed column
- [ ] WHERE with NULL comparison
- [ ] WHERE with type mismatch (string = int)
- [ ] Range query with start > end
- [ ] Range query with same start and end
- [ ] Point query for non-existent key
- [ ] Query during concurrent insert
- [ ] Query with corrupted data

### 3. Index Path
- [ ] Index with 0 keys
- [ ] Index with 1 key
- [ ] Index with duplicate keys
- [ ] Index with keys in reverse order
- [ ] Index with gaps in keys (1, 100, 10000)
- [ ] Index with negative keys
- [ ] Index retraining during queries
- [ ] Index with more keys than capacity
- [ ] Learned index prediction returns out of bounds

### 4. Storage Path
- [ ] Empty Arrow batch
- [ ] Single row Arrow batch
- [ ] Very large Arrow batch (millions of rows)
- [ ] Persistence with no data
- [ ] Persistence with disk full
- [ ] Load from corrupted Parquet file
- [ ] Load from missing file
- [ ] Load with schema mismatch
- [ ] Concurrent writes to same Parquet file

### 5. WAL Path
- [ ] WAL with no entries
- [ ] WAL recovery with corrupted entry
- [ ] WAL recovery with partial entry
- [ ] WAL recovery with out-of-order entries
- [ ] WAL with disk full during write
- [ ] WAL checkpoint with concurrent writes
- [ ] Multiple WAL files
- [ ] WAL recovery after crash mid-operation

### 6. Catalog Path
- [ ] Catalog with no tables
- [ ] Create table with existing name
- [ ] Drop non-existent table
- [ ] Query non-existent table
- [ ] Concurrent table creation
- [ ] Catalog persistence failure
- [ ] Catalog load with corrupted metadata

---

## 🧪 Test Categories

### Category 1: Correctness Tests ✅ HIGH PRIORITY
**Goal**: Verify results are always correct

1. **SQL Correctness**
   - Compare SELECT results with expected rows
   - Verify WHERE filters correctly
   - Verify INSERT actually stores data
   - Test boundary conditions (min/max values)

2. **Index Correctness**
   - Verify get() returns correct row
   - Verify range_query() returns all rows in range
   - Verify no false positives or false negatives
   - Compare learned index results with linear scan

3. **Data Integrity**
   - Verify no data loss on restart
   - Verify WAL recovery produces same state
   - Verify persistence doesn't corrupt data
   - Verify concurrent operations don't lose data

### Category 2: Edge Case Tests ✅ HIGH PRIORITY
**Goal**: Find all the ways to break the system

1. **Boundary Values**
   - Test with 0 rows
   - Test with 1 row
   - Test with 1M rows
   - Test with i64::MAX
   - Test with empty strings
   - Test with very large strings

2. **Invalid Inputs**
   - Malformed SQL
   - Type mismatches
   - NULL values
   - Out-of-range values
   - Non-UTF8 strings

3. **Resource Limits**
   - Out of memory
   - Out of disk space
   - Too many tables
   - Too many columns
   - Too many rows

### Category 3: Performance Tests ⚠️ MEDIUM PRIORITY
**Goal**: Verify performance claims hold at scale

1. **Large Scale**
   - Test with 1M rows (validate 9.85x claim)
   - Test with 10M rows
   - Measure memory usage
   - Measure disk usage
   - Verify no memory leaks

2. **Performance Degradation**
   - Test performance after many inserts
   - Test performance with fragmented data
   - Test performance without retraining
   - Compare with B-tree at same scale

### Category 4: Concurrency Tests ⚠️ MEDIUM PRIORITY
**Goal**: Find race conditions and deadlocks

1. **Concurrent Reads**
   - Multiple threads reading same table
   - Read while inserting
   - Read while persisting

2. **Concurrent Writes**
   - Multiple threads inserting to same table
   - Concurrent inserts with same key
   - Write while reading
   - Write while checkpointing WAL

### Category 5: Recovery Tests ⚠️ MEDIUM PRIORITY
**Goal**: Verify durability guarantees

1. **Crash Scenarios**
   - Crash during INSERT
   - Crash during CREATE TABLE
   - Crash during persistence
   - Crash during WAL write

2. **Corruption Scenarios**
   - Corrupted Parquet file
   - Corrupted WAL file
   - Corrupted catalog metadata
   - Partial writes

### Category 6: Security Tests 🔴 LOW PRIORITY (for now)
**Goal**: Prevent malicious inputs

1. **SQL Injection**
   - Test with `'; DROP TABLE users--`
   - Test with injection in WHERE clause
   - Test with injection in VALUES

2. **Path Traversal**
   - Test with `../../etc/passwd` in table names
   - Test with special characters in filenames

3. **DoS**
   - Very large SQL queries
   - Many small queries (exhaust resources)
   - Infinite loops in queries

---

## 🎯 Immediate Priorities

### Phase 1: Critical Correctness (TODAY)
1. ✅ Test duplicate primary keys
2. ✅ Test NULL values
3. ✅ Test empty tables
4. ✅ Test WHERE correctness (compare with scan)
5. ✅ Test index correctness (compare with linear search)

### Phase 2: Edge Cases (TOMORROW)
1. Test boundary values (0 rows, 1 row, MAX values)
2. Test invalid inputs (type mismatches, malformed SQL)
3. Test resource limits (OOM, disk full)

### Phase 3: Scale (NEXT 2 DAYS)
1. Test with 1M rows
2. Test with 10M rows
3. Measure memory leaks
4. Validate performance claims

### Phase 4: Concurrency & Recovery (NEXT 3 DAYS)
1. Test concurrent operations
2. Test WAL recovery scenarios
3. Test crash scenarios

---

## 🚨 Known Issues to Investigate

### From Code Review:
1. **TableIndex**: Does binary search handle edge cases?
2. **SQL Engine**: What happens with type mismatches?
3. **WAL**: Is sequence number always correct?
4. **Storage**: Can Arrow batch be empty?
5. **Catalog**: What if metadata file is corrupted?

### Suspected Issues:
1. **Duplicate keys**: No unique constraint enforcement?
2. **NULL handling**: Not clear if supported
3. **Error propagation**: Some `unwrap()` calls may panic
4. **Memory leaks**: No explicit test for leaks
5. **Concurrent access**: RwLock usage seems correct but untested

---

## 📊 Success Criteria

### Before Open Source Release:
- [ ] All critical correctness tests pass
- [ ] All edge case tests pass
- [ ] No data loss scenarios
- [ ] No panics in normal operation
- [ ] Performance claims validated at scale
- [ ] All known issues documented or fixed
- [ ] Comprehensive test coverage (>80%)
- [ ] Security basics covered (SQL injection prevented)

### Acceptable Trade-offs:
- ✅ Some SQL features missing (documented)
- ✅ Some performance edge cases (random data is slower)
- ✅ Some operational features missing (no monitoring)
- ❌ Data loss or corruption (NOT ACCEPTABLE)
- ❌ Crashes on valid inputs (NOT ACCEPTABLE)
- ❌ Security vulnerabilities (NOT ACCEPTABLE)

---

**Next Step**: Start executing Phase 1 tests immediately.