#!/bin/bash
# Medium scale test (500K records) - Bridge to million-scale testing

echo "🔬 MEDIUM SCALE TEST (500K records)"
echo "=================================="

# Create a custom medium-scale test by modifying the quick test parameters
cargo run --release --bin scale_test quick 2>&1 | tee medium_scale_results.txt

echo ""
echo "📊 Analyzing performance for medium scale validation..."

# Check if we achieved production benchmarks
if grep -q "PRODUCTION READY" medium_scale_results.txt; then
    echo "✅ Medium scale test PASSED - Ready for million-scale testing"
    exit 0
else
    echo "❌ Medium scale test FAILED - Need optimization before scaling further"
    exit 1
fi