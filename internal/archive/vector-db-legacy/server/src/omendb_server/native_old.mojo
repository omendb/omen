"""
Working OmenDB native module using proper Mojo-Python interop.

Based on the Modular examples in external/modular/examples/mojo/python-interop/
"""

from python import PythonObject, Python
from python.bindings import PythonModuleBuilder
from os import abort
from collections import List, Dict
from math import sqrt
from memory import UnsafePointer, Span
from algorithm import vectorize
from sys import simdwidthof

# Import our core modules
from core.vector import Float32Vector, from_list
from core.metadata import Metadata
from core.record import VectorRecord
# HNSW removed - using RoarGraph only
from algorithms.roar_graph import RoarGraphIndex

# Debug flag - set to False for production
alias DEBUG_LOGGING = False

# Database state management structure
# RoarGraph-only architecture - no index selection needed
alias ROARGRAPH_ONLY = True

struct DatabaseState(Movable):
    var roargraph_index: Optional[RoarGraphIndex]  # Primary storage and search
    var dimension: Int
    var database_path: String
    var is_open: Bool
    var vectors_cached: Bool
    
    fn __init__(out self):
        self.roargraph_index = Optional[RoarGraphIndex]()
        self.dimension = 0
        self.database_path = ""
        self.is_open = False
        self.vectors_cached = False

# Global database state - safe module-level storage
var _database_state = DatabaseState()

fn get_state() -> UnsafePointer[DatabaseState]:
    """Get the global database state."""
    return UnsafePointer.address_of(_database_state)

fn init_state():
    """Initialize the global state."""
    _database_state = DatabaseState()

# Export the Python module initialization function
@export
fn PyInit_native() -> PythonObject:
    """
    Python module initialization function.
    
    This function is called when Python imports the native module.
    """
    try:
        # Initialize global state
        init_state()
        
        # Create Python module matching this Mojo module name
        var module = PythonModuleBuilder("native")
        
        # Register basic functions
        module.def_function[test_connection]("test_connection")
        module.def_function[get_version]("get_version")
        
        # Register database operations
        module.def_function[create_database]("create_database")
        module.def_function[set_dimension]("set_dimension")
        module.def_function[insert_vector]("insert_vector")
        module.def_function[insert_vector]("add_vector")  # API compatibility
        module.def_function[insert_vector_with_metadata]("insert_vector_with_metadata")
        module.def_function[search_vectors]("search_vectors")
        module.def_function[search_vectors_with_filter]("search_vectors_with_filter")
        module.def_function[delete_vector]("delete_vector")
        module.def_function[get_vector]("get_vector")
        module.def_function[get_stats]("get_stats")
        module.def_function[is_healthy]("is_healthy")
        module.def_function[flush_database]("flush_database")
        module.def_function[flush_database]("save_to_file")  # API compatibility
        module.def_function[load_from_file]("load_from_file")
        module.def_function[close_database]("close_database")
        module.def_function[reset_state]("reset_state")
        module.def_function[cosine_similarity_test]("cosine_similarity_test")
        
        # Register HNSW index operations
        # HNSW functions removed - using RoarGraph only
        
        # Register RoarGraph index operations (commented out due to compilation issues)
        # module.def_function[create_roargraph_index]("create_roargraph_index")
        # module.def_function[roargraph_insert]("roargraph_insert")
        # module.def_function[roargraph_search]("roargraph_search")
        # module.def_function[roargraph_size]("roargraph_size")
        # module.def_function[roargraph_save]("roargraph_save")
        
        return module.finalize()
        
    except e:
        return abort[PythonObject](
            String("Failed to create OmenDB native module: ", e)
        )

fn test_connection() raises -> PythonObject:
    """Test that the native module is working."""
    return PythonObject("Connection successful")

fn create_database(path: PythonObject, read_only: PythonObject, log_level: PythonObject) raises -> PythonObject:
    """Create or open database file with comprehensive error handling."""
    try:
        var path_str = String(path)
        
        # Validate path is not empty
        if len(path_str) == 0:
            print("Native: Error - Empty database path provided")
            return PythonObject(False)
        
        # Check if database is already open
        var state = get_state()
        if state[].is_open and state[].database_path == path_str:
            print("Native: Database already open at " + path_str)
            return PythonObject(True)
        
        # Close existing database if open
        if state[].is_open:
            print("Native: Closing existing database before opening new one")
            _ = close_database()
        
        state[].database_path = path_str
        
        # Create database file if it doesn't exist (with error recovery)
        try:
            _create_database_file(path_str)
        except file_error:
            print("Native: Failed to create database file: " + String(file_error))
            state[].database_path = ""
            return PythonObject(False)
        
        # Load existing vectors from file (with error recovery)
        try:
            _load_vectors_from_file(path_str)
        except load_error:
            print("Native: Warning - Failed to load existing vectors: " + String(load_error))
            print("Native: Continuing with empty database")
            # Clear any partial state
            state[].vectors.clear()
            state[].vector_ids.clear()
            state[].metadata.clear()
            state[].vectors_cached = True
        
        state[].is_open = True
        
        # Enable HNSW indexing by default for performance
        @parameter
        if DEBUG_LOGGING:
            print("Native: HNSW enabled status: " + String(state[].index_type == INDEX_HNSW))
        
        if state[].index_type != INDEX_HNSW:
            @parameter
            if DEBUG_LOGGING:
                print("Native: Creating HNSW index...")
            var hnsw_index = HnswIndex(16, 200)  # Default M=16, ef_construction=200
            state[].hnsw_index = Optional[HnswIndex](hnsw_index)
            state[].index_type = INDEX_HNSW
            @parameter
            if DEBUG_LOGGING:
                print("Native: HNSW index enabled with default parameters")
        else:
            @parameter
            if DEBUG_LOGGING:
                print("Native: HNSW already enabled")
        
        @parameter
        if DEBUG_LOGGING:
            print("Native: Database opened at " + path_str + " with " + String(len(state[].vector_ids)) + " vectors")
        return PythonObject(True)
        
    except e:
        print("Native: Critical error in create_database: " + String(e))
        # Ensure clean state on failure
        var state = get_state()
        state[].is_open = False
        state[].database_path = ""
        state[].vectors.clear()
        state[].vector_ids.clear()
        state[].metadata.clear()
        state[].dimension = 0
        state[].vectors_cached = False
        return PythonObject(False)

fn get_version() raises -> PythonObject:
    """Get version information."""
    return PythonObject("0.1.0-native")

fn set_dimension(dimension: PythonObject) raises -> PythonObject:
    """Set vector dimension."""
    try:
        var dim = Int(dimension)
        var state = get_state()
        state[].dimension = dim
        print("Native: Set dimension to " + String(dim))
        return PythonObject(True)
    except:
        return PythonObject(False)

