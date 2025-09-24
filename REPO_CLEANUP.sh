#!/bin/bash
# Repository cleanup script - Review before running!
# Date: September 20, 2025

set -e  # Exit on error

echo "=== OmenDB Repository Cleanup ==="
echo "This script will reorganize files. Review changes before committing."
echo

# Create necessary directories
echo "Creating directory structure..."
mkdir -p tests/
mkdir -p internal/archive/old_architecture/
mkdir -p internal/archive/old_tests/
mkdir -p internal/archive/old_analysis/
mkdir -p internal/research/
mkdir -p scripts/

# Move test files from root to tests/
echo "Moving test files to tests/ directory..."
for file in test_*.py; do
    if [ -f "$file" ]; then
        echo "  Moving $file to tests/"
        mv "$file" tests/
    fi
done

# Move benchmark and analysis scripts
echo "Moving analysis scripts..."
[ -f "benchmark_industry_standards.py" ] && mv benchmark_industry_standards.py scripts/
[ -f "competitive_analysis_2025.py" ] && mv competitive_analysis_2025.py scripts/
[ -f "optimization_audit.py" ] && mv optimization_audit.py scripts/
[ -f "profile_detailed.py" ] && mv profile_detailed.py scripts/
[ -f "profile_insertion_detailed.py" ] && mv profile_insertion_detailed.py scripts/
[ -f "debug_hnsw_recall.py" ] && mv debug_hnsw_recall.py tests/
[ -f "debug_hnsw_state.py" ] && mv debug_hnsw_state.py tests/

# Archive old documentation from root
echo "Archiving old documentation..."
[ -f "ARCHITECTURE.md" ] && mv ARCHITECTURE.md internal/archive/
[ -f "DOCUMENTATION_INDEX.md" ] && mv DOCUMENTATION_INDEX.md internal/archive/
[ -f "OPTIMIZATION_SUMMARY.md" ] && mv OPTIMIZATION_SUMMARY.md internal/archive/
[ -f "RESEARCH.md" ] && mv RESEARCH.md internal/archive/
[ -f "STATUS.md" ] && mv STATUS.md internal/archive/
[ -f "RUST_COMPETITIVE_OPTIMIZATION.mojo" ] && mv RUST_COMPETITIVE_OPTIMIZATION.mojo internal/archive/

# Clean up internal/ - Archive redundant architecture docs
echo "Archiving redundant architecture documents..."
cd internal/

# Keep only the consolidated docs, archive the rest
[ -f "ARCHITECTURE_DECISION_FINAL.md" ] && mv ARCHITECTURE_DECISION_FINAL.md archive/old_architecture/
[ -f "MASTER_ARCHITECTURE_DECISION_2025.md" ] && mv MASTER_ARCHITECTURE_DECISION_2025.md archive/old_architecture/
[ -f "PURE_MOJO_ARCHITECTURE_FINAL.md" ] && mv PURE_MOJO_ARCHITECTURE_FINAL.md archive/old_architecture/
[ -f "UNIFIED_ARCHITECTURE_FINAL.md" ] && mv UNIFIED_ARCHITECTURE_FINAL.md archive/old_architecture/
[ -f "FINAL_ARCHITECTURE_DECISION.md" ] && mv FINAL_ARCHITECTURE_DECISION.md archive/old_architecture/
[ -f "MOJO_VALIDATION_AND_FUTURE_PROOF.md" ] && mv MOJO_VALIDATION_AND_FUTURE_PROOF.md archive/old_architecture/
[ -f "FFI_OVERHEAD_ANALYSIS.md" ] && mv FFI_OVERHEAD_ANALYSIS.md archive/old_architecture/
[ -f "RUST_VS_MOJO_DECISION.md" ] && mv RUST_VS_MOJO_DECISION.md archive/old_architecture/
[ -f "MOJO_REALITY_CHECK.md" ] && mv MOJO_REALITY_CHECK.md archive/old_architecture/
[ -f "ARCHITECTURE_COMPARISON.md" ] && mv ARCHITECTURE_COMPARISON.md archive/old_architecture/
[ -f "REFACTOR_RECOMMENDATION.md" ] && mv REFACTOR_RECOMMENDATION.md archive/old_architecture/
[ -f "STATE_OF_THE_ART_ARCHITECTURE.md" ] && mv STATE_OF_THE_ART_ARCHITECTURE.md archive/old_architecture/
[ -f "OMENDB_NEXT_GEN_PLAN.md" ] && mv OMENDB_NEXT_GEN_PLAN.md archive/old_architecture/
[ -f "HYBRID_IMPLEMENTATION_PROTOTYPE.mojo" ] && mv HYBRID_IMPLEMENTATION_PROTOTYPE.mojo archive/old_architecture/

