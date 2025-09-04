#!/usr/bin/env python3
"""
Analyze why SIMD gains vary by dimension.
"""


def analyze_strategies():
    """Analyze SIMD strategy efficiency by dimension."""

    # Historical gains from v0.1.0
    results = {
        32: {"gain": "+70%", "baseline": 11024, "simd": 18741},
        64: {"gain": "+4.8%", "baseline": 9900, "simd": 10375},
        128: {"gain": "-3.6%", "baseline": 5500, "simd": 5301},
        256: {"gain": "+26.1%", "baseline": 2200, "simd": 2774},
    }

    # ARM NEON SIMD width = 4 floats
    SIMD_WIDTH = 4

    print("SIMD Strategy Analysis")
    print("=" * 70)

    for dim, data in sorted(results.items()):
        print(f"\n{dim}D Analysis:")
        print("-" * 40)

        # Determine strategy
        if dim <= 64:
            strategy = "Small (4x scalar unroll)"
            unroll_factor = 4
            uses_simd = False
        elif dim <= 256:
            strategy = "Medium (2x SIMD unroll)"
            unroll_factor = 2 * SIMD_WIDTH
            uses_simd = True
        else:
            strategy = "Large (cache blocking)"
            unroll_factor = SIMD_WIDTH
            uses_simd = True

        print(f"Strategy: {strategy}")
        print(
            f"Performance: {data['baseline']} → {data['simd']} vec/s ({data['gain']})"
        )

        # Analyze efficiency
        if uses_simd:
            simd_ops = dim // SIMD_WIDTH
            remainder = dim % SIMD_WIDTH
            print(f"SIMD operations: {simd_ops} ({remainder} scalar remainder)")

            if dim <= 256:  # Medium strategy
                double_ops = dim // (SIMD_WIDTH * 2)
                single_ops = (dim - double_ops * SIMD_WIDTH * 2) // SIMD_WIDTH
                scalar_ops = dim % SIMD_WIDTH
                print(f"  2x SIMD loops: {double_ops}")
                print(f"  1x SIMD loops: {single_ops}")
                print(f"  Scalar ops: {scalar_ops}")
        else:
            # Small strategy (scalar unrolling)
            unrolled_iters = dim // 4
            remainder = dim % 4
            print(
                f"4x unrolled iterations: {unrolled_iters} ({remainder} scalar remainder)"
            )

        # Memory analysis
        bytes_per_vec = dim * 4
        cache_lines = bytes_per_vec / 64
        print(f"Memory: {bytes_per_vec} bytes ({cache_lines:.1f} cache lines)")

        # Explain performance
        print("\nPerformance explanation:")
        if dim == 32:
            print("✅ Excellent: 4x scalar unrolling perfect for small vectors")
            print("   - Maximum instruction-level parallelism")
            print("   - Fits in registers, minimal memory pressure")
        elif dim == 64:
            print("➖ Modest gain: At boundary of small strategy")
            print("   - Still benefits from scalar unrolling")
            print("   - But approaching register pressure limits")
        elif dim == 128:
            print("❌ Regression: Medium strategy suboptimal")
            print("   - 2x SIMD unrolling may have overhead")
            print("   - Perfect SIMD alignment but strategy switch cost")
            print("   - Could benefit from 4x or 8x SIMD unrolling")
        elif dim == 256:
            print("✅ Good gain: Benefits from approaching cache strategy")
            print("   - Better memory access patterns")
            print("   - Near boundary helps performance")

    print("\n" + "=" * 70)
    print("\nOptimization Opportunities:")
    print("1. Tune thresholds: 64→96, 256→384 might work better")
    print("2. Improve medium strategy: Use 4x or 8x SIMD unrolling")
    print("3. Special-case 128D: Most common dimension (OpenAI ada-002)")
    print("4. Consider dimension-specific tuning instead of ranges")

    print("\nWhy Adaptive SIMD Didn't Prevent Regression:")
    print("- Thresholds chosen were somewhat arbitrary (64, 256)")
    print("- Medium strategy (2x unroll) less aggressive than small (4x)")
    print("- No special handling for common dimensions (128D)")
    print("- Strategy switches at boundaries cause discontinuities")


if __name__ == "__main__":
    analyze_strategies()
