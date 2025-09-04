# Mojo Common Errors - Quick Reference

**Quick Reference**: Check this file first when encountering Mojo compilation errors.

## 🚨 Critical: Vectorize Index Bug (Silent Data Corruption)

**Most dangerous Mojo pattern - causes silent data loss:**

```mojo
# ❌ WRONG - Causes 75% data loss!
@parameter
fn process_elements[width: Int](idx: Int):
    vector_list[idx] = source[idx]  # idx is NOT element index!

vectorize[process_elements, simd_width](total_elements)
```

**Root Cause**: `vectorize` calls function with indices `0, simd_width, 2*simd_width...` not `0, 1, 2, 3...`

**Fix**: Use manual loop for element-wise access:
```mojo
# ✅ CORRECT - Process all elements
for i in range(total_elements):
    vector_list[i] = source[i]
```

## Common Compilation Errors

### 1. Import Resolution
```bash
error: unable to locate module 'core'
```
**Fix**: Remove `omendb/__init__.mojo` and use `-I omendb` flag

### 2. Unknown Function Errors
```bash
error: use of unknown declaration 'str'
error: use of unknown declaration 'int'
```
**Fix**: Use explicit constructors:
```mojo
String(value)    # not str(value)
Int(value)       # not int(value)
Float32(value)   # not float(value)
```

### 3. Mixed Type Print Errors
```bash
error: could not deduce parameter 'Ts' of callee 'print'
```
**Fix**: Cast types explicitly:
```mojo
print("Value:", Float32(some_value))
```

## Build Commands

```bash
# Build with correct flags
pixi run mojo build -I omendb -o target.so source.mojo
pixi run mojo -I omendb tests/test_file.mojo
```

## Quick Debug Checklist

1. **Import Error**: Remove `omendb/__init__.mojo`, use `-I omendb`
2. **Function Error**: Use `String()`, `Int()`, `Float32()` instead of Python names
3. **Type Error**: Add explicit type conversions
4. **Vectorize Bug**: Use manual loops for element-wise operations

---

**Remember**: Be explicit with types and use simple patterns when in doubt.