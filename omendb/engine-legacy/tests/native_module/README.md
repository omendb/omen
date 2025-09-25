# Native Module Tests

Tests for the Mojo native module functionality and Python-Mojo integration.

## Test Files

- `test_production_simple.py` - Core functionality test for production_native.mojo
- `test_safe_native.py` - Tests for working_native.mojo (safe validation approach)  
- `test_simple_native.py` - Tests for simple_native.mojo (with memory management)
- `test_vector_operations.py` - Comprehensive vector operations testing
- `test_working_native.py` - Basic working_native.mojo integration test

## Running Tests

```bash
# Run individual tests
pixi run python tests/native_module/test_production_simple.py

# Run from root directory
pixi run python -m pytest tests/native_module/
```

## Test Hierarchy

1. **Core Module Tests** - Basic compilation and import testing
2. **Functionality Tests** - Vector operations, similarity calculations  
3. **Integration Tests** - Python-Native API integration (see ../integration/)

## Performance Benchmarks

The production_simple test includes performance validation:
- Vector insertion rate: 10,000+ vectors/second
- Search performance: <1ms average
- Similarity calculation: <1ms average