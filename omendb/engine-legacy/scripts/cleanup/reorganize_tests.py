#!/usr/bin/env python3
"""Reorganize test directory structure for clarity and maintainability."""

import os
import shutil
from pathlib import Path

# Test organization plan
REORGANIZATION = {
    # Unit tests - fast, isolated
    "test/unit/": [
        "test/unit/test_cosine_distance.py",
        "test/unit/test_vector_storage.py",
        "test/unit/test_basic_api.py",
        "test/python/test_basic_crud.py",
        "test/python/test_dimension_validation.py",
        "test/python/test_edge_cases.py",
    ],
    # Integration tests - full system
    "test/integration/": [
        "test/integration/test_comprehensive.py",
        "test/integration/test_file_persistence.py",
        "test/python/test_api_standards.py",
        "test/python/test_enhanced_api.py",
        "test/python/test_upsert.py",
        "test/native_module/test_native_api.py",
        "test/native_module/test_working_native.py",
    ],
    # Performance benchmarks
    "test/benchmarks/": [
        "test/performance/test_current_performance.py",
        "test/performance/test_scale_validation.py",
        "test/performance/test_batch_api_numpy.py",
        "test/performance/test_incremental_performance.py",
        "test/diskann/test_diskann_scale.py",
        "test/performance/test_diskann_buffer_comparison.py",
    ],
    # Regression tests for bugs
    "test/regression/": [
        "test/regression/test_numpy_optimization.py",
        "test/python/test_memory_leaks.py",
        "test/validation/test_500_vector_accuracy.py",
        "test/test_bug_fixes.py",  # Our new bug fix tests
        "test/test_numpy_batch.py",  # NumPy batch bug test
    ],
    # Keep debug directory but clean it up
    "test/debug/": [
        # Only keep actively useful debug tools
        "test/debug/debug_buffer_flush.py",
        "test/debug/profile_bottleneck.py",
        "test/debug/test_memory_profiling.py",
    ],
    # Test fixtures and data
    "test/fixtures/": [
        "test/data/",
        "test/artifacts/",
    ],
}

# Files to delete (duplicates or obsolete)
TO_DELETE = [
    "test/test_migration_*.py",  # Old migration tests
    "test/test_incremental_migration.py",
    "test/test_instant_startup.py",  # Not meaningful anymore
    "test/consolidate_tests.py",  # Meta file
    "test/embedded_minimal_suite.py",  # Duplicate
]


def create_new_structure():
    """Create the new test directory structure."""
    print("🏗️  Creating new test structure...")

    # Create new directories
    for new_dir in REORGANIZATION.keys():
        Path(new_dir).mkdir(parents=True, exist_ok=True)
        print(f"  📁 Created {new_dir}")

    # Create proper README for each directory
    readmes = {
        "test/unit/": """# Unit Tests

Fast, isolated tests for individual components.
Run frequently during development.

```bash
pytest test/unit/
```
""",
        "test/integration/": """# Integration Tests

Full system tests that verify end-to-end functionality.

```bash
pytest test/integration/
```
""",
        "test/benchmarks/": """# Performance Benchmarks

Performance tests and competitive comparisons.

```bash
python test/benchmarks/test_current_performance.py
```
""",
        "test/regression/": """# Regression Tests

Tests that prevent previously fixed bugs from reoccurring.

```bash
pytest test/regression/
```
""",
    }

    for path, content in readmes.items():
        readme_path = Path(path) / "README.md"
        readme_path.write_text(content)
        print(f"  📝 Created {readme_path}")


def move_tests():
    """Move tests to their new locations."""
    print("\n🚚 Moving tests to new structure...")

    moved = 0
    failed = 0

    for target_dir, source_files in REORGANIZATION.items():
        for source in source_files:
            if "*" in source or not source.endswith(".py"):
                continue  # Skip patterns and directories for now

            source_path = Path(source)
            if source_path.exists():
                # Get just the filename
                filename = source_path.name
                target_path = Path(target_dir) / filename

                try:
                    # Copy instead of move to preserve originals for now
                    shutil.copy2(source_path, target_path)
                    moved += 1
                    print(f"  ✅ {source} → {target_path}")
                except Exception as e:
                    failed += 1
                    print(f"  ❌ Failed to move {source}: {e}")

    print(f"\n📊 Moved {moved} files ({failed} failures)")


def cleanup_old_tests():
    """Remove obsolete test files."""
    print("\n🗑️  Cleaning up obsolete tests...")

    deleted = 0
    for pattern in TO_DELETE:
        if "*" in pattern:
            # Handle glob patterns
            from glob import glob

            for file in glob(pattern):
                try:
                    os.remove(file)
                    deleted += 1
                    print(f"  🗑️  Deleted {file}")
                except:
                    pass
        else:
            # Single file
            try:
                if os.path.exists(pattern):
                    os.remove(pattern)
                    deleted += 1
                    print(f"  🗑️  Deleted {pattern}")
            except:
                pass

    print(f"\n📊 Deleted {deleted} obsolete files")


def create_test_runner():
    """Create a unified test runner script."""
    runner_content = '''#!/usr/bin/env python3
"""Unified test runner for OmenDB."""

import sys
import subprocess
from pathlib import Path

def run_tests(category=None):
    """Run tests by category."""
    
    categories = {
        "unit": "test/unit/",
        "integration": "test/integration/",
        "benchmarks": "test/benchmarks/",
        "regression": "test/regression/",
        "all": "test/",
    }
    
    if category and category in categories:
        path = categories[category]
        print(f"🧪 Running {category} tests...")
        cmd = f"pytest {path} -v"
    else:
        print("🧪 Running all tests...")
        cmd = "pytest test/ -v"
    
    subprocess.run(cmd, shell=True)

if __name__ == "__main__":
    category = sys.argv[1] if len(sys.argv) > 1 else "all"
    run_tests(category)
'''

    runner_path = Path("test/run_tests.py")
    runner_path.write_text(runner_content)
    runner_path.chmod(0o755)
    print(f"✅ Created test runner: {runner_path}")


def main():
    """Main reorganization process."""
    print("🧹 OmenDB Test Reorganization")
    print("=" * 60)

    # Check if we're in the right directory
    if not os.path.exists("test"):
        print("❌ Error: 'test' directory not found. Run from project root.")
        return

    # Create new structure
    create_new_structure()

    # Move tests
    move_tests()

    # Clean up old tests
    cleanup_old_tests()

    # Create test runner
    create_test_runner()

    print("\n" + "=" * 60)
    print("✅ Test reorganization complete!")
    print("\n📋 Next steps:")
    print("  1. Review the new structure in test/")
    print("  2. Update any import paths in moved tests")
    print("  3. Run tests to ensure they still work:")
    print("     - python test/run_tests.py unit")
    print("     - python test/run_tests.py integration")
    print("     - python test/run_tests.py benchmarks")
    print("  4. Delete old test files after verification")


if __name__ == "__main__":
    main()