fn insert_vector(id: PythonObject, vector_data: PythonObject) raises -> PythonObject:
    """Insert vector into storage with comprehensive error handling."""
    try:
        var state = get_state()
        # Check if database is open
        if not state[].is_open:
            print("Native: Error - Database not open")
            return PythonObject(False)
        
        var id_str = String(id)
        
        # Validate ID
        if len(id_str) == 0:
            print("Native: Error - Empty vector ID")
            return PythonObject(False)
        
        # Check if vector already exists
        if id_str in state[].vectors:
            print("Native: Warning - Vector '" + id_str + "' already exists, overwriting")
            # Remove from vector_ids to avoid duplicates
            for i in range(len(state[].vector_ids)):
                if state[].vector_ids[i] == id_str:
                    _ = state[].vector_ids.pop(i)
                    break
        
        # Convert Python list to Mojo List[Float32] with validation
        var mojo_data = List[Float32]()
        try:
            var vector_len = Int(vector_data.__len__())
            
            if vector_len == 0:
                print("Native: Error - Empty vector data")
                return PythonObject(False)
            
            # Check dimension
            if state[].dimension > 0 and vector_len != state[].dimension:
                print("Native: Dimension mismatch - expected " + String(state[].dimension) + ", got " + String(vector_len))
                return PythonObject(False)
            
            # Convert Python floats to Mojo Float32 with validation
            for i in range(vector_len):
                try:
                    var py_val = Float64(vector_data[i])
                    var float_val = Float32(py_val)
                    
                    # Check for invalid values (NaN, infinity)
                    if float_val != float_val:  # NaN check
                        print("Native: Error - Invalid float value at index " + String(i))
                        return PythonObject(False)
                    
                    mojo_data.append(float_val)
                except conversion_error:
                    print("Native: Error - Failed to convert vector element at index " + String(i))
                    return PythonObject(False)
            
        except data_error:
            print("Native: Error - Failed to process vector data: " + String(data_error))
            return PythonObject(False)
        
        # Create Float32Vector
        var vector = from_list[DType.float32](mojo_data)
        
        # Store in memory
        state[].vectors[id_str] = vector
        state[].vector_ids.append(id_str)
        state[].vectors_cached = True  # We have vectors in memory
        
        # Save to file immediately (with error recovery)
        try:
            _save_vector_to_file(id_str, vector)
            @parameter
            if DEBUG_LOGGING:
                print("Native: Vector saved to file: " + state[].database_path)
        except file_error:
            print("Native: Warning - Failed to save vector to file: " + String(file_error))
            print("Native: Vector stored in memory only")
            # Continue - vector is still in memory
        
        # Add to HNSW index if enabled
        try:
            if state[].index_type == INDEX_HNSW and state[].hnsw_index:
                var record = VectorRecord[DType.float32](id_str, vector)
                var index = state[].hnsw_index.value()
                index.insert(record)
                state[].hnsw_index = Optional[HnswIndex](index)
                @parameter
                if DEBUG_LOGGING:
                    print("Native: Added vector to HNSW index")
        except hnsw_error:
            # HNSW indexing failed, but vector is still in memory and file
            print("Native: Warning - Failed to add vector to HNSW index: " + String(hnsw_error))
        
        @parameter
        if DEBUG_LOGGING:
            print("Native: Inserted vector '" + id_str + "' with dimension " + String(vector.dimension()))
        return PythonObject(True)
        
    except e:
        print("Native: Insert failed - " + String(e))
        # Clean up any partial state
        try:
            var id_str = String(id)
            var state = get_state()
            if id_str in state[].vectors:
                # Remove from vectors if insertion failed after memory storage
                for i in range(len(state[].vector_ids)):
                    if state[].vector_ids[i] == id_str:
                        _ = state[].vector_ids.pop(i)
                        break
        except:
            pass
        return PythonObject(False)

fn insert_vector_with_metadata(id: PythonObject, vector_data: PythonObject, metadata_dict: PythonObject) raises -> PythonObject:
    """Insert vector with metadata into storage."""
    try:
        var id_str = String(id)
        
        # Convert Python list to Mojo List[Float32]
        var mojo_data = List[Float32]()
        var vector_len = Int(vector_data.__len__())
        
        # Check dimension
        var state = get_state()
        if state[].dimension > 0 and vector_len != state[].dimension:
            print("Native: Dimension mismatch - expected " + String(state[].dimension) + ", got " + String(vector_len))
            return PythonObject(False)
        
        # Convert Python floats to Mojo Float32
        for i in range(vector_len):
            var py_val = Float64(vector_data[i])
            mojo_data.append(Float32(py_val))
        
        # Create Float32Vector
        var vector = from_list[DType.float32](mojo_data)
        
        # Create metadata from Python dict
        var metadata = Metadata()
        
        # Extract metadata from Python dict
        if metadata_dict is not None:
            try:
                # Use direct Python dict access for known keys
                # This is a simplified approach for testing
                
                # Try common metadata keys
                var test_keys = List[String]()
                test_keys.append("category")
                test_keys.append("type")
                test_keys.append("author")
                test_keys.append("description")
                test_keys.append("tags")
                
                for i in range(len(test_keys)):
                    var key = test_keys[i]
                    try:
                        var value = metadata_dict[key]
                        if value is not None:
                            metadata.set(key, String(value))
                    except:
                        # Key doesn't exist, skip
                        pass
                    
            except e:
                print("Native: Failed to parse metadata: " + String(e))
        
        # Store in memory
        state[].vectors[id_str] = vector
        state[].metadata[id_str] = metadata
        state[].vector_ids.append(id_str)
        state[].vectors_cached = True
        
        # Save to file with metadata
        _save_vector_with_metadata_to_file(id_str, vector, metadata)
        
        print("Native: Inserted vector '" + id_str + "' with metadata and dimension " + String(vector.dimension()) + " (saved to file)")
        return PythonObject(True)
        
    except e:
        print("Native: Insert with metadata failed - " + String(e))
        return PythonObject(False)

