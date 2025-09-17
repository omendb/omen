#!/usr/bin/env python3
"""
Compare Monolithic vs Segmented Search Quality
October 2025

Test the same data with both approaches to isolate the issue.
"""

import numpy as np
import sys
import os

# Add path to native module
engine_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.append(os.path.join(engine_dir, 'python', 'omendb'))

# Import native module
import native

def test_search_quality(vectors, test_name, expected_threshold):
    """Test search quality and return results"""
    ids = [f'{test_name}_{i}' for i in range(len(vectors))]
    metadata = [{}] * len(vectors)

    # Build index
    native.clear_database()
    print(f"\n🏗️  Building {test_name} index with {len(vectors)} vectors...")
    native.add_vector_batch(ids, vectors, metadata)

    # Test with first few vectors
    perfect_matches = 0
    total_tests = min(10, len(vectors))

    print(f"Testing search quality with {total_tests} queries...")

    for i in range(total_tests):
        query = vectors[i]
        results = native.search_vectors(query, 5, {})

        expected_id = f'{test_name}_{i}'
        if len(results) > 0:
            actual_id = results[0].get('id', '')
            actual_distance = results[0].get('distance', float('inf'))

            if actual_id == expected_id and actual_distance < 0.001:
                perfect_matches += 1
                status = "✅"
            else:
                status = "❌"

            print(f"  Query {i}: Expected '{expected_id}', Got '{actual_id}', Distance: {actual_distance:.6f} {status}")
        else:
            print(f"  Query {i}: No results returned ❌")

    accuracy = (perfect_matches / total_tests) * 100
    print(f"\n📊 {test_name} Results:")
    print(f"  Perfect matches: {perfect_matches}/{total_tests} ({accuracy:.1f}%)")
    print(f"  Search mode: {'Segmented' if len(vectors) >= expected_threshold else 'Monolithic'}")

    return accuracy

def main():
    """Compare monolithic vs segmented search quality"""
    print("="*70)
    print("🔍 MONOLITHIC vs SEGMENTED QUALITY COMPARISON")
    print("="*70)

    # Use same seed for identical data
    np.random.seed(42)
    dimension = 128

    # Test 1: Small batch (monolithic mode, <10K threshold)
    mono_size = 5000
    mono_vectors = np.random.randn(mono_size, dimension).astype(np.float32)

    mono_accuracy = test_search_quality(mono_vectors, "mono", 10000)

    # Test 2: Large batch (segmented mode, >=10K threshold)
    seg_size = 10000
    seg_vectors = np.random.randn(seg_size, dimension).astype(np.float32)

    seg_accuracy = test_search_quality(seg_vectors, "seg", 10000)

    # Test 3: Same exact data, but subset to force monolithic
    # Use first 5000 vectors from the segmented set
    mono_subset_vectors = seg_vectors[:5000]

    print(f"\n" + "="*70)
    print("🧪 CONTROL TEST: Same vectors in monolithic mode")
    print("="*70)

    mono_subset_accuracy = test_search_quality(mono_subset_vectors, "subset", 10000)

    # Summary comparison
    print(f"\n" + "="*70)
    print("📊 QUALITY COMPARISON SUMMARY")
    print("="*70)

    print(f"{'Mode':<20} {'Size':<8} {'Perfect Matches':<15} {'Quality'}")
    print("-" * 60)
    print(f"{'Monolithic':<20} {mono_size:<8} {mono_accuracy:<15.1f}% {'✅ Expected' if mono_accuracy >= 90 else '⚠️  Poor'}")
    print(f"{'Segmented':<20} {seg_size:<8} {seg_accuracy:<15.1f}% {'✅ Good' if seg_accuracy >= 90 else '❌ BROKEN'}")
    print(f"{'Mono (control)':<20} {'5000':<8} {mono_subset_accuracy:<15.1f}% {'✅ Expected' if mono_subset_accuracy >= 90 else '⚠️  Poor'}")

    # Analysis
    print(f"\n🔍 ANALYSIS:")

    if mono_accuracy >= 90 and seg_accuracy < 50:
        print("❌ SEGMENTED MODE IS BROKEN - Quality severely degraded")
    elif mono_accuracy >= 90 and seg_accuracy >= 90:
        print("✅ Both modes working correctly")
    elif mono_accuracy < 50:
        print("❌ FUNDAMENTAL ISSUE - Even monolithic mode is broken")
    else:
        print("⚠️  Partial issues detected")

    print(f"\n🎯 ROOT CAUSE:")
    if seg_accuracy < mono_accuracy * 0.5:
        print("The segmented implementation is fundamentally broken.")
        print("Likely issues:")
        print("  • insert_bulk_wip() not building HNSW graph correctly")
        print("  • ID mapping issues between segmented and monolithic paths")
        print("  • Simplified segmented implementation using wrong internal structure")
    else:
        print("Issue may be with test data or methodology")

if __name__ == "__main__":
    main()