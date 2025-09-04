"""
Python bindings for OmenDB embedded database.

This module provides a C-compatible interface for Python bindings using FFI.
The C interface allows Python to interact with the embedded database through
ctypes without requiring direct Mojo-Python interop.

Architecture:
- EmbeddedDatabase (Mojo) ↔ C Interface (this module) ↔ Python (ctypes)
- Opaque handles for memory management
- Error codes for exception handling
- Type conversion between Mojo, C, and Python types
"""

from sys.ffi import external_call, DLHandle
from memory import UnsafePointer, memcpy
from collections import List, Dict, Optional
from storage.embedded_db import EmbeddedDatabase
from core.vector import Float32Vector  
from core.metadata import Metadata
from util.logging import LogLevel

# C-compatible types
alias c_char = UInt8
alias c_int = Int32
alias c_uint = UInt32
alias c_float = Float32
alias c_size_t = UInt64

# Error codes for C interface
alias OMENDB_SUCCESS: c_int = 0
alias OMENDB_ERROR_INVALID_HANDLE: c_int = -1
alias OMENDB_ERROR_INVALID_PARAMS: c_int = -2
alias OMENDB_ERROR_DATABASE_ERROR: c_int = -3
alias OMENDB_ERROR_MEMORY_ERROR: c_int = -4
alias OMENDB_ERROR_IO_ERROR: c_int = -5

# Opaque handle structures for Python
struct OmenDBHandle:
    """Opaque handle wrapping EmbeddedDatabase for C interface."""
    var database: EmbeddedDatabase
    var is_valid: Bool
    
    fn __init__(out self, database: EmbeddedDatabase):
        self.database = database^
        self.is_valid = True
    
    fn __moveinit__(out self, owned existing: Self):
        self.database = existing.database^
        self.is_valid = existing.is_valid

struct VectorHandle:
    """Opaque handle wrapping Float32Vector for C interface."""
    var vector: Float32Vector
    var is_valid: Bool
    
    fn __init__(out self, vector: Float32Vector):
        self.vector = vector^
        self.is_valid = True
    
    fn __moveinit__(out self, owned existing: Self):
        self.vector = existing.vector^
        self.is_valid = existing.is_valid

struct MetadataHandle:
    """Opaque handle wrapping Metadata for C interface.""" 
    var metadata: Metadata
    var is_valid: Bool
    
    fn __init__(out self, metadata: Metadata):
        self.metadata = metadata^
        self.is_valid = True
    
    fn __moveinit__(out self, owned existing: Self):
        self.metadata = existing.metadata^
        self.is_valid = existing.is_valid

# Global handle storage for C interface
var _database_handles = Dict[UInt64, OmenDBHandle]()
var _vector_handles = Dict[UInt64, VectorHandle]()
var _metadata_handles = Dict[UInt64, MetadataHandle]()
var _next_handle_id: UInt64 = 1

# Handle management functions
fn _create_database_handle(database: EmbeddedDatabase) -> UInt64:
    """Create a new database handle and return its ID."""
    var handle_id = _next_handle_id
    _next_handle_id += 1
    _database_handles[handle_id] = OmenDBHandle(database^)
    return handle_id

fn _create_vector_handle(vector: Float32Vector) -> UInt64:
    """Create a new vector handle and return its ID."""
    var handle_id = _next_handle_id
    _next_handle_id += 1
    _vector_handles[handle_id] = VectorHandle(vector^)
    return handle_id

fn _create_metadata_handle(metadata: Metadata) -> UInt64:
    """Create a new metadata handle and return its ID."""
    var handle_id = _next_handle_id
    _next_handle_id += 1
    _metadata_handles[handle_id] = MetadataHandle(metadata^)
    return handle_id

fn _get_database_handle(handle_id: UInt64) -> Optional[OmenDBHandle]:
    """Get database handle by ID."""
    if handle_id in _database_handles:
        return _database_handles[handle_id]
    return Optional[OmenDBHandle]()

fn _get_vector_handle(handle_id: UInt64) -> Optional[VectorHandle]:
    """Get vector handle by ID."""
    if handle_id in _vector_handles:
        return _vector_handles[handle_id]
    return Optional[VectorHandle]()