fn search_vectors(query_data: PythonObject, limit: PythonObject) raises -> PythonObject:
    """Search for similar vectors with optimized caching, sorting, and error handling."""
    try:
        # Check if database is open
        var state = get_state()
        if not state[].is_open:
            print("Native: Error - Database not open")
            return PythonObject("")
        
        var limit_int = Int(limit)
        var result_count = len(state[].vector_ids)
        
        # Validate limit parameter
        if limit_int <= 0:
            print("Native: Error - Invalid limit: " + String(limit_int))
            return PythonObject("")
        
        if result_count == 0:
            print("Native: No vectors stored")
            return PythonObject("")
        
        # Ensure vectors are cached in memory
        if not state[].vectors_cached:
            try:
                _ensure_vectors_cached()
            except cache_error:
                print("Native: Error - Failed to cache vectors: " + String(cache_error))
                return PythonObject("")
        
        # Try HNSW search first for better performance
        if state[].index_type == INDEX_HNSW and state[].hnsw_index:
            try:
                var hnsw_results = hnsw_search(query_data, limit, PythonObject(50))  # Default ef=50
                # Convert HNSW results to similarity format
                var python = Python.import_module("builtins")
                var similarity_results = python.list()
                
                for i in range(len(hnsw_results)):
                    var result = hnsw_results[i]
                    var result_dict = python.dict()
                    result_dict["id"] = result["id"]
                    # Convert distance to similarity (1 - distance for cosine)
                    var distance = Float64(result["distance"])
                    result_dict["similarity"] = 1.0 - distance
                    similarity_results.append(result_dict)
                
                @parameter
                if DEBUG_LOGGING:
                    print("Native: HNSW search returned " + String(len(similarity_results)) + " results")
                return similarity_results
            except hnsw_error:
                print("Native: Warning - HNSW search failed, falling back to brute force: " + String(hnsw_error))
                # Fall through to brute force search
        
        
        # Convert Python list to Mojo List[Float32] with validation
        var mojo_data = List[Float32]()
        try:
            var vector_len = Int(query_data.__len__())
            
            if vector_len == 0:
                print("Native: Error - Empty query vector")
                return PythonObject("")
            
            # Check dimension
            if state[].dimension > 0 and vector_len != state[].dimension:
                print("Native: Query dimension mismatch - expected " + String(state[].dimension) + ", got " + String(vector_len))
                return PythonObject("")
            
            for i in range(vector_len):
                try:
                    var py_val = Float64(query_data[i])
                    var float_val = Float32(py_val)
                    
                    # Check for invalid values
                    if float_val != float_val:  # NaN check
                        print("Native: Error - Invalid query vector value at index " + String(i))
                        return PythonObject("")
                    
                    mojo_data.append(float_val)
                except conversion_error:
                    print("Native: Error - Failed to convert query vector element at index " + String(i))
                    return PythonObject("")
                    
        except query_error:
            print("Native: Error - Failed to process query vector: " + String(query_error))
            return PythonObject("")
        
        var query_vector = from_list[DType.float32](mojo_data)
        
        # Calculate similarities with top-k selection optimization and error handling
        var similarities = List[Float64]()
        var indices = List[Int]()
        var valid_results = 0
        
        for i in range(result_count):
            try:
                var vector_id = state[].vector_ids[i]
                if vector_id in state[].vectors:
                    var stored_vector = state[].vectors[vector_id]
                    var similarity = _cosine_similarity(query_vector, stored_vector)
                    
                    # Validate similarity result
                    if similarity == similarity:  # Not NaN
                        similarities.append(similarity)
                        indices.append(i)
                        valid_results += 1
                    else:
                        print("Native: Warning - Invalid similarity for vector " + vector_id)
                else:
                    print("Native: Warning - Vector ID not found in storage: " + vector_id)
            except similarity_error:
                print("Native: Warning - Failed to compute similarity for vector " + String(i) + ": " + String(similarity_error))
                continue
        
        if valid_results == 0:
            print("Native: No valid search results")
            return PythonObject("")
        
        # Use selection sort for top-k results (O(k*n) vs O(n²) for full sort)
        var max_results = limit_int if limit_int < valid_results else valid_results
        try:
            _partial_sort_topk(similarities, indices, max_results)
        except sort_error:
            print("Native: Warning - Sort failed, returning unsorted results: " + String(sort_error))
            max_results = valid_results if valid_results < limit_int else limit_int
        
        # Build result list with top results
        var python = Python.import_module("builtins")
        var result_list = python.list()
        
        for i in range(max_results):
            if i < len(indices) and indices[i] < len(state[].vector_ids):
                var result_dict = python.dict()
                result_dict["id"] = state[].vector_ids[indices[i]]
                result_dict["similarity"] = similarities[i]
                result_list.append(result_dict)
        
        @parameter
        if DEBUG_LOGGING:
            print("Native: Search returned " + String(max_results) + " results from " + String(valid_results) + " valid comparisons")
        return result_list
        
    except e:
        print("Native: Search failed - " + String(e))
        var python = Python.import_module("builtins")
        return python.list()

fn search_vectors_with_filter(query_data: PythonObject, limit: PythonObject, filter_dict: PythonObject) raises -> PythonObject:
    """Search for similar vectors with metadata filtering."""
    try:
        var state = get_state()
        var limit_int = Int(limit)
        var result_count = len(state[].vector_ids)
        
        if result_count == 0:
            print("Native: No vectors stored")
            return PythonObject("")
        
        # Ensure vectors are cached in memory
        if not state[].vectors_cached:
            _ensure_vectors_cached()
        
        # Convert Python list to Mojo List[Float32]
        var mojo_data = List[Float32]()
        var vector_len = Int(query_data.__len__())
        
        # Check dimension
        if state[].dimension > 0 and vector_len != state[].dimension:
            print("Native: Query dimension mismatch")
            return PythonObject("")
        
        for i in range(vector_len):
            var py_val = Float64(query_data[i])
            mojo_data.append(Float32(py_val))
        
        var query_vector = from_list[DType.float32](mojo_data)
        
        # Calculate similarities with metadata filtering
        var similarities = List[Float64]()
        var indices = List[Int]()
        
        for i in range(result_count):
            var vector_id = state[].vector_ids[i]
            
            # Check if vector passes metadata filter
            if filter_dict is not None and _metadata_passes_filter(vector_id, filter_dict):
                var stored_vector = state[].vectors[vector_id]
                var similarity = _cosine_similarity(query_vector, stored_vector)
                similarities.append(similarity)
                indices.append(i)
            elif filter_dict is None:
                # No filter, include all vectors
                var stored_vector = state[].vectors[vector_id]
                var similarity = _cosine_similarity(query_vector, stored_vector)
                similarities.append(similarity)
                indices.append(i)
        
        # Use selection sort for top-k results
        var filtered_count = len(similarities)
        var max_results = limit_int if limit_int < filtered_count else filtered_count
        
        if max_results > 0:
            _partial_sort_topk(similarities, indices, max_results)
        
        # Build result string with top results
        var result = String("")
        
        for i in range(max_results):
            if i > 0:
                result += ","
            result += state[].vector_ids[indices[i]]
        
        print("Native: Filtered search returned " + String(max_results) + " results from " + String(filtered_count) + " matches")
        return PythonObject(result)
        
    except e:
        print("Native: Filtered search failed - " + String(e))
        return PythonObject("")

