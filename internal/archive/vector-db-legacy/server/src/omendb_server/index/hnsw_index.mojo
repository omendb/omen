"""
Hierarchical Navigable Small World (HNSW) graph index implementation.

This module implements the HNSW graph structure for efficient approximate nearest
neighbor search, refactored for Mojo 25.5.0 compatibility.
"""

from collections import Dict, List, Optional
from random import random_float64
from math import log

from core.vector import Vector, Float32Vector
from core.record import VectorRecord
from util.logging import Logger, LogLevel

# Using alias for compile-time constants instead of global variables
alias DEFAULT_M: Int = 16
alias DEFAULT_EF_CONSTRUCTION: Int = 200
alias DEFAULT_EF: Int = 50

# Helper struct for search results
struct DistanceResult(Copyable, Movable):
    """Result of a distance calculation, used in priority queues."""
    var id: String
    var distance: Float64

    fn __init__(out self, id: String, distance: Float64):
        self.id = id
        self.distance = distance

    fn __copyinit__(out self, other: Self):
        self.id = other.id
        self.distance = other.distance

    fn __moveinit__(out self, owned existing: Self):
        self.id = existing.id^
        self.distance = existing.distance

    fn __lt__(self, other: Self) -> Bool:
        return self.distance < other.distance

    fn __gt__(self, other: Self) -> Bool:
        return self.distance > other.distance

# Min-heap implementation for candidate selection
struct MinHeap(Copyable, Movable):
    """A simple min-heap priority queue for managing search candidates."""
    var heap: List[DistanceResult]

    fn __init__(out self):
        self.heap = List[DistanceResult]()

    fn __copyinit__(out self, other: Self):
        self.heap = other.heap

    fn __moveinit__(out self, owned existing: Self):
        self.heap = existing.heap^

    fn size(self) -> Int:
        return len(self.heap)

    fn is_empty(self) -> Bool:
        return self.size() == 0

    fn push(mut self, item: DistanceResult):
        self.heap.append(item)
        self._sift_up(self.size() - 1)

    fn pop(mut self) raises -> DistanceResult:
        if self.is_empty():
            raise Error("Cannot pop from an empty heap")

        var last_idx = self.size() - 1
        # Manual swap since List doesn't have swap method
        var temp = self.heap[0]
        self.heap[0] = self.heap[last_idx]
        self.heap[last_idx] = temp

        var item = self.heap.pop()
        if not self.is_empty():
            self._sift_down(0)
        return item

    fn peek(self) raises -> DistanceResult:
        if self.is_empty():
            raise Error("Cannot peek into an empty heap")
        return self.heap[0]

    fn _parent(self, i: Int) -> Int:
        return (i - 1) // 2

    fn _left_child(self, i: Int) -> Int:
        return 2 * i + 1

    fn _right_child(self, i: Int) -> Int:
        return 2 * i + 2

    fn _sift_up(mut self, i: Int):
        var current_idx = i
        while current_idx > 0 and self.heap[current_idx] < self.heap[self._parent(current_idx)]:
            var parent_idx = self._parent(current_idx)
            # Manual swap since List doesn't have swap method
            var temp = self.heap[current_idx]
            self.heap[current_idx] = self.heap[parent_idx]
            self.heap[parent_idx] = temp
            current_idx = parent_idx

    fn _sift_down(mut self, i: Int):
        var min_index = i
        var left = self._left_child(i)
        if left < self.size() and self.heap[left] < self.heap[min_index]:
            min_index = left

        var right = self._right_child(i)
        if right < self.size() and self.heap[right] < self.heap[min_index]:
            min_index = right

        if i != min_index:
            # Manual swap since List doesn't have swap method
            var temp = self.heap[i]
            self.heap[i] = self.heap[min_index]
            self.heap[min_index] = temp
            self._sift_down(min_index)

