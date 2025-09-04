# Storage Engine Comprehensive Audit

## Current Architecture Overview

### Components
1. **VectorStore** (native.mojo) - Main storage coordinator
2. **VectorBuffer** (vector_buffer.mojo) - Hot buffer for incoming vectors  
3. **DiskANNIndex** (diskann_csr.mojo) - Main index using CSR graph
4. **VamanaGraph** (csr_graph.mojo) - CSR storage for vectors and edges
5. **MemoryMappedStorage** (memory_mapped_storage.mojo) - Persistence layer
6. **ComponentMemoryStats** (memory_tracker.mojo) - Memory tracking

## Data Flow Analysis

### Add Vector Path
```
User API (add) 
    ↓
VectorStore.add_vector()
    ↓
Check quantization flags → Apply if enabled (NOT WORKING)
    ↓
VectorBuffer.add() → Store in buffer.data array
    ↓
When buffer full (10K) → _flush_buffer_to_main()
    ↓
DiskANNIndex.add() → VamanaGraph.add_node()
    ↓
NORMALIZATION HAPPENS HERE (line 110 csr_graph.mojo)
    ↓
Store normalized vector in graph.vectors array
```

### Get Vector Path
```
User API (get)
    ↓
VectorStore.get_vector()
    ↓
Check if in buffer (-1) or main index (1)
    ↓
If in buffer: Extract from buffer.data
If in main: graph.get_vector_ptr() 
    ↓
Return NORMALIZED vector (not original!)
```

### Persistence Path
```
set_persistence()
    ↓
Creates MemoryMappedStorage
    ↓
On checkpoint():
    - Should write vectors to mmap files
    - BROKEN: Writes 0 vectors
    ↓
On recovery():
    - Should load from mmap files  
    - BROKEN: Loads 0 vectors
```

## Critical Issues Found

### 1. 🔴 Vector Normalization Changes Data
**Location**: csr_graph.mojo line 110
```mojo
self.vectors[start_idx + i] = vector[i] * inv_norm
```
**Problem**: Stores normalized vectors, returns normalized on retrieval
**Impact**: Users get different values than they stored
**Fix**: Either store original + normalized, or document this behavior

### 2. 🔴 Memory-Mapped Storage Broken
**Location**: memory_mapped_storage.mojo checkpoint/recovery
**Problem**: 
- Checkpoint reports success but saves 0 vectors
- Recovery loads nothing
**Root Cause**: Need to investigate checkpoint logic
**Fix**: Debug why vectors aren't written to mmap

### 3. 🔴 Quantization Never Applied
**Location**: native.mojo lines 526-548, 747-769
```mojo
if self.use_quantization:  # This is False by default!
```
**Problem**: 
- Flags default to False (line 140, 142)
- enable_quantization() sets flag but doesn't affect existing DB
**Fix**: Apply quantization in add path when enabled

### 4. 🔴 Memory Tracking Broken
**Location**: Multiple accumulation points
**Problems**:
- Graph memory accumulated incorrectly
- CSR graph memory_bytes() not called
- Double counting in some paths
**Fix**: Centralize memory tracking

### 5. ⚠️ No Original Vector Storage
**Problem**: After removing vector_store dict, no way to get original
**Impact**: 
- Quantization can't work (needs original for comparison)
- Users can't get exact values back
**Fix**: Need to store originals somewhere

## Storage States & Transitions

### Vector Lifecycle
1. **Buffer Stage**: Raw vector in buffer.data array
2. **Flush Trigger**: Buffer full (10K vectors)
3. **Index Build**: Add to DiskANNIndex (gets normalized)
4. **Graph Storage**: Normalized vector in CSR graph
5. **Persistence**: Should checkpoint to mmap (broken)

### Memory Locations
- **buffer.data**: Raw vectors (Float32 array)
- **graph.vectors**: Normalized vectors (Float32 array)
- **quantized_vectors**: Dict (unused - quantization not applied)
- **binary_vectors**: Dict (unused - quantization not applied)
- **vector_store**: Dict (removed - was causing double storage)

## Root Cause Analysis

### Why Persistence Fails
1. Memory-mapped storage initialized but not integrated with flush
2. Checkpoint logic doesn't iterate through vectors
3. Recovery doesn't know where to look for data

### Why Quantization Doesn't Work
1. Flags default to False
2. No global state change when enabled mid-session
3. Quantization check happens per-vector, not globally
4. Original vectors not stored after our fix

### Why Memory Tracking is Wrong
1. Multiple accumulation points
2. Graph memory added on each flush
3. CSR graph's memory_bytes() not integrated
4. No central tracking authority

## Recommended Fixes (Priority Order)

### 1. Fix Vector Storage (CRITICAL)
- Store both original and normalized vectors
- Or clearly document normalization behavior
- Enable quantization to work

### 2. Fix Memory-Mapped Persistence (CRITICAL)
- Integrate with flush logic
- Ensure checkpoint writes actual data
- Fix recovery to load correctly

### 3. Fix Quantization (HIGH)
- Make flags affect current session
- Store originals for quantization
- Apply in add path

### 4. Fix Memory Tracking (MEDIUM)
- Single source of truth
- Call graph.memory_bytes()
- Remove accumulation

### 5. Add Integration Tests (HIGH)
- End-to-end persistence test
- Quantization verification
- Memory tracking validation

## Architecture Recommendations

### Short Term
1. Add original vector storage back (but efficiently)
2. Fix persistence integration points
3. Make quantization work globally
4. Centralize memory tracking

### Long Term
1. Consider multi-segment architecture (like Lucene)
2. Implement proper compaction strategy
3. Add incremental persistence (not just checkpoint)
4. Support different vector types natively

## Conclusion

The storage engine has solid bones but critical integration issues:
- Normalization changes user data
- Persistence is completely broken  
- Quantization is disabled despite API
- Memory tracking is fiction

These must be fixed before optimization work.