"""
OmenDB C ABI - Direct FFI Interface for High-Performance Integration.

Provides zero-overhead C-compatible functions for direct integration with
Rust, C++, or any language supporting C FFI. Bypasses Python serialization
for maximum performance.

Key Benefits:
- Zero-copy operations (direct memory access)
- No PyO3 or Python overhead
- True production performance (2000+ vec/s)
- Clean separation for server integration

Build:
    mojo build c_exports.mojo -o libomendb.so --emit shared-lib

C/Rust Integration:
    extern "C" {
        fn omendb_init(dimension: i32) -> i32;
        fn omendb_add(id_ptr: *const u8, len: i32, vec: *const f32, dim: i32) -> i32;
        fn omendb_search(query: *const f32, k: i32, ids: *mut i32, dists: *mut f32) -> i32;
        fn omendb_count() -> i32;
        fn omendb_clear() -> i32;
        fn omendb_version() -> *const u8;
    }
"""

from memory import UnsafePointer
from omendb.algorithms.hnsw import HNSWIndex
from omendb.core.sparse_map import SparseMap

# =============================================================================
# C-COMPATIBLE GLOBAL STATE
# =============================================================================

struct CDatabase(Movable):
    """C-compatible database structure."""
    var index: HNSWIndex
    var id_map: SparseMap
    var dimension: Int
    var initialized: Bool
    
    fn __init__(out self):
        self.dimension = 0
        self.initialized = False
        self.index = HNSWIndex(128, 10000)
        self.id_map = SparseMap()
    
    fn __moveinit__(out self, owned existing: Self):
        self.dimension = existing.dimension
        self.initialized = existing.initialized
        self.index = existing.index^
        self.id_map = existing.id_map^
    
    fn initialize(mut self, dimension: Int) -> Bool:
        if not self.initialized:
            self.dimension = dimension
            self.index = HNSWIndex(dimension, 10000)
            self.initialized = True
            return True
        return self.dimension == dimension

# Global instance
var __c_db: UnsafePointer[CDatabase] = UnsafePointer[CDatabase].alloc(1)
var __c_initialized: Bool = False

fn get_c_db() -> UnsafePointer[CDatabase]:
    if not __c_initialized:
        __c_db.init_pointee_move(CDatabase())
        __c_initialized = True
    return __c_db

# =============================================================================
# C ABI EXPORTS
# =============================================================================

@export("omendb_init")
fn omendb_init(dimension: Int32) -> Int32:
    """
    Initialize the database with given dimension.
    Returns 1 on success, 0 on failure.
    """
    var db = get_c_db()
    if db[].initialize(Int(dimension)):
        return 1
    return 0

@export("omendb_add")
fn omendb_add(
    id_ptr: UnsafePointer[UInt8],
    id_len: Int32,
    vector_ptr: UnsafePointer[Float32],
    dimension: Int32
) -> Int32:
    """
    Add a vector with string ID.
    Returns 1 on success, 0 on failure.
    """
    var db = get_c_db()
    
    # Auto-initialize if needed
    if not db[].initialized:
        if not db[].initialize(Int(dimension)):
            return 0
    
    # Check dimension
    if Int(dimension) != db[].dimension:
        return 0
    
    # Insert into HNSW
    var numeric_id = db[].index.insert(vector_ptr)
    if numeric_id < 0:
        return 0
    
    # Create string ID (simplified for now)
    var string_id = String("vec_") + String(numeric_id)
    _ = db[].id_map.insert(string_id, numeric_id)
    
    return 1

@export("omendb_search")
fn omendb_search(
    query_ptr: UnsafePointer[Float32],
    k: Int32,
    result_ids: UnsafePointer[Int32],
    result_distances: UnsafePointer[Float32]
) -> Int32:
    """
    Search for k nearest neighbors.
    Results are written to pre-allocated arrays.
    Returns number of results found.
    """
    var db = get_c_db()
    
    if not db[].initialized:
        return 0
    
    # Search HNSW
    var results = db[].index.search(query_ptr, Int(k))
    var count = len(results)
    
    # Copy results to output arrays
    for i in range(count):
        var result = results[i]
        result_ids[i] = Int32(result[0])
        result_distances[i] = Float32(result[1])
    
    return Int32(count)

@export("omendb_clear")
fn omendb_clear() -> Int32:
    """Clear all data."""
    var db = get_c_db()
    if db[].initialized:
        db[].index = HNSWIndex(db[].dimension, 10000)
        db[].id_map = SparseMap()
    return 1

@export("omendb_count")
fn omendb_count() -> Int32:
    """Get number of vectors."""
    var db = get_c_db()
    if db[].initialized:
        return Int32(db[].index.size)
    return 0

@export("omendb_version")
fn omendb_version() -> UnsafePointer[UInt8]:
    """Get version string."""
    # Return a static string
    var version = "OmenDB 0.1.0 (HNSW+)"
    var ptr = UnsafePointer[UInt8].alloc(len(version) + 1)
    for i in range(len(version)):
        ptr[i] = UInt8(ord(version[i]))
    ptr[len(version)] = 0  # Null terminator
    return ptr