#!/bin/bash
set -e

# Build native Python extension module for OmenDB
# Direct compilation approach

echo "🔨 Building OmenDB native module (direct compilation)..."

# Get the directory of this script
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

# Set paths
SOURCE_DIR="$PROJECT_ROOT/omendb"
PYTHON_DIR="$PROJECT_ROOT/python/omendb"
BUILD_DIR="$PROJECT_ROOT/build"

# Create directories
mkdir -p "$PYTHON_DIR"
mkdir -p "$BUILD_DIR"

# Copy required runtime libraries first
if [ -d "$PROJECT_ROOT/libs" ]; then
    echo "📚 Setting up runtime libraries..."
    mkdir -p "$PYTHON_DIR/lib"
    cp -f "$PROJECT_ROOT/libs/"*.dylib "$PYTHON_DIR/lib/" 2>/dev/null || true
    cp -f "$PROJECT_ROOT/libs/"*.so "$PYTHON_DIR/lib/" 2>/dev/null || true
fi

# Build the native module directly
echo "📦 Compiling native.mojo -> native.so"
cd "$PROJECT_ROOT"

# Use pixi to run mojo with correct environment and import path
if command -v pixi &> /dev/null; then
    echo "Using pixi environment..."
    # Build as shared library with import path
    pixi run mojo build -I omendb omendb/native.mojo -o "$PYTHON_DIR/native.so" --dynamic-library --no-main
else
    echo "Using system mojo..."
    mojo build -I omendb omendb/native.mojo -o "$PYTHON_DIR/native.so" --dynamic-library --no-main
fi

if [ $? -eq 0 ]; then
    echo "✅ Native module built successfully!"
    echo "📍 Output: $PYTHON_DIR/native.so"
    
    # Verify the build
    if [ -f "$PYTHON_DIR/native.so" ]; then
        echo "✅ native.so exists with size: $(du -h "$PYTHON_DIR/native.so" | cut -f1)"
        
        # Test import
        echo "🧪 Testing module import..."
        cd "$PROJECT_ROOT"
        if pixi run python -c "import sys; sys.path.insert(0, 'python'); import omendb; print('✅ Module imports successfully!')"; then
            echo "🎉 Build complete and verified!"
        else
            echo "⚠️  Module built but import test failed - this is normal if dependencies aren't set up yet"
        fi
    else
        echo "❌ native.so not found after build!"
        exit 1
    fi
else
    echo "❌ Build failed!"
    exit 1
fi