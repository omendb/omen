# OmenDB Development Setup

**Environment setup for OmenDB contributors and developers**

## 🛠️ Prerequisites

### Required Tools
- **Mojo 24.5+** - [Install from Modular](https://www.modular.com/download)
- **Python 3.9+** - For Python bindings and testing
- **Pixi** - Package manager (recommended)
- **Git** - Version control

### System Requirements
- **macOS** (Intel/Apple Silicon) or **Linux** (x86_64)
- **8GB+ RAM** - For compilation and testing
- **10GB+ disk space** - For dependencies and cache

## 🚀 Quick Setup

### 1. Clone Repository
```bash
git clone https://github.com/your-org/omendb
cd omendb
```

### 2. Install Dependencies
```bash
# Using Pixi (recommended)
pixi install
pixi shell

# Or manual setup
mojo --version  # Verify Mojo installation
python -m pip install -r requirements.txt
```

### 3. Build Native Module  
```bash
# Build Mojo native module
mojo build omendb/native.mojo -o omendb_native.so

# Test build
python -c "import omendb; print('✅ OmenDB imported successfully')"
```

### 4. Run Tests
```bash
# Quick validation
python test_current_status.py

# Full test suite
python -m pytest test/ -v

# Native module tests
mojo -I omendb omendb/native.mojo
```

## 📁 Project Structure

### Core Implementation
```
omendb/
├── native.mojo              # Main native module (Python-Mojo interop)
├── algorithms/roar_graph.mojo  # RoarGraph algorithm (PRIMARY)
├── index/hnsw_index.mojo    # HNSW algorithm (fallback)
├── core/                    # Vector operations, distance metrics
└── storage/                 # File persistence, WAL

python/omendb/
├── api.py                   # Modern Python API (DB, RoarGraphIndex)
├── __init__.py              # Module exports and lazy loading
└── exceptions.py            # Error handling framework
```

### Development Files
```
test/                       # Test suite
benchmarks/                  # Performance benchmarks  
examples/                    # Usage examples
docs/dev/                    # Developer documentation
```

## 🧪 Development Workflow

### 1. Code Changes
```bash
# Make changes to Mojo files
vim omendb/algorithms/roar_graph.mojo

# Rebuild native module
mojo build omendb/native.mojo -o omendb_native.so

# Test changes
python test_roargraph_integration.py
```

### 2. Testing Protocol
```bash
# Reality check - what actually works
python test_current_status.py

# Test specific components  
python test/test_roargraph_python_api.py

# Performance validation
python benchmarks/roar_graph_vs_hnsw.mojo
```

### 3. Code Formatting
```bash
# Format Mojo code
mojo format omendb/ test/

# Format Python code
black python/omendb/
isort python/omendb/
```

## 🐛 Troubleshooting

### Common Issues

#### "No module named 'max'"
```bash
# Verify Modular installation
modular --version

# Update Mojo
modular update mojo
```

#### Native Module Import Errors
```bash
# Rebuild native module
rm omendb_native.so omendb/*.so
mojo build omendb/native.mojo -o omendb_native.so

# Check Python path
export PYTHONPATH=$PWD/python:$PYTHONPATH
```

#### Compilation Errors
```bash
# Clean cache
rm -rf __mojocache__/ omendb/__mojocache__/

# Full rebuild
pixi run clean
pixi run build
```

### Performance Issues
```bash
# Profile memory usage
python -m memory_profiler test_current_status.py

# Profile performance
python -m cProfile -o profile.stats test_roargraph_integration.py
```

## 📋 Development Guidelines

### Code Style
- **Mojo**: Follow stdlib conventions, use type hints
- **Python**: Black formatting, type hints, docstrings
- **Commit messages**: Conventional commits format

### Testing Requirements
- **All features must have tests** - No feature complete without validation
- **Reality check first** - Use `test_current_status.py` before claiming working features
- **Performance validation** - Benchmark claims with real data

### Architecture Rules
- **RoarGraph is primary** - Focus indexing work on RoarGraph algorithm
- **HNSW is fallback** - Maintain but don't actively develop
- **Embedded-first** - Optimize for single-process deployment

## 🔧 Advanced Setup

### Custom Build Options
```bash
# Debug build
mojo build -D DEBUG omendb/native.mojo

# Release with optimizations  
mojo build -O3 omendb/native.mojo

# Profile build
mojo build --debug-info omendb/native.mojo
```

### IDE Configuration
- **VS Code**: Mojo extension + Python extension
- **Vim/Neovim**: Mojo syntax highlighting
- **PyCharm**: Python-only development

---

**Need help?** Check [troubleshooting guide](troubleshooting.md) or open an issue on GitHub