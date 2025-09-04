#!/bin/bash

# OmenDB Test Runner
#
# This script runs test using Mojo's built-in test framework with proper module imports.
# Uses `mojo test -I omendb test` pattern to add omendb/ to the import search path.
#
# Usage: ./run-test.sh [test_file_or_pattern]
# Examples:
#   ./run-test.sh                           # Run all test
#   ./run-test.sh test_vector               # Run specific test
#   ./run-test.sh core                      # Run all core test
#   ./run-test.sh core/test_vector.mojo     # Run specific test file

set -e

# Colors for better output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Check if pixi is available
if ! command -v pixi &>/dev/null; then
    echo -e "${RED}Error: pixi is not installed or not in PATH${NC}"
    echo "Please install pixi package manager first."
    exit 1
fi

# Check if we're in the project root
if [ ! -f "pixi.toml" ] || [ ! -d "omendb" ] || [ ! -d "test" ]; then
    echo -e "${RED}Error: This script must be run from the OmenDB project root directory${NC}"
    echo "Current directory: $(pwd)"
    echo "Expected files: pixi.toml, omendb/, test/"
    exit 1
fi

# Function to list available test
list_test() {
    echo -e "${YELLOW}Available test categories:${NC}"
    for dir in test/*/; do
        if [ -d "$dir" ]; then
            category=$(basename "$dir")
            echo -e "  ${GREEN}$category${NC}"
            if ls "$dir"*.mojo 1>/dev/null 2>&1; then
                ls "$dir"*.mojo | sed "s|$dir||" | sed 's/\.mojo$//' | sed 's/^/    /'
            fi
        fi
    done
    echo ""
    echo -e "${YELLOW}Usage examples:${NC}"
    echo -e "  ${CYAN}./run-test.sh${NC}                    - Run all test"
    echo -e "  ${CYAN}./run-test.sh core${NC}               - Run all core test"
    echo -e "  ${CYAN}./run-test.sh test_vector${NC}        - Run specific test"
    echo -e "  ${CYAN}./run-test.sh core/test_vector.mojo${NC} - Run specific test file"
}

# Function to run test with mojo test command
run_test() {
    local test_pattern="$1"
    local test_path="test"

    if [ -n "$test_pattern" ]; then
        # Check if it's a specific file
        if [[ "$test_pattern" == *.mojo ]]; then
            test_path="test/$test_pattern"
        elif [ -d "test/$test_pattern" ]; then
            # It's a category directory
            test_path="test/$test_pattern"
        elif [ -f "test/$test_pattern.mojo" ]; then
            # It's a test file without extension
            test_path="test/$test_pattern.mojo"
        else
            # Search for the test in subdirectories
            found_test=""
            for test_dir in test/*/; do
                if [ -f "$test_dir$test_pattern.mojo" ]; then
                    found_test="$test_dir$test_pattern.mojo"
                    break
                fi
            done

            if [ -n "$found_test" ]; then
                test_path="$found_test"
            else
                echo -e "${RED}Error: Test '$test_pattern' not found${NC}"
                echo ""
                list_test
                return 1
            fi
        fi
    fi

    if [ ! -e "$test_path" ]; then
        echo -e "${RED}Error: Test path '$test_path' does not exist${NC}"
        return 1
    fi

    echo -e "${BLUE}🧪 Running OmenDB test${NC}"
    echo -e "${BLUE}========================${NC}"
    echo -e "${CYAN}Test path: $test_path${NC}"
    echo -e "${CYAN}Command: mojo test -I omendb $test_path${NC}"
    echo ""

    # Run the test using Mojo's test framework
    # -I omendb adds the omendb directory to the import search path
    # This allows test to import like: from core.vector import Vector
    if pixi run mojo test -I omendb "$test_path"; then
        echo ""
        echo -e "${GREEN}✅ Tests completed successfully!${NC}"
        return 0
    else
        echo ""
        echo -e "${RED}❌ Some test failed${NC}"
        return 1
    fi
}

# Main script logic
if [ $# -eq 0 ]; then
    echo -e "${BLUE}OmenDB Test Runner${NC}"
    echo -e "${BLUE}==================${NC}"
    echo ""
    echo -e "${YELLOW}Running all test...${NC}"
    echo ""
    run_test
elif [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo -e "${BLUE}OmenDB Test Runner${NC}"
    echo -e "${BLUE}==================${NC}"
    echo ""
    echo -e "${YELLOW}Usage:${NC} $0 [test_file_or_pattern]"
    echo ""
    echo -e "${YELLOW}Examples:${NC}"
    echo -e "  $0                           ${CYAN}# Run all test${NC}"
    echo -e "  $0 core                      ${CYAN}# Run all core test${NC}"
    echo -e "  $0 test_vector               ${CYAN}# Run specific test${NC}"
    echo -e "  $0 core/test_vector.mojo     ${CYAN}# Run specific test file${NC}"
    echo ""
    list_test
    echo ""
    echo -e "${YELLOW}Note:${NC} Tests import modules directly from omendb/ using: ${CYAN}from core.vector import Vector${NC}"
    echo -e "${YELLOW}      The -I omendb flag adds omendb/ to Mojo's import search path.${NC}"
else
    run_test "$1"
fi

exit $?
