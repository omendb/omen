# FFI Implementation Decision: Python FFI vs C FFI

**Decision**: Use Python FFI (PyO3) instead of C FFI for Rust-Mojo integration

## Context

OmenDB Server needs to bridge between Rust (server orchestration) and Mojo (vector computation). Two main approaches:

1. **C FFI**: Direct C function calls between Rust and Mojo
2. **Python FFI**: Rust calls Python API which calls Mojo native module

## Decision Factors

### Performance Analysis

| Aspect | C FFI | Python FFI | Impact |
|--------|-------|------------|--------|
| Call overhead | 1-10μs | 50-200μs | <0.2% of 100ms P99 budget |
| Memory copies | 1x | 2x | Minimal for vector data |
| Type safety | Manual | Automatic | Development velocity |
| Error handling | Complex | Pythonic | Debugging ease |

### Real-World Context

**Target Performance**: 10K QPS, 100ms P99 latency
- Network: 1-10ms
- Storage I/O: 0.1-10ms  
- Vector computation: 1-50ms
- **FFI overhead**: 0.05-0.2ms

**FFI represents <5% of total latency budget**

### Implementation Complexity

**C FFI Challenges**:
```
❌ ABI compilation issues across Mojo versions
❌ Manual memory management and cleanup
❌ Complex error propagation
❌ Platform-specific build configuration
❌ Debugging requires deep systems knowledge
```

**Python FFI Benefits**:
```
✅ Uses existing, tested Python API
✅ Automatic memory management
✅ Rich error information from Python tracebacks
✅ Cross-platform compatibility
✅ Rapid prototyping and debugging
```

## Implementation Details

### Build Configuration
```bash
# No special environment variables needed with PyO3 v0.25+
cargo build

# Dependencies in Cargo.toml (no version pinning)
pyo3 = { features = ["auto-initialize"] }
```

### Core Implementation Pattern
```rust
// Pattern: Rust → spawn_blocking → Python GIL → Mojo
task::spawn_blocking(move || {
    Python::with_gil(|py| {
        let omendb = py.import("omendb.native")?;
        let result = omendb.call_method1("operation", args)?;
        Ok(result)
    })
}).await??
```

### File Locations
- **Main bridge**: `src/python_ffi.rs`
- **Engine manager**: `src/engine.rs` (updated for Python FFI)
- **Error types**: `src/error.rs` (added Python error variants)

## Python 3.13 Compatibility Resolution

**Previous Issue**: PyO3 v0.20.3 only supported Python ≤3.12

**Solution**: Upgraded to PyO3 v0.25+
```toml
# Cargo.toml - no version pinning, gets latest compatible
pyo3 = { features = ["auto-initialize"] }
```

**Benefits**:
- ✅ Native Python 3.13 support with version-specific bindings
- ✅ Optimal performance (no stable ABI overhead)
- ✅ No build flags or compatibility workarounds needed
- ✅ Future-proof for Python updates

## When to Reconsider

Migrate to C FFI if:
- **Ultra-low latency required** (sub-millisecond P99)
- **High-frequency operations** (>100K QPS per node)
- **Memory-constrained** environments
- **C FFI compilation issues resolved** in Mojo ecosystem

## Monitoring

Track these metrics to validate decision:
- `ffi_call_duration_seconds` - FFI call latency
- `python_gil_acquisition_seconds` - GIL overhead
- `memory_usage_mb` - Memory impact of double-copying

**Alert threshold**: FFI overhead >5% of total request latency

## Conclusion

Python FFI provides **working solution immediately** with negligible performance impact for our target workload. The reliability and development velocity benefits outweigh the minor performance cost.

**Status**: ✅ Implemented and functional
**Next review**: After load testing at 10K QPS