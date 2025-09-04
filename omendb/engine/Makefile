# OmenDB Makefile
# Convenient commands for building, testing, and development

.PHONY: help build test test-core test-storage test-index test-compression test-embedded examples clean dev install check lint format benchmark benchmark-quick benchmark-standard benchmark-enterprise benchmark-clean benchmark-deps

# Default target
help:
	@echo "OmenDB Development Commands"
	@echo "=========================="
	@echo ""
	@echo "Building:"
	@echo "  build          Build the main OmenDB binary"
	@echo "  build-debug    Build with debug symbols and memory checking"
	@echo ""
	@echo "Testing:"
	@echo "  test           Run all tests"
	@echo "  test-core      Run core module tests"
	@echo "  test-storage   Run storage module tests"
	@echo "  test-index     Run indexing module tests"
	@echo "  test-compression Run compression module tests"
	@echo "  test-embedded  Run embedded database tests"
	@echo ""
	@echo "Benchmarking:"
	@echo "  benchmark-quick       Quick benchmark (1K-10K vectors)"
	@echo "  benchmark-standard    Standard benchmark (1K-100K vectors)"
	@echo "  benchmark-enterprise  Enterprise benchmark (10K-500K vectors)"
	@echo "  benchmark-clean       Clean benchmark results"
	@echo "  benchmark-deps        Install benchmark dependencies"
	@echo ""
	@echo "Examples:"
	@echo "  examples       List available examples"
	@echo "  run-vector     Run vector operations demo"
	@echo "  run-search     Run search demo"
	@echo "  run-embedded   Run embedded database demo"
	@echo ""
	@echo "Development:"
	@echo "  dev            Enter development environment (pixi shell)"
	@echo "  install        Install dependencies"
	@echo "  check          Run type checking and validation"
	@echo "  clean          Clean build artifacts"
	@echo ""
	@echo "Environment:"
	@echo "  status         Show project status and environment info"

# Build commands
build:
	@echo "🔨 Building OmenDB native module..."
	pixi run mojo build -I omendb omendb/native.mojo -o python/omendb/native.so --emit shared-lib
	@echo "✅ Build complete!"

build-old:
	@echo "🔨 Building OmenDB..."
	pixi run mojo build -I omendb src/main.mojo

build-debug:
	@echo "🔨 Building OmenDB with debug symbols..."
	pixi run mojo build -D DEBUG=1 --debug-memory -I omendb src/main.mojo

# Test commands
test:
	@echo "🧪 Running all tests..."
	./scripts/run-test.sh

test-core:
	@echo "🧪 Running core module tests..."
	./scripts/run-test.sh core

test-storage:
	@echo "🧪 Running storage module tests..."
	./scripts/run-test.sh storage

test-index:
	@echo "🧪 Running indexing module tests..."
	./scripts/run-test.sh index

test-compression:
	@echo "🧪 Running compression module tests..."
	./scripts/run-test.sh compression

test-embedded:
	@echo "🧪 Running embedded database tests..."
	./scripts/run-test.sh embedded

# DiskANN specific tests
test-diskann:
	@echo "🧪 Testing DiskANN implementation..."
	PYTHONPATH=python pixi run python test_diskann_simple.py

test-hnsw:
	@echo "🧪 Testing HNSW implementation..."
	PYTHONPATH=python pixi run python test_hnsw_simple.py

test-algorithms:
	@echo "🧪 Testing all algorithms..."
	@make test-hnsw
	@make test-diskann

debug-diskann:
	@echo "🐛 Debugging DiskANN issues..."
	@echo "1. Checking native module build..."
	@make build
	@echo "2. Testing DiskANN initialization..."
	PYTHONPATH=python pixi run python -c 'import omendb; db = omendb.DB(algorithm="diskann", buffer_size=10); print("DiskANN init successful, vectors:", db.count())'
	@echo "3. Testing basic add/search..."
	PYTHONPATH=python pixi run python -c 'import omendb; db = omendb.DB(algorithm="diskann"); db.add("test", [1.0, 2.0, 3.0]); results = db.search([1.0, 2.0, 3.0], limit=1); print("Search results:", len(results), results)'

quick-test: build
	@echo "⚡ Running quick functional tests..."
	@make test-algorithms

