#!/usr/bin/env python3
"""
Analyze ultra-low latency requirements and our capabilities.
"""

import sys
import os
import time
import numpy as np

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "python"))

import omendb


def analyze_current_latency_distribution():
    """Measure current query latency distribution in detail."""
    print("📊 Current Latency Distribution Analysis")
    print("=" * 45)

    db = omendb.DB()
    dimensions = 128

    # Add vectors to get realistic performance
    print("Setting up test database...")
    vectors = []
    for i in range(1000):  # Smaller dataset for latency focus
        vector = [float(np.random.randn()) for _ in range(dimensions)]
        vectors.append(vector)
        db.add(f"vec_{i}", vector)

    # Measure query latencies with high precision
    print("Measuring query latencies...")
    latencies = []

    for i in range(1000):  # Many measurements for distribution
        query = [float(np.random.randn()) for _ in range(dimensions)]

        start = time.perf_counter()
        results = db.search(query, limit=10)
        end = time.perf_counter()

        latency_us = (end - start) * 1_000_000  # Convert to microseconds
        latencies.append(latency_us)

    # Statistical analysis
    latencies = np.array(latencies)

    print(f"\n📈 Latency Statistics (microseconds):")
    print(f"   Mean: {np.mean(latencies):.1f}μs")
    print(f"   Median: {np.median(latencies):.1f}μs")
    print(f"   P95: {np.percentile(latencies, 95):.1f}μs")
    print(f"   P99: {np.percentile(latencies, 99):.1f}μs")
    print(f"   P99.9: {np.percentile(latencies, 99.9):.1f}μs")
    print(f"   Min: {np.min(latencies):.1f}μs")
    print(f"   Max: {np.max(latencies):.1f}μs")

    return latencies


def compare_financial_requirements():
    """Compare our performance with financial trading requirements."""
    print(f"\n💰 Financial Trading Latency Requirements")
    print("=" * 45)

    # Financial trading latency budgets (realistic 2025 numbers)
    requirements = {
        "Market Data Processing": 1_000,  # 1ms - market data ingestion
        "Signal Generation": 5_000,  # 5ms - trading signal calculation
        "Risk Check": 2_000,  # 2ms - risk management
        "Order Placement": 1_000,  # 1ms - order to exchange
        "Total Budget": 10_000,  # 10ms total for entire trade
    }

    # Our current performance
    latencies = analyze_current_latency_distribution()
    our_p99 = np.percentile(latencies, 99)

    print("\n🎯 Requirement Analysis:")
    for component, budget_us in requirements.items():
        if component == "Total Budget":
            continue

        can_meet = our_p99 < budget_us
        margin = (
            (budget_us - our_p99) / budget_us * 100
            if can_meet
            else (our_p99 - budget_us) / budget_us * 100
        )

        status = "✅ MEET" if can_meet else "❌ EXCEED"
        print(
            f"   {component}: {budget_us}μs budget, our P99: {our_p99:.0f}μs ({status}, {margin:.0f}% margin)"
        )

    # Overall assessment
    total_budget = requirements["Total Budget"]
    can_meet_total = our_p99 < total_budget

    print(f"\n🏆 Overall Assessment:")
    print(f"   Total budget: {total_budget}μs")
    print(f"   Our P99 latency: {our_p99:.0f}μs")
    print(f"   Can meet requirements: {'✅ YES' if can_meet_total else '❌ NO'}")

    if can_meet_total:
        remaining_budget = total_budget - our_p99
        print(f"   Remaining budget: {remaining_budget:.0f}μs for other components")
    else:
        excess = our_p99 - total_budget
        improvement_needed = excess / our_p99 * 100
        print(f"   Need {improvement_needed:.0f}% latency reduction to be competitive")


def theoretical_minimum_latency():
    """Calculate theoretical minimum latency based on hardware limits."""
    print(f"\n🔬 Theoretical Minimum Latency Analysis")
    print("=" * 45)

    # Hardware-based latency components (2025 state-of-the-art)
    components = {
        "Memory Access (L1 cache)": 1,  # 1ns - data already in L1
        "Memory Access (L2 cache)": 3,  # 3ns - data in L2
        "Memory Access (L3 cache)": 12,  # 12ns - data in L3
        "Memory Access (RAM)": 100,  # 100ns - DRAM access
        "SIMD Computation (128D)": 10,  # 10ns - distance calculation
        "Function Call Overhead": 2,  # 2ns - Mojo function call
        "Python/Mojo Interface": 1000,  # 1000ns - Python binding overhead
    }

    print("⚡ Hardware Latency Components:")
    total_theoretical = 0
    for component, latency_ns in components.items():
        print(f"   {component}: {latency_ns}ns")
        if (
            component != "Python/Mojo Interface"
        ):  # Exclude Python overhead for pure Mojo
            total_theoretical += latency_ns

    print(f"\n🎯 Theoretical Minimums:")
    print(
        f"   Pure Mojo (no Python): {total_theoretical}ns = {total_theoretical / 1000:.2f}μs"
    )
    print(
        f"   With Python overhead: {total_theoretical + 1000}ns = {(total_theoretical + 1000) / 1000:.2f}μs"
    )
    print(
        f"   Current actual P99: {np.percentile(analyze_current_latency_distribution(), 99):.1f}μs"
    )

    # Calculate optimization potential
    current_p99 = np.percentile(analyze_current_latency_distribution(), 99)
    theoretical_min = (total_theoretical + 1000) / 1000  # Include Python overhead

    optimization_potential = (current_p99 - theoretical_min) / current_p99 * 100
    print(
        f"   Optimization potential: {optimization_potential:.0f}% latency reduction possible"
    )


if __name__ == "__main__":
    print("⚡ Ultra-Low Latency Capability Analysis")
    print("=" * 50)

    latencies = analyze_current_latency_distribution()
    compare_financial_requirements()
    theoretical_minimum_latency()

    print(f"\n🎯 Strategic Assessment:")
    p99 = np.percentile(latencies, 99)

    if p99 < 10_000:  # 10ms total budget
        print("✅ VIABLE: We can compete in financial applications")
        print("   Focus on: Latency consistency and optimization")
    elif p99 < 50_000:  # 50ms budget for less critical applications
        print("🔄 PARTIAL: Viable for some financial use cases")
        print("   Focus on: Significant latency reduction needed")
    else:
        print("❌ NOT VIABLE: Too slow for financial applications")
        print("   Focus on: Different market segments")
