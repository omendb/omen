#!/bin/bash
# Run regression tests for critical dimensions

echo "OMENDB REGRESSION TEST SUITE"
echo "============================"
echo ""

# Test critical dimensions
dimensions=(32 64 128 256 384 512)

for dim in "${dimensions[@]}"; do
    echo "Testing ${dim}D..."
    echo "----------------------------"
    pixi run python test/performance/test_single_dimension.py $dim
    echo ""
done

echo "============================"
echo "REGRESSION TEST COMPLETE"