# Max-heap implementation for result selection
struct MaxHeap(Copyable, Movable):
    """A simple max-heap priority queue for managing search results."""
    var heap: List[DistanceResult]

    fn __init__(out self):
        self.heap = List[DistanceResult]()

    fn __copyinit__(out self, other: Self):
        self.heap = other.heap

    fn __moveinit__(out self, owned existing: Self):
        self.heap = existing.heap^

    fn size(self) -> Int:
        return len(self.heap)

    fn is_empty(self) -> Bool:
        return self.size() == 0

    fn push(mut self, item: DistanceResult):
        self.heap.append(item)
        self._sift_up(self.size() - 1)

    fn pop(mut self) raises -> DistanceResult:
        if self.is_empty():
            raise Error("Cannot pop from an empty heap")

        var last_idx = self.size() - 1
        # Manual swap since List doesn't have swap method
        var temp = self.heap[0]
        self.heap[0] = self.heap[last_idx]
        self.heap[last_idx] = temp

        var item = self.heap.pop()
        if not self.is_empty():
            self._sift_down(0)
        return item

    fn peek(self) raises -> DistanceResult:
        if self.is_empty():
            raise Error("Cannot peek into an empty heap")
        return self.heap[0]

    fn to_sorted_list(self) raises -> List[DistanceResult]:
        """Returns the elements of the heap sorted in descending order."""
        var sorted_list = List[DistanceResult]()
        var temp_heap = self
        while not temp_heap.is_empty():
            sorted_list.append(temp_heap.pop())
        return sorted_list

    fn _parent(self, i: Int) -> Int:
        return (i - 1) // 2

    fn _left_child(self, i: Int) -> Int:
        return 2 * i + 1

    fn _right_child(self, i: Int) -> Int:
        return 2 * i + 2

    fn _sift_up(mut self, i: Int):
        var current_idx = i
        while current_idx > 0 and self.heap[current_idx] > self.heap[self._parent(current_idx)]:
            var parent_idx = self._parent(current_idx)
            # Manual swap since List doesn't have swap method
            var temp = self.heap[current_idx]
            self.heap[current_idx] = self.heap[parent_idx]
            self.heap[parent_idx] = temp
            current_idx = parent_idx

    fn _sift_down(mut self, i: Int):
        var max_index = i
        var left = self._left_child(i)
        if left < self.size() and self.heap[left] > self.heap[max_index]:
            max_index = left

        var right = self._right_child(i)
        if right < self.size() and self.heap[right] > self.heap[max_index]:
            max_index = right

        if i != max_index:
            # Manual swap since List doesn't have swap method
            var temp = self.heap[i]
            self.heap[i] = self.heap[max_index]
            self.heap[max_index] = temp
            self._sift_down(max_index)

struct HnswNode(Copyable, Movable):
    """A node in the HNSW graph."""
    var id: String
    var vector: Float32Vector
    var level: Int
    var connections: Dict[Int, List[String]]

    fn __init__(out self, id: String, vector: Float32Vector, level: Int):
        self.id = id
        self.vector = vector
        self.level = level
        self.connections = Dict[Int, List[String]]()
        for i in range(level + 1):
            self.connections[i] = List[String]()

    fn __copyinit__(out self, other: Self):
        self.id = other.id
        self.vector = other.vector
        self.level = other.level
        self.connections = other.connections

    fn __moveinit__(out self, owned existing: Self):
        self.id = existing.id^
        self.vector = existing.vector^
        self.level = existing.level
        self.connections = existing.connections^

