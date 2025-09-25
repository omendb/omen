#!/bin/bash
# Clone reference implementations

echo "Cloning reference implementations..."
git clone --depth 1 https://github.com/learnedsystems/RMI.git 2>/dev/null || echo "RMI already exists"
git clone --depth 1 https://github.com/learnedsystems/SOSD.git 2>/dev/null || echo "SOSD already exists"
git clone --depth 1 https://github.com/learnedsystems/RadixSpline.git 2>/dev/null || echo "RadixSpline already exists"
git clone --depth 1 https://github.com/microsoft/ALEX.git 2>/dev/null || echo "ALEX already exists"

echo "Reference implementations ready"
ls -la
