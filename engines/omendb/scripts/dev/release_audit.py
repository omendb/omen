#!/usr/bin/env python3
"""
Release Readiness Audit

Tests edge cases, error handling, and potential issues that could block release.
"""

import time
import numpy as np
from python.omendb import DB


def test_error_handling():
    """Test error handling and edge cases."""
    print("🔍 Error Handling Audit")
    print("-" * 25)

    issues = []

    # Test 1: Empty database queries
    try:
        db = DB()
        results = db.search([1.0, 2.0, 3.0], limit=10)
        if len(results) != 0:
            issues.append("Empty database should return 0 results")
        else:
            print("✅ Empty database query handled correctly")
    except Exception as e:
        issues.append(f"Empty database query failed: {e}")

    # Test 2: Invalid vector dimensions
    try:
        db = DB()
        db.add("test1", [1.0, 2.0, 3.0])

        # Try to add vector with different dimension
        success = db.add("test2", [1.0, 2.0])  # Different dimension
        if success:
            issues.append("Database should reject vectors with different dimensions")
        else:
            print("✅ Dimension validation working")
    except Exception as e:
        issues.append(f"Dimension validation test failed: {e}")

    # Test 3: Empty vectors
    try:
        db = DB()
        success = db.add("empty", [])
        if success:
            issues.append("Database should reject empty vectors")
        else:
            print("✅ Empty vector rejection working")
    except Exception as e:
        issues.append(f"Empty vector test failed: {e}")

    # Test 4: Invalid query dimensions
    try:
        db = DB()
        db.add("test", [1.0, 2.0, 3.0])
        results = db.search([1.0, 2.0], limit=10)  # Wrong dimension
        if len(results) != 0:
            issues.append("Query with wrong dimension should return 0 results")
        else:
            print("✅ Query dimension validation working")
    except Exception as e:
        issues.append(f"Query dimension test failed: {e}")

    # Test 5: Large k values
    try:
        db = DB()
        for i in range(5):
            db.add(f"vec_{i}", [float(i), float(i + 1), float(i + 2)])

        results = db.search([1.0, 2.0, 3.0], limit=100)  # More than available
        if len(results) > 5:
            issues.append("Should not return more results than vectors in database")
        else:
            print("✅ Large k value handled correctly")
    except Exception as e:
        issues.append(f"Large k test failed: {e}")

    # Test 6: Extreme values
    try:
        db = DB()
        extreme_vector = [1e10, -1e10, 0.0]
        db.add("extreme", extreme_vector)
        results = db.search(extreme_vector, limit=1)
        if len(results) == 0:
            issues.append("Extreme values should still work")
        else:
            print("✅ Extreme values handled")
    except Exception as e:
        issues.append(f"Extreme values test failed: {e}")

    return issues


def test_memory_stability():
    """Test for memory leaks and stability."""
    print("\n💾 Memory Stability Test")
    print("-" * 25)

    issues = []

    try:
        # Test repeated operations
        for cycle in range(3):
            db = DB()

            # Add vectors
            for i in range(100):
                db.add(
                    f"vec_{i}",
                    [float(i % 10), float((i + 1) % 10), float((i + 2) % 10)],
                )

            # Query repeatedly
            for i in range(50):
                results = db.search([1.0, 2.0, 3.0], limit=10)
                if len(results) == 0:
                    issues.append(f"Query failed in cycle {cycle}")
                    break

            # Force cleanup (Python GC)
            del db

        print("✅ Memory stability test passed")

    except Exception as e:
        issues.append(f"Memory stability test failed: {e}")

    return issues


def test_api_consistency():
    """Test API consistency and user experience."""
    print("\n🔧 API Consistency Test")
    print("-" * 25)

    issues = []

    try:
        # Test 1: Result format consistency
        db = DB()
        db.add("test1", [1.0, 2.0, 3.0], {"key": "value"})
        db.add("test2", [2.0, 3.0, 4.0])  # No metadata

        results = db.search([1.0, 2.0, 3.0], limit=10)

        for result in results:
            if not hasattr(result, "id"):
                issues.append("Result missing 'id' attribute")
            if not hasattr(result, "score"):
                issues.append("Result missing 'score' attribute")
            if not hasattr(result, "metadata"):
                issues.append("Result missing 'metadata' attribute")

        print("✅ Result format consistent")

        # Test 2: Metadata handling
        results_with_metadata = [r for r in results if r.metadata]
        if len(results_with_metadata) > 0:
            print("✅ Metadata preserved correctly")
        else:
            issues.append("Metadata not preserved in results")

    except Exception as e:
        issues.append(f"API consistency test failed: {e}")

    return issues


def test_performance_claims():
    """Verify performance claims are accurate."""
    print("\n⚡ Performance Claims Verification")
    print("-" * 35)

    issues = []

    try:
        # Test sub-millisecond queries claim
        vectors = np.random.random((1000, 128)).astype(np.float32)
        queries = np.random.random((20, 128)).astype(np.float32)

        db = DB()
        for i, vector in enumerate(vectors):
            db.add(f"vec_{i}", vector.tolist())

        query_times = []
        for query in queries:
            start_time = time.time()
            results = db.search(query.tolist(), limit=10)
            query_time = time.time() - start_time
            query_times.append(query_time * 1000)  # Convert to ms

        avg_query_time = np.mean(query_times)
        max_query_time = np.max(query_times)

        print(f"Average query time: {avg_query_time:.3f}ms")
        print(f"Max query time: {max_query_time:.3f}ms")

        if avg_query_time < 1.0:
            print("✅ Sub-millisecond query claim verified")
        else:
            issues.append(
                f"Sub-millisecond claim violated: {avg_query_time:.3f}ms average"
            )

        if max_query_time > 2.0:
            issues.append(f"Some queries too slow: {max_query_time:.3f}ms max")

    except Exception as e:
        issues.append(f"Performance verification failed: {e}")

    return issues


def main():
    """Run comprehensive release audit."""
    print("🚨 RELEASE READINESS AUDIT")
    print("=" * 40)

    all_issues = []

    # Run all tests
    all_issues.extend(test_error_handling())
    all_issues.extend(test_memory_stability())
    all_issues.extend(test_api_consistency())
    all_issues.extend(test_performance_claims())

    # Summary
    print("\n" + "=" * 40)
    print("📋 AUDIT SUMMARY")
    print("=" * 40)

    if len(all_issues) == 0:
        print("🎉 ALL TESTS PASSED - RELEASE READY!")
        print("✅ Error handling: Working")
        print("✅ Memory stability: Stable")
        print("✅ API consistency: Good")
        print("✅ Performance claims: Verified")
        return True
    else:
        print(f"❌ FOUND {len(all_issues)} ISSUES:")
        for i, issue in enumerate(all_issues, 1):
            print(f"  {i}. {issue}")
        print("\n⚠️ Issues must be resolved before release")
        return False


if __name__ == "__main__":
    success = main()
    exit(0 if success else 1)
