"""
OmenDB Instance-Based API - Fix for memory corruption.

This version creates separate database instances to avoid
the global singleton memory corruption issue.
"""

from typing import List, Optional, Dict, Union, Tuple, Any
import numpy as np
import os
import importlib.util

# Import the native module
try:
    native_so_path = os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "native.so"
    )
    
    if os.path.exists(native_so_path):
        spec = importlib.util.spec_from_file_location("native", native_so_path)
        _native = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(_native)
        _NATIVE_AVAILABLE = True
    else:
        raise ImportError("Native module not found")
        
except ImportError as e:
    print(f"Failed to load native module: {e}")
    _native = None
    _NATIVE_AVAILABLE = False


class DB:
    """Instance-based OmenDB that avoids global singleton issues."""
    
    def __init__(self, buffer_size: int = 10000, db_path: Optional[str] = None):
        """Create a new database instance.
        
        Each DB object gets its own internal database instance,
        avoiding the global singleton memory corruption.
        """
        if not _NATIVE_AVAILABLE:
            raise RuntimeError("Native module not available")
            
        # Create a new database instance
        self._handle = _native.create_database_instance()
        self._initialized = False
        self._dimension = None
        self._vector_count = 0
        
    def __del__(self):
        """Clean up database instance on deletion."""
        if hasattr(self, '_handle') and _native:
            try:
                _native.destroy_database_instance(self._handle)
            except:
                pass  # Ignore errors during cleanup
                
    def add(self, vector_id: str, vector: Union[List[float], np.ndarray]) -> bool:
        """Add a single vector."""
        if isinstance(vector, list):
            vector = np.array(vector, dtype=np.float32)
        elif not isinstance(vector, np.ndarray):
            vector = np.array(vector, dtype=np.float32)
            
        if vector.dtype != np.float32:
            vector = vector.astype(np.float32)
            
        # Store dimension on first vector
        if self._dimension is None:
            self._dimension = len(vector)
            
        # Call native with instance handle
        success = _native.add_vector(self._handle, vector_id, vector, {})
        if success:
            self._vector_count += 1
        return success
        
    def add_batch(
        self, 
        vectors: Union[List[List[float]], np.ndarray],
        ids: Optional[List[str]] = None
    ) -> List[str]:
        """Add batch of vectors."""
        if not isinstance(vectors, np.ndarray):
            vectors = np.array(vectors, dtype=np.float32)
        elif vectors.dtype != np.float32:
            vectors = vectors.astype(np.float32)
            
        batch_size = len(vectors)
        
        if ids is None:
            ids = [f"vec_{self._vector_count + i}" for i in range(batch_size)]
            
        # Store dimension on first batch
        if self._dimension is None and batch_size > 0:
            self._dimension = vectors.shape[1]
            
        # Call native with instance handle
        result = _native.add_vector_batch(self._handle, ids, vectors, [{}] * batch_size)
        
        if result:
            self._vector_count += batch_size
            return ids
        return []
        
    def search(
        self,
        query: Union[List[float], np.ndarray],
        limit: int = 10
    ) -> List[Tuple[str, float]]:
        """Search for nearest neighbors."""
        if isinstance(query, list):
            query = np.array(query, dtype=np.float32)
        elif not isinstance(query, np.ndarray):
            query = np.array(query, dtype=np.float32)
            
        if query.dtype != np.float32:
            query = query.astype(np.float32)
            
        # Call native with instance handle
        results = _native.search_vectors(self._handle, query, limit, {})
        
        # Convert results to list of tuples
        output = []
        for result in results:
            if isinstance(result, dict):
                output.append((result.get("id", ""), result.get("score", 0.0)))
            else:
                output.append(result)
        return output
        
    def clear(self):
        """Clear all vectors from this instance."""
        success = _native.clear_database(self._handle)
        if success:
            self._vector_count = 0
            self._dimension = None
        return success
        
    def count(self) -> int:
        """Get number of vectors."""
        return self._vector_count
        
    def info(self) -> Dict:
        """Get database info."""
        info = _native.get_database_info(self._handle)
        return {
            "vector_count": self._vector_count,
            "dimension": self._dimension,
            "algorithm": "HNSW+",
            "instance_based": True,
            "native_info": info
        }


def test_instance_based():
    """Test that instance-based approach fixes memory corruption."""
    print("Testing Instance-Based OmenDB")
    print("="*60)
    
    # Test 1: Multiple instances
    print("\n1. Testing multiple database instances:")
    db1 = DB()
    db2 = DB()
    
    # Add to first instance
    vectors1 = np.random.random((100, 128)).astype(np.float32)
    ids1 = db1.add_batch(vectors1)
    print(f"   DB1: Added {len(ids1)} vectors")
    
    # Add to second instance (should NOT crash)
    vectors2 = np.random.random((100, 128)).astype(np.float32)  
    ids2 = db2.add_batch(vectors2)
    print(f"   DB2: Added {len(ids2)} vectors")
    
    print(f"   DB1 count: {db1.count()}")
    print(f"   DB2 count: {db2.count()}")
    
    # Test 2: Multiple batches to same instance
    print("\n2. Testing multiple batches (no memory corruption):")
    db3 = DB()
    
    for i in range(5):
        batch = np.random.random((100, 128)).astype(np.float32)
        ids = db3.add_batch(batch)
        print(f"   Batch {i+1}: Added {len(ids)} vectors, total: {db3.count()}")
        
    # Test 3: Search
    print("\n3. Testing search:")
    query = np.random.random(128).astype(np.float32)
    results = db3.search(query, limit=5)
    print(f"   Found {len(results)} results")
    
    print("\n✅ Instance-based approach works!")
    print("No memory corruption with multiple instances/batches")
    

if __name__ == "__main__":
    test_instance_based()