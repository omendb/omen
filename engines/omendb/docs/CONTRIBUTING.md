# Contributing to OmenDB

## Quick Start for Contributors

1. **Setup Development Environment**
   ```bash
   pixi install
   pixi run python -m pytest test/
   ```

2. **Common Mojo Issues**: See `dev/MOJO_COMMON_ERRORS.md` for syntax troubleshooting

3. **Build System**: See `dev/BUILD_SYSTEM_ISSUE.md` if you encounter import errors

## Development Guidelines

### Code Standards
- **Mojo**: Follow `docs/MOJO_STYLE_GUIDE.md`
- **Python**: PEP 8 with type hints
- **Performance**: Always benchmark changes
- **Tests**: Comprehensive coverage required

### Testing
```bash
# Run full test suite
pixi run python -m pytest test/

# Performance validation
PYTHONPATH=python pixi run python test/performance/test_current_performance.py

# Mojo tests (if import issues, see dev/MOJO_COMMON_ERRORS.md)
pixi run mojo -I omendb tests/core/test_vector.mojo
```

### Documentation
- User docs: `docs/user/`
- Architecture: `docs/architecture/`
- Development: `docs/dev/`

### Known Issues
- **Build System**: Native module rebuild currently blocked (see `dev/BUILD_SYSTEM_ISSUE.md`)
- **Memory Bug**: Tiered storage disabled due to corruption (see `TIERED_STORAGE_MEMORY_BUG.md`)

See `docs/KNOWN_ISSUES.md` for complete list.