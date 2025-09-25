"""
Production-quality sparse map implementation for OmenDB.

This module provides a memory-efficient hash map using open addressing
with linear probing, designed to replace Mojo's Dict[String, Int] which
uses ~8KB per entry. Targets ~44 bytes per entry similar to tsl::sparse_map.

Key features:
- Open addressing with linear probing
- 75% max load factor with automatic resizing
- Power-of-2 capacity for fast modulo operations
- Thread-safe for single writer (OmenDB design)
- UnsafePointer-based storage for maximum efficiency

Performance characteristics:
- Insert: O(1) average, O(n) worst case (rare with good hash function)
- Get/Contains: O(1) average, O(n) worst case
- Memory: ~44 bytes per entry vs 8KB with Dict[String, Int]
"""

from memory import UnsafePointer
from collections import Optional
from math import log2


@value
struct Entry:
    """Hash map entry with FastDict control byte optimization."""
    var key: String
    var value: Int
    var is_occupied: Bool
    var is_deleted: Bool  # For tombstone marking
    var control_byte: Int8  # FastDict optimization: Top 7 bits of hash for fast rejection

    fn __init__(out self):
        """Initialize empty entry."""
        self.key = String()
        self.value = 0
        self.is_occupied = False
        self.is_deleted = False
        self.control_byte = 0

    fn __init__(out self, key: String, value: Int, control_byte: Int8 = 0):
        """Initialize entry with key-value pair and control byte."""
        self.key = key
        self.value = value
        self.is_occupied = True
        self.is_deleted = False
        self.control_byte = control_byte

    fn clear(mut self):
        """Clear entry marking as deleted."""
        self.key = String()
        self.value = 0
        self.is_occupied = False
        self.is_deleted = True
        self.control_byte = 0

    fn is_available(self) -> Bool:
        """Check if entry slot is available for insertion."""
        return not self.is_occupied


