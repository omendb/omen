"""
Fix for segment merging issue - DRAFT CODE

Current bug: self.main_index = new_segment (replaces instead of merging)
Solution: Merge new segment into existing main_index
"""

fn _flush_buffer_to_main(mut self) raises:
    """Fixed version with proper segment merging."""
    if self.buffer.size == 0:
        return
    
    var buffer_ids = self.buffer.ids
    var buffer_size = self.buffer.size
    
    # Check if main_index exists and has data
    if self.main_index.size() > 0:
        # MERGE MODE: Add buffer vectors to existing index
        
        # Add each buffered vector to existing main index
        for i in range(buffer_size):
            var vector = List[Float32]()
            for j in range(self.dimension):
                vector.append(self.buffer.data[i * self.dimension + j])
            
            # Add to existing index (will connect to existing nodes)
            _ = self.main_index.add(buffer_ids[i], vector)
        
        print("📦 Merged", buffer_size, "vectors into main index")
        print("   Main index now has", self.main_index.size(), "vectors")
        
    else:
        # INITIAL BUILD: Create new index from buffer
        var new_segment = DiskANNIndex(
            dimension=self.dimension,
            expected_nodes=buffer_size
        )
        
        # Add all buffered vectors to new segment
        for i in range(buffer_size):
            var vector = List[Float32]()
            for j in range(self.dimension):
                vector.append(self.buffer.data[i * self.dimension + j])
            _ = new_segment.add(buffer_ids[i], vector)
        
        # First segment becomes main index
        self.main_index = new_segment
        print("📦 Created initial index with", buffer_size, "vectors")
    
    # Update memory stats
    var csr_stats = self.main_index.get_memory_stats()
    self.memory_stats.graph_memory = csr_stats.graph_memory  # Don't accumulate
    self.memory_stats.metadata_memory = csr_stats.metadata_memory
    
    # Update id tracking
    for i in range(buffer_size):
        var id = buffer_ids[i]
        self.id_to_idx[id] = 1  # Mark as in main index
    
    # Clear buffer
    self.buffer.clear()
    
    # Don't increment total_vectors here - already done during add

"""
Alternative approach: Multi-segment architecture

Instead of merging into one index, maintain multiple segments:
- segments: List[DiskANNIndex]  
- Search across all segments
- Periodically compact segments

Pros: 
- No expensive merge operations
- Better for concurrent writes
- Used by Lucene, Elasticsearch

Cons:
- More complex search (multi-segment)
- Need compaction strategy
"""