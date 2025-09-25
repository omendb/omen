#!/bin/bash
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