fn delete_vector(id: PythonObject) raises -> PythonObject:
    """Delete vector by ID."""
    try:
        var id_str = String(id)
        var state = get_state()
        
        # Find and remove vector
        for i in range(len(state[].vector_ids)):
            if state[].vector_ids[i] == id_str:
                _ = state[].vector_ids.pop(i)
                # Note: Dict removal is complex, so we'll leave it for now
                print("Native: Deleted vector '" + id_str + "'")
                return PythonObject(True)
        
        print("Native: Vector '" + id_str + "' not found")
        return PythonObject(False)
        
    except e:
        print("Native: Delete failed - " + String(e))
        return PythonObject(False)

fn get_stats() raises -> PythonObject:
    """Get storage statistics."""
    var state = get_state()
    var count = len(state[].vector_ids)
    
    # Create Python dict for stats
    var python = Python.import_module("builtins")
    var stats = python.dict()
    stats["vector_count"] = count
    stats["dimension"] = state[].dimension
    stats["database_path"] = state[].database_path
    stats["is_open"] = state[].is_open
    stats["vectors_cached"] = state[].vectors_cached
    stats["status"] = "ready"
    
    return stats

fn is_healthy() raises -> PythonObject:
    """Check if database is healthy."""
    return PythonObject(True)

fn flush_database() raises -> PythonObject:
    """Flush database changes to disk."""
    try:
        var state = get_state()
        if state[].database_path != "":
            _flush_all_vectors_to_file()
            print("Native: Database flushed to " + state[].database_path)
        return PythonObject(True)
    except e:
        print("Native: Flush failed: " + String(e))
        return PythonObject(False)

fn close_database() raises -> PythonObject:
    """Close database and ensure all data is saved with comprehensive cleanup."""
    var success = True
    
    try:
        var state = get_state()
        if state[].is_open and state[].database_path != "":
            print("Native: Closing database at " + state[].database_path)
            
            # Attempt to flush all data
            try:
                _flush_all_vectors_to_file()
                print("Native: All vectors flushed to file")
            except flush_error:
                print("Native: Warning - Failed to flush vectors: " + String(flush_error))
                success = False  # Mark as partial failure but continue cleanup
            
            print("Native: Database closed (path: " + state[].database_path + ")")
        elif state[].is_open:
            print("Native: Closing database (no file path set)")
        else:
            print("Native: Database already closed")
        
    except e:
        print("Native: Error during close: " + String(e))
        success = False
    
    # Always perform cleanup regardless of errors
    try:
        var state = get_state()
        state[].is_open = False
        state[].database_path = ""
        state[].vectors_cached = False
        print("Native: Database state cleaned up")
    except cleanup_error:
        print("Native: Warning - Cleanup error: " + String(cleanup_error))
        success = False
    
    return PythonObject(success)

fn reset_state() raises -> PythonObject:
    """Reset all state for testing with comprehensive cleanup."""
    try:
        print("Native: Resetting state...")
        
        # Close database if open
        var state = get_state()
        if state[].is_open:
            print("Native: Closing open database before reset")
            _ = close_database()
        
        # Clear all data structures
        state[].vectors.clear()
        state[].vector_ids.clear()
        state[].metadata.clear()
        
        # Reset all state variables
        state[].dimension = 0
        state[].database_path = ""
        state[].is_open = False
        state[].vectors_cached = False
        
        print("Native: State reset complete")
        return PythonObject(True)
    except reset_error:
        print("Native: Reset failed: " + String(reset_error))
        # Force reset critical state even if some operations failed
        var state = get_state()
        state[].is_open = False
        state[].database_path = ""
        state[].vectors_cached = False
        return PythonObject(False)

fn get_vector(id: PythonObject) raises -> PythonObject:
    """Get a vector by ID."""
    try:
        var id_str = String(id)
        var state = get_state()
        
        # Check if database is open
        if not state[].is_open:
            print("Native: Error - Database not open")
            return PythonObject()
        
        # Load vectors if not cached
        if not state[].vectors_cached:
            _ensure_vectors_cached()
        
        # Check if vector exists
        if id_str not in state[].vectors:
            return PythonObject()  # Vector not found
        
        # Get the vector
        var vector = state[].vectors[id_str]
        
        # Convert to Python list
        var python = Python.import_module("builtins")
        var py_list = python.list()
        
        for i in range(vector.dimension()):
            _ = py_list.append(Float64(vector[i]))
        
        return py_list
        
    except e:
        print("Native: Failed to get vector: " + String(e))
        return PythonObject()

fn load_from_file(path: PythonObject) raises -> PythonObject:
    """Load database from file and return number of vectors loaded."""
    try:
        var path_str = String(path)
        
        # Set up database state
        var state = get_state()
        state[].database_path = path_str
        
        # Load vectors from file
        var vectors_before = len(state[].vector_ids)
        _load_vectors_from_file(path_str)
        var vectors_after = len(state[].vector_ids)
        var loaded_count = vectors_after - vectors_before
        
        state[].is_open = True
        state[].vectors_cached = True
        
        # Enable HNSW indexing by default for performance
        @parameter
        if DEBUG_LOGGING:
            print("Native: HNSW enabled status: " + String(state[].index_type == INDEX_HNSW))
        
        if state[].index_type != INDEX_HNSW:
            @parameter
            if DEBUG_LOGGING:
                print("Native: Creating HNSW index...")
            var hnsw_index = HnswIndex(16, 200)  # Default M=16, ef_construction=200
            state[].hnsw_index = Optional[HnswIndex](hnsw_index)
            state[].index_type = INDEX_HNSW
            @parameter
            if DEBUG_LOGGING:
                print("Native: HNSW index enabled with default parameters")
        else:
            @parameter
            if DEBUG_LOGGING:
                print("Native: HNSW already enabled")
        
        print("Native: Loaded " + String(loaded_count) + " vectors from " + path_str)
        return PythonObject(loaded_count)
        
    except e:
        print("Native: Failed to load from file: " + String(e))
        return PythonObject(0)

fn cosine_similarity_test(vector1: PythonObject, vector2: PythonObject) raises -> PythonObject:
    """Calculate cosine similarity between two vectors for testing."""
    try:
        # Convert Python lists to Mojo vectors
        var v1_list = List[Float32]()
        var v2_list = List[Float32]()
        
        var len1 = Int(vector1.__len__())
        var len2 = Int(vector2.__len__())
        
        if len1 != len2:
            return PythonObject(0.0)
        
        # Convert to Float32 lists
        for i in range(len1):
            v1_list.append(Float32(Float64(vector1[i])))
            v2_list.append(Float32(Float64(vector2[i])))
        
        # Create Float32Vector objects
        var vec1 = from_list[DType.float32](v1_list)
        var vec2 = from_list[DType.float32](v2_list)
        
        # Calculate cosine similarity
        var similarity = _cosine_similarity(vec1, vec2)
        
        return PythonObject(similarity)
        
    except e:
        print("Native: Cosine similarity test failed: " + String(e))
        return PythonObject(0.0)

