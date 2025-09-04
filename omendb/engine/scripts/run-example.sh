#!/bin/bash

# OmenDB Example Runner
#
# This script runs examples from the root directory to ensure proper module imports.
# Due to Mojo's module resolution system, examples cannot be run directly from the
# examples/ directory - they must be executed from the project root.
#
# Usage: ./run-example.sh <example_name>
# Example: ./run-example.sh vector_operations_demo

set -e

# Colors for better output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if pixi is available
if ! command -v pixi &>/dev/null; then
    echo -e "${RED}Error: pixi is not installed or not in PATH${NC}"
    echo "Please install pixi package manager first."
    exit 1
fi

# Check if example name is provided
if [ $# -eq 0 ]; then
    echo -e "${BLUE}OmenDB Example Runner${NC}"
    echo -e "${BLUE}=====================${NC}"
    echo ""
    echo -e "${YELLOW}Usage:${NC} $0 <example_name>"
    echo ""
    echo -e "${YELLOW}Recommended examples (start here):${NC}"
    echo -e "  ${GREEN}vector_operations_demo${NC}  - ⭐ Complete Phase 1 MVP demonstration"
    echo -e "  ${GREEN}basic_usage${NC}             - Simple introduction to OmenDB"
    echo -e "  ${GREEN}enhanced_storage_example${NC} - Storage and persistence features"
    echo ""
    echo -e "${YELLOW}All available examples:${NC}"
    if ls examples/*.mojo 1>/dev/null 2>&1; then
        ls examples/*.mojo | sed 's/examples\//  /' | sed 's/\.mojo$//'
    else
        echo "  No examples found in examples/ directory"
    fi
    echo ""
    echo -e "${YELLOW}Note:${NC} Examples must be run from the project root directory due to Mojo's module system."
    exit 1
fi

EXAMPLE_NAME="$1"
EXAMPLE_FILE="examples/${EXAMPLE_NAME}.mojo"

# Check if example file exists
if [ ! -f "$EXAMPLE_FILE" ]; then
    echo -e "${RED}Error: Example file '$EXAMPLE_FILE' not found${NC}"
    echo ""
    echo -e "${YELLOW}Available examples:${NC}"
    if ls examples/*.mojo 1>/dev/null 2>&1; then
        ls examples/*.mojo | sed 's/examples\//  /' | sed 's/\.mojo$//'
    else
        echo "  No examples found in examples/ directory"
    fi
    exit 1
fi

echo -e "${BLUE}🚀 Running OmenDB example: ${GREEN}$EXAMPLE_NAME${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""

# Run the example from the root directory using pixi
if pixi run mojo "$EXAMPLE_FILE"; then
    echo ""
    echo -e "${GREEN}✅ Example completed successfully!${NC}"
    echo -e "${YELLOW}💡 Try other examples or check examples/README.md for more information.${NC}"
else
    echo ""
    echo -e "${RED}❌ Example failed to run.${NC}"
    echo -e "${YELLOW}💡 Check the error messages above and ensure you're running from the project root.${NC}"
    exit 1
fi
