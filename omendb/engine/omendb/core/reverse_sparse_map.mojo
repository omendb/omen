"""
Production-quality reverse sparse map implementation for OmenDB.

This module provides a memory-efficient hash map for Int->String mappings
using open addressing with linear probing, designed to replace Mojo's 
Dict[Int, String] which uses excessive memory.

Key features:
- Open addressing with linear probing
- 75% max load factor with automatic resizing
- Power-of-2 capacity for fast modulo operations
- Memory efficient: ~44 bytes per entry vs 8KB with Dict[Int, String]
"""

from memory import UnsafePointer
from collections import Optional


@value
struct ReverseEntry:
    """Reverse hash map entry for Int->String mapping."""
    var key: Int
    var value: String
    var is_occupied: Bool
    var is_deleted: Bool

    fn __init__(out self):
        """Initialize empty entry."""
        self.key = 0
        self.value = String()
        self.is_occupied = False
        self.is_deleted = False

    fn __init__(out self, key: Int, value: String):
        """Initialize entry with key-value pair."""
        self.key = key
        self.value = value
        self.is_occupied = True
        self.is_deleted = False

    fn clear(mut self):
        """Clear entry marking as deleted."""
        self.key = 0
        self.value = String()
        self.is_occupied = False
        self.is_deleted = True

    fn is_available(self) -> Bool:
        """Check if entry slot is available for insertion."""
        return not self.is_occupied