# Binary helper functions for V3 format
fn _int_to_4bytes(val: Int) -> String:
    """Convert integer to 4-byte little-endian string."""
    var b1 = chr(val & 0xFF)
    var b2 = chr((val >> 8) & 0xFF)
    var b3 = chr((val >> 16) & 0xFF)
    var b4 = chr((val >> 24) & 0xFF)
    return b1 + b2 + b3 + b4

fn _float_to_4bytes(val: Float32) -> String:
    """Convert float32 to 4-byte little-endian string."""
    # For now, convert to int representation of float bits
    var int_val = Int(val * 1000000)  # Scale and convert to int
    return _int_to_4bytes(int_val)

fn _4bytes_to_int(bytes: String) -> Int:
    """Convert 4-byte little-endian string to integer."""
    if len(bytes) != 4:
        return 0
    var b1 = ord(bytes[0])
    var b2 = ord(bytes[1])
    var b3 = ord(bytes[2])
    var b4 = ord(bytes[3])
    return b1 + (b2 << 8) + (b3 << 16) + (b4 << 24)

fn _4bytes_to_float(bytes: String) -> Float32:
    """Convert 4-byte little-endian string to float32."""
    var int_val = _4bytes_to_int(bytes)
    return Float32(int_val) / 1000000.0  # Scale back to float

fn _ensure_vectors_cached() raises:
    """Ensure all vectors are loaded and cached in memory."""
    var state = get_state()
    if not state[].vectors_cached and state[].database_path != "":
        # Vectors are already loaded in _load_vectors_from_file during create_database
        state[].vectors_cached = True
        print("Native: Vectors cached for search optimization")

fn _partial_sort_topk(mut similarities: List[Float64], mut indices: List[Int], k: Int) raises:
    """Partial sort to get top-k results using selection algorithm (O(n) average)."""
    var n = len(similarities)
    
    # For small k, use selection sort on just the top k elements
    if k <= 10:
        for i in range(k):
            var max_idx = i
            for j in range(i + 1, n):
                if similarities[j] > similarities[max_idx]:
                    max_idx = j
            
            if max_idx != i:
                # Swap similarities
                var temp_sim = similarities[i]
                similarities[i] = similarities[max_idx]
                similarities[max_idx] = temp_sim
                
                # Swap indices
                var temp_idx = indices[i]
                indices[i] = indices[max_idx]
                indices[max_idx] = temp_idx
    else:
        # For larger k, use quickselect to partition around k-th element
        _quickselect_partition(similarities, indices, 0, n - 1, k)

fn _quickselect_partition(mut similarities: List[Float64], mut indices: List[Int], left: Int, right: Int, k: Int) raises:
    """Partition around k-th largest element using quickselect."""
    if left >= right:
        return
    
    var pivot_idx = _partition_around_pivot(similarities, indices, left, right)
    
    if pivot_idx == k - 1:
        return  # Found k-th element
    elif pivot_idx > k - 1:
        _quickselect_partition(similarities, indices, left, pivot_idx - 1, k)
    else:
        _quickselect_partition(similarities, indices, pivot_idx + 1, right, k)

fn _partition_around_pivot(mut similarities: List[Float64], mut indices: List[Int], left: Int, right: Int) raises -> Int:
    """Partition array around pivot (last element) for quickselect."""
    var pivot = similarities[right]
    var i = left - 1
    
    for j in range(left, right):
        if similarities[j] >= pivot:  # Descending order
            i += 1
            # Swap similarities
            var temp_sim = similarities[i]
            similarities[i] = similarities[j]
            similarities[j] = temp_sim
            
            # Swap indices
            var temp_idx = indices[i]
            indices[i] = indices[j]
            indices[j] = temp_idx
    
    # Place pivot in correct position
    i += 1
    var temp_sim = similarities[i]
    similarities[i] = similarities[right]
    similarities[right] = temp_sim
    
    var temp_idx = indices[i]
    indices[i] = indices[right]
    indices[right] = temp_idx
    
    return i

fn _cosine_similarity(vec1: Float32Vector, vec2: Float32Vector) raises -> Float64:
    """Calculate cosine similarity between two vectors using SIMD optimization."""
    if vec1.dimension() != vec2.dimension():
        return 0.0
    
    var dim = vec1.dimension()
    var dot_product: Float64 = 0.0
    var norm1: Float64 = 0.0
    var norm2: Float64 = 0.0
    
    # SIMD optimization for vector operations
    alias simd_width = simdwidthof[DType.float32]()
    
    # Process vectors in SIMD chunks
    var simd_chunks = dim // simd_width
    var remainder = dim % simd_width
    
    # SIMD vectorized computation
    for i in range(simd_chunks):
        var offset = i * simd_width
        var v1_simd = SIMD[DType.float32, simd_width]()
        var v2_simd = SIMD[DType.float32, simd_width]()
        
        # Load values into SIMD registers
        for j in range(simd_width):
            v1_simd[j] = vec1[offset + j]
            v2_simd[j] = vec2[offset + j]
        
        # Compute dot product and norms using SIMD
        var dot_simd = v1_simd * v2_simd
        var norm1_simd = v1_simd * v1_simd
        var norm2_simd = v2_simd * v2_simd
        
        # Accumulate results using SIMD reduce operations
        dot_product += Float64(dot_simd.reduce_add())
        norm1 += Float64(norm1_simd.reduce_add())
        norm2 += Float64(norm2_simd.reduce_add())
    
    # Process remaining elements
    for i in range(remainder):
        var offset = simd_chunks * simd_width + i
        var v1_val = Float64(vec1[offset])
        var v2_val = Float64(vec2[offset])
        
        dot_product += v1_val * v2_val
        norm1 += v1_val * v1_val
        norm2 += v2_val * v2_val
    
    # Avoid division by zero
    if norm1 == 0.0 or norm2 == 0.0:
        return 0.0
    
    # Calculate cosine similarity
    return dot_product / (sqrt(norm1) * sqrt(norm2))

fn _metadata_passes_filter(vector_id: String, filter_dict: PythonObject) raises -> Bool:
    """Check if vector's metadata passes the filter criteria."""
    try:
        # Check if we have metadata for this vector
        var state = get_state()
        if vector_id not in state[].metadata:
            return False
        
        var metadata = state[].metadata[vector_id]
        
        # Check each filter criterion using direct access
        # Try common metadata keys that might be filtered
        var test_keys = List[String]()
        test_keys.append("category")
        test_keys.append("type") 
        test_keys.append("author")
        test_keys.append("description")
        test_keys.append("tags")
        
        for i in range(len(test_keys)):
            var filter_key = test_keys[i]
            try:
                var filter_value = filter_dict[filter_key]
                if filter_value is not None:
                    var filter_value_str = String(filter_value)
                    
                    # Check if metadata has this key and value matches
                    if not metadata.contains(filter_key):
                        return False
                    
                    var metadata_value = metadata.get(filter_key)
                    if metadata_value != filter_value_str:
                        return False
            except:
                # Key doesn't exist in filter, skip
                pass
        
        return True
        
    except e:
        print("Native: Metadata filter error: " + String(e))
        return False

