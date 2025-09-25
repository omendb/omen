# 🔧 OmenDB Engine Development Instructions

## 🚨 CRITICAL: Debugging Philosophy

### NEVER DO THIS:
- ❌ **Disable features** when they don't work perfectly
- ❌ **Replace implementations** instead of fixing bugs  
- ❌ **Panic and turn things off** when tests fail
- ❌ **Give up on optimizations** at first failure
- ❌ **Assume something is "fundamentally broken"** without proof

### ALWAYS DO THIS:
- ✅ **Debug systematically** to find root causes
- ✅ **Fix the actual bugs** in our code
- ✅ **Make features work properly** through iteration
- ✅ **Test with quality metrics**, then optimize
- ✅ **Understand WHY** something fails before changing it

## 📊 Current Issues to Debug (Not Disable!)

### 1. HNSW Poor Recall Issue
**Problem**: Low Recall@1 (0-70%) depending on scale
**DO NOT**: Disable HNSW or replace with flat buffer
**DO**: 
- Debug the search traversal logic
- Check if graph connectivity is correct
- Verify distance calculations
- Test if entry point selection affects results
- Fix the actual bug causing poor recall

### 2. Binary Quantization Distance Issues  
**Problem**: May be causing incorrect distance calculations
**DO NOT**: Just disable binary quantization
**DO**:
- Debug the binary_distance function
- Verify Hamming distance conversion
- Test with known vectors
- Fix the calculation if wrong

### 3. Hub Highway Navigation
**Problem**: May affect search quality
**DO NOT**: Just set use_flat_graph = False
**DO**:
- Understand how Hub Highway works
- Debug the hub selection logic
- Test with/without to understand impact
- Fix issues to make it work properly

### 4. Hash Map Crashes During Migration ⚠️ CRITICAL
**Problem**: SparseMap and ReverseSparseMap crash at indices 116-117 during bulk migration
**ROOT CAUSE IDENTIFIED**: Multiple issues in our custom hash map implementations:
- Recursive calls during resize (partially fixed)
- Hash collision or capacity management bugs at specific load factors
- Complex interaction between paired hash maps

**CURRENT STATUS**:
- ✅ Zero crashes achieved with safe workaround
- ⚠️ Limited functionality: Only first 115 vectors get ID mapping
- 🔍 Vectors insert successfully but unmapped vectors not searchable by string ID

**PROPER FIX NEEDED**:
- Rewrite SparseMap with proper capacity management
- Fix hash collision handling
- Add comprehensive unit tests for edge cases
- Consider using different hash map implementation

## 🔬 Systematic Debugging Approach

### Step 1: Reproduce Issues
```python
# Create minimal test case that shows the problem
vectors = known_test_set
ground_truth = compute_exact_neighbors(vectors)
hnsw_results = test_hnsw(vectors)
print(f"Recall: {compute_recall(hnsw_results, ground_truth)}")
```

### Step 2: Isolate Components
```python
# Test each component separately
test_distance_calculation()  # Is distance calc correct?
test_graph_connectivity()    # Is graph properly connected?
test_search_traversal()       # Is search logic correct?
test_candidate_selection()    # Are we selecting right candidates?
```

### Step 3: Fix Root Cause
```mojo
// Don't just change parameters - fix the actual bug
// Example: If distance calc is wrong, fix it:
fn fixed_distance(a: Vector, b: Vector) -> Float32:
    // Debug and fix the actual calculation
    return correct_calculation(a, b)
```

### Step 4: Validate Fix
```python
# Ensure fix actually improves quality
assert compute_recall(hnsw_results, ground_truth) > 0.9
assert performance_still_good()
```

## 🎯 Quality Targets (Don't Compromise!)

- **Recall@1**: Must be >90% (fix bugs until achieved)
- **Recall@10**: Must be >95% (industry standard)
- **Performance**: Maintain speed while fixing quality
- **All optimizations**: Must work correctly, not be disabled

## 💡 Remember

Our implementations are valuable and worth fixing:
- **Binary quantization**: 32x memory reduction when working
- **Hub Highway**: Novel optimization worth debugging
- **HNSW implementation**: Custom Mojo code we control
- **SIMD optimizations**: Significant speedups

Don't throw away months of work - debug and fix properly!

## 🔧 Current Focus

1. **✅ Zero crashes achieved** - Hash map workaround prevents segfaults
2. **Fix hash map implementations** - Rewrite SparseMap/ReverseSparseMap properly
3. **Restore full ID mapping** - All vectors should be searchable by string ID
4. **Maintain 26K+ vec/s performance** - Current bulk insertion is working
5. **Add hash map unit tests** - Prevent similar issues in future

---
*Philosophy: Every bug is solvable. Debug systematically. Fix properly.*