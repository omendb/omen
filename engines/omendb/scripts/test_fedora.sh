#!/bin/bash
# Test script for OmenDB on Fedora/Intel/NVIDIA system

echo "=== OmenDB Fedora Testing Script ==="
echo "Testing on Intel/NVIDIA Linux system"
echo

# System info
echo "1. System Information:"
echo "---------------------"
uname -a
echo
lscpu | grep -E "Model name|CPU\(s\)|Thread|Core|Socket|NUMA|CPU MHz|Architecture"
echo
nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv 2>/dev/null || echo "No NVIDIA GPU detected"
echo

# Python environment
echo "2. Python Environment:"
echo "---------------------"
python3 --version
pip3 --version || echo "pip3 not installed"
echo

# Test basic functionality
echo "3. Testing OmenDB Import:"
echo "------------------------"
cd /home/nick/github/omendb/omenDB
export PYTHONPATH=python:$PYTHONPATH

python3 -c "
import sys
sys.path.insert(0, 'python')
try:
    import omendb
    print('✅ Import successful')
    db = omendb.DB()
    print(f'✅ DB created: {db}')
except Exception as e:
    print(f'❌ Error: {e}')
"

# Test performance
echo
echo "4. Quick Performance Test:"
echo "-------------------------"
python3 -c "
import sys
import time
sys.path.insert(0, 'python')
try:
    import omendb
    import random
    
    db = omendb.DB()
    
    # Test with 128D vectors
    dim = 128
    num_vectors = 1000
    
    # Generate random vectors
    vectors = [[random.random() for _ in range(dim)] for _ in range(num_vectors)]
    ids = [f'vec_{i}' for i in range(num_vectors)]
    
    # Time insertion
    start = time.perf_counter()
    db.add_batch([(ids[i], vectors[i], {}) for i in range(num_vectors)])
    insert_time = time.perf_counter() - start
    
    # Time query
    query = [random.random() for _ in range(dim)]
    start = time.perf_counter()
    results = db.search(query, limit=10)
    query_time = (time.perf_counter() - start) * 1000  # ms
    
    print(f'Inserted {num_vectors} vectors in {insert_time:.3f}s')
    print(f'Insertion rate: {num_vectors/insert_time:.0f} vec/s')
    print(f'Query time: {query_time:.2f}ms')
    print(f'Found {len(results)} results')
    
except Exception as e:
    print(f'❌ Performance test failed: {e}')
"

echo
echo "5. Compare with HNSW (if available):"
echo "-----------------------------------"
python3 -c "
try:
    import hnswlib
    print('✅ HNSW available for comparison')
except:
    print('❌ HNSW not installed - install with: pip3 install hnswlib')
"

echo
echo "=== Testing Complete ==="
echo
echo "Next steps:"
echo "1. Install pixi: curl -fsLS https://pixi.sh/install.sh | bash"
echo "2. Run full test suite: pixi run python test/python/test_api_standards.py"
echo "3. Run benchmarks: pixi run python benchmarks/comprehensive.py"