struct HnswIndex(Copyable, Movable):
    """HNSW graph implementation, refactored for Mojo 25.5.0."""
    var m: Int
    var m_max_0: Int
    var ef_construction: Int
    var level_mult: Float64
    var nodes: Dict[String, HnswNode]
    var entry_point: Optional[String]
    var max_level: Int
    var logger: Logger

    fn __init__(out self, m: Int = DEFAULT_M, ef_construction: Int = DEFAULT_EF_CONSTRUCTION):
        self.m = m
        self.m_max_0 = 2 * m
        self.ef_construction = ef_construction
        self.level_mult = 1.0 / log(Float64(m))
        self.nodes = Dict[String, HnswNode]()
        self.entry_point = Optional[String]()
        self.max_level = -1
        self.logger = Logger(0)  # Use Int instead of String for logger ID

    fn __copyinit__(out self, other: Self):
        self.m = other.m
        self.m_max_0 = other.m_max_0
        self.ef_construction = other.ef_construction
        self.level_mult = other.level_mult
        self.nodes = other.nodes
        self.entry_point = other.entry_point
        self.max_level = other.max_level
        self.logger = Logger(0)  # Create new logger since Logger is not copyable

    fn __moveinit__(out self, owned existing: Self):
        self.m = existing.m
        self.m_max_0 = existing.m_max_0
        self.ef_construction = existing.ef_construction
        self.level_mult = existing.level_mult
        self.nodes = existing.nodes^
        self.entry_point = existing.entry_point^
        self.max_level = existing.max_level
        self.logger = Logger(0)  # Create new logger since Logger is not movable

    fn _get_random_level(self) -> Int:
        return Int(-log(random_float64(0, 1)) * self.level_mult)

    fn _distance(self, v1: Float32Vector, v2: Float32Vector) raises -> Float64:
        return v1.euclidean_distance(v2)

    fn _search_layer(self, query: Float32Vector, entry_point_id: String, ef: Int, level: Int) raises -> MaxHeap:
        var visited = Dict[String, Bool]()
        visited[entry_point_id] = True

        var entry_node = self.nodes[entry_point_id]
        var dist = self._distance(query, entry_node.vector)

        var candidates = MinHeap()
        candidates.push(DistanceResult(entry_point_id, dist))

        var results = MaxHeap()
        results.push(DistanceResult(entry_point_id, dist))

        while not candidates.is_empty():
            var current = candidates.pop()
            var furthest_result = results.peek()

            if current.distance > furthest_result.distance and results.size() >= ef:
                break

            var current_node = self.nodes[current.id]
            var neighbors = current_node.connections[level]

            for neighbor_id in neighbors:
                if neighbor_id not in visited:
                    visited[neighbor_id] = True
                    var neighbor_node = self.nodes[neighbor_id]
                    var neighbor_dist = self._distance(query, neighbor_node.vector)

                    if neighbor_dist < furthest_result.distance or results.size() < ef:
                        candidates.push(DistanceResult(neighbor_id, neighbor_dist))
                        results.push(DistanceResult(neighbor_id, neighbor_dist))
                        if results.size() > ef:
                            _ = results.pop()

        return results

    fn insert(mut self, record: VectorRecord[DType.float32]) raises:
        """Insert a new vector into the HNSW index."""
        var id = record.id
        var vector = record.vector

        # Check if vector already exists
        if id in self.nodes:
            return

        var level = self._get_random_level()
        var new_node = HnswNode(id, vector, level)

        if not self.entry_point:
            self.nodes[id] = new_node
            self.entry_point = id
            self.max_level = level
            return

        var current_id = self.entry_point.value()
        var current_level = self.max_level

        # Search from top level down to level+1
        while current_level > level:
            var closest_dist = self._distance(vector, self.nodes[current_id].vector)
            var closest_id = current_id

            var current_node = self.nodes[current_id]
            var neighbors = current_node.connections[current_level]

            for neighbor_id in neighbors:
                var neighbor_node = self.nodes[neighbor_id]
                var dist = self._distance(vector, neighbor_node.vector)
                if dist < closest_dist:
                    closest_dist = dist
                    closest_id = neighbor_id

            current_id = closest_id
            current_level -= 1

        # Search and connect from level down to 0
        while current_level >= 0:
            var neighbors_heap = self._search_layer(vector, current_id, self.ef_construction, current_level)
            var m_max = self.m if current_level > 0 else self.m_max_0

            var neighbors = neighbors_heap.to_sorted_list()
            var num_neighbors = min(m_max, len(neighbors))

            # Connect new node to M closest neighbors
            for i in range(num_neighbors):
                var neighbor_id = neighbors[i].id
                new_node.connections[current_level].append(neighbor_id)

                # Add bidirectional connection
                var neighbor_node = self.nodes[neighbor_id]
                neighbor_node.connections[current_level].append(id)

                # Prune connections if necessary
                if len(neighbor_node.connections[current_level]) > m_max:
                    # Simple pruning - remove the farthest neighbor
                    var farthest_idx = 0
                    var farthest_dist = Float64(0.0)
                    var connections = neighbor_node.connections[current_level]

                    for j in range(len(connections)):
                        var conn_id = connections[j]
                        var conn_node = self.nodes[conn_id]
                        var dist = self._distance(neighbor_node.vector, conn_node.vector)
                        if dist > farthest_dist:
                            farthest_dist = dist
                            farthest_idx = j

                    # Remove the farthest connection
                    _ = neighbor_node.connections[current_level].pop(farthest_idx)

                self.nodes[neighbor_id] = neighbor_node

            current_level -= 1

        self.nodes[id] = new_node
        if level > self.max_level:
            self.max_level = level
            self.entry_point = id

    fn search(self, query: Float32Vector, k: Int, ef: Int = DEFAULT_EF) raises -> List[DistanceResult]:
        """Search for k nearest neighbors."""
        if not self.entry_point:
            return List[DistanceResult]()

        var current_id = self.entry_point.value()
        var current_level = self.max_level

        # Search from top level down to level 1
        while current_level > 0:
            var closest_dist = self._distance(query, self.nodes[current_id].vector)
            var closest_id = current_id

            var current_node = self.nodes[current_id]
            var neighbors = current_node.connections[current_level]

            for neighbor_id in neighbors:
                var neighbor_node = self.nodes[neighbor_id]
                var dist = self._distance(query, neighbor_node.vector)
                if dist < closest_dist:
                    closest_dist = dist
                    closest_id = neighbor_id

            current_id = closest_id
            current_level -= 1

        # Search level 0 with ef
        var search_ef = max(ef, k)
        var results_heap = self._search_layer(query, current_id, search_ef, 0)
        var results = results_heap.to_sorted_list()

        var final_results = List[DistanceResult]()
        var count = min(k, len(results))
        for i in range(count):
            final_results.append(results[i])

        # The list is already sorted by the heap logic, but it's max-heap so it's descending.
        # Search results should be ascending by distance. Let's reverse it.
        var reversed_results = List[DistanceResult]()
        for i in range(len(final_results)):
            reversed_results.append(final_results[len(final_results) - 1 - i])

        return reversed_results

    fn size(self) -> Int:
        return len(self.nodes)

    fn get_node(self, id: String) raises -> Optional[HnswNode]:
        if id in self.nodes:
            return Optional[HnswNode](self.nodes[id])
        return None
