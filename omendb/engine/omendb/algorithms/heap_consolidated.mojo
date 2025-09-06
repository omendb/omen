"""
Consolidated heap implementations for OmenDB.
Uses state-of-the-art patterns from Modular's MAX kernels and production HNSW.

Three specialized heaps for different use cases:
1. DynamicMinHeap - For search candidates (grows as needed)
2. FixedMaxHeap - For top-k results (fixed size, evicts worst)
3. BatchHeap - For batch operations (pre-sized)
"""

from collections import List
from memory import UnsafePointer
from math import min, max

# =============================================================================
# Common Types
# =============================================================================

struct HeapItem(Copyable, Movable):
    """Universal heap item for all heap types."""
    var id: UInt32
    var distance: Float32
    var metadata: UInt32  # Can store visited flag or other data
    
    fn __init__(out self, id: UInt32, distance: Float32, metadata: UInt32 = 0):
        self.id = id
        self.distance = distance
        self.metadata = metadata
    
    fn __copyinit__(out self, existing: Self):
        self.id = existing.id
        self.distance = existing.distance
        self.metadata = existing.metadata
    
    fn __moveinit__(out self, owned existing: Self):
        self.id = existing.id
        self.distance = existing.distance
        self.metadata = existing.metadata
    
    fn __lt__(self, other: Self) -> Bool:
        """Less than comparison for min-heap."""
        return self.distance < other.distance
    
    fn __gt__(self, other: Self) -> Bool:
        """Greater than comparison for max-heap."""
        return self.distance > other.distance

# =============================================================================
# Dynamic Min-Heap (For Search Candidates)
# =============================================================================

struct DynamicMinHeap(Copyable, Movable):
    """
    Dynamic min-heap for HNSW search candidates.
    Based on production hnsw_index.mojo implementation.
    
    Features:
    - Grows dynamically with List
    - No upfront allocation
    - Simple and reliable
    """
    var items: List[HeapItem]
    
    fn __init__(out self):
        """Initialize empty heap."""
        self.items = List[HeapItem]()
    
    fn __copyinit__(out self, existing: Self):
        self.items = existing.items
    
    fn __moveinit__(out self, owned existing: Self):
        self.items = existing.items^
    
    fn size(self) -> Int:
        return len(self.items)
    
    fn is_empty(self) -> Bool:
        return self.size() == 0
    
    fn push(mut self, item: HeapItem):
        """Add item to heap. O(log n)."""
        self.items.append(item)
        self._sift_up(self.size() - 1)
    
    fn pop(mut self) -> HeapItem:
        """Remove and return minimum. O(log n)."""
        if self.is_empty():
            return HeapItem(UInt32.MAX, Float32.MAX)
        
        var min_item = self.items[0]
        var last_idx = self.size() - 1
        
        # Move last to root
        self.items[0] = self.items[last_idx]
        _ = self.items.pop()
        
        # Restore heap property
        if not self.is_empty():
            self._sift_down(0)
        
        return min_item
    
    fn peek(self) -> HeapItem:
        """View minimum without removing."""
        if self.is_empty():
            return HeapItem(UInt32.MAX, Float32.MAX)
        return self.items[0]
    
    fn clear(mut self):
        """Clear all items."""
        self.items = List[HeapItem]()
    
    @always_inline
    fn _parent(self, i: Int) -> Int:
        return (i - 1) // 2
    
    @always_inline
    fn _left_child(self, i: Int) -> Int:
        return 2 * i + 1
    
    @always_inline
    fn _right_child(self, i: Int) -> Int:
        return 2 * i + 2
    
    fn _sift_up(mut self, start_idx: Int):
        """Bubble up to maintain min-heap property."""
        var idx = start_idx
        while idx > 0:
            var parent_idx = self._parent(idx)
            if self.items[idx] < self.items[parent_idx]:
                # Swap with parent
                var temp = self.items[idx]
                self.items[idx] = self.items[parent_idx]
                self.items[parent_idx] = temp
                idx = parent_idx
            else:
                break
    
    fn _sift_down(mut self, start_idx: Int):
        """Bubble down to maintain min-heap property."""
        var idx = start_idx
        var size = self.size()
        
        while True:
            var min_idx = idx
            var left = self._left_child(idx)
            var right = self._right_child(idx)
            
            if left < size and self.items[left] < self.items[min_idx]:
                min_idx = left
            
            if right < size and self.items[right] < self.items[min_idx]:
                min_idx = right
            
            if min_idx != idx:
                # Swap with smaller child
                var temp = self.items[idx]
                self.items[idx] = self.items[min_idx]
                self.items[min_idx] = temp
                idx = min_idx
            else:
                break

# =============================================================================
# Fixed Max-Heap (For Top-K Results)
# =============================================================================