fn _get_metadata_handle(handle_id: UInt64) -> Optional[MetadataHandle]:
    """Get metadata handle by ID."""
    if handle_id in _metadata_handles:
        return _metadata_handles[handle_id]
    return Optional[MetadataHandle]()

fn _free_database_handle(handle_id: UInt64):
    """Free a database handle."""
    if handle_id in _database_handles:
        _ = _database_handles.pop(handle_id)

fn _free_vector_handle(handle_id: UInt64):
    """Free a vector handle."""
    if handle_id in _vector_handles:
        _ = _vector_handles.pop(handle_id)

fn _free_metadata_handle(handle_id: UInt64):
    """Free a metadata handle."""
    if handle_id in _metadata_handles:
        _ = _metadata_handles.pop(handle_id)

# C interface functions for Python bindings

@external("C")
fn omendb_create(path: UnsafePointer[c_char], read_only: c_int, log_level: c_int) -> UInt64:
    """
    Create/open embedded database.
    
    Args:
        path: C string path to database file
        read_only: 1 for read-only, 0 for read-write
        log_level: Logging level (0-4)
    
    Returns:
        Database handle ID, or 0 on error
    """
    try:
        # Convert C string to Mojo String
        var mojo_path = String()
        var i = 0
        while path[i] != 0:
            mojo_path += chr(Int(path[i]))
            i += 1
        
        # Create database
        var database = EmbeddedDatabase(mojo_path, read_only != 0, Int(log_level))
        return _create_database_handle(database^)
    
    except:
        return 0  # Error - return invalid handle

@external("C")
fn omendb_close(db_handle: UInt64) -> c_int:
    """
    Close database and free resources.
    
    Args:
        db_handle: Database handle ID
        
    Returns:
        OMENDB_SUCCESS or error code
    """
    var handle_opt = _get_database_handle(db_handle)
    if not handle_opt:
        return OMENDB_ERROR_INVALID_HANDLE
    
    var handle = handle_opt.value()
    handle.database.close()
    _free_database_handle(db_handle)
    return OMENDB_SUCCESS

@external("C") 
fn omendb_is_healthy(db_handle: UInt64) -> c_int:
    """
    Check database health.
    
    Args:
        db_handle: Database handle ID
        
    Returns:
        1 if healthy, 0 if not, -1 on error
    """
    var handle_opt = _get_database_handle(db_handle)
    if not handle_opt:
        return OMENDB_ERROR_INVALID_HANDLE
    
    var handle = handle_opt.value()
    return 1 if handle.database.is_healthy() else 0

@external("C")
fn omendb_set_dimension(db_handle: UInt64, dimension: c_uint) -> c_int:
    """
    Set vector dimension.
    
    Args:
        db_handle: Database handle ID
        dimension: Vector dimension
        
    Returns:
        OMENDB_SUCCESS or error code
    """
    var handle_opt = _get_database_handle(db_handle)
    if not handle_opt:
        return OMENDB_ERROR_INVALID_HANDLE
    
    try:
        var handle = handle_opt.value()
        handle.database.set_dimension(UInt32(dimension))
        return OMENDB_SUCCESS
    except:
        return OMENDB_ERROR_DATABASE_ERROR

@external("C")
fn vector_create_f32(data: UnsafePointer[c_float], dimension: c_size_t) -> UInt64:
    """
    Create Float32Vector from array.
    
    Args:
        data: Array of float32 values
        dimension: Vector dimension
        
    Returns:
        Vector handle ID, or 0 on error
    """
    try:
        # Convert C array to Mojo List
        var vector_data = List[Float32]()
        for i in range(Int(dimension)):
            vector_data.append(data[i])
        
        # Create vector
        var vector = Float32Vector(vector_data)
        return _create_vector_handle(vector^)
    
    except:
        return 0

