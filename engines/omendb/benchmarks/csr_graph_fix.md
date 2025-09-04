# CSRGraph Memory Bug Analysis

## Root Cause
Found multiple memory management issues in CSRGraph/VamanaGraph:

### 1. Copy Constructor Bug (CRITICAL)
**Location**: `csr_graph.mojo` line 143
```mojo
// CURRENT (WRONG):
self.row_offsets = UnsafePointer[Int32].alloc(existing.num_nodes + 1)

// SHOULD BE:
self.row_offsets = UnsafePointer[Int32].alloc(existing.capacity + 1)
```

The copy constructor allocates based on `num_nodes` but should preserve `capacity`.
This causes buffer overflow when adding nodes after copying.

### 2. Reallocation Off-by-One Error
**Location**: `csr_graph.mojo` line 182
```mojo
// Check if we need to reallocate
if node_idx >= self.capacity:
    self._reallocate(max(self.capacity * 2, node_idx + 100))
```

When `node_idx == capacity - 1`, we don't reallocate, but then access `row_offsets[node_idx + 1]` which equals `row_offsets[capacity]`. This is technically valid if we allocated `capacity + 1` elements, but it's cutting it close.

### 3. Memory Calculation Issues in Reallocation
Various `memcpy` calls use inconsistent size calculations that could cause issues.

## The Fix

### Fix 1: Copy Constructor
```mojo
fn __copyinit__(mut self, existing: CSRGraph) -> None:
    # ... other code ...
    
    # Preserve capacity, not just num_nodes
    self.capacity = existing.capacity  # ADD THIS LINE
    
    # Allocate based on capacity
    self.row_offsets = UnsafePointer[Int32].alloc(self.capacity + 1)
    memcpy(self.row_offsets, existing.row_offsets, (existing.num_nodes + 1) * sizeof[Int32]())
```

### Fix 2: Safer Reallocation Check
```mojo
fn add_node(mut self, id: String, vector: List[Float32]) -> Int:
    var node_idx = self.num_nodes
    
    # Check if we need space for node_idx + 1 in row_offsets
    if node_idx >= self.capacity - 1:  # Changed from >= to ensure buffer
        self._reallocate(max(self.capacity * 2, node_idx + 100))
```

## Impact
This bug causes segfault when:
1. Graph is copied (happens during operations)
2. New nodes are added to the copy
3. Access to row_offsets exceeds the incorrectly sized allocation

## Testing
The crash at 10K vectors is likely due to buffer flush creating a copy and then adding to it.