fn _parse_metadata_json(json_str: String) -> Metadata:
    """Parse simple JSON-like metadata format."""
    var metadata = Metadata()
    
    try:
        var trimmed = json_str.strip()
        if trimmed.startswith("{") and trimmed.endswith("}"):
            # Remove braces
            var content = trimmed[1:-1]
            
            if len(content) > 0:
                # Split by commas to get key-value pairs
                var pairs = content.split(",")
                
                for pair in pairs:
                    var kv_parts = pair.split(":")
                    if len(kv_parts) >= 2:
                        var key = kv_parts[0].strip()
                        var value = kv_parts[1].strip()
                        
                        # Remove quotes if present
                        if key.startswith("\"") and key.endswith("\""):
                            key = key[1:-1]
                        if value.startswith("\"") and value.endswith("\""):
                            value = value[1:-1]
                        
                        metadata.set(String(key), String(value))
    except:
        # If parsing fails, return empty metadata
        pass
    
    return metadata

# File persistence functions
fn _create_database_file(path: String) raises:
    """Create database file with true binary V3 format."""
    try:
        # Check if file already exists
        var file_handle = open(path, "r")
        file_handle.close()
        # File exists, don't overwrite
        return
    except:
        # File doesn't exist, create it
        pass
    
    # Create new file with binary V3 format
    var file_handle = open(path, "w")
    
    # V3 Binary Header (fixed 64 bytes)
    file_handle.write("OMENDB_V3_BINARY\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00")
    
    # Metadata section (32 bytes)
    file_handle.write("\x00\x00\x00\x00")  # dimension (4 bytes)
    file_handle.write("\x00\x00\x00\x00")  # vector_count (4 bytes)
    file_handle.write("\x00\x00\x00\x40")  # header_size = 64 (4 bytes, little endian)
    file_handle.write("\x00\x00\x00\x00")  # reserved (4 bytes)
    file_handle.write("\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00")  # padding (16 bytes)
    
    file_handle.close()

fn _save_vector_to_file(id: String, vector: Float32Vector) raises:
    """Save single vector to database file using Python's file I/O."""
    var state = get_state()
    if state[].database_path == "":
        return
    
    # Use Python's file I/O since Mojo's open() is broken
    var python = Python.import_module("builtins")
    var file_handle = python.open(state[].database_path, "a")
    
    # Text format: id,dimension,v1,v2,v3,...\n
    var line = id + "," + String(vector.dimension())
    
    # Write vector data
    for i in range(vector.dimension()):
        line += "," + String(Float64(vector[i]))
    
    line += "\n"
    file_handle.write(line)
    file_handle.close()

fn _save_vector_with_metadata_to_file(id: String, vector: Float32Vector, metadata: Metadata) raises:
    """Save vector with metadata to database file in binary format."""
    var state = get_state()
    if state[].database_path == "":
        return
    
    # Use binary format for maximum I/O performance
    var file_handle = open(state[].database_path, "a")
    
    # Binary format: [id_len:4][id:id_len][dim:4][vector_data:dim*4][metadata_len:4][metadata:metadata_len]
    var id_len = len(id)
    
    # Write id length and id
    file_handle.write(_int_to_4bytes(id_len))
    file_handle.write(id)
    
    # Write dimension and vector data
    file_handle.write(_int_to_4bytes(vector.dimension()))
    for i in range(vector.dimension()):
        file_handle.write(_float_to_4bytes(vector[i]))
    
    # Serialize metadata as compact JSON
    var metadata_json = String("{")
    if len(metadata) > 0:
        var first = True
        try:
            var keys = metadata.get_all_keys()
            for i in range(len(keys)):
                var key = keys[i]
                var value = metadata.get(key)
                if not first:
                    metadata_json += ","
                metadata_json += "\"" + key + "\":\"" + value + "\""
                first = False
        except:
            pass
    metadata_json += "}"
    
    # Write metadata length and data
    file_handle.write(_int_to_4bytes(len(metadata_json)))
    file_handle.write(metadata_json)
    
    file_handle.close()


# Binary format helper functions
fn _int32_to_bytes(value: Int) -> List[UInt8]:
    """Convert Int32 to little-endian bytes."""
    var bytes = List[UInt8]()
    var val = value
    bytes.append(UInt8(val & 0xFF))
    bytes.append(UInt8((val >> 8) & 0xFF))
    bytes.append(UInt8((val >> 16) & 0xFF))
    bytes.append(UInt8((val >> 24) & 0xFF))
    return bytes

fn _float32_to_bytes(value: Float32) -> List[UInt8]:
    """Convert Float32 to little-endian bytes."""
    var bytes = List[UInt8]()
    # Use UnsafePointer to get the raw bytes of the Float32
    var ptr = UnsafePointer[Float32].alloc(1)
    ptr[] = value
    var byte_ptr = ptr.bitcast[UInt8]()
    bytes.append(byte_ptr[0])
    bytes.append(byte_ptr[1])
    bytes.append(byte_ptr[2])
    bytes.append(byte_ptr[3])
    ptr.free()
    return bytes

fn _bytes_to_int32(bytes: List[UInt8], offset: Int) -> Int:
    """Convert little-endian bytes to Int32."""
    var val = Int(bytes[offset])
    val |= Int(bytes[offset + 1]) << 8
    val |= Int(bytes[offset + 2]) << 16
    val |= Int(bytes[offset + 3]) << 24
    return val

fn _bytes_to_float32(bytes: List[UInt8], offset: Int) -> Float32:
    """Convert little-endian bytes to Float32."""
    var ptr = UnsafePointer[UInt8].alloc(4)
    ptr[0] = bytes[offset]
    ptr[1] = bytes[offset + 1]
    ptr[2] = bytes[offset + 2]
    ptr[3] = bytes[offset + 3]
    var float_ptr = ptr.bitcast[Float32]()
    var result = float_ptr[]
    ptr.free()
    return result

