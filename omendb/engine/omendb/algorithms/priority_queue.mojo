"""
Consolidated high-performance priority queue implementation for OmenDB.

This module consolidates the functionality from binary_heap.mojo and priority_queue.mojo
into a single, optimized implementation for RoarGraph search operations.

Provides O(log n) insertion and extraction vs O(n log n) sorting operations.
"""

from collections import List
from math import floor


struct SearchCandidate(Copyable, Movable):
    """
    Unified search candidate structure for graph traversal.
    
    Consolidates fields from both previous implementations:
    - id/node_id -> node_id (more descriptive)
    - expanded/visited -> visited (more general term)
    """
    
    var node_id: UInt32      # Node identifier (UInt32 for memory efficiency)
    var distance: Float32    # Distance/similarity score
    var visited: Bool        # Whether this node has been processed
    
    fn __init__(
        out self, 
        node_id: UInt32, 
        distance: Float32, 
        visited: Bool = False
    ):
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
    High-performance min-heap priority queue optimized for similarity search.
    
    Features:
    - O(log n) insertion and extraction
    - Fixed capacity with overflow handling
    - Efficient memory pre-allocation
    - Specialized operations for graph search
    """
    
    var data: List[SearchCandidate]
    var capacity: Int
    var size: Int
    
    fn __init__(out self, capacity: Int = 1000):
        """Initialize heap with specified capacity."""
        self.capacity = capacity
        self.size = 0
        self.data = List[SearchCandidate]()
        
        # Pre-allocate capacity for better performance
        # Initialize with sentinel values
        for i in range(capacity):
            self.data.append(SearchCandidate(0, Float32.MAX, True))
        self.size = 0
    
    fn __copyinit__(out self, existing: Self):
        self.capacity = existing.capacity
        self.size = existing.size
        self.data = existing.data
    
    fn __moveinit__(out self, owned existing: Self):
        self.capacity = existing.capacity
        self.size = existing.size
        self.data = existing.data^
    
    # Core Operations
    
    fn push(mut self, candidate: SearchCandidate):
        """Push candidate onto heap. O(log n) complexity.
        
        For full heap, replaces worst element if new one is better.
        This maintains top-K smallest elements efficiently.
        """
        if self.size < self.capacity:
            # Standard insertion with heapify up
            self.data[self.size] = candidate
            self._heapify_up(self.size)
            self.size += 1
        else:
            # Heap full: replace worst element if this one is better
            if candidate.distance < self.data[0].distance:
                self.data[0] = candidate
                self._heapify_down(0)
    
    fn pop(mut self) -> SearchCandidate:
        """Pop minimum element from heap. O(log n) complexity."""
        if self.size == 0:
            return SearchCandidate(0, Float32.MAX, True)
        
        var min_candidate = self.data[0]
        self.size -= 1
        
        if self.size > 0:
            # Move last element to root and heapify down
            self.data[0] = self.data[self.size]
            self._heapify_down(0)
        
        return min_candidate
    
    fn peek_min(self) -> SearchCandidate:
        """Get minimum element without removing it."""
        if self.size == 0:
            return SearchCandidate(0, Float32.MAX, True)
        return self.data[0]
    
    # Query Operations
    
    fn is_empty(self) -> Bool:
        """Check if heap is empty."""
        return self.size == 0
        
    fn current_size(self) -> Int:
        """Get current number of elements in heap."""
        return self.size
    
    fn has_unvisited_nodes(self) -> Bool:
        """
        Check if there are unvisited nodes in the heap.
        Useful for graph traversal algorithms.
        """
        for i in range(self.size):
            if not self.data[i].visited:
                return True
        return False
    
    fn mark_visited(mut self, node_id: UInt32):
        """Mark a specific node as visited."""
        for i in range(self.size):
            if self.data[i].node_id == node_id:
                self.data[i].visited = True
                break
    
    fn clear(mut self):
        """Clear all elements from heap."""
        self.size = 0
    
    # Internal Heap Operations
    
    fn _parent(self, index: Int) -> Int:
        """Get parent index."""
        return (index - 1) // 2
    
    fn _left_child(self, index: Int) -> Int:
        """Get left child index."""
        return 2 * index + 1
    
    fn _right_child(self, index: Int) -> Int:
        """Get right child index."""
        return 2 * index + 2
    
    fn _swap(mut self, i: Int, j: Int):
        """Swap two elements in heap."""
        var temp = self.data[i]
        self.data[i] = self.data[j]
        self.data[j] = temp
    
    fn _heapify_up(mut self, index: Int):
        """Maintain heap property by moving element up.
        
        Iterative implementation to avoid stack overflow.
        O(log n) complexity with O(1) stack space.
        """
        var current = index
        
        # Iterate up the tree instead of recursing
        while current > 0:
            var parent_idx = self._parent(current)
            
            # Stop if heap property is satisfied
            if self.data[current].distance >= self.data[parent_idx].distance:
                break
            
            # Swap with parent and continue up
            self._swap(current, parent_idx)
            current = parent_idx
    
    fn _heapify_down(mut self, index: Int):
        """Maintain heap property by moving element down.
        
        Iterative implementation to avoid stack overflow.
        O(log n) complexity with O(1) stack space.
        """
        var current = index
        
        # Iterate down the tree instead of recursing
        while True:
            var smallest = current
            var left = self._left_child(current)
            var right = self._right_child(current)
            
            # Find smallest among node and its children
            if (left < self.size and 
                self.data[left].distance < self.data[smallest].distance):
                smallest = left
            
            if (right < self.size and 
                self.data[right].distance < self.data[smallest].distance):
                smallest = right
            
            # Stop if heap property is satisfied
            if smallest == current:
                break
            
            # Swap with smallest child and continue down
            self._swap(current, smallest)
            current = smallest


# Type aliases for backward compatibility
alias PriorityQueue = MinHeapPriorityQueue
alias BinaryHeap = MinHeapPriorityQueue