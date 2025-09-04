# Module Import and Compilation Fixes

Quick reference for resolving common Mojo 25.5.0 module import and compilation issues in OmenDB.

## 🚨 CRITICAL: Package Import Resolution Issues

### Problem: `unable to locate module 'core'` or `unable to locate module 'omendb'`

**Root Cause**: Problematic package `__init__.mojo` files causing import conflicts.

**🎯 BREAKTHROUGH SOLUTION**: 

The issue is typically caused by complex `__init__.mojo` files in the root package directory that create circular dependencies or import conflicts.

**Step 1: Remove Problematic Package Init Files**
```bash
# Remove the root package __init__.mojo file if it exists
rm omendb/__init__.mojo  # This often fixes the issue immediately
```

**Step 2: Use Correct Import Pattern**
```bash
# ✅ Use -I flag to add package directory to import path
pixi run mojo -I omendb tests/test_file.mojo
./scripts/run-test.sh test_name  # Uses correct -I omendb pattern
```

**Step 3: Import Syntax in Test Files**
```mojo
# ✅ CORRECT: Direct module imports (with -I omendb flag)
from core.vector import Vector
from compression.binary_quantization import BinaryQuantizer

# ❌ INCORRECT: Package-prefixed imports 
from omendb.core.vector import Vector
from src.core.vector import Vector
```

### Verified Working Pattern

1. **Directory Structure**: `omendb/core/vector.mojo`, `omendb/compression/`, etc.
2. **No Root Package Init**: Remove `omendb/__init__.mojo` if it exists
3. **Keep Subpackage Inits**: Keep `omendb/core/__init__.mojo`, `omendb/compression/__init__.mojo`
4. **Import Path Flag**: Use `-I omendb` to add to search path
5. **Simple Imports**: Use `from core.vector import Vector` syntax

## Print Statement Compatibility (Mojo 25.5.0)

### Problem: `could not deduce parameter 'Ts' of callee 'print'`

**Root Cause**: Mixed types in print statements confuse Mojo's type inference.

**Solutions**:

```mojo
# ✅ Print simple values directly
print("Dimensions:", vector.dim)
print("Memory usage:", memory_size, "bytes")

# ✅ Cast complex types explicitly
print("Ratio:", float(ratio))
print("Count:", int(count))

# ❌ AVOID: Complex expressions or str() calls
print("Value:", str(complex_expression))  # str() doesn't exist in Mojo
print("Mixed:", some_float, some_int, "text")  # Mixed types can fail
```

## Type Conversion Issues

### Problem: `cannot implicitly convert 'SIMD[float64, 1]' to 'SIMD[float32, 1]'`

**Solution**: Always use explicit type casting:

```mojo
# ✅ Explicit conversion for random values
vector.data[i] = Float32(random_float64(-1.0, 1.0))

# ✅ Explicit conversion for arithmetic
var accuracy_retention = Float64(1.0) - accuracy_loss

# ❌ Implicit conversion fails
vector.data[i] = random_float64(-1.0, 1.0)
```

## Method Name Updates (OmenDB Vector API)

### Vector class methods:
- ✅ `vector.memory_footprint()` - Returns memory usage in bytes
- ❌ `vector.memory_usage()` - Method doesn't exist
- ✅ `len(list_object)` - Use function for List sizes
- ❌ `list_object.size` - Attribute doesn't exist

## Function Availability Issues

### Missing Functions in Mojo:
- ❌ `str()` - Not available, use direct printing or explicit casting
- ✅ `len()` - Available for List objects
- ✅ `int()`, `float()` - Available for type casting
- ✅ `Float32()`, `Float64()` - Explicit SIMD type constructors

## Test Runner Best Practices

### Correct Test Execution:

```bash
# ✅ Use provided test script (includes correct -I flag)
./scripts/run-test.sh test_name

# ✅ Manual execution with correct flag
pixi run mojo -I omendb tests/path/test_file.mojo

# ✅ Using mojo test command
pixi run mojo test -I omendb tests/path/test_file.mojo

# ❌ AVOID: Running without -I flag
pixi run mojo tests/test_file.mojo  # Will fail with import errors
```

### Test File Import Pattern:

```mojo
# ✅ CORRECT: Test file imports (assumes -I omendb flag)
from core.vector import Vector
from compression.binary_quantization import BinaryQuantizer
from testing import assert_true

# Test functions should be marked with raises if needed
fn test_function() raises:
    # Test code here
```

## Advanced Debugging Steps

### If imports still fail after following above steps:

1. **Verify Directory Structure**:
   ```bash
   ls -la omendb/core/      # Should contain vector.mojo and __init__.mojo
   ls -la omendb/           # Should NOT contain __init__.mojo
   ```

2. **Check Individual Module Compilation**:
   ```bash
   pixi run mojo build omendb/core/vector.mojo  # Should fail (no main)
   # But error should be "no main function", not import errors
   ```

3. **Test Simple Import**:
   ```bash
   # Create minimal test file and run
   pixi run mojo -I omendb tests/test_import_debug.mojo
   ```

4. **Check for Circular Dependencies**:
   - Review `__init__.mojo` files for complex import chains
   - Simplify imports to only essential components
   - Remove unused imports

## Memory Management Issues

### Runtime Crashes:

If tests import correctly but crash during execution:

```mojo
# ✅ Use explicit defer for cleanup
var ptr = UnsafePointer[Float32].alloc(size)
defer:
    ptr.free()

# ✅ Ensure proper ownership with Copyable, Movable traits
struct MyStruct(Copyable, Movable):
    # Implementation
```

## Quick Reference Checklist

When facing import issues, check in this order:

1. ☐ Remove `omendb/__init__.mojo` if it exists
2. ☐ Use `-I omendb` flag or test script
3. ☐ Use simple import syntax: `from core.vector import Vector`
4. ☐ Run from project root directory
5. ☐ Check for `str()` usage and replace with direct printing
6. ☐ Add explicit type conversions: `Float32(value)`
7. ☐ Use `memory_footprint()` instead of `memory_usage()`
8. ☐ Add `raises` to functions that call methods that may raise

## Working Examples

### Successful Test File Template:

```mojo
"""
Test file template with working import pattern.
"""

from collections import List
from testing import assert_true
from core.vector import Vector
from compression.binary_quantization import BinaryQuantizer

fn test_vector_creation() raises:
    """Test basic vector functionality."""
    var vec = Vector[DType.float32](128)
    assert_true(vec.dimension() == 128)
    print("✅ Vector creation successful")

fn main() raises:
    """Run all tests."""
    test_vector_creation()
    print("🎉 All tests passed!")
```

### Successful Build Command:

```bash
# From project root
./scripts/run-test.sh test_vector_creation
# or
pixi run mojo -I omendb tests/core/test_vector.mojo
```

---

## Last Updated

Based on breakthrough debugging session that resolved critical import infrastructure issues in OmenDB. The key insight was that complex package `__init__.mojo` files cause import conflicts that prevent the `-I` flag from working correctly.