fn _load_vectors_from_file(path: String) raises:
    """Load vectors from database file (supports V2+ and V3 binary formats)."""
    try:
        # Read first 16 bytes to check format
        var file_handle = open(path, "r")
        var header = file_handle.read(16)
        file_handle.close()
        
        # Check format type
        if header.startswith("OMENDB_V3_BINARY"):
            _load_vectors_from_v3_binary_file(path)
            return
        elif header.startswith("OMENDB_V2_OPTIMIZED") or header.startswith("OMENDB_V2_BINARY"):
            _load_vectors_from_v2_text_file(path)
            return
        
        # Fall back to legacy text format
        _load_vectors_from_text_file(path)
        return
        
    except:
        # File doesn't exist or is empty - that's ok
        var state = get_state()
        state[].vectors_cached = True


fn _load_vectors_from_v3_binary_file(path: String) raises:
    """Load vectors from V3 binary format file."""
    try:
        var state = get_state()
        var file_handle = open(path, "r")
        var content = file_handle.read()
        file_handle.close()
        
        # Skip header (64 bytes)
        var offset = 64
        
        while offset < len(content):
            # Read id length
            if offset + 4 > len(content):
                break
            var id_len = _4bytes_to_int(content[offset:offset+4])
            offset += 4
            
            # Read id
            if offset + id_len > len(content):
                break
            var id = content[offset:offset+id_len]
            offset += id_len
            
            # Read dimension
            if offset + 4 > len(content):
                break
            var dim = _4bytes_to_int(content[offset:offset+4])
            offset += 4
            
            # Read vector data
            if offset + dim * 4 > len(content):
                break
            var vector_data = List[Float32]()
            for i in range(dim):
                var float_bytes = content[offset:offset+4]
                vector_data.append(_4bytes_to_float(float_bytes))
                offset += 4
            
            # Create vector
            var vector = from_list(vector_data)
            
            # Check if there's metadata
            var metadata = Metadata()
            if offset + 4 <= len(content):
                var metadata_len = _4bytes_to_int(content[offset:offset+4])
                offset += 4
                
                if offset + metadata_len <= len(content):
                    var metadata_json = content[offset:offset+metadata_len]
                    offset += metadata_len
                    
                    # Parse basic JSON metadata (simple key-value pairs)
                    if metadata_json.startswith("{") and metadata_json.endswith("}"):
                        var json_content = metadata_json[1:-1]  # Remove braces
                        if json_content != "":
                            var pairs = json_content.split(",")
                            for pair in pairs:
                                var kv = pair.split(":")
                                if len(kv) == 2:
                                    var key = String(kv[0].strip().strip("\""))
                                    var value = String(kv[1].strip().strip("\""))
                                    metadata.set(key, value)
            
            # Store vector and metadata
            state[].vectors[id] = vector
            state[].vector_ids.append(id)
            state[].metadata[id] = metadata
        
        state[].vectors_cached = True
        print("Native: Loaded " + String(len(state[].vector_ids)) + " vectors from V3 binary file")
        
    except e:
        print("Native: Failed to load V3 binary file: " + String(e))
        var state = get_state()
        state[].vectors_cached = True

fn _load_vectors_from_v2_text_file(path: String) raises:
    """Load vectors from V2 text format file."""
    try:
        var state = get_state()
        var file_handle = open(path, "r")
        var content = file_handle.read()
        file_handle.close()
        
        var lines = content.split("\n")
        var past_header = False
        
        for line in lines:
            var trimmed = line.strip()
            if trimmed == "---":
                past_header = True
                continue
            
            if not past_header or len(trimmed) == 0:
                continue
            
            # Parse vector line: id:dimension:val1,val2,val3,...
            var parts = trimmed.split(":")
            if len(parts) >= 3:
                var id_str = parts[0]
                var dim_str = parts[1]
                var values_str = parts[2]
                
                # Skip if already loaded
                var id_key = String(id_str)
                var state = get_state()
                if id_key in state[].vectors:
                    continue
                
                # Parse vector values
                var value_strs = values_str.split(",")
                var mojo_data = List[Float32]()
                
                for val_str in value_strs:
                    if len(val_str.strip()) > 0:
                        mojo_data.append(Float32(atof(val_str.strip())))
                
                if len(mojo_data) > 0:
                    var vector = from_list[DType.float32](mojo_data)
                    var id_key = String(id_str)
                    state[].vectors[id_key] = vector
                    state[].vector_ids.append(id_key)
                    
                    # Parse metadata if present (V2 format with metadata)
                    var metadata = Metadata()
                    if len(parts) >= 4:
                        var metadata_str = parts[3]
                        metadata = _parse_metadata_json(String(metadata_str))
                    
                    state[].metadata[id_key] = metadata
                    
                    # Update dimension if needed
                    if state[].dimension == 0:
                        state[].dimension = vector.dimension()
        
        print("Native: Loaded " + String(len(state[].vector_ids)) + " vectors from V2 text file")
        state[].vectors_cached = True
        
    except:
        # File doesn't exist or is empty - that's ok
        var state = get_state()
        state[].vectors_cached = True

fn _load_vectors_from_text_file(path: String) raises:
    """Load vectors from legacy text format file."""
    try:
        var state = get_state()
        var file_handle = open(path, "r")
        var content = file_handle.read()
        file_handle.close()
        
        var lines = content.split("\n")
        var past_header = False
        
        for line in lines:
            var trimmed = line.strip()
            if trimmed == "---":
                past_header = True
                continue
            
            if not past_header or len(trimmed) == 0:
                continue
            
            # Parse vector line: id:dimension:val1,val2,val3,...
            var parts = trimmed.split(":")
            if len(parts) >= 3:
                var id_str = parts[0]
                var dim_str = parts[1]
                var values_str = parts[2]
                
                # Skip if already loaded
                var id_key = String(id_str)
                var state = get_state()
                if id_key in state[].vectors:
                    continue
                
                # Parse vector values
                var value_strs = values_str.split(",")
                var mojo_data = List[Float32]()
                
                for val_str in value_strs:
                    if len(val_str.strip()) > 0:
                        mojo_data.append(Float32(atof(val_str.strip())))
                
                if len(mojo_data) > 0:
                    var vector = from_list[DType.float32](mojo_data)
                    var id_key = String(id_str)
                    state[].vectors[id_key] = vector
                    state[].vector_ids.append(id_key)
                    
                    # Update dimension if needed
                    if state[].dimension == 0:
                        state[].dimension = vector.dimension()
        
        print("Native: Loaded " + String(len(state[].vector_ids)) + " vectors from text file")
        state[].vectors_cached = True
        
    except:
        # File doesn't exist or is empty - that's ok
        var state = get_state()
        state[].vectors_cached = True

