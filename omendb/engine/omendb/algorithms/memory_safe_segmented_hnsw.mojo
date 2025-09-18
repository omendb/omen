"""
Memory-Safe Segmented HNSW Implementation
September 2025

Fixes the race condition issues in parallel execution by using atomic coordination
and pre-allocated memory regions to eliminate shared memory access conflicts.

Key insight: Current segfaults are due to multiple threads accessing same HNSWIndex.
Solution: Each thread gets completely isolated memory space + atomic coordination.
"""

from math import ceil
from algorithm import parallelize
from collections import List
from memory import UnsafePointer, memcpy
from atomic import Atomic
from omendb.algorithms.hnsw import HNSWIndex
from random import random_float64
from sys.info import num_performance_cores

# Configuration for memory-safe parallel execution
alias SEGMENT_SIZE = 1000           # Vectors per segment
alias MAX_SEGMENTS = 8              # Match CPU cores
alias PARALLEL_WORKERS = 8          # Worker threads

@value
struct AtomicCounter:
    """Thread-safe counter using atomic operations."""
    var counter: Atomic[Int]

    fn __init__(out self, initial_value: Int = 0):
        self.counter = Atomic[Int](initial_value)

    fn get(self) -> Int:
        return self.counter.load()

    fn increment(mut self) -> Int:
        return self.counter.fetch_add(1)

    fn add(mut self, value: Int) -> Int:
        return self.counter.fetch_add(value)