struct FixedMaxHeap(Copyable, Movable):
    """
    Fixed-size max-heap for maintaining top-k smallest elements.
    Based on heap_topk.mojo pattern.
    
    Features:
    - Fixed capacity (no reallocations)
    - Maintains k smallest by evicting largest
    - Memory efficient
    """
    var items: UnsafePointer[HeapItem]
    var capacity: Int
    var size: Int
    
    fn __init__(out self, k: Int):
        """Initialize with capacity k."""
        self.capacity = k
        self.size = 0
        self.items = UnsafePointer[HeapItem].alloc(k)
        
        # Initialize with sentinel values
        for i in range(k):
            self.items[i] = HeapItem(UInt32.MAX, Float32.MAX)
    
    fn __copyinit__(out self, existing: Self):
        self.capacity = existing.capacity
        self.size = existing.size
        self.items = UnsafePointer[HeapItem].alloc(self.capacity)
        for i in range(self.capacity):
            self.items[i] = existing.items[i]
    
    fn __moveinit__(out self, owned existing: Self):
        self.capacity = existing.capacity
        self.size = existing.size
        self.items = existing.items
        existing.items = UnsafePointer[HeapItem]()
    
    fn __del__(owned self):
        if self.items:
            self.items.free()
    
    @always_inline
    fn push(mut self, item: HeapItem):
        """
        Add item if it's better than worst (for top-k smallest).
        Uses max-heap to evict largest when full.
        """
        if self.size < self.capacity:
            # Not full, just add
            self.items[self.size] = item
            self._bubble_up(self.size)
            self.size += 1
        elif item.distance < self.items[0].distance:
            # Replace root (largest) with smaller item
            self.items[0] = item
            self._bubble_down(0)
    
    fn get_sorted_results(self) -> List[HeapItem]:
        """Extract all items in sorted order (smallest to largest)."""
        var results = List[HeapItem]()
        
        # Copy items to temporary array
        var temp = UnsafePointer[HeapItem].alloc(self.size)
        for i in range(self.size):
            temp[i] = self.items[i]
        
        # Simple insertion sort (fine for small k)
        for i in range(self.size):
            var min_idx = i
            for j in range(i + 1, self.size):
                if temp[j].distance < temp[min_idx].distance:
                    min_idx = j
            
            # Swap if needed
            if min_idx != i:
                var tmp = temp[i]
                temp[i] = temp[min_idx]
                temp[min_idx] = tmp
            
            results.append(temp[i])
        
        temp.free()
        return results
    
    @always_inline
    fn _bubble_up(mut self, idx: Int):
        """Maintain max-heap property upward."""
        var current = idx
        while current > 0:
            var parent = (current - 1) // 2
            if self.items[current] > self.items[parent]:
                # Swap with parent
                var temp = self.items[current]
                self.items[current] = self.items[parent]
                self.items[parent] = temp
                current = parent
            else:
                break
    
    @always_inline
    fn _bubble_down(mut self, idx: Int):
        """Maintain max-heap property downward."""
        var current = idx
        while True:
            var left = 2 * current + 1
            var right = 2 * current + 2
            var largest = current
            
            if left < self.size and self.items[left] > self.items[largest]:
                largest = left
            
            if right < self.size and self.items[right] > self.items[largest]:
                largest = right
            
            if largest != current:
                # Swap with larger child
                var temp = self.items[current]
                self.items[current] = self.items[largest]
                self.items[largest] = temp
                current = largest
            else:
                break

# =============================================================================
# Batch Heap (For Bulk Operations)
# =============================================================================

struct BatchHeap:
    """
    Pre-sized heap for batch operations.
    Optimized for known-size workloads.
    """
    var items: UnsafePointer[HeapItem]
    var capacity: Int
    var size: Int
    
    fn __init__(out self, expected_size: Int):
        """Pre-allocate for expected size."""
        # Add 20% buffer for growth
        self.capacity = expected_size + expected_size // 5
        self.size = 0
        self.items = UnsafePointer[HeapItem].alloc(self.capacity)
    
    fn __del__(owned self):
        if self.items:
            self.items.free()
    
    fn add_batch(mut self, items: UnsafePointer[HeapItem], count: Int):
        """Add multiple items efficiently."""
        # Ensure capacity
        if self.size + count > self.capacity:
            self._grow(self.size + count)
        
        # Copy items
        for i in range(count):
            self.items[self.size + i] = items[i]
        
        self.size += count
        
        # Heapify in bulk (more efficient than individual inserts)
        self._build_heap()
    
    fn _grow(mut self, new_size: Int):
        """Grow capacity to accommodate new size."""
        var new_capacity = max(new_size, self.capacity * 2)
        var new_items = UnsafePointer[HeapItem].alloc(new_capacity)
        
        # Copy existing
        for i in range(self.size):
            new_items[i] = self.items[i]
        
        self.items.free()
        self.items = new_items
        self.capacity = new_capacity
    
    fn _build_heap(mut self):
        """Build heap from unordered array. O(n)."""
        # Start from last non-leaf
        for i in range(self.size // 2 - 1, -1, -1):
            self._sift_down(i)
    
    fn _sift_down(mut self, start_idx: Int):
        """Standard sift-down for min-heap."""
        var idx = start_idx
        while True:
            var left = 2 * idx + 1
            var right = 2 * idx + 2
            var smallest = idx
            
            if left < self.size and self.items[left].distance < self.items[smallest].distance:
                smallest = left
            
            if right < self.size and self.items[right].distance < self.items[smallest].distance:
                smallest = right
            
            if smallest != idx:
                var temp = self.items[idx]
                self.items[idx] = self.items[smallest]
                self.items[smallest] = temp
                idx = smallest
            else:
                break

# =============================================================================
# Helper Functions
# =============================================================================

fn select_heap_type(use_case: String) -> String:
    """
    Recommend heap type based on use case.
    
    Returns implementation guidance.
    """
    if use_case == "search_candidates":
        return "Use DynamicMinHeap - grows as needed, no pre-allocation"
    elif use_case == "top_k_results":
        return "Use FixedMaxHeap - maintains k smallest efficiently"
    elif use_case == "batch_insert":
        return "Use BatchHeap - optimized for bulk operations"
    elif use_case == "thread_safe":
        return "Wrap any heap with mutex/spinlock for thread safety"
    else:
        return "Use DynamicMinHeap as default"