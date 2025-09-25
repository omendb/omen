#!/usr/bin/env python3
"""Identify and optionally remove dead code from removed algorithms."""

import os
import glob

# Files to remove (confirmed unused)
DEAD_FILES = [
    # Old migration strategy files
    "omendb/core/lazy_indexing.mojo",
    "omendb/core/incremental_migration.mojo",
    "omendb/core/simple_migration.mojo",
    # Old Mojo-only API that references HNSW
    "omendb/mojo_api.mojo",
    # Test files for removed algorithms
    "test/roargraph/",
    "test/migration/",
    "test/debug/test_hnsw_*.py",
    "test/debug/test_buffer_*.py",
    "test/debug/test_algo_comparison.py",
    "test/debug/test_compare_algorithms.py",
    "test/debug/test_direct_hnsw.py",
    "test/debug/test_pure_hnsw.py",
    "test/debug/test_no_buffer_comparison.py",
]

# Keywords that indicate old algorithm references
DEAD_KEYWORDS = [
    "hnsw",
    "HNSW",
    "roargraph",
    "RoarGraph",
    "tiered",
    "TieredStorage",
    "migration_threshold",
    "force_algorithm",
    "algorithm='hnsw'",
    "algorithm='roargraph'",
]


def find_dead_code():
    """Find files with references to removed algorithms."""
    print("🔍 Searching for dead code references...\n")

    dead_references = []

    # Search all .mojo and .py files
    for pattern in ["**/*.mojo", "**/*.py"]:
        for filepath in glob.glob(pattern, recursive=True):
            # Skip this script
            if filepath.endswith("cleanup_dead_code.py"):
                continue

            # Skip files in .git or pixi directories
            if ".git" in filepath or ".pixi" in filepath:
                continue

            with open(filepath, "r") as f:
                try:
                    content = f.read()
                    for keyword in DEAD_KEYWORDS:
                        if keyword in content:
                            # Special case: diskann.mojo mentions HNSW in comments
                            if (
                                "diskann" in filepath.lower()
                                and keyword.lower() == "hnsw"
                            ):
                                continue
                            dead_references.append((filepath, keyword))
                            break
                except:
                    pass

    return dead_references


def list_dead_files():
    """List files that should be removed."""
    print("📁 Files to remove:\n")

    existing_dead = []
    for pattern in DEAD_FILES:
        if pattern.endswith("/"):
            # Directory
            if os.path.exists(pattern.rstrip("/")):
                existing_dead.append(pattern)
                print(f"  ❌ {pattern} (directory)")
        elif "*" in pattern:
            # Glob pattern
            matches = glob.glob(pattern)
            for match in matches:
                existing_dead.append(match)
                print(f"  ❌ {match}")
        else:
            # Single file
            if os.path.exists(pattern):
                existing_dead.append(pattern)
                print(f"  ❌ {pattern}")

    return existing_dead


def main():
    """Main cleanup analysis."""
    print("🧹 OmenDB Dead Code Analysis")
    print("=" * 60)

    # Find dead files
    dead_files = list_dead_files()

    print(f"\nFound {len(dead_files)} files/directories to remove")

    # Find references to dead code
    print("\n" + "=" * 60)
    references = find_dead_code()

    if references:
        print(f"\n⚠️  Found {len(references)} files with dead code references:\n")
        for filepath, keyword in references:
            print(f"  {filepath}: contains '{keyword}'")
    else:
        print("\n✅ No dead code references found in active files")

    # Summary
    print("\n" + "=" * 60)
    print("📊 Summary:")
    print(f"  - Files to remove: {len(dead_files)}")
    print(f"  - Files with dead references: {len(set(f for f, _ in references))}")

    print("\n💡 Recommendations:")
    print("  1. Remove identified dead files")
    print("  2. Update files with dead references")
    print("  3. Clean up test directory structure")
    print("  4. Update documentation")


if __name__ == "__main__":
    main()
