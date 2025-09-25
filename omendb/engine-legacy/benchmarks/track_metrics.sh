#!/bin/bash
# Quick script to track performance metrics over time

echo "======================================"
echo "OmenDB Performance Tracking"
echo "Date: $(date)"
echo "======================================"

# Set Python path
export PYTHONPATH=python:$PYTHONPATH

# Run regression tracker
python benchmarks/regression_tracker.py

# Check if regression history exists and show trend
if [ -f "benchmarks/regression_history.json" ]; then
    echo ""
    echo "======================================"
    echo "Performance Trend (Last 5 runs)"
    echo "======================================"
    
    python -c "
import json
with open('benchmarks/regression_history.json') as f:
    data = json.load(f)
    benchmarks = data['benchmarks'][-5:]
    
    for b in benchmarks:
        print(f\"\n{b['date']}:\")
        for r in b['results']:
            if 'error' not in r:
                print(f\"  {r['name']:8s}: {r['insertion_throughput']:,} vec/s, {r['search_p50_ms']:.2f}ms\")
    "
fi

echo ""
echo "To run this regularly, add to crontab:"
echo "0 */6 * * * cd $(pwd) && ./benchmarks/track_metrics.sh >> benchmarks/metrics.log 2>&1"