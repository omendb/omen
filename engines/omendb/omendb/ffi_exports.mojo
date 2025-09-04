"""
FFI exports for OmenDB - C-compatible functions for Rust server integration.

This module provides C-compatible function exports that the Rust server
can call via FFI to access the Mojo vector database engine.
"""

from python import PythonObject
from collections import List, Dict
from memory import Pointer, UnsafePointer
from native import VectorStore


# Global store instance for FFI operations  
var __global_store: UnsafePointer[VectorStore] = UnsafePointer[VectorStore]()


@export("C")
fn omendb_create_store(dimension: Int32) -> UnsafePointer[UInt8]:
    """Create a new vector store instance."""
    try:
        var store = VectorStore()
        # Initialize with the specified dimension
        if dimension > 0:
            store.dimension = Int(dimension)
            # Initialize will happen on first vector add
            
            # Store globally (simplified approach for FFI)
            if not __global_store:
                __global_store = UnsafePointer[VectorStore].alloc(1)
            __global_store.init_pointee_move(store)
            
            # Return a non-null pointer to indicate success
            return UnsafePointer[UInt8].alloc(1)
        
        return UnsafePointer[UInt8]()
    except:
        return UnsafePointer[UInt8]()


@export("C") 
fn omendb_add_vector(
    store_ptr: Pointer[UInt8],
    id_ptr: Pointer[Int8],
    id_len: Int32,
    vector_ptr: Pointer[Float32],
    vector_len: Int32
) -> Int32:
    """Add a vector to the store. Returns 1 on success, 0 on failure."""
    try:
        if not __global_store:
            return 0
            
        # Convert C string to Mojo String
        var id_str = String()
        for i in range(Int(id_len)):
            id_str += chr(Int(id_ptr[i]))
        
        # Convert C array to Mojo List
        var vector = List[Float32]()
        for i in range(Int(vector_len)):
            vector.append(vector_ptr[i])
        
        # Add to store
        var success = __global_store.value().add(id_str, vector, Dict[String, PythonObject]())
        return 1 if success else 0
        
    except:
        return 0


@export("C")
fn omendb_query_vectors(
    store_ptr: Pointer[UInt8], 
    query_ptr: Pointer[Float32],
    query_len: Int32,
    k: Int32,
    results_ptr: Pointer[FFISearchResult]
) -> Int32:
    """Query similar vectors. Returns number of results found."""
    try:
        if not __global_store:
            return 0
            
        # Convert C array to Mojo List
        var query = List[Float32]()
        for i in range(Int(query_len)):
            query.append(query_ptr[i])
        
        # Query the store
        var results = __global_store.value().query(query, Int(k))
        var num_results = min(len(results), Int(k))
        
        # Convert results to C structure
        for i in range(num_results):
            var result = results[i]
            
            # Copy ID (simplified - assuming max 256 chars)
            var id_bytes = result.id.as_bytes()
            var copy_len = min(len(id_bytes), 255)
            for j in range(copy_len):
                results_ptr[i].id[j] = id_bytes[j]
            results_ptr[i].id[copy_len] = 0  # Null terminate
            
            # Copy distance
            results_ptr[i].distance = result.distance
            
            # Copy metadata (simplified as empty JSON)
            results_ptr[i].metadata[0] = ord('{')
            results_ptr[i].metadata[1] = ord('}')
            results_ptr[i].metadata[2] = 0
        
        return num_results
        
    except:
        return 0


@export("C")
fn omendb_get_stats(
    store_ptr: Pointer[UInt8],
    vector_count_ptr: Pointer[Int32],
    algorithm_ptr: Pointer[Int8],
    algorithm_len: Int32
) -> Int32:
    """Get store statistics. Returns 1 on success, 0 on failure."""
    try:
        if not __global_store:
            return 0
            
        var stats = __global_store.value().get_stats()
        
        # Set vector count
        vector_count_ptr[0] = stats.get("vector_count", PythonObject(0)).to_float64().to_Int()
        
        # Set algorithm name
        var algorithm = python.str(stats.get("algorithm", PythonObject("unknown")))
        var algo_bytes = algorithm.as_bytes()
        var copy_len = min(len(algo_bytes), Int(algorithm_len) - 1)
        for i in range(copy_len):
            algorithm_ptr[i] = algo_bytes[i]
        algorithm_ptr[copy_len] = 0  # Null terminate
        
        return 1
        
    except:
        return 0


@export("C")
fn omendb_destroy_store(store_ptr: Pointer[UInt8]) -> Int32:
    """Destroy the store instance. Returns 1 on success."""
    try:
        __global_store = UnsafePointer[VectorStore]()
        if store_ptr:
            store_ptr.free()
        return 1
    except:
        return 0


# FFI result structure matching Rust side
@value
struct FFISearchResult:
    """Search result structure for FFI communication."""
    var id: UnsafePointer[Int8]         # Vector ID (null-terminated)
    var distance: Float32               # Distance/similarity score
    var metadata: UnsafePointer[Int8]   # Metadata JSON (null-terminated)
    
    fn __init__(out self):
        # Allocate fixed-size buffers
        self.id = UnsafePointer[Int8].alloc(256)
        self.distance = 0.0
        self.metadata = UnsafePointer[Int8].alloc(1024)
        
        # Initialize buffers to zero
        for i in range(256):
            self.id[i] = 0
        for i in range(1024):
            self.metadata[i] = 0
