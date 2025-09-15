"""
High-performance min-heap priority queue optimized for HNSW search.

Replaces O(n²) candidate queue operations with O(log n) heap operations.
This is the critical optimization for graph traversal performance.
"""

from memory import UnsafePointer

struct SearchCandidate(Copyable, Movable):
    """Candidate node for HNSW search with distance."""
    var distance: Float32
    var node_id: Int

    fn __init__(out self, distance: Float32, node_id: Int):
        self.distance = distance
        self.node_id = node_id

    fn __copyinit__(out self, existing: Self):
        self.distance = existing.distance
        self.node_id = existing.node_id

    fn __moveinit__(out self, owned existing: Self):
        self.distance = existing.distance
        self.node_id = existing.node_id

struct FastMinHeap(Copyable, Movable):
    """
    Optimized min-heap for HNSW candidate queue.

    PERFORMANCE CRITICAL: Replaces O(n²) operations with O(log n):
    - find_min_idx(): O(n) -> O(1)
    - remove_min(): O(n) -> O(log n)
    - add(): O(n) -> O(log n)

    Expected speedup: 10-50x for typical ef values (16-64)
    """
    var heap: UnsafePointer[SearchCandidate]
    var capacity: Int
    var size: Int

    fn __init__(out self, initial_capacity: Int = 128):
        """Initialize with pre-allocated capacity to avoid reallocations."""
        self.capacity = initial_capacity
        self.size = 0
        self.heap = UnsafePointer[SearchCandidate].alloc(initial_capacity)

        # Initialize with sentinel values to avoid uninitialized memory
        for i in range(initial_capacity):
            self.heap[i] = SearchCandidate(Float32.MAX, -1)

    fn __copyinit__(out self, existing: Self):
        """Copy constructor."""
        self.capacity = existing.capacity
        self.size = existing.size
        self.heap = UnsafePointer[SearchCandidate].alloc(self.capacity)

        # Copy all elements
        for i in range(self.size):
            self.heap[i] = existing.heap[i]

    fn __moveinit__(out self, owned existing: Self):
        """Move constructor."""
        self.capacity = existing.capacity
        self.size = existing.size
        self.heap = existing.heap
        existing.heap = UnsafePointer[SearchCandidate]()

    fn __del__(owned self):
        """Free allocated memory."""
        if self.heap:
            self.heap.free()

    @always_inline
    fn parent(self, i: Int) -> Int:
        """Get parent index."""
        return (i - 1) // 2

    @always_inline
    fn left_child(self, i: Int) -> Int:
        """Get left child index."""
        return 2 * i + 1

    @always_inline
    fn right_child(self, i: Int) -> Int:
        """Get right child index."""
        return 2 * i + 2

    @always_inline
    fn len(self) -> Int:
        """Get current size."""
        return self.size

    @always_inline
    fn is_empty(self) -> Bool:
        """Check if heap is empty."""
        return self.size == 0

    @always_inline
    fn peek_min(self) -> SearchCandidate:
        """Get minimum element without removing it. O(1)"""
        if self.size > 0:
            return self.heap[0]
        return SearchCandidate(Float32.MAX, -1)

    fn add(mut self, distance: Float32, node_id: Int):
        """Add element to heap. O(log n)"""
        if self.size >= self.capacity:
            self._resize()

        # Add at end and bubble up
        var idx = self.size
        self.heap[idx] = SearchCandidate(distance, node_id)
        self.size += 1

        # Bubble up to maintain heap property
        self._bubble_up(idx)

    fn extract_min(mut self) -> SearchCandidate:
        """Remove and return minimum element. O(log n)"""
        if self.size == 0:
            return SearchCandidate(Float32.MAX, -1)

        var min_element = self.heap[0]

        # Move last element to root and bubble down
        self.size -= 1
        if self.size > 0:
            self.heap[0] = self.heap[self.size]
            var root_idx = 0
            self._bubble_down(root_idx)

        return min_element

    fn clear(mut self):
        """Clear all elements. O(1)"""
        self.size = 0

    # CRITICAL: These are the optimized replacements for O(n²) operations

    @always_inline
    fn find_min_idx(self) -> Int:
        """OPTIMIZED: O(1) instead of O(n). Always returns 0 for min-heap."""
        return 0 if self.size > 0 else -1

    @always_inline
    fn get_distance(self, idx: Int) -> Float32:
        """Get distance at index. O(1)"""
        if idx == 0 and self.size > 0:
            return self.heap[0].distance
        return Float32.MAX

    @always_inline
    fn get_node_id(self, idx: Int) -> Int:
        """Get node ID at index. O(1)"""
        if idx == 0 and self.size > 0:
            return self.heap[0].node_id
        return -1

    fn remove_at(mut self, idx: Int) -> Bool:
        """OPTIMIZED: Remove element at index. O(log n) for idx=0."""
        if idx == 0:
            _ = self.extract_min()
            return True
        return False

    # Private helper methods

    fn _bubble_up(mut self, mut idx: Int):
        """Bubble element up to maintain heap property."""
        while idx > 0:
            var parent_idx = self.parent(idx)
            if self.heap[idx].distance >= self.heap[parent_idx].distance:
                break

            # Swap with parent
            var temp = self.heap[idx]
            self.heap[idx] = self.heap[parent_idx]
            self.heap[parent_idx] = temp

            idx = parent_idx

    fn _bubble_down(mut self, mut idx: Int):
        """Bubble element down to maintain heap property."""
        while True:
            var left = self.left_child(idx)
            var right = self.right_child(idx)
            var smallest = idx

            # Find smallest among node and its children
            if left < self.size and self.heap[left].distance < self.heap[smallest].distance:
                smallest = left

            if right < self.size and self.heap[right].distance < self.heap[smallest].distance:
                smallest = right

            if smallest == idx:
                break

            # Swap with smallest child
            var temp = self.heap[idx]
            self.heap[idx] = self.heap[smallest]
            self.heap[smallest] = temp

            idx = smallest

    fn _resize(mut self):
        """Resize heap when capacity is exceeded."""
        var new_capacity = self.capacity * 2
        var new_heap = UnsafePointer[SearchCandidate].alloc(new_capacity)

        # Copy existing elements
        for i in range(self.size):
            new_heap[i] = self.heap[i]

        # Initialize new elements
        for i in range(self.size, new_capacity):
            new_heap[i] = SearchCandidate(Float32.MAX, -1)

        # Free old memory and update
        self.heap.free()
        self.heap = new_heap
        self.capacity = new_capacity

