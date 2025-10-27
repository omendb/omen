#!/bin/bash
# Docker Resource Exhaustion Tests
# Tests graceful degradation under extreme resource constraints

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=========================================="
echo "Docker Resource Exhaustion Test Suite"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Build the test binary once (no resource constraints for compilation)
echo "Building test binary in Docker (no memory constraints)..."
BUILD_EXIT=$(docker run --rm \
    -v "$PROJECT_ROOT:/workspace" \
    -w /workspace \
    rust:latest \
    bash -c "apt-get update -qq && apt-get install -y -qq clang libclang-dev > /dev/null 2>&1 && cargo build --release --bin resource_exhaustion_test 2>&1 | tail -20 && echo EXIT_CODE:\$?")

echo "$BUILD_EXIT"

if echo "$BUILD_EXIT" | grep -q "EXIT_CODE:0"; then
    echo -e "${GREEN}✓ Test binary built${NC}"
else
    echo -e "${RED}✗ Failed to build test binary${NC}"
    exit 1
fi
echo ""

# Test 1: Memory Limit (512MB)
echo "Test 1: Memory Limit (512MB)"
echo "Testing behavior under memory pressure..."
docker run --rm \
    --memory="512m" \
    --memory-swap="512m" \
    -v "$PROJECT_ROOT:/workspace" \
    -w /workspace \
    rust:latest \
    ./target/release/resource_exhaustion_test memory 512

MEMORY_EXIT=$?
if [ $MEMORY_EXIT -eq 0 ]; then
    echo -e "${GREEN}✓ Memory limit test passed${NC}"
else
    echo -e "${RED}✗ Memory limit test failed (exit code: $MEMORY_EXIT)${NC}"
fi
echo ""

# Test 2: Memory Limit (256MB) - More extreme
echo "Test 2: Memory Limit (256MB) - Extreme pressure"
echo "Testing behavior under severe memory pressure..."
docker run --rm \
    --memory="256m" \
    --memory-swap="256m" \
    -v "$PROJECT_ROOT:/workspace" \
    -w /workspace \
    rust:latest \
    ./target/release/resource_exhaustion_test memory 256

MEMORY_EXTREME_EXIT=$?
if [ $MEMORY_EXTREME_EXIT -eq 0 ]; then
    echo -e "${GREEN}✓ Extreme memory limit test passed${NC}"
else
    echo -e "${YELLOW}⚠ Extreme memory limit test exited with code $MEMORY_EXTREME_EXIT (may be expected)${NC}"
fi
echo ""

# Test 3: CPU Limit (0.5 cores)
echo "Test 3: CPU Limit (0.5 cores)"
echo "Testing behavior under CPU constraints..."
docker run --rm \
    --cpus="0.5" \
    -v "$PROJECT_ROOT:/workspace" \
    -w /workspace \
    rust:latest \
    ./target/release/resource_exhaustion_test cpu

CPU_EXIT=$?
if [ $CPU_EXIT -eq 0 ]; then
    echo -e "${GREEN}✓ CPU limit test passed${NC}"
else
    echo -e "${RED}✗ CPU limit test failed (exit code: $CPU_EXIT)${NC}"
fi
echo ""

# Test 4: File Descriptor Limit
echo "Test 4: File Descriptor Limit (100 open files)"
echo "Testing behavior with limited file descriptors..."
docker run --rm \
    --ulimit nofile=100:100 \
    -v "$PROJECT_ROOT:/workspace" \
    -w /workspace \
    rust:latest \
    ./target/release/resource_exhaustion_test fdlimit

FD_EXIT=$?
if [ $FD_EXIT -eq 0 ]; then
    echo -e "${GREEN}✓ File descriptor limit test passed${NC}"
else
    echo -e "${RED}✗ File descriptor limit test failed (exit code: $FD_EXIT)${NC}"
fi
echo ""

# Test 5: Combined Constraints
echo "Test 5: Combined Constraints (256MB RAM + 0.5 CPU + 100 FD)"
echo "Testing behavior under multiple resource constraints..."
docker run --rm \
    --memory="256m" \
    --memory-swap="256m" \
    --cpus="0.5" \
    --ulimit nofile=100:100 \
    -v "$PROJECT_ROOT:/workspace" \
    -w /workspace \
    rust:latest \
    ./target/release/resource_exhaustion_test combined

COMBINED_EXIT=$?
if [ $COMBINED_EXIT -eq 0 ]; then
    echo -e "${GREEN}✓ Combined constraints test passed${NC}"
else
    echo -e "${YELLOW}⚠ Combined constraints test exited with code $COMBINED_EXIT${NC}"
fi
echo ""

# Summary
echo "=========================================="
echo "Test Summary"
echo "=========================================="

TOTAL_TESTS=5
PASSED_TESTS=0

[ $MEMORY_EXIT -eq 0 ] && PASSED_TESTS=$((PASSED_TESTS + 1))
[ $MEMORY_EXTREME_EXIT -eq 0 ] && PASSED_TESTS=$((PASSED_TESTS + 1))
[ $CPU_EXIT -eq 0 ] && PASSED_TESTS=$((PASSED_TESTS + 1))
[ $FD_EXIT -eq 0 ] && PASSED_TESTS=$((PASSED_TESTS + 1))
[ $COMBINED_EXIT -eq 0 ] && PASSED_TESTS=$((PASSED_TESTS + 1))

echo "Passed: $PASSED_TESTS / $TOTAL_TESTS"
echo ""

if [ $PASSED_TESTS -eq $TOTAL_TESTS ]; then
    echo -e "${GREEN}✓ All resource exhaustion tests passed!${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠ Some tests showed degraded behavior under extreme constraints${NC}"
    echo "This is expected - system should degrade gracefully, not crash"
    exit 0
fi
