"""
Fix for Memory-Mapped Storage Integration

Problem: Vectors never make it to memory-mapped storage's hot_vectors
Solution: Integrate memory-mapped storage with buffer and flush
"""

# Option 1: Add to hot_vectors during add_vector
fn add_vector(mut self, id: String, vector: List[Float32], metadata: Metadata) raises -> Bool:
    """Fixed version that adds to memory-mapped storage."""
    try:
        # ... existing buffer add logic ...
        var success = self.buffer.add(id, vector)
        
        if success:
            # ... existing metadata and quantization logic ...
            
            # NEW: Also add to memory-mapped storage if enabled
            if self.storage_type.value == StorageType.MEMORY_MAPPED and self.memory_mapped_storage:
                var storage = self.memory_mapped_storage.unsafe_value()
                var metadata_dict = Dict[String, String]()
                # Add to hot vectors for persistence
                _ = storage.save_vector(id, vector, metadata_dict)
            
            self.total_vectors += 1
            return True
            
# Option 2: Transfer during checkpoint (better for batch efficiency)
fn checkpoint(mut self) raises -> Bool:
    """Fixed checkpoint that transfers buffer to memory-mapped storage."""
    try:
        var saved_count = 0
        var success = False
        
        # Handle memory-mapped storage
        if self.storage_type.value == StorageType.MEMORY_MAPPED and self.memory_mapped_storage:
            var storage = self.memory_mapped_storage.unsafe_value()
            
            # NEW: Transfer buffer vectors to hot_vectors
            print("📦 Batching", self.buffer.size, "vectors for checkpoint...")
            
            # Add all buffered vectors
            for i in range(self.buffer.size):
                var id = self.buffer.ids[i]
                var vector = List[Float32]()
                for j in range(self.dimension):
                    vector.append(self.buffer.data[i * self.dimension + j])
                
                var metadata_dict = Dict[String, String]()
                if storage.save_vector(id, vector, metadata_dict):
                    saved_count += 1
            
            # Also add vectors from main index
            if self.main_index.size() > 0:
                # Need to iterate through CSR graph nodes
                for node_idx in range(self.main_index.graph.num_nodes):
                    var id = self.main_index.graph.get_node_id(node_idx)
                    if id != "":
                        var vec_ptr = self.main_index.graph.get_vector_ptr(node_idx)
                        var vector = List[Float32]()
                        for j in range(self.dimension):
                            vector.append(vec_ptr[j])
                        
                        var metadata_dict = Dict[String, String]()
                        if storage.save_vector(id, vector, metadata_dict):
                            saved_count += 1
            
            # Now checkpoint with actual data
            success = storage.checkpoint_async()
            if success:
                print("✅ Memory-mapped async checkpoint:", saved_count, "vectors")
        
        return success

# Option 3: Best solution - integrate at flush time
fn _flush_buffer_to_main(mut self) raises:
    """Fixed flush that also updates memory-mapped storage."""
    if self.buffer.size == 0:
        return
    
    var buffer_ids = self.buffer.ids
    var buffer_size = self.buffer.size
    
    # Add to main index (existing logic)
    # ... existing merge logic ...
    
    # NEW: If memory-mapped storage enabled, add vectors there too
    if self.storage_type.value == StorageType.MEMORY_MAPPED and self.memory_mapped_storage:
        var storage = self.memory_mapped_storage.unsafe_value()
        
        for i in range(buffer_size):
            var id = buffer_ids[i]
            var vector = List[Float32]()
            for j in range(self.dimension):
                vector.append(self.buffer.data[i * self.dimension + j])
            
            var metadata_dict = Dict[String, String]()
            _ = storage.save_vector(id, vector, metadata_dict)
    
    self.buffer.clear()