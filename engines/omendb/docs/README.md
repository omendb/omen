# OmenDB Documentation

Welcome to OmenDB documentation. **Major Update: Vamana implementation eliminates all performance cliffs!**

## 🎉 Latest Achievement
- **[Vamana Success](performance/VAMANA_SUCCESS_SUMMARY.md)** - 100K vectors with NO cliffs (32,500x improvement!)
- **[Current Status](performance/STATUS.md)** - 85% production ready

## 📚 Documentation Structure

### Getting Started
- **[Quickstart Guide](quickstart.md)** - 5-minute introduction
- **[Python API Reference](api/python.md)** - Complete API documentation
- **[Why OmenDB?](why-omendb.md)** - Benchmarks and advantages

### User Guides
- **[API Reference](user/api-reference.md)** - Detailed API documentation
- **[Performance Guide](user/performance.md)** - Optimization tips ✅ Updated
- **[Quickstart](user/quickstart.md)** - Getting started guide
- **[Single Database Design](user/single-database-design.md)** - Architecture explanation

### Developer Documentation
- **[Testing Guide](dev/testing.md)** - How to run tests
- **[Setup Guide](dev/setup.md)** - Development environment
- **[Distribution](dev/distribution.md)** - Packaging and distribution
- **[Pixi Setup](dev/setup/pixi.md)** - Pixi environment setup
- **[Mojo Common Errors](dev/MOJO_COMMON_ERRORS.md)** - Troubleshooting Mojo
- **[Mojo Import Fixes](dev/troubleshooting/mojo-import-fixes.md)** - Import issues
- **[Common Issues](dev/troubleshooting/common-issues.md)** - General troubleshooting

### Architecture & Implementation
- **[Architecture Decisions](architecture/ARCHITECTURE_DECISIONS.md)** - Key design choices
- **[Vamana Design](implementation/ENTERPRISE_DISKANN_DESIGN.md)** - DiskANN architecture
- **[Migration Plan](implementation/VAMANA_MIGRATION_PLAN.md)** - Production rollout
- **[Performance Analysis](performance/PROJECT_STATUS_SUMMARY.md)** - Current metrics

### Guides
- **[Dimension Validation](guides/DIMENSION_VALIDATION.md)** - Working with dimensions
- **[Multi-Dimension Usage](guides/MULTI_DIMENSION_USAGE.md)** - Multiple dimension support

### Style Guides
- **[Documentation Style Guide](DOCUMENTATION_STYLE_GUIDE.md)** - Writing docs
- **[Mojo Style Guide](MOJO_STYLE_GUIDE.md)** - Code style
- **[Contributing](CONTRIBUTING.md)** - Contribution guidelines

### Key Decisions
- **[Vamana Refactor](decisions/VAMANA_REFACTOR_DECISION.md)** - Why we rewrote the core
- **[Dict vs SparseMap](decisions/DICT_VS_SPARSEMAP.md)** - Memory optimization choices
- **[Memory Guide](decisions/MEMORY_OPTIMIZATION_GUIDE.md)** - Optimization strategies

## 🎯 Quick Links

### Key Documents
- **[STATUS.md](../STATUS.md)** - 🎯 MASTER AUTHORITY for project status
- **[README.md](../README.md)** - Project introduction
- **[CHANGELOG.md](../CHANGELOG.md)** - Version history
- **[BUILD.md](../BUILD.md)** - Build instructions
- **[Examples](../examples/)** - 29 working code examples
- **[Benchmarks](../benchmarks/)** - Performance tests

### Verified Performance (December 2024 - Vamana)
- **Insertion**: 0.36ms average (100K vectors tested)
- **P99 Latency**: 0.42ms (incredibly predictable)
- **Max Vectors**: 100,000+ (no cliffs!)
- **Search**: 2-3ms at 100K scale
- **Memory**: 309 bytes/vector

### Known Issues
- Collections API disabled (Mojo limitation)
- Windows not supported (Mojo limitation)
- Fedora build issues with Mojo 25.4.0

### Example Issues Found
Many examples use suboptimal patterns:
```python
# Suboptimal: Converting arrays to lists reduces performance
db.add_batch(vectors=numpy_array.tolist(), ...)

# Optimal: Pass NumPy arrays directly for zero-copy optimization
db.add_batch(vectors=numpy_array, ...)
```

---

**Note**: This index only lists files that actually exist. See STATUS.md for roadmap of planned documentation.