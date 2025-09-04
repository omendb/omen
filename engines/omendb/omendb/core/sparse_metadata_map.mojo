"""
Efficient sparse metadata storage for OmenDB.

This module provides a memory-efficient replacement for Dict[String, Metadata]
that avoids the 8KB per entry overhead of Mojo's Dict.

Since most vectors don't have metadata, and those that do typically have
only a few key-value pairs, we optimize for the sparse case.
"""

from memory import UnsafePointer
from collections import Optional, List
from .sparse_map import SparseMap
from .metadata import Metadata


struct CompactMetadata(Copyable, Movable):
    """
    Compact metadata storage using parallel arrays instead of Dict.
    
    For vectors with metadata, we typically see 1-5 key-value pairs.
    Using parallel arrays is much more memory efficient than Dict[String, String].
    """
    var keys: List[String]
    var values: List[String]
    
    fn __init__(out self):
        """Initialize empty metadata."""
        self.keys = List[String]()
        self.values = List[String]()
    
    fn __init__(out self, metadata: Metadata):
        """Initialize from regular Metadata."""
        self.keys = List[String]()
        self.values = List[String]()
        
        # Copy from parallel arrays
        for i in range(len(metadata.keys)):
            self.keys.append(metadata.keys[i])
            self.values.append(metadata.values[i])
    
    fn __copyinit__(out self, existing: Self):
        """Copy constructor."""
        self.keys = existing.keys
        self.values = existing.values
    
    fn __moveinit__(out self, owned existing: Self):
        """Move constructor."""
        self.keys = existing.keys^
        self.values = existing.values^
    
    fn to_metadata(self) -> Metadata:
        """Convert back to regular Metadata."""
        var metadata = Metadata()
        for i in range(len(self.keys)):
            try:
                metadata.set(self.keys[i], self.values[i])
            except:
                pass  # Skip on error
        return metadata
    
    fn get(self, key: String) -> Optional[String]:
        """Get value for a key."""
        for i in range(len(self.keys)):
            if self.keys[i] == key:
                return Optional[String](self.values[i])
        return Optional[String]()
    
    fn set(mut self, key: String, value: String):
        """Set a key-value pair."""
        # Check if key exists
        for i in range(len(self.keys)):
            if self.keys[i] == key:
                self.values[i] = value
                return
        
        # Add new key-value pair
        self.keys.append(key)
        self.values.append(value)
    
    fn len(self) -> Int:
        """Return number of key-value pairs."""
        return len(self.keys)
    
    fn is_empty(self) -> Bool:
        """Check if metadata is empty."""
        return len(self.keys) == 0


struct SparseMetadataMap(Copyable, Movable):
    """
    Memory-efficient metadata storage using SparseMap and compact storage.
    
    Instead of Dict[String, Metadata] where each entry uses 8KB,
    we use SparseMap[String, CompactMetadata] with ~200 bytes per entry.
    
    Memory savings: ~40x reduction for vectors with metadata.
    """
    
    var entries: List[CompactMetadata]  # Actual metadata storage
    var id_to_index: SparseMap  # Map vector ID to index in entries
    
    fn __init__(out self, initial_capacity: Int = 64):
        """Initialize sparse metadata map."""
        self.entries = List[CompactMetadata](capacity=initial_capacity)
        self.id_to_index = SparseMap(initial_capacity)
    
    fn __copyinit__(out self, existing: Self):
        """Copy constructor."""
        self.entries = existing.entries
        self.id_to_index = existing.id_to_index
    
    fn __moveinit__(out self, owned existing: Self):
        """Move constructor."""
        self.entries = existing.entries^
        self.id_to_index = existing.id_to_index^
    
    fn set(mut self, vector_id: String, metadata: Metadata) -> Bool:
        """
        Store metadata for a vector.
        
        Args:
            vector_id: The vector ID
            metadata: The metadata to store
            
        Returns:
            True if stored successfully
        """
        # Skip empty metadata
        if len(metadata) == 0:
            return False
        
        # Convert to compact format
        var compact = CompactMetadata(metadata)
        
        # Check if vector already has metadata
        var existing_idx = self.id_to_index.get(vector_id)
        if existing_idx:
            # Update existing
            var idx = existing_idx.value()
            if idx < len(self.entries):
                self.entries[idx] = compact
                return True
        
        # Add new entry
        var new_idx = len(self.entries)
        self.entries.append(compact)
        _ = self.id_to_index.insert(vector_id, new_idx)
        return True
    
    fn get(self, vector_id: String) -> Optional[Metadata]:
        """
        Get metadata for a vector.
        
        Args:
            vector_id: The vector ID
            
        Returns:
            Optional[Metadata] containing the metadata if found
        """
        var idx_opt = self.id_to_index.get(vector_id)
        if not idx_opt:
            return Optional[Metadata]()
        
        var idx = idx_opt.value()
        if idx >= len(self.entries):
            return Optional[Metadata]()
        
        # Convert compact metadata back to regular Metadata
        var metadata = self.entries[idx].to_metadata()
        return Optional[Metadata](metadata)
    
    fn contains(self, vector_id: String) -> Bool:
        """Check if a vector has metadata."""
        return self.id_to_index.contains(vector_id)
    
    fn remove(mut self, vector_id: String) -> Bool:
        """
        Remove metadata for a vector.
        
        Args:
            vector_id: The vector ID
            
        Returns:
            True if metadata was removed
        """
        var idx_opt = self.id_to_index.get(vector_id)
        if idx_opt:
            _ = self.id_to_index.remove(vector_id)
        if not idx_opt:
            return False
        
        # Note: We don't actually remove from entries array to avoid shifting
        # This is a memory vs performance trade-off
        # The slot will be reused or cleaned up on clear()
        
        return True
    
    fn clear(mut self):
        """Clear all metadata."""
        self.entries = List[CompactMetadata]()
        self.id_to_index = SparseMap(64)  # Reset with default capacity
    
    fn len(self) -> Int:
        """Return number of vectors with metadata."""
        return self.id_to_index.len()
    
    fn is_empty(self) -> Bool:
        """Check if map is empty."""
        return self.id_to_index.is_empty()
    
    fn stats(self) -> String:
        """Return statistics about the metadata map."""
        var stats = String()
        stats += "SparseMetadataMap Statistics:\n"
        stats += "  Vectors with metadata: " + String(self.id_to_index.len()) + "\n"
        
        # Calculate average metadata size
        var total_kvs = 0
        for i in range(len(self.entries)):
            total_kvs += self.entries[i].len()
        
        var avg_kvs = Float32(total_kvs) / Float32(len(self.entries)) if len(self.entries) > 0 else 0
        stats += "  Average KV pairs per vector: " + String(avg_kvs) + "\n"
        
        # Estimate memory usage
        # SparseMap: ~44 bytes per entry
        # CompactMetadata: ~50 bytes base + (key_len + value_len) * avg_kvs
        var sparse_map_mem = self.id_to_index.len() * 44
        var metadata_mem = len(self.entries) * (50 + Int(avg_kvs * 40))  # Assume 40 bytes per KV
        stats += "  Estimated memory: ~" + String(sparse_map_mem + metadata_mem) + " bytes\n"
        stats += "  vs Dict[String, Metadata]: ~" + String(self.id_to_index.len() * 8000) + " bytes\n"
        stats += "  Savings: " + String(Float32(self.id_to_index.len() * 8000) / Float32(sparse_map_mem + metadata_mem)) + "x\n"
        
        return stats