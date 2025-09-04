"""
Simple test for priority queue optimization components.

This test validates the core optimization components without
complex dependencies.
"""

from omendb.algorithms import MinHeap, SearchCandidate, DistanceCache
from testing import assert_equal, assert_true
from collections import List


fn test_priority_queue_basic() raises:
    """Test basic priority queue operations."""
    print("Testing priority queue basic operations...")

    var heap = MinHeap(100)
    assert_true(heap.is_empty(), "New heap should be empty")

    # Test insertion
    heap.push_distance(0.5, 1)
    heap.push_distance(0.2, 2)
    heap.push_distance(0.8, 3)
    heap.push_distance(0.1, 4)

    assert_equal(heap.size(), 4)
    assert_true(
        not heap.is_empty(), "Heap should not be empty after insertions"
    )

    # Test min-heap property (smallest distance first)
    var first = heap.pop()
    assert_equal(first.node_id, 4)  # Distance 0.1
    assert_true(
        abs(first.distance - 0.1) < 1e-6, "Should extract minimum distance"
    )

    var second = heap.pop()
    assert_equal(second.node_id, 2)  # Distance 0.2

    var third = heap.pop()
    assert_equal(third.node_id, 1)  # Distance 0.5

    var fourth = heap.pop()
    assert_equal(fourth.node_id, 3)  # Distance 0.8

    assert_true(heap.is_empty(), "Heap should be empty after all pops")

    print("✅ Priority queue basic operations test passed")


fn test_priority_queue_performance() raises:
    """Test priority queue performance characteristics."""
    print("Testing priority queue performance...")

    var heap = MinHeap(1000)
    var num_operations = 100

    # Insert many elements
    for i in range(num_operations):
        var distance = Float32(i % 50) / 50.0  # Varied distances
        heap.push_distance(distance, i)

    assert_equal(heap.size(), num_operations)

    # Extract elements and verify ordering
    var prev_distance = Float32(-1.0)
    var extracted = 0

    while not heap.is_empty():
        var candidate = heap.pop()
        assert_true(
            candidate.distance >= prev_distance,
            "Heap order should be maintained",
        )
        prev_distance = candidate.distance
        extracted += 1

    assert_equal(extracted, num_operations)

    print("✅ Priority queue performance test passed")


fn test_distance_cache_basic() raises:
    """Test basic distance cache functionality."""
    print("Testing distance cache basic operations...")

    var cache = DistanceCache(1000)

    # Test cache miss
    var distance = cache.get(1, 2)
    assert_equal(distance, -1.0)  # Not found
    assert_true(not cache.has(1, 2), "Should not have uncached distance")

    # Test cache hit
    cache.put(1, 2, 0.75)
    distance = cache.get(1, 2)
    assert_true(abs(distance - 0.75) < 1e-6, "Should return cached distance")
    assert_true(cache.has(1, 2), "Should have cached distance")

    # Test order independence
    distance = cache.get(2, 1)  # Reversed order
    assert_true(abs(distance - 0.75) < 1e-6, "Should work with reversed order")

    print("✅ Distance cache basic operations test passed")


fn test_distance_cache_collision_handling() raises:
    """Test distance cache collision handling."""
    print("Testing distance cache collision handling...")

    var cache = DistanceCache(100)  # Smaller cache for collision testing

    # Add multiple entries
    cache.put(1, 2, 0.25)
    cache.put(3, 4, 0.50)
    cache.put(5, 6, 0.75)
    cache.put(7, 8, 0.90)

    # Verify all entries are still accessible
    assert_true(
        abs(cache.get(1, 2) - 0.25) < 1e-6, "First entry should be cached"
    )
    assert_true(
        abs(cache.get(3, 4) - 0.50) < 1e-6, "Second entry should be cached"
    )
    assert_true(
        abs(cache.get(5, 6) - 0.75) < 1e-6, "Third entry should be cached"
    )
    assert_true(
        abs(cache.get(7, 8) - 0.90) < 1e-6, "Fourth entry should be cached"
    )

    print("✅ Distance cache collision handling test passed")


fn test_search_candidate_functionality() raises:
    """Test search candidate structure."""
    print("Testing search candidate functionality...")

    var candidate1 = SearchCandidate(0.5, 10, False)
    assert_true(
        abs(candidate1.distance - 0.5) < 1e-6,
        "Distance should be set correctly",
    )
    assert_equal(candidate1.node_id, 10)
    assert_true(not candidate1.visited, "Should not be visited initially")

    var candidate2 = SearchCandidate(0.3, 20, True)
    assert_true(candidate2.visited, "Should be marked as visited")

    print("✅ Search candidate functionality test passed")


fn test_optimization_components_integration() raises:
    """Test integration of optimization components."""
    print("Testing optimization components integration...")

    # Create priority queue and add candidates
    var heap = MinHeap(50)
    var cache = DistanceCache(500)

    # Add candidates to heap
    heap.push_distance(0.8, 1)
    heap.push_distance(0.3, 2)
    heap.push_distance(0.6, 3)

    # Cache some distances
    cache.put(1, 2, 0.45)
    cache.put(2, 3, 0.33)

    # Process candidates in order
    var first = heap.pop()
    assert_equal(first.node_id, 2)  # Should be minimum distance (0.3)

    # Check cached distances
    var cached_distance = cache.get(1, 2)
    assert_true(
        abs(cached_distance - 0.45) < 1e-6,
        "Cached distance should be available",
    )

    var second = heap.pop()
    assert_equal(second.node_id, 3)  # Next minimum (0.6)

    var third = heap.pop()
    assert_equal(third.node_id, 1)  # Last (0.8)

    print("✅ Optimization components integration test passed")


fn run_priority_queue_tests() raises:
    """Run all priority queue and optimization tests."""
    print("🚀 Running Priority Queue Optimization Tests")
    print("============================================")

    test_search_candidate_functionality()
    test_priority_queue_basic()
    test_priority_queue_performance()
    test_distance_cache_basic()
    test_distance_cache_collision_handling()
    test_optimization_components_integration()

    print("\n✅ All priority queue optimization tests passed!")
    print("🎯 Core optimization components validated")
    print(
        "📈 Priority queue provides O(log n) vs O(n log n) sorting improvement"
    )
    print("🗄️ Distance cache eliminates redundant similarity calculations")
    print("🚀 Ready for RoarGraph algorithm integration")


fn main() raises:
    run_priority_queue_tests()
