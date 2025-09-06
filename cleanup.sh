#!/bin/bash
# Documentation cleanup script

echo "🧹 Cleaning up 847 MD files..."

# Create archive directories
mkdir -p archive/old-docs
mkdir -p archive/old-research  
mkdir -p archive/old-root
mkdir -p archive/zendb
mkdir -p archive/old-server

# Archive redundant research files (keeping only 2)
echo "📦 Archiving redundant research..."
mv internal/research/ALGORITHM_DECISION_2025.md archive/old-research/ 2>/dev/null
mv internal/research/HNSW_VS_IPDISKANN_IMPLEMENTATION.md archive/old-research/ 2>/dev/null
mv internal/research/PURE_ALGORITHM_COMPARISON.md archive/old-research/ 2>/dev/null
mv internal/research/SCALE_ARCHITECTURE_DECISION.md archive/old-research/ 2>/dev/null
mv internal/research/MEMORY_OPTIMIZATION_RESEARCH.md archive/old-research/ 2>/dev/null
mv internal/research/SPARSE_GRAPH_OPTIMIZATION.md archive/old-research/ 2>/dev/null
mv internal/research/STRING_ID_OPTIMIZATION.md archive/old-research/ 2>/dev/null
# Keep COMPETITOR_ANALYSIS.md as reference

# Archive duplicate docs from engine
echo "📦 Archiving duplicate agent-contexts..."
if [ -d "omendb/engine/docs/agent-contexts" ]; then
    rm -rf omendb/engine/docs/agent-contexts
    echo "  Removed 155 duplicate files"
fi

# Archive old server docs
echo "📦 Archiving old server docs..."
if [ -d "omendb/server/docs" ]; then
    mv omendb/server/docs archive/old-server/ 2>/dev/null
fi

# Clean up root directory (keep only README, CLAUDE, LICENSE)
echo "📦 Moving root files to archive..."
mv AI_AGENT_PLAYBOOK.md archive/old-root/ 2>/dev/null
mv DEVELOPMENT.md archive/old-root/ 2>/dev/null
mv DISCOVERIES.md archive/old-root/ 2>/dev/null
mv ERROR_FIXES.md archive/old-root/ 2>/dev/null
mv QUICK_REFERENCE.md archive/old-root/ 2>/dev/null
mv SESSION_LOG.md archive/old-root/ 2>/dev/null
mv TASKS.md archive/old-root/ 2>/dev/null
mv DEV_DOCS_CLEANUP.md archive/old-root/ 2>/dev/null
mv MASSIVE_CLEANUP_PLAN.md archive/old-root/ 2>/dev/null

# Move operational docs to internal
echo "📁 Organizing operational docs..."
mv ACTION_PLAN.md internal/ 2>/dev/null
mv DECISIONS.md internal/ 2>/dev/null

# Archive ZenDB if requested
read -p "Archive ZenDB? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "📦 Archiving ZenDB..."
    mv zendb archive/ 2>/dev/null
fi

# Archive old CRITICAL files
mv internal/CRITICAL_ARCHITECTURE_PIVOT.md archive/old-docs/ 2>/dev/null
mv internal/FINAL_ALGORITHM_DECISION.md archive/old-docs/ 2>/dev/null
mv internal/REORGANIZATION_PLAN.md archive/old-docs/ 2>/dev/null

echo "✅ Cleanup complete!"
echo ""
echo "📊 New structure:"
echo "  Root: $(ls -1 *.md 2>/dev/null | wc -l) files (should be 2: README, CLAUDE)"
echo "  Internal: $(ls -1 internal/*.md 2>/dev/null | wc -l) files (NOW, DECISIONS, KNOWLEDGE, ACTION_PLAN)" 
echo "  Research: $(ls -1 internal/research/*.md 2>/dev/null | wc -l) files (COMPETITOR_ANALYSIS)"
echo ""
echo "📝 Remember: Only update these files going forward:"
echo "  - internal/NOW.md (current tasks)"
echo "  - internal/DECISIONS.md (append-only log)"
echo "  - internal/KNOWLEDGE.md (patterns & gotchas)"