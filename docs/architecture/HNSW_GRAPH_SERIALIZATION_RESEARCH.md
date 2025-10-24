# HNSW Graph Serialization Research

**Date**: October 24, 2025
**Goal**: Implement fast HNSW graph serialization to reduce load time from 30 minutes → <1 second

---

## Problem Statement

Current implementation:
- **Save**: Serialize vectors only with bincode (0.25s) ✅
- **Load**: Deserialize vectors + **rebuild HNSW** (~1800s = 30 minutes) ❌
- **Root cause**: HNSW rebuild has O(n log n) complexity

Target:
- **Load**: <1 second by serializing HNSW graph directly

---

## hnsw_rs API Analysis

### File Dump API

```rust
// From api.rs
fn file_dump(&self, path: &Path, file_basename: &str) -> anyhow::Result<String>
```

**Behavior**:
- Creates two files:
  - `{basename}.hnsw.graph` - Graph structure (topology)
  - `{basename}.hnsw.data` - Vector data
- Returns actual basename used (may append random number if files exist)
- Does NOT overwrite if mmap is active
- Uses `DumpMode::Full` to dump both graph and data

**Example**:
```rust
hnsw.file_dump(Path::new("/tmp/store"), "vectors")?;
// Creates: /tmp/store/vectors.hnsw.graph and /tmp/store/vectors.hnsw.data
```

### Load API

```rust
// From hnswio.rs
pub fn load_hnsw<'b, 'a, T, D>(&'a mut self) -> Result<Hnsw<'b, T, D>>
where
    T: 'static + Serialize + DeserializeOwned + Clone + Sized + Send + Sync + std::fmt::Debug,
    D: Distance<T> + Default + Send + Sync,
    'a: 'b,
```

**Lifetime constraints**:
- `'a` - Lifetime of the `HnswIo` loader
- `'b` - Lifetime of the returned `Hnsw` struct
- **Constraint**: `'a: 'b` (loader must outlive the Hnsw)
- **Implication**: Returned `Hnsw` borrows from the loader

**Example**:
```rust
let mut loader = HnswIo::new(path, filename)?;
let hnsw = loader.load_hnsw::<f32, DistL2>()?;
// hnsw borrows from loader - can't return both from function easily
```

### Load with Distance

```rust
pub fn load_hnsw_with_dist<'b, 'a, T, D>(&'a self, f: D) -> anyhow::Result<Hnsw<'b, T, D>>
```

Same lifetime constraints, but allows custom distance function (useful for DistPtr types that don't implement Default).

---

## Lifetime Issue Explained

**The Problem**:
```rust
fn load() -> Hnsw<'static, f32, DistL2> {
    let mut loader = HnswIo::new(...)?;
    let hnsw = loader.load_hnsw()?;  // hnsw borrows from loader
    hnsw  // ❌ ERROR: can't return hnsw because loader is dropped
}
```

**Why**:
- `HnswIo` reads files and may use memory mapping
- `Hnsw` may contain references to the loader's internal data
- When loader is dropped, those references become invalid
- Rust prevents this at compile time

---

## Solutions

### Option 1: Store Both Loader and Hnsw (RECOMMENDED)

```rust
pub struct VectorStore {
    pub vectors: Vec<Vector>,
    pub hnsw_index: Option<HNSWIndex<'static>>,

    // NEW: Store the loader to keep it alive
    hnsw_loader: Option<Box<HnswIo>>,

    dimensions: usize,
}
```

**Pros**:
- Safe Rust (no unsafe)
- Loader stays alive as long as Hnsw
- Minimal memory overhead (loader is small)

**Cons**:
- Slightly more complex storage

### Option 2: Disable mmap and use 'static (SIMPLER)

**Key insight**: The lifetime dependency exists because of potential mmap usage. If we ensure mmap is disabled, we might be able to use 'static.

```rust
let mut loader = HnswIo::new(path, filename)?;
loader.set_options(ReloadOptions::default());  // no mmap
let hnsw = loader.load_hnsw()?;
// Still has lifetime issue, but data is fully loaded
```

**Issue**: Even without mmap, the lifetime is still tied to the loader.

### Option 3: Box::leak (UNSAFE-ISH)

```rust
let loader = Box::leak(Box::new(HnswIo::new(...)?));
let hnsw = loader.load_hnsw()?;
// loader lives forever, so hnsw can be 'static
```

**Pros**:
- Simple to implement

**Cons**:
- Memory leak (loader never freed)
- Not idiomatic Rust
- Accumulates leaks on multiple loads

### Option 4: Unsafe Transmute (VERY DANGEROUS)

```rust
let hnsw: Hnsw<'static, f32, DistL2> = unsafe {
    std::mem::transmute(hnsw)
};
```

**Pros**:
- Works around lifetime system

**Cons**:
- **EXTREMELY DANGEROUS**
- Can cause undefined behavior
- Violates Rust safety guarantees
- **DO NOT USE**

---

## Recommended Approach

### Option 1: Store Loader with Hnsw

```rust
pub struct HNSWIndex<'a> {
    index: Hnsw<'a, f32, DistL2>,

    // NEW: Keep loader alive
    _loader: Option<Box<HnswIo>>,

    // ... other fields
}

impl HNSWIndex<'static> {
    pub fn from_file_dump(path: &str, filename: &str) -> Result<Self> {
        let mut loader = Box::new(HnswIo::new(
            Path::new(path),
            filename
        )?);

        let hnsw = loader.load_hnsw::<f32, DistL2>()?;

        Ok(Self {
            index: hnsw,
            _loader: Some(loader),
            // ... initialize other fields
        })
    }
}
```

**How it works**:
- Store `HnswIo` loader in `_loader` field
- Load `Hnsw` from the loader
- Both are stored in `HNSWIndex`
- Loader stays alive as long as `HNSWIndex` exists
- When `HNSWIndex` is dropped, loader is dropped (safe)

---

## Implementation Plan

1. ✅ Research hnsw_rs API (DONE)
2. Modify `HNSWIndex` to store loader
3. Add `from_file_dump()` method to HNSWIndex
4. Modify `VectorStore` to use graph serialization
5. Update `save_to_disk()` to call `file_dump()`
6. Update `load_from_disk()` to use `from_file_dump()`
7. Test roundtrip with 100K vectors
8. Benchmark: Verify <1s load time

---

## Expected Performance

| Operation | Current | With Graph Serialization |
|-----------|---------|-------------------------|
| Save | 0.25s | ~0.5s (graph + vectors) |
| Load | ~1800s | <1s (just deserialize) |
| Total | ~1800s | <2s |

**Improvement**: 900x faster load time

---

## Files to Modify

1. `src/vector/hnsw_index.rs`:
   - Add `_loader: Option<Box<HnswIo>>` field
   - Add `from_file_dump()` method
   - Update constructor to accept loader

2. `src/vector/store.rs`:
   - Update `save_to_disk()` to use `file_dump()`
   - Update `load_from_disk()` to use `from_file_dump()`
   - Remove rebuild logic (no longer needed!)

3. Tests:
   - Update to test graph serialization
   - Verify load time <1s

---

**Status**: Research complete ✅
**Next**: Implementation (estimated 2-3 hours)