struct SparseMap(Copyable, Movable):
    """
    Memory-efficient hash map with open addressing and linear probing.
    
    Designed to replace Dict[String, Int] with significantly lower memory usage.
    Uses power-of-2 capacities and maintains load factor below 75%.
    
    Key Design Decisions:
    - Linear probing: Simple, cache-friendly, good performance in practice
    - Power-of-2 capacity: Fast modulo using bitwise AND
    - 75% load factor: Good balance between space and performance
    - Tombstone deletion: Maintains probe sequences after deletions
    """
    
    var entries: UnsafePointer[Entry]
    var capacity: Int
    var size: Int
    var load_factor_threshold: Float32

    # Static constants
    alias MIN_CAPACITY = 8
    alias DEFAULT_LOAD_FACTOR = 0.90  # FastDict optimization: 90% vs typical 75%

    fn __init__(out self, initial_capacity: Int = Self.MIN_CAPACITY):
        """
        Initialize sparse map with given initial capacity.
        
        Args:
            initial_capacity: Starting capacity (will be rounded to next power of 2)
        """
        # Ensure capacity is power of 2 and at least MIN_CAPACITY
        var cap = Self.MIN_CAPACITY
        var requested_cap = Self._next_power_of_2(initial_capacity)
        if requested_cap > cap:
            cap = requested_cap
        
        self.capacity = cap
        self.size = 0
        self.load_factor_threshold = Self.DEFAULT_LOAD_FACTOR
        
        # Allocate and initialize entries
        self.entries = UnsafePointer[Entry].alloc(self.capacity)
        for i in range(self.capacity):
            # Initialize each entry as empty
            (self.entries + i).init_pointee_copy(Entry())

    fn __copyinit__(out self, existing: Self):
        """Copy constructor."""
        self.capacity = existing.capacity
        self.size = existing.size
        self.load_factor_threshold = existing.load_factor_threshold
        
        # Allocate new memory and copy entries
        self.entries = UnsafePointer[Entry].alloc(self.capacity)
        for i in range(self.capacity):
            (self.entries + i).init_pointee_copy((existing.entries + i)[])

    fn __moveinit__(out self, owned existing: Self):
        """Move constructor."""
        self.entries = existing.entries
        self.capacity = existing.capacity
        self.size = existing.size
        self.load_factor_threshold = existing.load_factor_threshold
        # CRITICAL: Null out the moved-from pointer to prevent double-free
        existing.entries = UnsafePointer[Entry]()

    fn __del__(owned self):
        """Destructor - free allocated memory."""
        if self.entries:
            # Destroy all entries
            for i in range(self.capacity):
                (self.entries + i).destroy_pointee()
            
            # Free the memory
            self.entries.free()

    # ========================================
    # Core Operations
    # ========================================

    fn insert(mut self, key: String, value: Int) -> Bool:
        """
        Insert key-value pair into map.

        Args:
            key: String key to insert
            value: Integer value associated with key

        Returns:
            True if new key was inserted, False if existing key was updated
        """
        # Check if resize needed before insertion
        if self._should_resize():
            self._resize()

        return self._insert_without_resize(key, value)

    fn _insert_without_resize(mut self, key: String, value: Int) -> Bool:
        """
        Insert key-value pair without triggering resize.
        Used internally during resize to avoid recursion.
        """
        var hash_val = self._hash(key)
        var control_byte = self._get_control_byte(hash_val)
        var index = hash_val & (self.capacity - 1)  # Fast modulo for power of 2
        var distance = 0  # For quadratic probing

        # FastDict optimization: Quadratic probing reduces clustering
        while distance < self.capacity:
            var entry_ptr = self.entries + index
            var entry = entry_ptr[]

            if entry.is_available():
                # Found empty slot - insert new entry with control byte
                entry_ptr.init_pointee_copy(Entry(key, value, control_byte))
                self.size += 1
                return True
            elif entry.is_occupied and entry.control_byte == control_byte and entry.key == key:
                # FastDict optimization: Control byte fast-reject before string comparison
                entry_ptr[].value = value
                return False

            # FastDict optimization: Quadratic probing
            distance += 1
            index = (hash_val + distance * distance) & (self.capacity - 1)

        # Should never reach here with proper capacity management
        return False

    fn get(self, key: String) -> Optional[Int]:
        """
        Get value for given key.
        
        Args:
            key: String key to look up
            
        Returns:
            Optional[Int] containing value if found, None otherwise
        """
        var hash_val = self._hash(key)
        var control_byte = self._get_control_byte(hash_val)
        var index = hash_val & (self.capacity - 1)
        var distance = 0
        
        # FastDict optimization: Quadratic probing with control byte fast-reject
        while distance < self.capacity:
            var entry = (self.entries + index)[]
            
            if not entry.is_occupied and not entry.is_deleted:
                # Empty slot - key not found
                return None
            elif entry.is_occupied and entry.control_byte == control_byte and entry.key == key:
                # FastDict optimization: Control byte fast-reject before string comparison
                return entry.value
            
            # FastDict optimization: Quadratic probing
            distance += 1
            index = (hash_val + distance * distance) & (self.capacity - 1)
        
        # Exhausted search
        return None

    fn contains(self, key: String) -> Bool:
        """
        Check if key exists in map.
        
        Args:
            key: String key to check
            
        Returns:
            True if key exists, False otherwise
        """
        return self.get(key) is not None

    fn remove(mut self, key: String) -> Bool:
        """
        Remove key from map using tombstone marking.
        
        Args:
            key: String key to remove
            
        Returns:
            True if key was found and removed, False otherwise
        """
        var hash_val = self._hash(key)
        var index = hash_val & (self.capacity - 1)
        var original_index = index
        
        # Linear probing to find key
        while True:
            var entry_ptr = self.entries + index
            var entry = entry_ptr[]
            
            if not entry.is_occupied and not entry.is_deleted:
                # Empty slot - key not found
                return False
            elif entry.is_occupied and entry.key == key:
                # Found key - mark as deleted (tombstone)
                entry_ptr[].clear()
                self.size -= 1
                return True
            
            # Continue probing
            index = (index + 1) & (self.capacity - 1)
            
            # Wrapped around - key not found
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

    fn _hash(self, key: String) -> Int:
        """
        Hash function for strings using FNV-1a algorithm.
        
        Args:
            key: String to hash
            
        Returns:
            Hash value as integer
        """
        # FNV-1a constants
        var hash = 2166136261  # FNV offset basis
        var prime = 16777619   # FNV prime
        
        # Hash each character using as_bytes()
        var key_bytes = key.as_bytes()
        for i in range(len(key_bytes)):
            var byte_val = Int(key_bytes[i])
            hash ^= byte_val
            hash *= prime
        
        return hash if hash >= 0 else -hash

    @always_inline
    fn _get_control_byte(self, hash_val: Int) -> Int8:
        """FastDict optimization: Extract top 7 bits of hash for fast rejection."""
        return Int8((hash_val >> 25) & 0x7F)  # Top 7 bits for 32-bit hash

    fn _should_resize(self) -> Bool:
        """Check if map should be resized based on load factor."""
        return self.load_factor() >= self.load_factor_threshold

    fn _resize(mut self):
        """Double the capacity and rehash all entries."""
        var old_entries = self.entries
        var old_capacity = self.capacity
        
        # Double capacity
        self.capacity *= 2
        self.size = 0  # Will be incremented during rehashing
        
        # Allocate new memory
        self.entries = UnsafePointer[Entry].alloc(self.capacity)
        for i in range(self.capacity):
            (self.entries + i).init_pointee_copy(Entry())
        
        # Rehash all existing entries
        for i in range(old_capacity):
            var entry = (old_entries + i)[]
            if entry.is_occupied:
                # Re-insert using new capacity (no resize check needed)
                _ = self._insert_without_resize(entry.key, entry.value)
        
        # Clean up old memory
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
        """Return statistics about the hash map."""
        var stats = String()
        stats += "SparseMap Statistics:\n"
        stats += "  Size: " + String(self.size) + "\n"
        stats += "  Capacity: " + String(self.capacity) + "\n"
        stats += "  Load Factor: " + String(self.load_factor()) + "\n"
        stats += "  Memory Usage: ~" + String(self.capacity * 44) + " bytes\n"
        return stats

    fn validate(self) -> Bool:
        """Validate internal consistency of the map."""
        var count = 0
        for i in range(self.capacity):
            var entry = (self.entries + i)[]
            if entry.is_occupied:
                count += 1
        
        return count == self.size