# Example commands
examples:
	@echo "📚 Available examples:"
	./scripts/run-example.sh

run-vector:
	@echo "🚀 Running vector operations demo..."
	./scripts/run-example.sh vector_operations_demo

run-search:
	@echo "🔍 Running search demo..."
	./scripts/run-example.sh search_demo

run-embedded:
	@echo "💾 Running embedded database demo..."
	./scripts/run-example.sh embedded_demo

# Development commands
dev:
	@echo "🛠️  Entering development environment..."
	@echo "Run 'exit' to leave the environment"
	pixi shell

install:
	@echo "📦 Installing dependencies..."
	pixi install

check:
	@echo "✅ Running type checking..."
	@echo "Note: Mojo uses built-in type checking during compilation"
	pixi run mojo --version
	@echo "Environment ready for development"

clean:
	@echo "🧹 Cleaning build artifacts..."
	@rm -f main
	@rm -rf __pycache__
	@rm -rf .mojo_cache
	@find . -name "*.pyc" -delete
	@echo "Clean complete"

# Status and info
status:
	@echo "📊 OmenDB Project Status"
	@echo "======================="
	@echo ""
	@echo "Environment:"
	@which pixi || echo "  ❌ pixi not found"
	@pixi --version 2>/dev/null || echo "  ❌ pixi not working"
	@pixi run mojo --version 2>/dev/null || echo "  ❌ mojo not available"
	@echo ""
	@echo "Project Structure:"
	@test -d omendb && echo "  ✅ omendb/ source directory" || echo "  ❌ omendb/ missing"
	@test -d tests && echo "  ✅ tests/ directory" || echo "  ❌ tests/ missing"
	@test -d examples && echo "  ✅ examples/ directory" || echo "  ❌ examples/ missing"
	@test -d docs && echo "  ✅ docs/ directory" || echo "  ❌ docs/ missing"
	@test -f pixi.toml && echo "  ✅ pixi.toml configuration" || echo "  ❌ pixi.toml missing"
	@echo ""
	@echo "Quick Start:"
	@echo "  make dev        # Enter development environment"
	@echo "  make test       # Run all tests"
	@echo "  make examples   # See available examples"

# Advanced targets for CI/CD
ci-test: install test
	@echo "✅ CI test pipeline completed"

ci-build: install build
	@echo "✅ CI build pipeline completed"

# Quick development workflow
quick: test examples
	@echo "🚀 Quick development check completed"

# Comprehensive Benchmarking System
# =================================

# Quick benchmark for development and CI
benchmark-quick:
	@echo "🚀 Running Quick Benchmark Suite"
	@echo "================================="
	@echo "Testing 1K-10K vectors with basic competitors"
	@echo ""
	pixi run python benchmarks/comprehensive_benchmark.py --quick
	@echo ""
	@echo "✅ Quick benchmark complete!"
	@echo "📊 Results saved to benchmarks/results/"

# Standard benchmark suite
benchmark-standard:
	@echo "🚀 Running Standard Benchmark Suite"
	@echo "===================================="
	@echo "Testing 1K-100K vectors with full competitor analysis"
	@echo ""
	pixi run python benchmarks/comprehensive_benchmark.py --standard
	@echo ""
	@echo "✅ Standard benchmark complete!"
	@echo "📊 Results saved to benchmarks/results/"

# Enterprise-scale benchmark
benchmark-enterprise:
	@echo "🚀 Running Enterprise Benchmark Suite"
	@echo "======================================"
	@echo "⚠️  Warning: This may take 30+ minutes and require 8GB+ RAM"
	@echo "Testing 10K-500K vectors with full stress testing"
	@echo ""
	@read -p "Continue? (y/N): " confirm && [ "$$confirm" = "y" ] || [ "$$confirm" = "Y" ]
	pixi run python benchmarks/comprehensive_benchmark.py --enterprise
	@echo ""
	@echo "✅ Enterprise benchmark complete!"
	@echo "📊 Results saved to benchmarks/results/"

# Default benchmark (standard)
benchmark: benchmark-standard