@external("C")
fn vector_get_data(vector_handle: UInt64, out_data: UnsafePointer[c_float], out_dimension: UnsafePointer[c_size_t]) -> c_int:
    """
    Get vector data as array.
    
    Args:
        vector_handle: Vector handle ID
        out_data: Output array (must be pre-allocated)
        out_dimension: Output dimension
        
    Returns:
        OMENDB_SUCCESS or error code
    """
    var handle_opt = _get_vector_handle(vector_handle)
    if not handle_opt:
        return OMENDB_ERROR_INVALID_HANDLE
    
    var handle = handle_opt.value()
    var vector = handle.vector
    var dimension = vector.dimension()
    
    # Copy data to output array
    for i in range(dimension):
        out_data[i] = vector[i]
    
    out_dimension[] = c_size_t(dimension)
    return OMENDB_SUCCESS

@external("C")
fn vector_free(vector_handle: UInt64):
    """Free vector handle."""
    _free_vector_handle(vector_handle)

@external("C")
fn metadata_create() -> UInt64:
    """
    Create empty metadata object.
    
    Returns:
        Metadata handle ID, or 0 on error
    """
    try:
        var metadata = Metadata()
        return _create_metadata_handle(metadata^)
    except:
        return 0

@external("C")
fn metadata_set(metadata_handle: UInt64, key: UnsafePointer[c_char], value: UnsafePointer[c_char]) -> c_int:
    """
    Set metadata key-value pair.
    
    Args:
        metadata_handle: Metadata handle ID
        key: C string key
        value: C string value
        
    Returns:
        OMENDB_SUCCESS or error code
    """
    var handle_opt = _get_metadata_handle(metadata_handle)
    if not handle_opt:
        return OMENDB_ERROR_INVALID_HANDLE
    
    try:
        # Convert C strings to Mojo Strings
        var mojo_key = String()
        var i = 0
        while key[i] != 0:
            mojo_key += chr(Int(key[i]))
            i += 1
        
        var mojo_value = String()
        i = 0
        while value[i] != 0:
            mojo_value += chr(Int(value[i]))
            i += 1
        
        var handle = handle_opt.value()
        handle.metadata.set(mojo_key, mojo_value)
        return OMENDB_SUCCESS
    
    except:
        return OMENDB_ERROR_DATABASE_ERROR

@external("C")
fn metadata_free(metadata_handle: UInt64):
    """Free metadata handle."""
    _free_metadata_handle(metadata_handle)

@external("C")
fn omendb_insert_vector(db_handle: UInt64, id: UnsafePointer[c_char], 
                       vector_handle: UInt64, metadata_handle: UInt64) -> c_int:
    """
    Insert vector into database.
    
    Args:
        db_handle: Database handle ID
        id: C string vector ID
        vector_handle: Vector handle ID
        metadata_handle: Metadata handle ID (0 for no metadata)
        
    Returns:
        OMENDB_SUCCESS or error code
    """
    var db_handle_opt = _get_database_handle(db_handle)
    if not db_handle_opt:
        return OMENDB_ERROR_INVALID_HANDLE
    
    var vector_handle_opt = _get_vector_handle(vector_handle)
    if not vector_handle_opt:
        return OMENDB_ERROR_INVALID_HANDLE
    
    try:
        # Convert C string ID to Mojo String
        var mojo_id = String()
        var i = 0
        while id[i] != 0:
            mojo_id += chr(Int(id[i]))
            i += 1
        
        var db_handle_val = db_handle_opt.value()
        var vector_handle_val = vector_handle_opt.value()
        
        # Get metadata if provided
        var metadata_opt = Optional[Metadata]()
        if metadata_handle != 0:
            var metadata_handle_opt = _get_metadata_handle(metadata_handle)
            if metadata_handle_opt:
                metadata_opt = metadata_handle_opt.value().metadata
        
        # Insert vector
        var success = db_handle_val.database.insert_vector(mojo_id, vector_handle_val.vector, metadata_opt)
        return OMENDB_SUCCESS if success else OMENDB_ERROR_DATABASE_ERROR
    
    except:
        return OMENDB_ERROR_DATABASE_ERROR

@external("C")
fn omendb_delete_vector(db_handle: UInt64, id: UnsafePointer[c_char]) -> c_int:
    """
    Delete vector from database.
    
    Args:
        db_handle: Database handle ID
        id: C string vector ID
        
    Returns:
        OMENDB_SUCCESS or error code
    """
    var handle_opt = _get_database_handle(db_handle)
    if not handle_opt:
        return OMENDB_ERROR_INVALID_HANDLE
    
    try:
        # Convert C string to Mojo String
        var mojo_id = String()
        var i = 0
        while id[i] != 0:
            mojo_id += chr(Int(id[i]))
            i += 1
        
        var handle = handle_opt.value()
        var success = handle.database.delete_vector(mojo_id)
        return OMENDB_SUCCESS if success else OMENDB_ERROR_DATABASE_ERROR
    
    except:
        return OMENDB_ERROR_DATABASE_ERROR

