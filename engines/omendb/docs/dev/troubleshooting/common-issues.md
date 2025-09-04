# Common Issues and Solutions

*Troubleshooting guide for OmenDB development challenges*

## 🚨 CRITICAL: Import System Issues

### Module Import Failures (`unable to locate module` errors)
**Problem**: Tests fail with "unable to locate module 'core'" or similar import errors
**Quick Fix**: 
1. Remove `omendb/__init__.mojo` if it exists: `rm omendb/__init__.mojo`
2. Use test runner: `./scripts/run-test.sh test_name`
3. Or manual: `pixi run mojo -I omendb tests/path/test_file.mojo`

**📋 Complete Solution**: See `docs/development/troubleshooting/module-import-fixes.md` for detailed breakthrough solution that resolves package import conflicts.

## Mojo-Specific Challenges

### 1. API Instability
**Problem**: Mojo APIs change between versions, breaking existing code
**Solutions**:
- Pin to specific Mojo version in `pixi.toml`
- Use abstraction layers for frequently changing APIs
- Test with each Mojo update before upgrading
- Keep backup implementations for critical functionality

### 2. Missing Standard Library Features
**Problem**: Standard library lacks common functionality found in other languages
**Solutions**:
- Implement custom alternatives in `src/util/`
- Check `external/modular/mojo/stdlib/` for available functionality
- Use C bindings for complex operations when needed
- Contribute implementations back to community

### 3. SIMD Limitations
**Problem**: SIMD operations don't work as expected or have limitations
**Solutions**:
- Fall back to scalar operations when SIMD fails
- Check vector dimensions are SIMD-width aligned
- Use `debug_assert` to validate SIMD preconditions
- Profile to ensure SIMD actually improves performance

### 4. Memory Management Issues
**Problem**: Memory leaks, dangling pointers, or ownership confusion
**Solutions**:
- Use RAII patterns consistently with `defer`
- Enable debug memory tracking: `mojo build -D DEBUG_MEMORY=1`
- Use memory profiling tools during development
- Implement custom memory tracking for complex operations

### 5. Module Import Resolution
**Problem**: Module imports fail or behave unpredictably
**Solutions**:
- **🚨 FIRST**: Check if `omendb/__init__.mojo` exists and remove it
- Always run examples from project root directory
- Use `-I omendb` flag or provided test runner script
- Use simple import syntax: `from core.vector import Vector`
- See `module-import-fixes.md` for complete solution

### 6. Print Statement Compatibility
**Problem**: Complex print statements fail with type errors
**Solutions**:
- Use explicit type casting: `str(value)` for complex types
- Break complex prints into multiple statements
- Use debug logging instead of print for complex data
- Format strings properly with String type

## Vector Database Issues

### Server Mode Issues

#### 1. Memory Pressure Under Load
**Problem**: Server runs out of memory with large datasets
**Solutions**:
- Implement intelligent tiering (hot/warm/cold storage)
- Use compression for less frequently accessed vectors
- Monitor memory usage and implement back-pressure
- Tune garbage collection parameters

#### 2. Search Accuracy Degradation
**Problem**: Search results become less accurate over time
**Solutions**:
- Tune HNSW parameters (M, ef_construction, ef_search)
- Adjust DiskANN graph quality settings
- Monitor and retrain BM25 parameters
- Implement compression ratio monitoring

#### 3. Concurrent Access Bottlenecks
**Problem**: Multiple threads competing for index access
**Solutions**:
- Use lock-free read operations where possible
- Implement fine-grained write locking by partition
- Use read-write locks instead of mutexes
- Consider lock-free data structures for hot paths

#### 4. ML Pipeline Integration Failures
**Problem**: Embedding model updates break existing functionality
**Solutions**:
- Implement model versioning and backward compatibility
- Use feature flags for gradual model rollouts
- Monitor embedding drift and alert on significant changes
- Maintain separate staging and production pipelines

#### 5. Real-time Ingestion Latency
**Problem**: Cannot meet sub-second ingestion-to-search requirements
**Solutions**:
- Implement async ingestion with search availability flags
- Use write-ahead logging for immediate acknowledgment
- Batch small updates for better throughput
- Consider eventual consistency for non-critical updates

### Embedded Mode Issues

#### 6. Resource Constraint Violations
**Problem**: Exceeding memory, CPU, or battery limits on mobile/edge devices
**Solutions**:
- Implement adaptive algorithms based on available resources
- Use more aggressive compression on resource-constrained devices
- Monitor and throttle CPU usage during idle periods
- Profile memory usage on target hardware configurations

#### 7. Cold Start Performance
**Problem**: Index loading takes too long for responsive user experience
**Solutions**:
- Implement lazy loading of index segments
- Use memory-mapped files for faster startup
- Cache frequently accessed index portions
- Optimize index serialization format for faster loading

