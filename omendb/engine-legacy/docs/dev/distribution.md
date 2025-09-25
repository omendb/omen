# PyPI Distribution Guide

## Overview
This guide covers building and distributing OmenDB to PyPI.

## Prerequisites
- Python 3.8+
- `build` package: `pip install build`
- `twine` package: `pip install twine`
- PyPI account and API token

## Build Process

### 1. Ensure Native Module is Built
```bash
pixi run mojo build omendb/native.mojo --emit shared-lib -o python/omendb/native.so
```

### 2. Build Distribution
```bash
python -m build
```

This creates:
- `dist/omendb-0.1.0-py3-none-any.whl`
- `dist/omendb-0.1.0.tar.gz`

### 3. Test Installation
```bash
pip install dist/omendb-0.1.0-py3-none-any.whl
python -c "from omendb import DB; db = DB(); print('Success!')"
```

## Upload to PyPI

### Test PyPI First
```bash
twine upload --repository-url https://test.pypi.org/legacy/ dist/*
```

### Production PyPI
```bash
twine upload dist/*
```

## Platform Considerations
- OmenDB only supports macOS and Linux (Mojo limitation)
- Ensure platform tags are correct in wheel
- Consider using cibuildwheel for multi-platform builds

## Troubleshooting
- Verify `pyproject.toml` configuration
- Check that native module is included in wheel
- Test in clean virtual environment