struct FastMaxHeap(Copyable, Movable):
    """
    Optimized max-heap for HNSW result pool (W).

    Maintains top-k smallest elements by evicting largest.
    Optimized for replace_furthest() operation.
    """
    var heap: UnsafePointer[SearchCandidate]
    var capacity: Int
    var size: Int
    var max_size: Int  # Fixed capacity for top-k

    fn __init__(out self, max_capacity: Int):
        """Initialize with fixed capacity for top-k results."""
        self.capacity = max_capacity
        self.max_size = max_capacity
        self.size = 0
        self.heap = UnsafePointer[SearchCandidate].alloc(max_capacity)

        # Initialize with sentinel values
        for i in range(max_capacity):
            self.heap[i] = SearchCandidate(0.0, -1)

    fn __copyinit__(out self, existing: Self):
        """Copy constructor."""
        self.capacity = existing.capacity
        self.max_size = existing.max_size
        self.size = existing.size
        self.heap = UnsafePointer[SearchCandidate].alloc(self.capacity)

        for i in range(self.size):
            self.heap[i] = existing.heap[i]

    fn __moveinit__(out self, owned existing: Self):
        """Move constructor."""
        self.capacity = existing.capacity
        self.max_size = existing.max_size
        self.size = existing.size
        self.heap = existing.heap
        existing.heap = UnsafePointer[SearchCandidate]()

    fn __del__(owned self):
        """Free allocated memory."""
        if self.heap:
            self.heap.free()

    @always_inline
    fn len(self) -> Int:
        """Get current size."""
        return self.size

    fn add(mut self, distance: Float32, node_id: Int):
        """Add element to max-heap. O(log n)"""
        if self.size < self.max_size:
            # Add normally
            var idx = self.size
            self.heap[idx] = SearchCandidate(distance, node_id)
            self.size += 1
            self._bubble_up_max(idx)
        else:
            # Replace maximum if this is smaller
            if distance < self.heap[0].distance:
                self.heap[0] = SearchCandidate(distance, node_id)
                var root_idx = 0
                self._bubble_down_max(root_idx)

    fn replace_furthest(mut self, distance: Float32, node_id: Int) -> Bool:
        """OPTIMIZED: Replace furthest (maximum) element if distance is closer."""
        if self.size < self.max_size:
            self.add(distance, node_id)
            return True
        elif distance < self.heap[0].distance:
            self.heap[0] = SearchCandidate(distance, node_id)
            var root_idx = 0
            self._bubble_down_max(root_idx)
            return True
        return False

    @always_inline
    fn get_node_id(self, idx: Int) -> Int:
        """Get node ID at index. O(1)"""
        if idx < self.size:
            return self.heap[idx].node_id
        return -1

    @always_inline
    fn get_distance(self, idx: Int) -> Float32:
        """Get distance at index. O(1)"""
        if idx < self.size:
            return self.heap[idx].distance
        return Float32.MAX

    fn sort_by_distance(mut self):
        """Sort heap elements by distance for result extraction."""
        # Simple bubble sort for small collections (K is typically 10-100)
        for i in range(self.size):
            for j in range(self.size - 1 - i):
                if self.heap[j].distance > self.heap[j + 1].distance:
                    var temp = self.heap[j]
                    self.heap[j] = self.heap[j + 1]
                    self.heap[j + 1] = temp

    # Private helpers for max-heap operations

    fn _bubble_up_max(mut self, mut idx: Int):
        """Bubble up for max-heap (parent > children)."""
        while idx > 0:
            var parent_idx = (idx - 1) // 2
            if self.heap[idx].distance <= self.heap[parent_idx].distance:
                break

            var temp = self.heap[idx]
            self.heap[idx] = self.heap[parent_idx]
            self.heap[parent_idx] = temp

            idx = parent_idx

    fn _bubble_down_max(mut self, mut idx: Int):
        """Bubble down for max-heap (parent > children)."""
        while True:
            var left = 2 * idx + 1
            var right = 2 * idx + 2
            var largest = idx

            if left < self.size and self.heap[left].distance > self.heap[largest].distance:
                largest = left

            if right < self.size and self.heap[right].distance > self.heap[largest].distance:
                largest = right

            if largest == idx:
                break

            var temp = self.heap[idx]
            self.heap[idx] = self.heap[largest]
            self.heap[largest] = temp

            idx = largest