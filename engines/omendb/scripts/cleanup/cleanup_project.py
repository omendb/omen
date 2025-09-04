#!/usr/bin/env python3
"""
Clean up and organize OmenDB project based on audit results.
"""

import os
import shutil
from pathlib import Path
import json


def move_files_to_proper_locations():
    """Move root-level files to appropriate directories."""

    # Create necessary directories
    dirs_to_create = [
        "test/debug",
        "scripts/cleanup",
        "results",
        "results/profiling",
    ]

    for dir_path in dirs_to_create:
        Path(dir_path).mkdir(parents=True, exist_ok=True)

    # Files to move
    moves = {
        # Test files
        "test_*.py": "test/debug/",
        "debug_*.py": "test/debug/",
        # Benchmark files
        "benchmark_diskann.py": "benchmarks/",
        # Cleanup scripts
        "cleanup_*.py": "scripts/cleanup/",
        "reorganize_tests.py": "scripts/cleanup/",
        "audit_project.py": "scripts/cleanup/",
        "cleanup_project.py": "scripts/cleanup/",
        # Results
        "benchmark_results_*.json": "results/",
        "profile_results_*.json": "results/profiling/",
        "audit_results.json": "results/",
    }

    moved = []
    for pattern, target_dir in moves.items():
        if "*" in pattern:
            import glob

            files = glob.glob(pattern)
        else:
            files = [pattern] if os.path.exists(pattern) else []

        for file in files:
            if os.path.isfile(file):
                target_path = os.path.join(target_dir, os.path.basename(file))
                try:
                    shutil.move(file, target_path)
                    moved.append((file, target_path))
                    print(f"  ✅ Moved {file} → {target_path}")
                except Exception as e:
                    print(f"  ❌ Failed to move {file}: {e}")

    return moved


def update_gitignore():
    """Add entries to .gitignore."""

    entries_to_add = [
        "\n# Cache directories",
        "embeddings_cache/",
        "__pycache__/",
        "*.pyc",
        "\n# Result files",
        "results/",
        "*.json",
        "\n# Temporary test files",
        "test/debug/",
        "\n# IDE",
        ".vscode/",
        ".idea/",
    ]

    gitignore_path = ".gitignore"

    # Read existing
    if os.path.exists(gitignore_path):
        with open(gitignore_path, "r") as f:
            existing = f.read()
    else:
        existing = ""

    # Add new entries if not present
    added = []
    for entry in entries_to_add:
        if entry not in existing and not entry.startswith("\n"):
            added.append(entry)

    if added:
        with open(gitignore_path, "a") as f:
            f.write("\n")
            for entry in entries_to_add:
                f.write(entry + "\n")
        print(f"  ✅ Updated .gitignore with {len(added)} entries")

    return added


def create_proper_structure():
    """Create proper project structure."""

    structure = {
        ".github/workflows": "GitHub Actions",
        "benchmarks": "Performance benchmarks",
        "ci": "CI/CD scripts",
        "docs/api": "API documentation",
        "docs/guides": "User guides",
        "docs/internal": "Internal docs (should be in private repo)",
        "examples": "Usage examples",
        "omendb": "Core package",
        "python/omendb": "Python bindings",
        "results": "Test/benchmark results (gitignored)",
        "scripts": "Utility scripts",
        "scripts/cleanup": "Cleanup utilities",
        "test/unit": "Unit tests",
        "test/integration": "Integration tests",
        "test/benchmarks": "Performance tests",
        "test/regression": "Regression tests",
        "test/debug": "Debug/experimental tests (gitignored)",
    }

    for path, purpose in structure.items():
        Path(path).mkdir(parents=True, exist_ok=True)
        print(f"  ✅ Ensured {path}/ exists")


def identify_private_docs():
    """Identify documentation that should be in private repo."""

    private_keywords = [
        "investor",
        "business",
        "revenue",
        "pricing",
        "monetization",
        "competitive",
        "strategy",
        "roadmap",
        "internal",
        "confidential",
        "proprietary",
    ]

    private_docs = []
    import glob

    for doc in glob.glob("docs/**/*.md", recursive=True):
        with open(doc, "r") as f:
            content = f.read().lower()

        for keyword in private_keywords:
            if keyword in content:
                # Check if it's actually sensitive
                if any(
                    term in content
                    for term in [
                        "competitive analysis",
                        "business plan",
                        "revenue model",
                        "internal only",
                    ]
                ):
                    private_docs.append(doc)
                    break

    return private_docs


def create_cleanup_script():
    """Create a script to automate the cleanup."""

    script = """#!/bin/bash
# OmenDB Project Cleanup Script

echo "🧹 Cleaning up OmenDB project..."

# Move files to proper locations
echo "📁 Organizing files..."
python scripts/cleanup/cleanup_project.py

# Remove empty directories
echo "🗑️  Removing empty directories..."
find . -type d -empty -delete

# Update documentation
echo "📝 Documentation reminders:"
echo "  - Update README.md with current performance (96K vec/s)"
echo "  - Remove HNSW/RoarGraph references"
echo "  - Update CHANGELOG.md"
echo "  - Move internal docs to private repo"

# Run tests to ensure nothing broke
echo "🧪 Running tests..."
python test/run_tests.py unit

echo "✅ Cleanup complete!"
"""

    script_path = "scripts/cleanup.sh"
    with open(script_path, "w") as f:
        f.write(script)
    os.chmod(script_path, 0o755)
    print(f"  ✅ Created {script_path}")


def main():
    """Main cleanup process."""
    print("🧹 OmenDB Project Cleanup")
    print("=" * 60)

    # Create proper structure
    print("\n📁 Creating proper directory structure...")
    create_proper_structure()

    # Move files
    print("\n🚚 Moving files to proper locations...")
    moved = move_files_to_proper_locations()
    print(f"  Moved {len(moved)} files")

    # Update .gitignore
    print("\n📝 Updating .gitignore...")
    update_gitignore()

    # Identify private docs
    print("\n🔒 Identifying private documentation...")
    private_docs = identify_private_docs()
    if private_docs:
        print("  These docs may need to move to private repo:")
        for doc in private_docs:
            print(f"    - {doc}")

    # Create cleanup script
    print("\n📜 Creating cleanup script...")
    create_cleanup_script()

    # Summary
    print("\n" + "=" * 60)
    print("📊 Cleanup Summary")
    print("=" * 60)
    print(f"  Files moved: {len(moved)}")
    print(f"  Private docs found: {len(private_docs)}")

    print("\n🎯 Next Steps:")
    print("  1. Review and commit these changes")
    print("  2. Update README.md with correct performance (96K vec/s)")
    print("  3. Remove HNSW/RoarGraph references from docs")
    print("  4. Add comprehensive edge case tests")
    print("  5. Fix GitHub Actions workflow")
    print("  6. Consider moving internal docs to omendb-cloud/")


if __name__ == "__main__":
    main()