# Archive old analysis docs
[ -f "BREAKTHROUGH_SEPT_20_2025.md" ] && mv BREAKTHROUGH_SEPT_20_2025.md archive/old_analysis/
[ -f "HONEST_REALITY_CHECK.md" ] && mv HONEST_REALITY_CHECK.md archive/old_analysis/
[ -f "MEMORY_STABILITY_ANALYSIS.md" ] && mv MEMORY_STABILITY_ANALYSIS.md archive/old_analysis/
[ -f "SEGMENTED_HNSW_RESULTS.md" ] && mv SEGMENTED_HNSW_RESULTS.md archive/old_analysis/
[ -f "THRESHOLD_UPDATE_RESULTS.md" ] && mv THRESHOLD_UPDATE_RESULTS.md archive/old_analysis/
[ -f "PERFORMANCE_ANALYSIS.md" ] && mv PERFORMANCE_ANALYSIS.md archive/old_analysis/
[ -f "FINAL_SUMMARY.md" ] && mv FINAL_SUMMARY.md archive/old_analysis/
[ -f "DOCS_CLEANUP_STATUS.md" ] && mv DOCS_CLEANUP_STATUS.md archive/old_analysis/

# Move research docs to proper location
[ -f "LANCEDB_ANALYSIS.md" ] && mv LANCEDB_ANALYSIS.md research/
[ -f "COMPETITIVE_ARCHITECTURE_ANALYSIS.md" ] && mv COMPETITIVE_ARCHITECTURE_ANALYSIS.md research/
[ -f "RESEARCH_CONSOLIDATED_2025.md" ] && mv RESEARCH_CONSOLIDATED_2025.md research/

# Archive old repo cleanup plans
[ -f "REPO_CLEANUP_PLAN.md" ] && mv REPO_CLEANUP_PLAN.md archive/
[ -f "REPO_CLEANUP_PLAN_REVISED.md" ] && mv REPO_CLEANUP_PLAN_REVISED.md archive/

# Remove unnecessary files
[ -f "AI_AGENT_CONTEXT.md" ] && rm AI_AGENT_CONTEXT.md  # Duplicate of CLAUDE.md
[ -f "DOCUMENTATION_STRUCTURE.md" ] && rm DOCUMENTATION_STRUCTURE.md  # Outdated
[ -f "README.md" ] && rm README.md  # Old internal README

cd ..

# Clean up zendb if it exists
if [ -d "zendb" ]; then
    echo "Removing zendb/ (separate project)..."
    rm -rf zendb/
fi

echo
echo "=== Cleanup Complete ==="
echo
echo "Final structure:"
echo "  core/"
echo "  ├── CLAUDE.md            # AI context"
echo "  ├── README.md            # Project README"
echo "  ├── tests/               # All test files"
echo "  ├── scripts/             # Utility scripts"
echo "  ├── internal/"
echo "  │   ├── ARCHITECTURE_SIMPLE.md  # Main architecture"
echo "  │   ├── STATUS.md        # Current status"
echo "  │   ├── TODO.md          # Active tasks"
echo "  │   ├── DECISIONS.md     # Decision log"
echo "  │   ├── RESEARCH.md      # Research findings"
echo "  │   ├── research/        # Detailed research"
echo "  │   └── archive/         # Old documents"
echo "  └── omendb/"
echo "      ├── engine/          # Mojo code"
echo "      ├── server/          # Python server"
echo "      └── web/             # Web UI"
echo
echo "Please review changes and commit when ready."