# Clean benchmark results
benchmark-clean:
	@echo "🧹 Cleaning benchmark results..."
	@mkdir -p benchmarks/results
	@rm -f benchmarks/results/*
	@echo "✅ Benchmark results cleaned."

# Install benchmark dependencies
benchmark-deps:
	@echo "📦 Installing benchmark dependencies..."
	@echo "Installing core dependencies..."
	pixi add numpy psutil
	@echo "Installing competitor libraries..."
	@echo "  - ChromaDB (embedded vector database)"
	pixi add --pypi chromadb
	@echo "  - Faiss (Facebook AI Similarity Search)"
	pixi add --pypi faiss-cpu
	@echo "  - Qdrant (high-performance vector database)"
	pixi add --pypi qdrant-client
	@echo ""
	@echo "✅ Dependencies installed."
	@echo ""
	@echo "🚀 Testing benchmark system..."
	@echo "Running quick validation test..."
	pixi run python benchmarks/comprehensive_benchmark.py --quick --no-stress --no-embeddings
	@echo ""
	@echo "✅ Benchmark system ready!"
	@echo "Run 'make benchmark-quick' to start benchmarking"

# Specific competitor benchmarks
benchmark-omendb:
	@echo "🎯 OmenDB-only benchmark"
	pixi run python benchmarks/comprehensive_benchmark.py --standard --competitors omendb

benchmark-chromadb:
	@echo "🎯 ChromaDB comparison benchmark"
	pixi run python benchmarks/comprehensive_benchmark.py --standard --competitors omendb chromadb

benchmark-faiss:
	@echo "🎯 Faiss comparison benchmark"
	pixi run python benchmarks/comprehensive_benchmark.py --standard --competitors omendb faiss

benchmark-qdrant:
	@echo "🎯 Qdrant comparison benchmark"
	pixi run python benchmarks/comprehensive_benchmark.py --standard --competitors omendb qdrant

# System requirements check
benchmark-check:
	@echo "🔍 Benchmark System Requirements Check"
	@echo "======================================"
	@echo "Platform: $$(uname -s) $$(uname -m)"
	@echo "Python: $$(python --version 2>/dev/null || echo 'Not found')"
	@echo "Available Memory: $$(free -h 2>/dev/null | grep '^Mem:' | awk '{print $$7}' || echo 'Unknown')"
	@echo "Disk Space: $$(df -h . | tail -1 | awk '{print $$4}' || echo 'Unknown')"
	@echo ""
	@echo "📊 Benchmark Dependencies:"
	@pixi run python -c "import numpy; print('  ✅ NumPy:', numpy.__version__)" 2>/dev/null || echo "  ❌ NumPy not available"
	@pixi run python -c "import psutil; print('  ✅ psutil:', psutil.__version__)" 2>/dev/null || echo "  ❌ psutil not available"
	@pixi run python -c "import chromadb; print('  ✅ ChromaDB:', chromadb.__version__)" 2>/dev/null || echo "  ⚠️  ChromaDB not available"
	@pixi run python -c "import faiss; print('  ✅ Faiss: available')" 2>/dev/null || echo "  ⚠️  Faiss not available"
	@pixi run python -c "import qdrant_client; print('  ✅ Qdrant: available')" 2>/dev/null || echo "  ⚠️  Qdrant not available"
	@echo ""
	@echo "💡 Recommendations:"
	@echo "  Quick benchmark:      2GB RAM, 1GB disk space"
	@echo "  Standard benchmark:   4GB RAM, 2GB disk space"
	@echo "  Enterprise benchmark: 8GB RAM, 5GB disk space"
	@echo ""
	@echo "🚀 To install missing dependencies: make benchmark-deps"

# Results analysis
benchmark-results:
	@echo "📊 Benchmark Results Analysis"
	@echo "============================="
	@echo "Results directory: benchmarks/results/"
	@echo ""
	@echo "📁 Available result files:"
	@ls -la benchmarks/results/ 2>/dev/null || echo "  No results found. Run 'make benchmark-quick' first."
	@echo ""
	@echo "📋 Latest summary:"
	@find benchmarks/results/ -name "benchmark_summary_*.txt" -type f -exec ls -t {} + 2>/dev/null | head -1 | xargs cat || echo "  No summary files found"
	@echo ""
	@echo "💡 To view detailed results, check benchmarks/results/benchmark_results_*.json"