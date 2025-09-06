"""
Fixed priority queue implementation with efficient memory management.
Addresses std::bad_alloc issues by using incremental allocation.
"""

from collections import List
from math import floor

struct SearchCandidate(Copyable, Movable):
    """Search candidate for graph traversal."""
    var node_id: UInt32
    var distance: Float32
    var visited: Bool
    
    fn __init__(out self, node_id: UInt32, distance: Float32, visited: Bool = False):
        self.node_id = node_id
        self.distance = distance
        self.visited = visited
    
    fn __copyinit__(out self, existing: Self):
        self.node_id = existing.node_id
        self.distance = existing.distance
        self.visited = existing.visited
    
    fn __moveinit__(out self, owned existing: Self):
        self.node_id = existing.node_id
        self.distance = existing.distance
        self.visited = existing.visited


struct MinHeapPriorityQueue(Copyable, Movable):
    """
    Memory-efficient min-heap priority queue.
    
    Key improvements:
    - Incremental allocation (starts small, grows as needed)
    - Efficient reallocation strategy
    - No upfront memory allocation
    """
    
    var data: List[SearchCandidate]
    var capacity: Int
    var size: Int
    
    fn __init__(out self, capacity: Int = 1000):
        """Initialize with capacity hint, but don't allocate."""
        self.capacity = capacity
        self.size = 0
        self.data = List[SearchCandidate]()
        # Don't pre-allocate - grow as needed
    
    fn __copyinit__(out self, existing: Self):
        self.capacity = existing.capacity
        self.size = existing.size
        self.data = existing.data
    
    fn __moveinit__(out self, owned existing: Self):
        self.capacity = existing.capacity
        self.size = existing.size
        self.data = existing.data^
    
    fn push(mut self, candidate: SearchCandidate):
        """Push with incremental allocation."""
        if self.size < self.capacity:
            # Ensure we have space for one more element
            if len(self.data) <= self.size:
                # Grow by small chunks to avoid large allocations
                var grow_size = min(16, self.capacity - len(self.data))
                for _ in range(grow_size):
                    self.data.append(SearchCandidate(0, Float32.MAX, True))
            
            self.data[self.size] = candidate
            self._heapify_up(self.size)
            self.size += 1
        else:
            # Replace worst if better
            if self.size > 0 and candidate.distance < self.data[0].distance:
                self.data[0] = candidate
                self._heapify_down(0)
    
    fn pop(mut self) -> SearchCandidate:
        """Pop minimum element."""
        if self.size == 0:
            return SearchCandidate(0, Float32.MAX, True)
        
        var min_candidate = self.data[0]
        self.size -= 1
        
        if self.size > 0:
            self.data[0] = self.data[self.size]
            self._heapify_down(0)
        
        return min_candidate
    
    fn peek_min(self) -> SearchCandidate:
        """Peek at minimum without removing."""
        if self.size == 0:
            return SearchCandidate(0, Float32.MAX, True)
        return self.data[0]
    
    fn is_empty(self) -> Bool:
        """Check if empty."""
        return self.size == 0
    
    fn current_size(self) -> Int:
        """Get current size."""
        return self.size
    
    fn clear(mut self):
        """Clear the queue efficiently."""
        self.size = 0
        # Keep allocated memory for reuse
    
    fn _parent(self, i: Int) -> Int:
        """Get parent index."""
        return (i - 1) // 2
    
    fn _left_child(self, i: Int) -> Int:
        """Get left child index."""
        return 2 * i + 1
    
    fn _right_child(self, i: Int) -> Int:
        """Get right child index."""
        return 2 * i + 2
    
    fn _swap(mut self, i: Int, j: Int):
        """Swap two elements."""
        var temp = self.data[i]
        self.data[i] = self.data[j]
        self.data[j] = temp
    
    fn _heapify_up(mut self, start_idx: Int):
        """Restore heap property upward."""
        var current = start_idx
        
        while current > 0:
            var parent_idx = self._parent(current)
            
            if self.data[current].distance >= self.data[parent_idx].distance:
                break
            
            self._swap(current, parent_idx)
            current = parent_idx
    
    fn _heapify_down(mut self, start_idx: Int):
        """Restore heap property downward."""
        var current = start_idx
        
        while True:
            var smallest = current
            var left = self._left_child(current)
            var right = self._right_child(current)
            
            if left < self.size and self.data[left].distance < self.data[smallest].distance:
                smallest = left
            
            if right < self.size and self.data[right].distance < self.data[smallest].distance:
                smallest = right
            
            if smallest == current:
                break
            
            self._swap(current, smallest)
            current = smallest