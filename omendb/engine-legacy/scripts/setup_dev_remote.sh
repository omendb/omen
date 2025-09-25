#!/bin/bash
# Setup development remote for OmenDB
# Use omendb-dev for development, switch to omendb for releases

echo "🔧 Setting up OmenDB development remote configuration..."

# Check if we're in the right directory
if [ ! -f "CLAUDE.md" ] || [ ! -d "omendb" ]; then
    echo "❌ Error: Run this script from the omendb repository root"
    exit 1
fi

# Remove old private remote if exists
if git remote | grep -q "private"; then
    echo "📝 Removing old 'private' remote..."
    git remote remove private
fi

# Add new dev remote (will be renamed from omendb-private to omendb-dev)
echo "➕ Adding 'dev' remote for omendb-dev repository..."
git remote add dev git@github.com:omendb/omendb-dev.git 2>/dev/null || true

# Update existing dev remote if needed
git remote set-url dev git@github.com:omendb/omendb-dev.git

# Set dev as default push remote
echo "🎯 Setting 'dev' as default push remote..."
git config remote.pushdefault dev

# Show current configuration
echo ""
echo "✅ Remote configuration updated:"
git remote -v

echo ""
echo "📋 Usage:"
echo "  Development work:  git push dev"
echo "  Release:          git push origin"
echo ""
echo "🔄 To rename the GitHub repo:"
echo "  1. Go to https://github.com/omendb/omendb-private/settings"
echo "  2. Rename to 'omendb-dev'"
echo "  3. Run this script again"