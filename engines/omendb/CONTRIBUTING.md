# Contributing to OmenDB

Thank you for your interest in contributing to OmenDB! This guide will help you get started.

## Development Setup

### Prerequisites

- Python 3.8+
- Pixi package manager: https://pixi.sh/latest/
- Mojo (installed via Pixi)
- macOS 10.14+ or Linux (glibc 2.17+)

### Getting Started

1. Fork and clone the repository:
```bash
git clone https://github.com/YOUR_USERNAME/omendb.git
cd omendb
```

2. Install development environment:
```bash
pixi install
```

3. Build the native module:
```bash
pixi run mojo build omendb/native.mojo --emit shared-lib -o python/omendb/native.so
```

4. Run tests to verify setup:
```bash
pixi run python test/python/test_api_standards.py
```

## Development Workflow

### Making Changes

1. Create a feature branch:
```bash
git checkout -b feature/your-feature-name
```

2. Make your changes following our coding standards

3. Run tests:
```bash
# Quick validation
pixi run python test/python/test_api_standards.py
pixi run python test_instant_startup.py

# Full test suite
PYTHONPATH=python pixi run python -m pytest test/python/ -v
```

4. Run benchmarks to ensure no performance regression:
```bash
pixi run python benchmarks/comprehensive.py --quick
```

### Code Style

#### Mojo Code
- Follow existing patterns in the codebase
- Use clear, descriptive variable names
- Add comments for complex logic
- Keep functions focused and small

#### Python Code  
- Follow PEP 8 style guide
- Use type hints for all public APIs
- Add docstrings to all functions and classes

### Testing

All changes must include appropriate tests:

1. **Unit tests**: Test individual functions
2. **Integration tests**: Test component interactions
3. **Performance tests**: For optimization changes

Example test:
```python
def test_dimension_validation():
    """Test that dimension mismatches are caught."""
    db = DB()
    db.add("vec1", [1, 2, 3])
    
    with pytest.raises(ValidationError):
        db.add("vec2", [1, 2, 3, 4])  # Different dimension
```

## Submitting Changes

### Pull Request Process

1. Update documentation for any API changes
2. Add tests for new functionality
3. Ensure all tests pass
4. Update CHANGELOG.md with your changes
5. Submit a pull request with a clear description

### Pull Request Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Performance improvement
- [ ] Documentation update

## Testing
- [ ] Tests pass locally
- [ ] No performance regression
- [ ] Documentation updated

## Checklist
- [ ] Code follows project style
- [ ] Self-review completed
- [ ] Tests added/updated
```

## Architecture Guidelines

### Single-Dimension Design

OmenDB uses a single-dimension-per-process architecture. This is intentional for performance. When contributing:

- Don't try to "fix" the global state
- Respect the single-dimension design
- See `docs/ARCHITECTURE_DECISIONS.md` for details

### Performance First

OmenDB prioritizes performance. When contributing:

- Benchmark before and after changes
- Use SIMD operations where possible
- Minimize allocations
- Profile your code

## Areas for Contribution

### Welcome Contributions

- Performance optimizations with benchmarks
- Bug fixes with tests
- Documentation improvements
- Example applications
- Test coverage improvements

### Future Areas (Discuss First)

- New index algorithms
- GPU acceleration
- Multi-dimension support
- New query types

## Getting Help

- **Issues**: https://github.com/omendb/omendb/issues
- **Discussions**: https://github.com/omendb/omendb/discussions

## License

By contributing, you agree that your contributions will be licensed under the Elastic License 2.0.

## Code of Conduct

Please be respectful and constructive in all interactions. We want OmenDB to be welcoming to all contributors.

Thank you for contributing to OmenDB! 🔥