fn _flush_all_vectors_to_file() raises:
    """Rewrite entire database file with current vectors in V3 binary format."""
    var state = get_state()
    if state[].database_path == "":
        return
    
    # Write complete V3 binary file
    var file_handle = open(state[].database_path, "w")
    
    # V3 Binary Header (64 bytes)
    file_handle.write("OMENDB_V3_BINARY\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00")
    
    # Write metadata section
    file_handle.write(_int_to_4bytes(state[].dimension))  # dimension
    file_handle.write(_int_to_4bytes(len(state[].vector_ids)))  # vector_count
    file_handle.write(_int_to_4bytes(64))  # header_size
    file_handle.write(_int_to_4bytes(0))   # reserved
    file_handle.write("\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00")  # padding
    
    # Write all vectors in binary format with metadata
    for i in range(len(state[].vector_ids)):
        var id_str = state[].vector_ids[i]
        var vector = state[].vectors[id_str]
        
        # Write id length and id
        file_handle.write(_int_to_4bytes(len(id_str)))
        file_handle.write(id_str)
        
        # Write dimension and vector data
        file_handle.write(_int_to_4bytes(vector.dimension()))
        for j in range(vector.dimension()):
            file_handle.write(_float_to_4bytes(vector[j]))
        
        # Write metadata
        var metadata_json = String("{")
        if id_str in state[].metadata:
            var metadata = state[].metadata[id_str]
            if len(metadata) > 0:
                var first = True
                try:
                    var keys = metadata.get_all_keys()
                    for k in range(len(keys)):
                        var key = keys[k]
                        var value = metadata.get(key)
                        if not first:
                            metadata_json += ","
                        metadata_json += "\"" + key + "\":\"" + value + "\""
                        first = False
                except:
                    pass
        metadata_json += "}"
        
        # Write metadata length and data
        file_handle.write(_int_to_4bytes(len(metadata_json)))
        file_handle.write(metadata_json)
    
    file_handle.close()

# ===-----------------------------------------------------------------------===#
# HNSW Index Functions
# ===-----------------------------------------------------------------------===#

# HNSW functions removed - using RoarGraph only

# ===-----------------------------------------------------------------------===#
# RoarGraph Index Functions
# ===-----------------------------------------------------------------------===#

fn create_roargraph_index(dimension: PythonObject) raises -> PythonObject:
    """Create a new RoarGraph index with specified dimension."""
    try:
        var dim_val = Int(dimension)
        
        # Create new RoarGraph index
        # var index = RoarGraphIndex(dim_val)
        # TODO: Add roargraph_index to DatabaseState struct
        # _roargraph_index = Optional[RoarGraphIndex](index)
        var state = get_state()
        state[].index_type = INDEX_ROARGRAPH
        
        print("Native: Created RoarGraph index with dimension=" + String(dim_val))
        return PythonObject(True)
        
    except e:
        print("Native: Failed to create RoarGraph index: " + String(e))
        return PythonObject(False)

fn roargraph_insert(id: PythonObject, vector: PythonObject) raises -> PythonObject:
    """Insert a vector into the RoarGraph index."""
    try:
        var state = get_state()
        if state[].index_type != INDEX_ROARGRAPH:
            return PythonObject(False)
        # TODO: Implement roargraph_index in DatabaseState
        # if not _roargraph_index:
        #     return PythonObject(False)
        
        var id_str = String(id)
        var vector_list = List[Float32]()
        
        # Convert Python list to Mojo list
        for i in range(len(vector)):
            vector_list.append(Float32(Float64(vector[i])))
        
        # Create vector
        var mojo_vector = from_list[DType.float32](vector_list)
        
        # Insert into RoarGraph index
        # TODO: Implement roargraph_index storage in DatabaseState
        # var index = _roargraph_index.value()
        # index.insert(mojo_vector, id_str)
        # _roargraph_index = Optional[RoarGraphIndex](index)
        
        return PythonObject(True)
        
    except e:
        print("Native: Failed to insert vector into RoarGraph index: " + String(e))
        return PythonObject(False)

fn roargraph_search(query: PythonObject, k: PythonObject) raises -> PythonObject:
    """Search for k nearest neighbors in the RoarGraph index."""
    try:
        var state = get_state()
        if state[].index_type != INDEX_ROARGRAPH:
            var python = Python.import_module("builtins")
            return python.list()
        # TODO: Implement roargraph_index in DatabaseState
        # if not _roargraph_index:
        #     return PythonObject([])
        
        var k_val = Int(k)
        var query_list = List[Float32]()
        
        # Convert Python list to Mojo list
        for i in range(len(query)):
            query_list.append(Float32(Float64(query[i])))
        
        # Create query vector
        var query_vector = from_list[DType.float32](query_list)
        
        # Search RoarGraph index
        # TODO: Implement roargraph_index storage in DatabaseState
        # var index = _roargraph_index.value()
        # var results = index.search(query_vector, k_val)
        
        # Convert results to Python format (placeholder for now)
        var python = Python.import_module("builtins")
        var python_results = python.list()
        # TODO: Implement actual search results
        # for i in range(len(results)):
        #     var result = results[i]
        #     var result_dict = PythonObject({})
        #     result_dict["id"] = PythonObject(result.id)
        #     result_dict["distance"] = PythonObject(result.distance)
        #     result_dict["similarity"] = PythonObject(result.similarity)
        #     python_results.append(result_dict)
        
        return python_results
        
    except e:
        print("Native: Failed to search RoarGraph index: " + String(e))
        var python = Python.import_module("builtins")
        return python.list()

fn roargraph_size() raises -> PythonObject:
    """Get the number of vectors in the RoarGraph index."""
    try:
        var state = get_state()
        if state[].index_type != INDEX_ROARGRAPH:
            return PythonObject(0)
        # TODO: Implement roargraph_index in DatabaseState
        # if not _roargraph_index:
        #     return PythonObject(0)
        
        # TODO: Implement roargraph_index storage in DatabaseState
        # var index = _roargraph_index.value()
        # return PythonObject(len(index.vectors))
        return PythonObject(0)
        
    except e:
        print("Native: Failed to get RoarGraph index size: " + String(e))
        return PythonObject(0)

fn roargraph_save(path: PythonObject) raises -> PythonObject:
    """Save the RoarGraph index to a file with efficient binary serialization."""
    try:
        var state = get_state()
        if state[].index_type != INDEX_ROARGRAPH:
            print("Native: No RoarGraph index to save")
            return PythonObject(False)
        # TODO: Implement roargraph_index in DatabaseState
        # if not _roargraph_index:
        #     print("Native: No RoarGraph index to save")
        #     return PythonObject(False)
        
        var path_str = String(path)
        print("Native: RoarGraph index save not yet implemented - path: " + path_str)
        
        # TODO: Implement RoarGraph index persistence with proper state management
        # Once roargraph_index is properly integrated into DatabaseState,
        # implement the full binary serialization format here
        
        # Placeholder implementation for now
        return PythonObject(False)
        
    except e:
        print("Native: Failed to save RoarGraph index: " + String(e))
        return PythonObject(False)

