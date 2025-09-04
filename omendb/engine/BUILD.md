# OmenDB Build Instructions

**Last Updated**: August 4, 2025  
**Mojo Version**: 25.5.0 (macOS) / 25.4.0 (Fedora)

## 🚨 Current Build Status

**macOS**: ✅ Working (Mojo 25.5.0)  
**Fedora**: ❌ Build issues with Mojo 25.4.0

## 📋 Prerequisites

1. **Install Mojo/MAX**: Follow instructions at https://docs.modular.com/max/install
2. **Install Pixi**: `curl -fsSL https://pixi.sh/install.sh | bash`
3. **Install dependencies**: `pixi install`

## 🔧 Build Commands

### Current Working Build (macOS, Mojo 25.5.0)

```bash
# Build the native Python extension module
pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib -I omendb

# Test the build
PYTHONPATH=python pixi run python -c "import omendb; print('Build successful!')"
```

### Deprecated Commands (No Longer Work)

```bash
# These flags were removed in Mojo 25.x:
# --dynamic-library (replaced by --emit shared-lib)
# --no-main (no longer needed)

# DON'T USE:
pixi run mojo build -I omendb omendb/native.mojo -o python/omendb/native.so --dynamic-library --no-main
```

### Build Script Updates Needed

The `build_native_direct.sh` script needs updating:
```bash
#!/bin/bash
# OLD (doesn't work):
# mojo build -I omendb omendb/native.mojo -o python/omendb/native.so --dynamic-library --no-main

# NEW (should work):
mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib -I omendb
```

## 🐧 Fedora-Specific Issues

### Known Problems (Mojo 25.4.0)

1. **Module Resolution**: Import errors even with `-I omendb` flag
   ```
   error: unable to locate module 'algorithms'
   error: unable to locate module 'core'
   ```

2. **Package Build Crash**: 
   ```bash
   pixi run mojo package omendb -o omendb.mojopkg
   # Error: mojo crashed! Please file a bug report
   ```

3. **Python Module Not Created**: The `--emit shared-lib` compiles but doesn't create a proper Python extension

### Potential Solutions

1. **Version Mismatch**: Try updating to Mojo 25.5.0 on Fedora
   ```bash
   modular update mojo-nightly
   ```

2. **Clean Build**:
   ```bash
   # Remove old artifacts
   rm -rf python/omendb/native.so
   rm -rf .mojo-cache
   
   # Try build with explicit paths
   cd /path/to/omendb
   pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib -I $(pwd)/omendb
   ```

3. **Environment Variables**:
   ```bash
   export MOJO_IMPORT_PATH=$(pwd)/omendb
   pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib
   ```

## 🔍 Verifying the Build

### Check Python Extension Symbol

A working Python extension must export `PyInit_native`:
```bash
# On Linux:
nm -D python/omendb/native.so | grep PyInit_native

# On macOS:
nm python/omendb/native.so | grep PyInit_native

# Should show:
# ... T PyInit_native
```

### Test Import

```bash
PYTHONPATH=python pixi run python -c "
import omendb
db = omendb.DB()
print(f'Version: {omendb.__version__}')
print('Build successful!')
"
```

## 📝 Build Troubleshooting

### Import Errors

If you get "unable to locate module" errors:
1. Ensure you're in the project root directory
2. Use absolute path for `-I` flag: `-I $(pwd)/omendb`
3. Check that `omendb/__init__.mojo` does NOT exist (it breaks imports)

### Shared Library Issues

If the .so file is created but Python can't import it:
1. Check for `PyInit_native` symbol (see above)
2. Ensure you're using `--emit shared-lib` not `--emit library`
3. Try `--emit pymodule` if available in your Mojo version

### Version Differences

Mojo is rapidly evolving. Check your version:
```bash
pixi run mojo --version
```

Key version differences:
- **< 25.0**: Uses `--dynamic-library --no-main`
- **>= 25.0**: Uses `--emit shared-lib`
- **>= 25.5**: Better module resolution

## 🎯 Working Build Configuration

Based on successful builds:
- **Mojo Version**: 25.5.0
- **Platform**: macOS (ARM64 and x86_64)
- **Command**: `pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib -I omendb`
- **Working Directory**: Project root (`/path/to/omendb`)

## 🆘 Getting Help

If builds continue to fail:
1. Share exact error messages
2. Include Mojo version: `pixi run mojo --version`
3. Include platform: `uname -a`
4. Try the MAX Discord community
5. File issue at: https://github.com/modularml/mojo/issues

## 📅 Build History

- **July 31, 2025**: Last successful Fedora build (old Mojo version)
- **August 1-3, 2025**: Various build attempts with new syntax
- **August 4, 2025**: Documented working macOS build command
- **TODO**: Verify Fedora build with Mojo 25.5.0

## 🔴 Historical Build Issue (July 30, 2025)

### Problem
The Mojo build system couldn't compile `native.mojo` due to module resolution failures, preventing SIMD optimizations from being tested.

### Root Cause
- Import resolution failed even with `-I omendb` flag
- Cascading errors from unknown types
- Different module resolution for shared library builds vs regular builds

### Impact
- Performance stuck at 5,300 vec/s instead of 7,000+ target
- Migration control features untested
- v0.1.0 could ship with current performance but missed optimization goal

### Resolution
- Build system was fixed with proper flags
- Current performance: 90,000+ vec/s (17x improvement achieved)
- SIMD optimizations implemented via @vectorize decorator