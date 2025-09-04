"""
Test buffer overhead vs direct operations.

This tests the fundamental question: Is buffering actually faster?
We simulate the cost of graph updates to see if batching helps.
"""

import time
import numpy as np


class GraphUpdateSimulator:
    """Simulates the cost of graph updates."""

    def __init__(self, update_cost_ms=0.1):
        """
        Args:
            update_cost_ms: Simulated cost of updating graph per vector
        """
        self.update_cost_ms = update_cost_ms
        self.graph_size = 0
        self.total_updates = 0

    def direct_insert(self, vector):
        """Simulate direct insertion with immediate graph update."""
        # Simulate graph navigation and update
        time.sleep(self.update_cost_ms / 1000)
        self.graph_size += 1
        self.total_updates += 1

        # As graph grows, updates get slightly more expensive
        self.update_cost_ms *= 1.00001  # Tiny growth factor

    def batch_insert(self, vectors):
        """Simulate batch insertion with optimized graph building."""
        n = len(vectors)

        # Batch building is more efficient (can optimize connections)
        # Typically O(n log n) instead of n * O(log n)
        batch_cost = self.update_cost_ms * n * 0.3  # 70% savings
        time.sleep(batch_cost / 1000)

        self.graph_size += n
        self.total_updates += 1  # One batch update


def test_direct_vs_buffered(n_vectors=10000, buffer_size=1000):
    """Compare direct insertion vs buffered insertion."""

    print(f"\n🧪 Testing {n_vectors} vectors")
    print(f"   Buffer size: {buffer_size}")
    print("=" * 50)

    # Test direct insertion
    print("\n1️⃣  DIRECT INSERTION (no buffer)")
    direct_sim = GraphUpdateSimulator(update_cost_ms=0.1)

    start = time.perf_counter()
    for i in range(n_vectors):
        direct_sim.direct_insert(i)  # Simulate vector
    direct_time = time.perf_counter() - start

    print(f"   Time: {direct_time:.2f}s")
    print(f"   Rate: {n_vectors / direct_time:.0f} vec/s")
    print(f"   Graph updates: {direct_sim.total_updates}")

    # Test buffered insertion
    print("\n2️⃣  BUFFERED INSERTION")
    buffered_sim = GraphUpdateSimulator(update_cost_ms=0.1)
    buffer = []

    start = time.perf_counter()
    for i in range(n_vectors):
        buffer.append(i)

        # Flush when buffer full
        if len(buffer) >= buffer_size:
            buffered_sim.batch_insert(buffer)
            buffer = []

    # Final flush
    if buffer:
        buffered_sim.batch_insert(buffer)

    buffered_time = time.perf_counter() - start

    print(f"   Time: {buffered_time:.2f}s")
    print(f"   Rate: {n_vectors / buffered_time:.0f} vec/s")
    print(f"   Graph updates: {buffered_sim.total_updates}")

    # Analysis
    print("\n📊 ANALYSIS")
    speedup = direct_time / buffered_time
    update_reduction = direct_sim.total_updates / buffered_sim.total_updates

    if speedup > 1:
        print(f"   ✅ Buffering is {speedup:.1f}x FASTER")
    else:
        print(f"   ❌ Direct is {1 / speedup:.1f}x faster")

    print(f"   📉 Graph updates reduced by {update_reduction:.0f}x")

    return speedup


def find_optimal_buffer_size(n_vectors=10000):
    """Find the optimal buffer size."""
    print("\n🔍 Finding optimal buffer size...")
    print("=" * 50)

    buffer_sizes = [0, 10, 100, 500, 1000, 5000, 10000]
    best_size = 0
    best_speedup = 0

    for size in buffer_sizes:
        if size == 0:
            print(f"\nNo buffer (direct insertion):")
            speedup = 1.0  # Baseline
        else:
            print(f"\nBuffer size {size}:")
            speedup = test_direct_vs_buffered(n_vectors, size)

        if speedup > best_speedup:
            best_speedup = speedup
            best_size = size

    print("\n" + "=" * 50)
    print(f"🏆 OPTIMAL BUFFER SIZE: {best_size}")
    print(f"   Speedup: {best_speedup:.1f}x")

    if best_size == 0:
        print("\n⚡ CONCLUSION: Skip the buffer! Direct insertion is fastest.")
    elif best_size < 1000:
        print(f"\n⚡ CONCLUSION: Small buffer ({best_size}) is optimal.")
    else:
        print(f"\n⚡ CONCLUSION: Large buffer ({best_size}) provides best performance.")


def simulate_real_workload():
    """Simulate a realistic streaming workload."""
    print("\n🌊 STREAMING WORKLOAD SIMULATION")
    print("=" * 50)
    print("Simulating real-time vector stream with bursts...")

    # Simulate bursty traffic
    burst_sizes = [1, 1, 1, 10, 1, 1, 100, 1, 1, 1, 1000]

    direct_sim = GraphUpdateSimulator(update_cost_ms=0.1)
    buffered_sim = GraphUpdateSimulator(update_cost_ms=0.1)
    buffer = []
    buffer_size = 100

    direct_start = time.perf_counter()
    for burst in burst_sizes:
        for _ in range(burst):
            direct_sim.direct_insert(1)
    direct_time = time.perf_counter() - direct_start

    buffered_start = time.perf_counter()
    for burst in burst_sizes:
        for _ in range(burst):
            buffer.append(1)
            if len(buffer) >= buffer_size:
                buffered_sim.batch_insert(buffer)
                buffer = []
    if buffer:
        buffered_sim.batch_insert(buffer)
    buffered_time = time.perf_counter() - buffered_start

    total_vectors = sum(burst_sizes)
    print(f"\nProcessed {total_vectors} vectors in bursts: {burst_sizes}")
    print(f"\nDirect: {direct_time:.3f}s ({total_vectors / direct_time:.0f} vec/s)")
    print(f"Buffered: {buffered_time:.3f}s ({total_vectors / buffered_time:.0f} vec/s)")

    if buffered_time < direct_time:
        print(f"\n✅ Buffer handles bursts {direct_time / buffered_time:.1f}x better!")
    else:
        print(f"\n❌ Direct handles bursts {buffered_time / direct_time:.1f}x better!")


def main():
    print("\n" + "=" * 60)
    print("🔬 BUFFER PERFORMANCE ANALYSIS FOR DISKANN")
    print("=" * 60)

    # Basic comparison
    test_direct_vs_buffered(10000, 1000)

    # Find optimal size
    # find_optimal_buffer_size(10000)

    # Realistic workload
    simulate_real_workload()

    print("\n" + "=" * 60)
    print("💡 KEY INSIGHTS:")
    print("=" * 60)
    print("""
    1. Buffering reduces graph update frequency
    2. Batch operations can optimize graph construction
    3. BUT: Adds complexity and search overhead
    4. Optimal depends on workload pattern
    
    For DiskANN specifically:
    - Each insert updates ~64 nodes (degree R)
    - Robust pruning has overhead
    - Buffer amortizes this cost
    """)


if __name__ == "__main__":
    main()