struct ReverseSparseMap(Copyable, Movable):
    """
    Memory-efficient reverse hash map (Int->String) using open addressing.
    
    Designed to replace Dict[Int, String] with significantly lower memory usage.
    Uses power-of-2 capacities and maintains load factor below 75%.
    """
    
    var entries: UnsafePointer[ReverseEntry]
    var capacity: Int
    var size: Int
    var load_factor_threshold: Float32

    # Static constants
    alias MIN_CAPACITY = 8
    alias DEFAULT_LOAD_FACTOR = 0.75

    fn __init__(out self, initial_capacity: Int = Self.MIN_CAPACITY):
        """Initialize reverse sparse map."""
        var cap = Self.MIN_CAPACITY
        var requested_cap = Self._next_power_of_2(initial_capacity)
        if requested_cap > cap:
            cap = requested_cap
        
        self.capacity = cap
        self.size = 0
        self.load_factor_threshold = Self.DEFAULT_LOAD_FACTOR
        
        self.entries = UnsafePointer[ReverseEntry].alloc(self.capacity)
        for i in range(self.capacity):
            (self.entries + i).init_pointee_copy(ReverseEntry())

    fn __copyinit__(out self, existing: Self):
        """Copy constructor."""
        self.capacity = existing.capacity
        self.size = existing.size
        self.load_factor_threshold = existing.load_factor_threshold
        
        self.entries = UnsafePointer[ReverseEntry].alloc(self.capacity)
        for i in range(self.capacity):
            (self.entries + i).init_pointee_copy((existing.entries + i)[])

    fn __moveinit__(out self, owned existing: Self):
        """Move constructor."""
        self.entries = existing.entries
        self.capacity = existing.capacity
        self.size = existing.size
        self.load_factor_threshold = existing.load_factor_threshold
        # CRITICAL: Null out the moved-from pointer to prevent double-free
        existing.entries = UnsafePointer[ReverseEntry]()

    fn __del__(owned self):
        """Destructor - free allocated memory."""
        if self.entries:
            for i in range(self.capacity):
                (self.entries + i).destroy_pointee()
            self.entries.free()

    # ========================================
    # Core Operations
    # ========================================

    fn insert(mut self, key: Int, value: String) -> Bool:
        """Insert key-value pair into map."""
        if self._should_resize():
            self._resize()

        return self._insert_without_resize(key, value)

    fn _insert_without_resize(mut self, key: Int, value: String) -> Bool:
        """Insert key-value pair without triggering resize. Used internally during resize."""
        var hash_val = self._hash(key)
        var index = hash_val & (self.capacity - 1)
        var original_index = index

        while True:
            var entry_ptr = self.entries + index
            var entry = entry_ptr[]

            if entry.is_available():
                entry_ptr.init_pointee_copy(ReverseEntry(key, value))
                self.size += 1
                return True
            elif entry.key == key:
                entry_ptr[].value = value
                return False

            index = (index + 1) & (self.capacity - 1)

            if index == original_index:
                # Table full - should not happen with proper capacity management
                return False

    fn get(self, key: Int) -> Optional[String]:
        """Get value for given key."""
        var hash_val = self._hash(key)
        var index = hash_val & (self.capacity - 1)
        var original_index = index
        
        while True:
            var entry = (self.entries + index)[]
            
            if not entry.is_occupied and not entry.is_deleted:
                return None
            elif entry.is_occupied and entry.key == key:
                return entry.value
            
            index = (index + 1) & (self.capacity - 1)
            
            if index == original_index:
                return None

    fn contains(self, key: Int) -> Bool:
        """Check if key exists in map."""
        return self.get(key) is not None

    fn remove(mut self, key: Int) -> Bool:
        """Remove key from map using tombstone marking."""
        var hash_val = self._hash(key)
        var index = hash_val & (self.capacity - 1)
        var original_index = index
        
        while True:
            var entry_ptr = self.entries + index
            var entry = entry_ptr[]
            
            if not entry.is_occupied and not entry.is_deleted:
                return False
            elif entry.is_occupied and entry.key == key:
                entry_ptr[].clear()
                self.size -= 1
                return True
            
            index = (index + 1) & (self.capacity - 1)
            
            if index == original_index:
                return False

    # ========================================
    # Utility Methods
    # ========================================

    fn len(self) -> Int:
        """Return number of entries in map."""
        return self.size

    fn is_empty(self) -> Bool:
        """Check if map is empty."""
        return self.size == 0

    fn load_factor(self) -> Float32:
        """Calculate current load factor."""
        if self.capacity == 0:
            return 0.0
        return Float32(self.size) / Float32(self.capacity)

    fn clear(mut self):
        """Clear all entries from the map."""
        # Mark all entries as empty/deleted
        for i in range(self.capacity):
            (self.entries + i)[].clear()
        self.size = 0

    # ========================================
    # Private Helper Methods
    # ========================================

    fn _hash(self, key: Int) -> Int:
        """Hash function for integers using integer hash."""
        var hash = key
        hash = ((hash >> 16) ^ hash) * 0x45d9f3b
        hash = ((hash >> 16) ^ hash) * 0x45d9f3b
        hash = (hash >> 16) ^ hash
        return hash if hash >= 0 else -hash

    fn _should_resize(self) -> Bool:
        """Check if map should be resized based on load factor."""
        return self.load_factor() >= self.load_factor_threshold

    fn _resize(mut self):
        """Double the capacity and rehash all entries."""
        var old_entries = self.entries
        var old_capacity = self.capacity
        
        self.capacity *= 2
        self.size = 0
        
        self.entries = UnsafePointer[ReverseEntry].alloc(self.capacity)
        for i in range(self.capacity):
            (self.entries + i).init_pointee_copy(ReverseEntry())
        
        for i in range(old_capacity):
            var entry = (old_entries + i)[]
            if entry.is_occupied:
                _ = self._insert_without_resize(entry.key, entry.value)
        
        for i in range(old_capacity):
            (old_entries + i).destroy_pointee()
        old_entries.free()

    @staticmethod
    fn _next_power_of_2(n: Int) -> Int:
        """Find the next power of 2 greater than or equal to n."""
        if n <= 0:
            return 1
        
        var power = 1
        while power < n:
            power *= 2
        return power

    # ========================================
    # Debug and Statistics
    # ========================================

    fn stats(self) -> String:
        """Return statistics about the reverse hash map."""
        var stats = String()
        stats += "ReverseSparseMap Statistics:\n"
        stats += "  Size: " + String(self.size) + "\n"
        stats += "  Capacity: " + String(self.capacity) + "\n"
        stats += "  Load Factor: " + String(self.load_factor()) + "\n"
        stats += "  Memory Usage: ~" + String(self.capacity * 44) + " bytes\n"
        stats += "  vs Dict[Int, String]: ~" + String(self.size * 8000) + " bytes\n"
        return stats