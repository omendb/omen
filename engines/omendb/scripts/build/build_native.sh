#!/bin/bash
# Build native.mojo as a Python module

echo "🔨 Building OmenDB native module..."

# Change to omendb directory
cd omendb

# Build the module
echo "Building native.mojo..."
pixi run mojo build native.mojo -o ../python/omendb/native.mojopkg

# Check if build succeeded
if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    
    # Try to create native.so from mojopkg
    cd ../python/omendb
    
    # For now, we'll use the existing native.so if available
    if [ -f "../../omendb/native.mojopkg" ]; then
        echo "📦 Native module package created"
    fi
else
    echo "❌ Build failed"
    exit 1
fi