@external("C")
fn omendb_search_vectors(db_handle: UInt64, query_handle: UInt64, limit: c_int,
                        results: UnsafePointer[UnsafePointer[c_char]], count: UnsafePointer[c_int]) -> c_int:
    """
    Search for similar vectors.
    
    Args:
        db_handle: Database handle ID
        query_handle: Query vector handle ID
        limit: Maximum number of results
        results: Output array of result IDs (must be pre-allocated)
        count: Output count of actual results
        
    Returns:
        OMENDB_SUCCESS or error code
    """
    var db_handle_opt = _get_database_handle(db_handle)
    if not db_handle_opt:
        return OMENDB_ERROR_INVALID_HANDLE
        
    var query_handle_opt = _get_vector_handle(query_handle)
    if not query_handle_opt:
        return OMENDB_ERROR_INVALID_HANDLE
    
    try:
        var db_handle_val = db_handle_opt.value()
        var query_handle_val = query_handle_opt.value()
        
        var result_ids = db_handle_val.database.search_vectors(query_handle_val.vector, Int(limit))
        
        # Convert results to C strings for Python consumption
        var actual_count = min(len(result_ids), Int(limit))
        count[] = c_int(actual_count)
        
        # Allocate and copy string results to C format
        for i in range(actual_count):
            var id_str = result_ids[i]
            var id_len = len(id_str)
            
            # Allocate C string (Python will manage this memory)
            var c_str = UnsafePointer[c_char].alloc(id_len + 1)
            
            # Copy string contents
            for j in range(id_len):
                c_str[j] = ord(id_str[j])
            c_str[id_len] = 0  # Null terminate
            
            results[i] = c_str
        
        return OMENDB_SUCCESS
    
    except:
        return OMENDB_ERROR_DATABASE_ERROR

@external("C")
fn omendb_flush(db_handle: UInt64) -> c_int:
    """
    Flush database to disk.
    
    Args:
        db_handle: Database handle ID
        
    Returns:
        OMENDB_SUCCESS or error code
    """
    var handle_opt = _get_database_handle(db_handle)
    if not handle_opt:
        return OMENDB_ERROR_INVALID_HANDLE
    
    try:
        var handle = handle_opt.value()
        handle.database.flush()
        return OMENDB_SUCCESS
    except:
        return OMENDB_ERROR_IO_ERROR

@external("C")
fn omendb_get_stats(db_handle: UInt64, stats_buffer: UnsafePointer[c_char], buffer_size: c_size_t) -> c_int:
    """
    Get database statistics.
    
    Args:
        db_handle: Database handle ID
        stats_buffer: Output buffer for stats string
        buffer_size: Size of output buffer
        
    Returns:
        OMENDB_SUCCESS or error code
    """
    var handle_opt = _get_database_handle(db_handle)
    if not handle_opt:
        return OMENDB_ERROR_INVALID_HANDLE
    
    try:
        var handle = handle_opt.value()
        var stats = handle.database.get_stats()
        
        # Copy stats to buffer (truncate if necessary)
        var copy_length = min(len(stats), Int(buffer_size) - 1)
        for i in range(copy_length):
            stats_buffer[i] = ord(stats[i])
        stats_buffer[copy_length] = 0  # Null terminate
        
        return OMENDB_SUCCESS
    except:
        return OMENDB_ERROR_DATABASE_ERROR

# Library initialization function
@external("C")
fn omendb_library_init() -> c_int:
    """Initialize the OmenDB library."""
    return OMENDB_SUCCESS

@external("C") 
fn omendb_library_cleanup() -> c_int:
    """Cleanup the OmenDB library and free all handles."""
    _database_handles.clear()
    _vector_handles.clear()
    _metadata_handles.clear()
    _next_handle_id = 1
    return OMENDB_SUCCESS