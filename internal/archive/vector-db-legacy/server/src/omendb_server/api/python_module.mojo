"""
OmenDB Python module using correct Mojo-Python interop approach.

Based on official Modular examples in external/modular/examples/mojo/python-interop/
"""

from python import PythonObject
from python.bindings import PythonModuleBuilder
from os import abort
from collections import List

from storage.embedded_db import EmbeddedDatabase
from core.vector import Float32Vector
from core.metadata import Metadata

# Export the Python module initialization function
@export
fn PyInit_omendb() -> PythonObject:
    """
    Python module initialization function.
    
    This function is called when Python imports the omendb module.
    """
    try:
        # Create Python module matching this Mojo module name
        var module = PythonModuleBuilder("omendb")
        
        # Register functions to be available from Python
        module.def_function[create_database]("create_database")
        module.def_function[create_vector]("create_vector")
        module.def_function[create_metadata]("create_metadata")
        module.def_function[test_vector_operations]("test_vector_operations") 
        module.def_function[validate_installation]("validate_installation")
        module.def_function[get_version]("get_version")
        
        return module.finalize()
        
    except e:
        return abort[PythonObject](
            String("Failed to create OmenDB Python module: ", e)
        )

fn create_database(path: PythonObject) raises -> PythonObject:
    """
    Create or open an embedded database.
    
    Args:
        path: Database file path as PythonObject.
        
    Returns:
        PythonObject indicating success/failure.
    """
    try:
        var path_str = String(path)
        var db = EmbeddedDatabase(path_str, False, 2)
        return PythonObject(True)
    except:
        return PythonObject(False)

fn test_vector_operations(dimension: PythonObject) raises -> PythonObject:
    """
    Test vector creation and operations.
    
    Args:
        dimension: Vector dimension as PythonObject.
        
    Returns:
        PythonObject indicating success/failure.
    """
    try:
        var dim = Int(dimension)
        var data = List[Float32]()
        
        # Create test vector data
        for i in range(dim):
            data.append(Float32(i) * 0.1)
        
        var vector = Float32Vector(data)
        
        # Verify vector creation worked
        if vector.dimension() == dim:
            return PythonObject(True)
        else:
            return PythonObject(False)
            
    except:
        return PythonObject(False)

fn validate_installation() raises -> PythonObject:
    """
    Validate OmenDB installation and core functionality.
    
    Returns:
        PythonObject indicating installation validity.
    """
    try:
        # Test database creation
        var db = EmbeddedDatabase("test_validation.omen", False, 2)
        
        # Test vector operations
        var data = List[Float32]()
        for i in range(128):
            data.append(Float32(i) * 0.01)
        var vector = Float32Vector(data)
        
        # Test metadata
        var metadata = Metadata()
        metadata.set("test_key", "test_value")
        
        if vector.dimension() == 128 and metadata.contains("test_key"):
            return PythonObject(True)
        else:
            return PythonObject(False)
            
    except:
        return PythonObject(False)

fn get_version() raises -> PythonObject:
    """
    Get OmenDB version information.
    
    Returns:
        Version string as PythonObject.
    """
    return PythonObject("0.1.0-dev")

fn create_vector(data: PythonObject) raises -> PythonObject:
    """
    Create a vector from Python list.
    
    Args:
        data: List of floats as PythonObject.
        
    Returns:
        PythonObject indicating success/failure.
    """
    try:
        # For now, just validate that we can create a vector
        # In the full implementation, this would return a handle
        var test_data = List[Float32]()
        test_data.append(1.0)
        test_data.append(2.0)
        test_data.append(3.0)
        var vector = Float32Vector(test_data)
        return PythonObject(True)
    except:
        return PythonObject(False)

fn create_metadata(data: PythonObject) raises -> PythonObject:
    """
    Create metadata from Python dict.
    
    Args:
        data: Dictionary as PythonObject.
        
    Returns:
        PythonObject indicating success/failure.
    """
    try:
        # For now, just validate that we can create metadata
        var metadata = Metadata()
        metadata.set("test", "value")
        return PythonObject(True)
    except:
        return PythonObject(False)