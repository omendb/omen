# String ID Memory Optimization Research
*Date: August 24, 2025*

## Executive Summary

String IDs consume ~5MB per 100K vectors. Converting to integer IDs would save significant memory but requires careful refactoring due to type system constraints in Mojo.

## Current State

### Memory Impact
- **String IDs**: ~50 bytes per ID (100K vectors = 5MB)
- **Integer IDs**: 8 bytes per ID (100K vectors = 0.8MB)
- **Potential Savings**: 84% reduction in ID storage

### Implementation Challenges

1. **Type System Constraints**
   - Mojo's Dict requires keys to implement `EqualityComparable` trait
   - Int64/SIMD types don't fully implement this trait
   - Must use plain `Int` type instead

2. **Mixed ID Types Throughout Codebase**
   - DiskANNIndex uses String IDs
   - Storage engines expect String IDs
   - Python API passes String IDs
   - Recovery paths use String IDs

3. **Backward Compatibility**
   - Need to maintain String ID support at API level
   - Requires dual mapping (String -> Int)
   - Hash collisions possible with simple hash functions

## Attempted Solution

### Approach
1. Changed internal storage to use `Dict[Int, ...]` instead of `Dict[String, ...]`
2. Added `string_id_map: Dict[String, Int]` for backward compatibility
3. Created `_get_or_create_int_id()` helper for conversion
4. Modified VectorBuffer to use `List[Int]` for IDs

### Why It Failed
- Too many interdependencies between components
- DiskANNIndex.build() expects List[String]
- Storage engines (MemoryMappedStorage, SnapshotStorage) use String IDs
- Python FFI functions deeply coupled with String IDs
- Over 30 locations requiring simultaneous changes

## Recommended Approach

### Phase 1: Measure & Profile
- [x] Quantify exact memory usage of String IDs
- [x] Profile where IDs are stored/duplicated
- [ ] Identify quick wins without major refactoring

### Phase 2: Incremental Migration
1. **Create ID abstraction layer**
   ```mojo
   struct VectorIDInt:
       var value: Int
       
   alias VectorID = VectorIDInt  # Can swap later
   ```

2. **Update components bottom-up**
   - Start with VectorBuffer (isolated component)
   - Then VectorStore internals
   - Then DiskANNIndex
   - Finally storage engines

3. **Maintain dual support**
   - Keep String ID methods for compatibility
   - Add Int ID methods alongside
   - Gradually deprecate String methods

### Phase 3: Optimize ID Generation
- Use sequential integers for new vectors
- Only create String mapping when needed
- Consider using Int32 instead of Int64 (4 bytes vs 8)

## Quick Wins (Do Now)

1. **Intern String IDs** - Use a global string pool to avoid duplicates
2. **Lazy ID generation** - Don't generate IDs if user doesn't need them
3. **Compress ID storage** - Use shorter prefixes or base64 encoding

## Performance Impact

### Current (String IDs)
- Memory: 50.5MB per 1M vectors (with int8 quantization)
- ID overhead: ~5MB per 100K vectors

### Target (Integer IDs)
- Memory: 45.5MB per 1M vectors
- ID overhead: ~0.8MB per 100K vectors
- **10% total memory reduction**

## Decision

**Defer full integer ID conversion** until after:
1. Python dict → Mojo structures (bigger impact)
2. Fix Python import time (usability issue)
3. Add beamwidth control (feature request)

The complexity of changing ID types throughout the codebase outweighs the 10% memory savings at this time. Focus on higher-impact optimizations first.

## Code References

Key files requiring changes:
- `omendb/native.mojo:100` - Dict declarations
- `omendb/core/vector_buffer.mojo:25` - ID storage
- `omendb/core/vector.mojo:634` - VectorID struct  
- `omendb/algorithms/diskann.mojo` - Index build methods
- `omendb/core/memory_mapped_storage.mojo` - Storage interface

## Lessons Learned

1. **Type system limitations** - Mojo's trait system is still evolving
2. **Tight coupling** - Components too interdependent for easy refactoring
3. **Abstraction needed** - Should have used ID abstraction from start
4. **Test coverage** - Need better tests before major refactoring