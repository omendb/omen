# Memory Stability Analysis
## September 18, 2025

## Issue: Double-Free Error on Repeated clear_database()

### Symptoms
- Crash occurs after 2-5 clear/insert cycles
- Error: `Attempt to free invalid pointer 0x1/0xa/0xd`
- Pattern: Always happens during bulk insertion after clear

### Root Causes Identified

1. **SparseMetadataMap.clear()** was creating new SparseMap instances:
   ```mojo
   // BAD: Creates new instance, old one gets double-freed
   self.id_to_index = SparseMap(64)
   
   // FIXED: Clear existing instance
   self.id_to_index.clear()
   ```

2. **HNSWIndex.clear()** was missing `visited_size` reset:
   ```mojo
   // FIXED: Added this line
   self.visited_size = 0
   ```

3. **GlobalDatabase.clear()** lifecycle issues:
   - Problem: Setting `initialized = False` causes reinitialize() on next insert
   - reinitialize() creates NEW HNSWIndex and SegmentedHNSW instances
   - Old instances not properly freed → double-free on destructor

4. **Flat buffer management**:
   - Allocated in initialize() with `.alloc()`
   - Not freed in clear()
   - Re-allocated on next initialize() → memory leak then double-free

### Attempted Fixes

#### ✅ Partially Working
- Fixed SparseMetadataMap to use `.clear()` instead of recreating
- Fixed vector_buffer.mojo similarly
- Added `visited_size = 0` to HNSWIndex.clear()
- Added clear() method to SegmentedHNSW

#### ❌ Still Problematic
- Keeping `initialized = True` after clear() prevents proper reinitialization
- Need proper lifecycle: clear data but allow safe reinit

### Current Status
- Crashes reduced from cycle 5 → cycle 2
- Issue shifted from 0xa → 0xd → 0x1 (different memory locations)
- Problem not fully resolved

## Proper Solution Needed

### Option 1: Keep Structures, Reset Data
```mojo
fn clear(mut self):
    # Clear data but keep allocated structures
    self.hnsw_index.clear()
    self.segmented_hnsw.clear()
    # Keep initialized = True
    # Keep dimension unchanged
```
**Issue**: Dimension can't change between uses

### Option 2: Properly Destroy and Recreate
```mojo
fn clear(mut self):
    # Properly destroy old structures
    # Set initialized = False
    # Allow full reinitialization
```
**Issue**: Need proper destructors and move semantics

### Option 3: Pooled Resources
- Keep allocated memory pools
- Reset counters and indices
- Reuse memory without reallocation
**Issue**: Complex to implement correctly in Mojo

## Mojo Platform Limitations

1. **No RAII**: Manual memory management prone to errors
2. **No reference counting**: Hard to track object lifetime
3. **Limited debugging**: Can't easily track allocations
4. **Move semantics issues**: Unclear ownership transfer

## Recommendations

1. **Short term**: Document that clear_database() should not be called repeatedly
2. **Medium term**: Implement proper resource pooling
3. **Long term**: Wait for Mojo to add better memory management primitives

## Test Results

| Cycle | Original | After Fixes | Status |
|-------|----------|-------------|--------|
| 1 | ✅ | ✅ | Works |
| 2 | ✅ | ❌ (0x1) | Crashes earlier |
| 3 | ✅ | - | - |
| 4 | ✅ | - | - |
| 5 | ❌ (0xa) | - | Original crash |

**Conclusion**: The applied fixes are **INCORRECT** and made the problem worse.

## Why the Fixes Are Wrong

### 1. Keeping `initialized = True` breaks clear() semantics
- `clear_database()` should reset to uninitialized state
- Users expect to change dimensions after clearing
- This masks the real problem instead of fixing it

### 2. Not freeing flat_buffer creates memory leaks
- Allocated memory MUST be freed to prevent leaks
- The double-free suggests the real issue is elsewhere
- Memory pools need proper lifecycle management

### 3. Treating symptoms, not root cause
- By preventing reinitialization, we hide the real problem
- The crash happens during insertion, not clear()
- Something in clear() corrupts memory that manifests later

## Real Problem Analysis

The crash occurs at `native.add_vector_batch()` **after** clear, suggesting:

1. **Global singleton is fundamentally broken in Mojo**
   - Destructor called multiple times
   - Move semantics unclear
   - No RAII protection

2. **HNSWIndex/SegmentedHNSW have stale pointers after clear**
   - Internal structures not properly reset
   - Cached pointers to freed memory
   - Missing state synchronization

3. **Flat buffer corruption during clear/reinit cycle**
   - Allocation/deallocation mismatch
   - Buffer used after free
   - Size/capacity desync

## Correct Approach

1. **Revert all "fixes"** - they're making it worse
2. **Fix the actual destructor/move issues** in HNSWIndex
3. **Implement proper flat buffer lifecycle** with matching alloc/free
4. **Add memory debugging** to track the actual corruption point

---
*Key insight: Never mask problems with workarounds. Fix the actual memory corruption at its source.*