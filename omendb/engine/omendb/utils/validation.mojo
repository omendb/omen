"""Validation and conversion utilities for OmenDB.

This module provides functions for validating inputs, converting between types,
and ensuring data integrity.
"""

from python import PythonObject, Python
from collections import List
from .types import DatabaseError, MAX_VECTOR_DIM

# =============================================================================
# VECTOR VALIDATION
# =============================================================================

fn validate_vector_dimension(vector_dim: Int, expected_dim: Int) raises:
    """Validate that a vector has the expected dimension.
    
    Args:
        vector_dim: The actual dimension of the vector
        expected_dim: The expected dimension
        
    Raises:
        Error if dimensions don't match
    """
    if vector_dim != expected_dim:
        raise Error(
            DatabaseError.DIMENSION_MISMATCH + 
            ": expected " + str(expected_dim) + 
            ", got " + str(vector_dim)
        )

fn validate_vector_size(size: Int) raises:
    """Validate that a vector size is within acceptable bounds.
    
    Args:
        size: The size to validate
        
    Raises:
        Error if size is invalid
    """
    if size <= 0:
        raise Error(DatabaseError.INVALID_PARAMETER + ": vector size must be positive")
    if size > MAX_VECTOR_DIM:
        raise Error(
            DatabaseError.INVALID_PARAMETER + 
            ": vector dimension " + str(size) + 
            " exceeds maximum " + str(MAX_VECTOR_DIM)
        )

fn validate_k_parameter(k: Int, max_vectors: Int) raises -> Int:
    """Validate and adjust k parameter for search.
    
    Args:
        k: Number of results requested
        max_vectors: Maximum available vectors
        
    Returns:
        Adjusted k value
        
    Raises:
        Error if k is invalid
    """
    if k <= 0:
        raise Error(DatabaseError.INVALID_PARAMETER + ": k must be positive")
    
    # Adjust k if it exceeds available vectors
    if k > max_vectors:
        return max_vectors
    return k

# =============================================================================
# PYTHON CONVERSIONS
# =============================================================================

fn python_list_to_float32(py_list: PythonObject) raises -> List[Float32]:
    """Convert a Python list or numpy array to List[Float32].
    
    Args:
        py_list: Python list or numpy array
        
    Returns:
        Mojo List[Float32]
        
    Raises:
        Error if conversion fails
    """
    try:
        var result = List[Float32]()
        var py = Python.import_module("builtins")
        var len_func = py.len
        var length = int(len_func(py_list))
        
        result.reserve(length)
        for i in range(length):
            result.append(float(py_list[i]).cast[DType.float32]())
        
        return result
    except:
        raise Error(DatabaseError.INVALID_PARAMETER + ": failed to convert vector data")

fn float32_list_to_python(mojo_list: List[Float32]) raises -> PythonObject:
    """Convert a Mojo List[Float32] to Python list.
    
    Args:
        mojo_list: Mojo list of floats
        
    Returns:
        Python list object
    """
    try:
        var py = Python.import_module("builtins")
        var py_list = py.list()
        
        for i in range(len(mojo_list)):
            _ = py_list.append(float(mojo_list[i]))
        
        return py_list
    except:
        raise Error(DatabaseError.INVALID_PARAMETER + ": failed to convert to Python list")

fn extract_numpy_data(np_array: PythonObject) raises -> List[Float32]:
    """Extract data from a numpy array as Float32 list.
    
    Args:
        np_array: Numpy array object
        
    Returns:
        List[Float32] with the array data
        
    Raises:
        Error if extraction fails
    """
    try:
        var np = Python.import_module("numpy")
        
        # Ensure float32 dtype
        var float32_array = np_array.astype(np.float32)
        
        # Flatten if multi-dimensional
        var flat_array = float32_array.flatten()
        
        # Convert to list
        var py_list = flat_array.tolist()
        
        return python_list_to_float32(py_list)
    except:
        raise Error(DatabaseError.INVALID_PARAMETER + ": failed to extract numpy data")

# =============================================================================
# STRING VALIDATION
# =============================================================================

