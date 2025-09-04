#!/usr/bin/env python3
"""
Compare benchmark results between branches for PR reviews.
"""

import json
import sys
from typing import Dict, Optional


def load_results(filename: str) -> Optional[Dict]:
    """Load benchmark results from file."""
    try:
        with open(filename, "r") as f:
            return json.load(f)
    except Exception as e:
        print(f"❌ Error loading {filename}: {e}")
        return None


def calculate_change(current: float, baseline: float) -> tuple[float, str]:
    """Calculate percentage change and emoji indicator."""
    if baseline == 0:
        return 0, "➖"

    change = (current - baseline) / baseline * 100

    if abs(change) < 1:
        emoji = "➖"  # No significant change
    elif change > 10:
        emoji = "🚀"  # Significant improvement
    elif change > 0:
        emoji = "✅"  # Improvement
    elif change < -10:
        emoji = "❌"  # Significant regression
    else:
        emoji = "⚠️"  # Minor regression

    return change, emoji


def format_number(value: float, is_time: bool = False) -> str:
    """Format number for display."""
    if is_time:
        return f"{value:.2f}ms"
    elif value > 1000000:
        return f"{value / 1000000:.1f}M"
    elif value > 1000:
        return f"{value / 1000:.1f}K"
    else:
        return f"{value:.0f}"


def compare_results(current: Dict, baseline: Dict) -> str:
    """Generate comparison markdown report."""
    report = []
    report.append("## 📊 Performance Comparison\n")

    if "OmenDB" not in current or "OmenDB" not in baseline:
        report.append("❌ Unable to compare - OmenDB results not found\n")
        return "\n".join(report)

    current_omen = current["OmenDB"]["benchmarks"]
    baseline_omen = baseline["OmenDB"]["benchmarks"]

    # Compare each benchmark size
    sizes = sorted(set(current_omen.keys()) & set(baseline_omen.keys()))

    for size in sizes:
        report.append(f"\n### {size:,} vectors\n")
        report.append("| Metric | Current | Baseline | Change |")
        report.append("|--------|---------|----------|--------|")

        curr = current_omen[size]
        base = baseline_omen[size]

        # Insertion throughput
        if "insertion" in curr and "insertion" in base:
            curr_tp = curr["insertion"].get("throughput", 0)
            base_tp = base["insertion"].get("throughput", 0)
            change, emoji = calculate_change(curr_tp, base_tp)

            report.append(
                f"| **Insertion** | {format_number(curr_tp)} vec/s | "
                f"{format_number(base_tp)} vec/s | "
                f"{emoji} {change:+.1f}% |"
            )

        # Query latency
        if "query" in curr and "query" in base:
            curr_p50 = curr["query"].get("p50_latency_ms", 0)
            base_p50 = base["query"].get("p50_latency_ms", 0)
            # For latency, negative change is good
            change, emoji = calculate_change(base_p50, curr_p50)

            report.append(
                f"| **Query P50** | {curr_p50:.2f}ms | "
                f"{base_p50:.2f}ms | "
                f"{emoji} {-change:+.1f}% |"
            )

            curr_p99 = curr["query"].get("p99_latency_ms", 0)
            base_p99 = base["query"].get("p99_latency_ms", 0)
            change, emoji = calculate_change(base_p99, curr_p99)

            report.append(
                f"| **Query P99** | {curr_p99:.2f}ms | "
                f"{base_p99:.2f}ms | "
                f"{emoji} {-change:+.1f}% |"
            )

        # Memory usage
        if "memory" in curr and "memory" in base:
            curr_mem = curr["memory"].get("bytes_per_vector", 0)
            base_mem = base["memory"].get("bytes_per_vector", 0)
            # For memory, negative change is good
            change, emoji = calculate_change(base_mem, curr_mem)

            report.append(
                f"| **Memory/vec** | {format_number(curr_mem)} B | "
                f"{format_number(base_mem)} B | "
                f"{emoji} {-change:+.1f}% |"
            )

    # Add summary
    report.append("\n### Summary\n")

    # Calculate overall assessment
    improvements = 0
    regressions = 0

    for size in sizes:
        curr = current_omen[size]
        base = baseline_omen[size]

        if "insertion" in curr and "insertion" in base:
            if (
                curr["insertion"].get("throughput", 0)
                > base["insertion"].get("throughput", 0) * 1.05
            ):
                improvements += 1
            elif (
                curr["insertion"].get("throughput", 0)
                < base["insertion"].get("throughput", 0) * 0.95
            ):
                regressions += 1

        if "query" in curr and "query" in base:
            if (
                curr["query"].get("p50_latency_ms", 999)
                < base["query"].get("p50_latency_ms", 999) * 0.95
            ):
                improvements += 1
            elif (
                curr["query"].get("p50_latency_ms", 999)
                > base["query"].get("p50_latency_ms", 999) * 1.05
            ):
                regressions += 1

    if regressions > 0:
        report.append(
            "⚠️ **Performance regressions detected** - Please review the changes above.\n"
        )
    elif improvements > 0:
        report.append("✅ **Performance improvements detected** - Great work!\n")
    else:
        report.append("➖ **No significant performance changes detected**\n")

    # Add legend
    report.append("\n---")
    report.append(
        "*Legend: 🚀 >10% improvement | ✅ improvement | ➖ no change | ⚠️ regression | ❌ >10% regression*"
    )

    return "\n".join(report)


def main():
    """Main comparison runner."""
    if len(sys.argv) != 3:
        print("Usage: python compare_benchmarks.py <current.json> <baseline.json>")
        sys.exit(1)

    current_file = sys.argv[1]
    baseline_file = sys.argv[2]

    print(f"📊 Comparing benchmarks...")
    print(f"  Current: {current_file}")
    print(f"  Baseline: {baseline_file}")

    current = load_results(current_file)
    baseline = load_results(baseline_file)

    if not current or not baseline:
        print("❌ Failed to load benchmark results")
        sys.exit(1)

    comparison = compare_results(current, baseline)

    # Write to file for GitHub Actions
    with open("comparison.md", "w") as f:
        f.write(comparison)

    # Also print to console
    print("\n" + comparison)

    # Check for regressions
    if "regressions detected" in comparison:
        sys.exit(1)  # Fail the CI check

    sys.exit(0)


if __name__ == "__main__":
    main()
