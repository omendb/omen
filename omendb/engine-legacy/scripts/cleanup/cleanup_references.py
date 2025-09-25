#!/usr/bin/env python3
"""Clean up references to removed algorithms (HNSW, RoarGraph, tiered storage)."""

import os
import re
import glob

# Replacements to make
REPLACEMENTS = {
    # Algorithm references (keep 'flat' as it's still supported)
    "algorithm='hnsw'": "algorithm='diskann'",
    "algorithm='roargraph'": "algorithm='diskann'",
    'algorithm="hnsw"': 'algorithm="diskann"',
    'algorithm="roargraph"': 'algorithm="diskann"',
    # Migration parameters (remove these)
    r",\s*migration_threshold=\d+": "",
    r"migration_threshold=\d+,?\s*": "",
    r",\s*force_algorithm='[\w]+'": "",
    r"force_algorithm='[\w]+',?\s*": "",
    # Comments about old algorithms
    "# Use HNSW": "# Use DiskANN",
    "# Use RoarGraph": "# Use DiskANN",
    "# HNSW algorithm": "# DiskANN algorithm",
    "# RoarGraph algorithm": "# DiskANN algorithm",
    "with HNSW": "with DiskANN",
    "with RoarGraph": "with DiskANN",
    # Tiered storage references
    "tiered_storage": "storage",
    "TieredStorage": "Storage",
}

# Files to skip (already removed or will be removed)
SKIP_FILES = {
    "cleanup_references.py",
    "cleanup_dead_code.py",
    ".git",
    ".pixi",
    "test/roargraph/",
    "test/migration/",
}


def should_skip(filepath):
    """Check if file should be skipped."""
    for skip in SKIP_FILES:
        if skip in filepath:
            return True
    return False


def clean_file(filepath):
    """Clean references in a single file."""
    if should_skip(filepath):
        return False

    try:
        with open(filepath, "r") as f:
            content = f.read()

        original = content

        # Apply replacements
        for old, new in REPLACEMENTS.items():
            if old.startswith("r"):
                # Regex replacement
                content = re.sub(old[1:], new, content)
            else:
                # Simple string replacement
                content = content.replace(old, new)

        # Remove entire lines that are now empty or just comments about removed features
        lines = content.split("\n")
        cleaned_lines = []
        for line in lines:
            # Skip lines that are just about removed algorithms
            if (
                any(x in line.lower() for x in ["hnsw", "roargraph", "tiered"])
                and "#" in line
            ):
                if "diskann" not in line.lower():
                    continue  # Skip pure comment lines about old algorithms
            cleaned_lines.append(line)

        content = "\n".join(cleaned_lines)

        if content != original:
            with open(filepath, "w") as f:
                f.write(content)
            return True
        return False

    except Exception as e:
        print(f"Error processing {filepath}: {e}")
        return False


def clean_python_api():
    """Special handling for Python API to simplify algorithm selection."""
    api_file = "python/omendb/api.py"
    if not os.path.exists(api_file):
        return

    with open(api_file, "r") as f:
        content = f.read()

    # Remove algorithm parameter from DB constructor docs
    content = re.sub(
        r"algorithm: str = 'diskann',.*?# .*?\n",
        "# DiskANN is the only algorithm - no rebuilds ever!\n",
        content,
    )

    # Simplify configure method
    content = re.sub(
        r"algorithm: Index algorithm \('diskann', 'hnsw', 'flat'\)",
        "# Algorithm is always DiskANN",
        content,
    )

    with open(api_file, "w") as f:
        f.write(content)


def main():
    """Clean up all references to removed algorithms."""
    print("🧹 Cleaning up algorithm references...")
    print("=" * 60)

    updated_files = []

    # Process Python files
    print("\n📝 Processing Python files...")
    for filepath in glob.glob("**/*.py", recursive=True):
        if clean_file(filepath):
            updated_files.append(filepath)
            print(f"  ✅ {filepath}")

    # Process Mojo files
    print("\n📝 Processing Mojo files...")
    for filepath in glob.glob("**/*.mojo", recursive=True):
        if clean_file(filepath):
            updated_files.append(filepath)
            print(f"  ✅ {filepath}")

    # Special handling for Python API
    print("\n📝 Simplifying Python API...")
    clean_python_api()

    print("\n" + "=" * 60)
    print(f"✅ Updated {len(updated_files)} files")
    print("\n💡 Next steps:")
    print("  1. Review changes with git diff")
    print("  2. Run tests to ensure nothing broke")
    print("  3. Commit the cleanup")


if __name__ == "__main__":
    main()
