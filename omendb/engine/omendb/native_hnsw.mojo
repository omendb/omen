"""
OmenDB native module with HNSW+ algorithm - clean implementation.
"""

from python import PythonObject, Python
from collections import List
from memory import UnsafePointer
from omendb.algorithms.hnsw import HNSWIndex
from omendb.core.sparse_map import SparseMap

# =============================================================================
# ID MAPPING
# =============================================================================

struct IDMapper:
    """Maps string IDs to numeric IDs for HNSW."""
    var string_to_int: SparseMap  # String ID -> Int ID
    var int_to_string: List[String]  # Int ID -> String ID
    var next_id: Int
    
    fn __init__(out self):
        self.string_to_int = SparseMap()
        self.int_to_string = List[String]()
        self.next_id = 0
    
    fn add(mut self, string_id: String, numeric_id: Int):
        """Add a mapping."""
        _ = self.string_to_int.insert(string_id, numeric_id)
        
        # Grow list if needed
        while len(self.int_to_string) <= numeric_id:
            self.int_to_string.append(String(""))
        self.int_to_string[numeric_id] = string_id
        
        if numeric_id >= self.next_id:
            self.next_id = numeric_id + 1
    
    fn get_numeric(self, string_id: String) -> Int:
        """Get numeric ID for string ID."""
        var result = self.string_to_int.get(string_id)
        if result:
            return result.value()
        return -1
    
    fn get_string(self, numeric_id: Int) -> String:
        """Get string ID for numeric ID."""
        if numeric_id < len(self.int_to_string):
            return self.int_to_string[numeric_id]
        return String("")
    
    fn clear(mut self):
        """Clear all mappings."""
        # Recreate SparseMap since it doesn't have clear
        self.string_to_int = SparseMap()
        self.int_to_string = List[String]()
        self.next_id = 0

# =============================================================================
# VECTOR DATABASE
# =============================================================================

struct VectorDB:
    """HNSW-based vector database with string ID support."""
    var index: HNSWIndex
    var id_mapper: IDMapper
    var dimension: Int
    var initialized: Bool
    
    fn __init__(out self):
        self.dimension = 0
        self.initialized = False
        self.index = HNSWIndex(1, 100)  # Placeholder
        self.id_mapper = IDMapper()
    
    fn initialize(mut self, dimension: Int) -> Bool:
        """Initialize with dimension."""
        if self.initialized and self.dimension != dimension:
            return False
        
        if not self.initialized:
            self.dimension = dimension
            self.index = HNSWIndex(dimension, 10000)
            self.initialized = True
        
        return True
    
    fn add(mut self, string_id: String, vector: UnsafePointer[Float32]) -> Bool:
        """Add vector with string ID."""
        if not self.initialized:
            return False
        
        # Check if ID exists
        var existing = self.id_mapper.get_numeric(string_id)
        if existing >= 0:
            return False  # Already exists
        
        # Insert into HNSW
        var numeric_id = self.index.insert(vector)
        
        # Map IDs
        self.id_mapper.add(string_id, numeric_id)
        
        return True
    
    fn search(self, query: UnsafePointer[Float32], k: Int) -> List[Tuple[String, Float32]]:
        """Search for k nearest neighbors."""
        var results = List[Tuple[String, Float32]]()
        
        if not self.initialized:
            return results
        
        # Search HNSW
        var raw_results = self.index.search(query, k)
        
        # Convert to string IDs
        for i in range(len(raw_results)):
            var pair = raw_results[i]
            var numeric_id = Int(pair[0])
            var distance = pair[1]
            
            var string_id = self.id_mapper.get_string(numeric_id)
            if len(string_id) > 0:
                results.append((string_id, distance))
        
        return results
    
    fn clear(mut self):
        """Clear all data."""
        if self.initialized:
            self.index = HNSWIndex(self.dimension, 10000)
            self.id_mapper.clear()
    
    fn size(self) -> Int:
        """Get number of vectors."""
        if not self.initialized:
            return 0
        return self.index.size

# =============================================================================
# SINGLETON MANAGEMENT
# =============================================================================

struct GlobalDB:
    """Singleton database manager."""
    var db: VectorDB
    var initialized: Bool
    
    fn __init__(out self):
        self.db = VectorDB()
        self.initialized = False
    
    fn get(mut self) -> UnsafePointer[VectorDB]:
        """Get database pointer."""
        return UnsafePointer[VectorDB].address_of(self.db)

# Global instance
var __global: GlobalDB = GlobalDB()

fn get_db() -> UnsafePointer[VectorDB]:
    """Get global database instance."""
    return __global.get()

# =============================================================================
# PYTHON EXPORTS
# =============================================================================

@export
fn init_db(dimension: Int) -> Bool:
    """Initialize database with dimension."""
    return get_db()[].initialize(dimension)

@export
fn add_vector(id_ptr: UnsafePointer[UInt8], id_len: Int, vector_ptr: UnsafePointer[Float32], dimension: Int) -> Bool:
    """Add a vector with string ID."""
    var db = get_db()
    
    # Auto-initialize if needed
    if not db[].initialized:
        _ = db[].initialize(dimension)
    
    # Convert C string to Mojo string (simplified)
    var id = String("vec_") + String(id_len)
    
    return db[].add(id, vector_ptr)

@export
fn search_vectors(
    query_ptr: UnsafePointer[Float32],
    k: Int,
    result_count: UnsafePointer[Int]
) -> UnsafePointer[Float32]:
    """Search and return distances. IDs handled separately."""
    var db = get_db()
    
    if not db[].initialized:
        result_count[0] = 0
        return UnsafePointer[Float32]()
    
    var results = db[].search(query_ptr, k)
    result_count[0] = len(results)
    
    # Allocate result array
    var distances = UnsafePointer[Float32].alloc(len(results))
    for i in range(len(results)):
        var (id, dist) = results[i]
        distances[i] = dist
    
    return distances

@export
fn clear_db():
    """Clear the database."""
    get_db()[].clear()

@export
fn db_size() -> Int:
    """Get database size."""
    return get_db()[].size()