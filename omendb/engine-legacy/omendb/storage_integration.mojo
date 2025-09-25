"""Integration layer connecting GlobalDatabase with memory-mapped storage."""

from omendb.native import GlobalDatabase
from omendb.storage.memory_mapped import MemoryMappedStorage, create_optimal_storage
from collections import List, Dict
from memory import UnsafePointer
from python import PythonObject


struct PersistentGlobalDatabase:
    """GlobalDatabase with persistent storage backend."""
    var db: GlobalDatabase
    var storage: MemoryMappedStorage
    var auto_checkpoint: Bool
    var checkpoint_interval: Int
    var operations_since_checkpoint: Int
    
    fn __init__(out self, storage_path: String = "omendb_data", 
                dimension: Int = 768, auto_checkpoint: Bool = True) raises:
        """Initialize with persistent storage."""
        self.db = GlobalDatabase()
        self.storage = create_optimal_storage(storage_path, dimension, True)
        self.auto_checkpoint = auto_checkpoint
        self.checkpoint_interval = 100  # Checkpoint every 100 operations
        self.operations_since_checkpoint = 0
        
        # Initialize database
        _ = self.db.initialize(dimension)
        
        # Try to recover from storage
        try:
            var recovered = self.storage.recover()
            if recovered > 0:
                print("Recovered", recovered, "vectors from storage")
                self._restore_vectors_to_memory()
        except:
            print("No existing data to recover")
    
    fn _restore_vectors_to_memory(mut self) raises:
        """Restore vectors from storage to in-memory index."""
        # Get all IDs from storage
        var ids = self.storage.get_all_ids()
        
        for i in range(len(ids)):
            var id = ids[i]
            
            # Load vector from storage
            var vector_opt = self.storage.load_vector(id)
            if not vector_opt:
                continue
                
            var vector_list = vector_opt.value()
            
            # Convert to UnsafePointer for GlobalDatabase
            var vector_ptr = UnsafePointer[Float32].alloc(len(vector_list))
            for j in range(len(vector_list)):
                vector_ptr[j] = vector_list[j]
            
            # Get metadata (empty for now)
            var metadata = Dict[String, PythonObject]()
            
            # Add to in-memory index
            _ = self.db.add_vector_with_metadata(id, vector_ptr, metadata)
            
            vector_ptr.free()
    
    fn add_vector(mut self, id: String, vector: UnsafePointer[Float32], 
                  dimension: Int, metadata: Dict[String, PythonObject] = Dict[String, PythonObject]()) raises -> Bool:
        """Add vector to both in-memory index and persistent storage."""
        # Add to in-memory database
        if not self.db.add_vector_with_metadata(id, vector, metadata):
            return False
        
        # Convert to List for storage
        var vector_list = List[Float32]()
        for i in range(dimension):
            vector_list.append(vector[i])
        
        # Convert metadata to storage format
        var storage_metadata = Dict[String, String]()
        # Note: For now, we don't persist complex metadata
        
        # Save to persistent storage
        _ = self.storage.save_vector(id, vector_list, storage_metadata)
        
        self.operations_since_checkpoint += 1
        
        # Auto-checkpoint if needed
        if self.auto_checkpoint and self.operations_since_checkpoint >= self.checkpoint_interval:
            self.checkpoint()
        
        return True
    
    fn search(mut self, query: UnsafePointer[Float32], k: Int) -> List[Tuple[String, Float32]]:
        """Search for k nearest neighbors."""
        return self.db.search_vectors(query, k)
    
    fn checkpoint(mut self) raises:
        """Checkpoint current state to disk."""
        _ = self.storage.checkpoint_async()
        self.storage.sync()
        self.operations_since_checkpoint = 0
        print("Checkpoint completed")
    
    fn get_vector(self, id: String) -> UnsafePointer[Float32]:
        """Get vector by ID from in-memory index."""
        return self.db.get_vector_data(id)
    
    fn count_vectors(self) -> Int:
        """Get total number of vectors."""
        return self.db.count_vectors()
    
    fn clear(mut self):
        """Clear all data."""
        self.db.clear()
        # Note: Storage clear not implemented yet


fn test_persistent_database() raises:
    """Test the persistent database integration."""
    print("\n=== Testing Persistent Database ===")
    
    # Create persistent database
    var db = PersistentGlobalDatabase("test_persistent", 128, True)
    
    # Add test vectors
    var num_vectors = 10
    for i in range(num_vectors):
        var id = "persist_vec_" + String(i)
        var vector = UnsafePointer[Float32].alloc(128)
        for j in range(128):
            vector[j] = Float32(i * j) * 0.001
        
        var success = db.add_vector(id, vector, 128)
        if success:
            print("Added vector:", id)
        
        vector.free()
    
    # Force checkpoint
    db.checkpoint()
    
    # Search test
    var query = UnsafePointer[Float32].alloc(128)
    for i in range(128):
        query[i] = Float32(i) * 0.01
    
    var results = db.search(query, 5)
    print("Search returned", len(results), "results")
    
    for i in range(len(results)):
        var result = results[i]
        print("  -", result[0], "distance:", result[1])
    
    query.free()
    
    print("Total vectors:", db.count_vectors())
    
    # Test recovery by creating new instance
    print("\n=== Testing Recovery ===")
    var db2 = PersistentGlobalDatabase("test_persistent", 128, True)
    print("Vectors after recovery:", db2.count_vectors())
    
    print("\n✓ Persistent database test complete!")


fn main() raises:
    """Run integration tests."""
    test_persistent_database()