fn validate_vector_id(vector_id: String) raises:
    """Validate a vector ID.
    
    Args:
        vector_id: The ID to validate
        
    Raises:
        Error if ID is invalid
    """
    if len(vector_id) == 0:
        raise Error(DatabaseError.INVALID_PARAMETER + ": vector ID cannot be empty")
    
    if len(vector_id) > 256:
        raise Error(DatabaseError.INVALID_PARAMETER + ": vector ID too long (max 256 chars)")

fn validate_collection_name(name: String) raises:
    """Validate a collection name.
    
    Args:
        name: The collection name to validate
        
    Raises:
        Error if name is invalid
    """
    if len(name) == 0:
        raise Error(DatabaseError.INVALID_PARAMETER + ": collection name cannot be empty")
    
    if len(name) > 128:
        raise Error(DatabaseError.INVALID_PARAMETER + ": collection name too long (max 128 chars)")
    
    # Check for invalid characters
    for char in name:
        if not (char.isalnum() or char == '_' or char == '-'):
            raise Error(
                DatabaseError.INVALID_PARAMETER + 
                ": collection name contains invalid character: " + char
            )

fn validate_file_path(path: String) raises:
    """Validate a file path.
    
    Args:
        path: The file path to validate
        
    Raises:
        Error if path is invalid
    """
    if len(path) == 0:
        raise Error(DatabaseError.INVALID_PARAMETER + ": file path cannot be empty")
    
    if len(path) > 4096:
        raise Error(DatabaseError.INVALID_PARAMETER + ": file path too long")

# =============================================================================
# METADATA VALIDATION
# =============================================================================

fn validate_metadata(metadata: PythonObject) raises -> Dict[String, String]:
    """Validate and convert metadata from Python dict.
    
    Args:
        metadata: Python dictionary object
        
    Returns:
        Mojo Dict[String, String]
        
    Raises:
        Error if metadata is invalid
    """
    var result = Dict[String, String]()
    
    if metadata is None:
        return result
    
    try:
        var py = Python.import_module("builtins")
        
        # Check if it's a dict
        if not py.isinstance(metadata, py.dict):
            return result
        
        # Convert each key-value pair
        for key in metadata.keys():
            var key_str = str(key)
            var value_str = str(metadata[key])
            
            # Validate key length
            if len(key_str) > 128:
                raise Error(DatabaseError.INVALID_PARAMETER + ": metadata key too long")
            
            # Validate value length  
            if len(value_str) > 1024:
                raise Error(DatabaseError.INVALID_PARAMETER + ": metadata value too long")
            
            result[key_str] = value_str
        
        return result
    except e:
        # Return empty dict if conversion fails
        return result

# =============================================================================
# NUMERIC VALIDATION
# =============================================================================

fn validate_positive_int(value: Int, param_name: String) raises:
    """Validate that an integer is positive.
    
    Args:
        value: The value to validate
        param_name: Name of the parameter for error messages
        
    Raises:
        Error if value is not positive
    """
    if value <= 0:
        raise Error(
            DatabaseError.INVALID_PARAMETER + 
            ": " + param_name + " must be positive, got " + str(value)
        )

fn validate_range(value: Int, min_val: Int, max_val: Int, param_name: String) raises:
    """Validate that a value is within a range.
    
    Args:
        value: The value to validate
        min_val: Minimum allowed value (inclusive)
        max_val: Maximum allowed value (inclusive)
        param_name: Name of the parameter for error messages
        
    Raises:
        Error if value is out of range
    """
    if value < min_val or value > max_val:
        raise Error(
            DatabaseError.INVALID_PARAMETER + 
            ": " + param_name + " must be between " + 
            str(min_val) + " and " + str(max_val) + 
            ", got " + str(value)
        )

fn clamp_value(value: Int, min_val: Int, max_val: Int) -> Int:
    """Clamp a value to a range.
    
    Args:
        value: The value to clamp
        min_val: Minimum value
        max_val: Maximum value
        
    Returns:
        Clamped value
    """
    if value < min_val:
        return min_val
    if value > max_val:
        return max_val
    return value