#### 8. Single-file Database Corruption
**Problem**: Database becomes corrupted due to partial writes or power failures
**Solutions**:
- Implement atomic write operations with rename
- Use write-ahead logging for crash recovery
- Add checksums for corruption detection
- Implement automatic backup and recovery mechanisms

#### 9. FFI Binding Stability
**Problem**: Memory safety issues when crossing language boundaries
**Solutions**:
- Use explicit memory management at FFI boundaries
- Implement reference counting for shared objects
- Add validation for all inputs crossing FFI boundaries
- Use defensive programming practices

#### 10. Platform Compatibility Issues
**Problem**: Code doesn't work across different architectures or operating systems
**Solutions**:
- Use feature detection for platform-specific optimizations
- Implement fallback algorithms for unsupported features
- Test on all target platforms during development
- Use conditional compilation for platform differences

### Universal Issues (Both Modes)

#### 11. Vector Dimension Mismatches
**Problem**: Vectors with different dimensions cause runtime errors
**Solutions**:
- Validate dimensions at ingestion time
- Store dimension metadata with each collection
- Implement dimension checking in all vector operations
- Use type system to enforce dimension consistency where possible

#### 12. Hybrid Search Optimization
**Problem**: Dense, sparse, and graph components poorly balanced
**Solutions**:
- Implement learned fusion models instead of fixed weights
- Use A/B testing to optimize fusion parameters
- Monitor component performance and adjust weights dynamically
- Implement query-specific optimization

#### 13. Multi-Vector Consistency
**Problem**: Cross-modal representations become inconsistent
**Solutions**:
- Implement transactional updates across all vector types
- Use consistent hashing for vector-to-document mapping
- Monitor cross-modal similarity distributions
- Implement consistency checks and repair mechanisms

#### 14. Compression Trade-offs
**Problem**: Memory savings come at too high accuracy cost
**Solutions**:
- Implement adaptive quantization based on query patterns
- Use different compression levels for different data tiers
- Monitor reconstruction error and adjust compression parameters
- Implement quality-based compression selection

#### 15. Graph-Vector Integration
**Problem**: Knowledge graph and vector embeddings become inconsistent
**Solutions**:
- Implement graph-aware embedding updates
- Use consistent entity resolution across graph and vector space
- Monitor graph structure changes and update embeddings accordingly
- Implement graph-vector consistency validation

## Development Environment Issues

### Build and Compilation Issues

#### Pixi Environment Problems
**Problem**: Pixi fails to create or activate environment
**Solutions**:
- Clear pixi cache: `pixi clean cache`
- Verify system dependencies are installed
- Check for conflicting conda installations
- Use `pixi info` to debug environment issues

#### Mojo Compiler Errors
**Problem**: Unclear or confusing compiler error messages
**Solutions**:
- Enable verbose compilation: `mojo build -v`
- Check for missing imports or circular dependencies
- Verify struct field initialization
- Use simpler test cases to isolate issues

### Performance and Profiling Issues

#### Unexpected Performance Degradation
**Problem**: Code performs worse than expected
**Solutions**:
- Profile with built-in Mojo profiler
- Check for accidental memory allocations in hot paths
- Verify SIMD operations are actually being used
- Compare with reference implementations

#### Memory Leaks in Long-Running Processes
**Problem**: Memory usage grows over time
**Solutions**:
- Use memory tracking tools during development
- Implement object pooling for frequently allocated objects
- Check for circular references in data structures
- Monitor memory usage in production

## Testing and Validation Issues

### Test Failures in CI/CD
**Problem**: Tests pass locally but fail in CI
**Solutions**:
- Use same Mojo version in CI as development
- Check for timing-dependent test failures
- Verify all test dependencies are available
- Use deterministic random seeds for reproducible tests

### Inconsistent Benchmarking Results
**Problem**: Performance benchmarks vary widely between runs
**Solutions**:
- Use stable benchmark environments
- Warm up algorithms before timing
- Run multiple iterations and report statistics
- Control for system load and other processes

## Recovery Procedures

### Database Corruption Recovery
1. Stop all database operations
2. Check integrity using built-in validation tools
3. Restore from most recent backup if available
4. Rebuild indices from raw vector data if necessary
5. Implement additional validation to prevent recurrence

### Performance Regression Recovery
1. Identify when regression was introduced using git bisect
2. Profile the problematic commit to identify root cause
3. Implement fix or revert if fix is not immediately available
4. Add performance regression tests to prevent recurrence
5. Update monitoring to catch similar issues earlier

### Development Environment Recovery
1. Document current state and error symptoms
2. Try clean rebuild: `pixi clean && pixi install`
3. Check for system-level dependencies or conflicts
4. Create fresh environment if necessary
5. Restore code and configuration from version control

This troubleshooting guide should be updated as new issues are discovered and resolved. Each issue should include clear problem description, root cause analysis, and step-by-step solutions.