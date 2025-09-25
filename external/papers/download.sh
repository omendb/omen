#!/bin/bash
# Download essential learned index papers

echo "Downloading core papers..."
wget -q https://arxiv.org/pdf/1712.01208.pdf -O 01-learned-index-structures.pdf
wget -q https://arxiv.org/pdf/1905.08898.pdf -O 02-alex-updatable.pdf  
wget -q https://arxiv.org/pdf/2004.14541.pdf -O 03-radixspline.pdf
wget -q https://arxiv.org/pdf/1911.13014.pdf -O 04-sosd-benchmark.pdf

echo "Downloading implementation papers..."
wget -q https://arxiv.org/pdf/1910.06169.pdf -O 05-pgm-index.pdf
wget -q https://arxiv.org/pdf/2104.05520.pdf -O 06-lipp-updatable.pdf

echo "Papers downloaded to external/papers/"
ls -la *.pdf 2>/dev/null || echo "No PDFs yet - run wget commands manually"
