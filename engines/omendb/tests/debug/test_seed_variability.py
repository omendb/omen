#!/usr/bin/env python3
"""Test accuracy variability with different random seeds"""

import omendb
import numpy as np


def test_with_seed(seed, n_vectors=500, dim=128):
    """Test accuracy with a specific random seed"""
    db = omendb.DB()
    db.clear()
    db.configure(buffer_size=100)

    # Generate random vectors with specific seed
    np.random.seed(seed)
    vectors = np.random.randn(n_vectors, dim).astype(np.float32)

    # Add all vectors
    for i in range(n_vectors):
        db.add(f"vec_{i}", vectors[i])

    # Test accuracy
    test_sample = 20
    correct = 0

    for i in range(test_sample):
        results = db.search(vectors[i], limit=1)
        if results and results[0].id == f"vec_{i}":
            correct += 1

    accuracy = (correct / test_sample) * 100
    return accuracy


def main():
    """Test with multiple seeds to check variability"""
    print("TESTING ACCURACY VARIABILITY ACROSS RANDOM SEEDS")
    print("=" * 60)
    print("Testing 500 vectors, 128 dimensions, 20 samples per test")
    print()

    seeds = range(10)  # Test 10 different seeds
    accuracies = []

    print(f"{'Seed':<6} {'Accuracy':<10} {'Status'}")
    print("-" * 30)

    for seed in seeds:
        acc = test_with_seed(seed)
        accuracies.append(acc)

        status = "✅" if acc >= 95 else "❌"
        print(f"{seed:<6} {acc:>6.1f}%    {status}")

    print("-" * 30)

    # Statistics
    mean_acc = np.mean(accuracies)
    std_acc = np.std(accuracies)
    min_acc = np.min(accuracies)
    max_acc = np.max(accuracies)

    print(f"\nStatistics:")
    print(f"  Mean:     {mean_acc:.1f}%")
    print(f"  Std Dev:  {std_acc:.1f}%")
    print(f"  Min:      {min_acc:.1f}%")
    print(f"  Max:      {max_acc:.1f}%")

    if min_acc < 95:
        print(f"\n⚠️ Some seeds have accuracy below 95% target")
        print(f"   Failing seeds: {[s for s, a in zip(seeds, accuracies) if a < 95]}")
    else:
        print(f"\n✅ All seeds achieve ≥95% accuracy!")

    # Test at different scales
    print("\n" + "=" * 60)
    print("TESTING SCALE EFFECT (seed=0)")
    print("=" * 60)

    scales = [100, 500, 1000, 2000, 5000]
    print(f"{'N Vectors':<10} {'Accuracy':<10} {'Status'}")
    print("-" * 30)

    for n in scales:
        acc = test_with_seed(0, n_vectors=n)
        status = "✅" if acc >= 95 else "❌"
        print(f"{n:<10} {acc:>6.1f}%    {status}")


if __name__ == "__main__":
    main()