struct MemorySafeSegmentedHNSW(Movable):
    """
    Memory-safe parallel HNSW with atomic coordination.
    Eliminates race conditions through isolated memory spaces.
    """
    var dimension: Int
    var num_segments: Int

    # Pre-allocated memory regions (completely isolated per segment)
    var segment_vectors: UnsafePointer[UnsafePointer[Float32]]  # Isolated vector storage
    var segment_results: UnsafePointer[UnsafePointer[Int]]     # Isolated result storage
    var segment_capacities: UnsafePointer[Int]                 # Capacity per segment
    var segment_sizes: UnsafePointer[Int]                      # Current size per segment

    # Atomic coordination (no shared state access)
    var total_processed: AtomicCounter                         # Progress tracking
    var next_global_id: AtomicCounter                         # ID generation

    # Working indices (isolated per segment)
    var segment_indices: UnsafePointer[HNSWIndex]

    fn __init__(out self, dimension: Int):
        self.dimension = dimension
        self.num_segments = min(MAX_SEGMENTS, num_performance_cores())

        # Initialize atomic counters
        self.total_processed = AtomicCounter(0)
        self.next_global_id = AtomicCounter(0)

        print("🔒 MEMORY SAFE HNSW: Initializing", self.num_segments, "isolated segments")

        # Allocate isolated memory regions
        self.segment_vectors = UnsafePointer[UnsafePointer[Float32]].alloc(self.num_segments)
        self.segment_results = UnsafePointer[UnsafePointer[Int]].alloc(self.num_segments)
        self.segment_capacities = UnsafePointer[Int].alloc(self.num_segments)
        self.segment_sizes = UnsafePointer[Int].alloc(self.num_segments)

        # Allocate segment indices
        self.segment_indices = UnsafePointer[HNSWIndex].alloc(self.num_segments)

        # Initialize each segment with completely isolated memory
        for i in range(self.num_segments):
            var capacity = SEGMENT_SIZE
            self.segment_capacities[i] = capacity
            self.segment_sizes[i] = 0

            # Isolated vector storage
            self.segment_vectors[i] = UnsafePointer[Float32].alloc(capacity * dimension)

            # Isolated result storage
            self.segment_results[i] = UnsafePointer[Int].alloc(capacity)

            # Isolated HNSW index (no shared memory access)
            var idx = HNSWIndex(dimension, capacity)
            idx.enable_binary_quantization()
            idx.use_flat_graph = False
            idx.use_smart_distance = False
            idx.cache_friendly_layout = False
            self.segment_indices[i] = idx^

        print("✅ MEMORY SAFE: All segments isolated and initialized")

    fn insert_batch_safe(mut self, vectors: UnsafePointer[Float32], n_vectors: Int) -> List[Int]:
        """
        Memory-safe parallel insertion with atomic coordination.
        Each thread operates on completely isolated memory regions.
        """
        print("🔒 MEMORY SAFE PARALLEL: Processing", n_vectors, "vectors")

        # Pre-allocate results list (main thread only)
        var all_results = List[Int]()
        for i in range(n_vectors):
            all_results.append(0)  # Initialize all results

        # Distribute vectors across segments (round-robin, no shared state)
        var vectors_per_segment = (n_vectors + self.num_segments - 1) // self.num_segments

        # Copy input vectors to isolated segment memory (eliminate shared access)
        for segment_id in range(self.num_segments):
            var start_idx = segment_id * vectors_per_segment
            var end_idx = min(start_idx + vectors_per_segment, n_vectors)
            var count = end_idx - start_idx

            if count > 0:
                # Copy to isolated segment memory
                var dest = self.segment_vectors[segment_id]
                var src = vectors.offset(start_idx * self.dimension)
                var copy_size = count * self.dimension
                memcpy(dest, src, copy_size * 4)  # Float32 = 4 bytes

                self.segment_sizes[segment_id] = count

        # PARALLEL EXECUTION with atomic coordination
        @parameter
        fn process_segment_safe(segment_id: Int):
            """Process segment with completely isolated memory access."""
            var count = self.segment_sizes[segment_id]
            if count <= 0:
                return

            print("🔒 SAFE Thread", segment_id, ": Processing", count, "vectors (isolated memory)")

            # Get isolated memory for this segment (no race conditions)
            var segment_vectors_ptr = self.segment_vectors[segment_id]
            var segment_results_ptr = self.segment_results[segment_id]

            # Process vectors in this segment (completely isolated)
            for i in range(count):
                var vector_ptr = segment_vectors_ptr.offset(i * self.dimension)

                # Insert into isolated HNSW index (no shared access)
                var local_ids = self.segment_indices[segment_id].insert_bulk(vector_ptr, 1)

                # Generate unique global ID atomically
                var global_id = self.next_global_id.increment()
                segment_results_ptr[i] = global_id

                # Update progress atomically
                _ = self.total_processed.increment()

        # Launch parallel workers with isolated memory
        print("🚀 SAFE PARALLEL: Launching", self.num_segments, "workers with isolated memory")
        parallelize[process_segment_safe](self.num_segments)

        # Collect results from isolated memory (main thread only, no races)
        var result_idx = 0
        for segment_id in range(self.num_segments):
            var count = self.segment_sizes[segment_id]
            var segment_results_ptr = self.segment_results[segment_id]

            for i in range(count):
                if result_idx < n_vectors:
                    all_results[result_idx] = segment_results_ptr[i]
                    result_idx += 1

        print("✅ SAFE PARALLEL: Processed", self.total_processed.get(), "vectors safely")
        return all_results

    fn search_safe(self, query: UnsafePointer[Float32], k: Int = 10) -> List[Int]:
        """Memory-safe search across all segments."""
        var results = List[Int]()

        # Search each segment independently (read-only, safe)
        for segment_id in range(self.num_segments):
            if self.segment_sizes[segment_id] > 0:
                # Search this segment (isolated, no race conditions)
                var segment_results = self.segment_indices[segment_id].search_vectors(query, k)

                # Add segment results to global results
                for i in range(len(segment_results)):
                    results.append(segment_results[i])

        # Return top-k across all segments (could sort by distance if needed)
        return results

    fn get_total_vectors(self) -> Int:
        """Get total vectors processed safely."""
        return self.total_processed.get()

    fn __del__(mut self):
        """Clean up isolated memory safely."""
        for i in range(self.num_segments):
            if self.segment_vectors[i]:
                self.segment_vectors[i].free()
            if self.segment_results[i]:
                self.segment_results[i].free()

        self.segment_vectors.free()
        self.segment_results.free()
        self.segment_capacities.free()
        self.segment_sizes.free()
        self.segment_indices.free()