# OmenDB Codebase Refactoring Plan
*Date: August 24, 2025*

## Current Architecture Issues

### Monolithic Structure
- **native.mojo**: 3000+ lines mixing API, storage, indexing, and configuration
- **Single responsibility violation**: One file handles too many concerns
- **Testing difficulty**: Hard to unit test individual components
- **Development bottleneck**: Multiple developers can't work on different parts

### Global State Problems
```mojo
var __buffer_size: Int = 10000
var __use_columnar: Bool = False
var __is_server: Bool = False
```
- Makes testing difficult (global state persists)
- Prevents multiple instances with different configs
- Concurrency issues with shared state
- Violates dependency injection principles

### Tight Coupling Issues
- VectorStore directly instantiates SparseDiskANNIndex
- Memory tracking hardcoded into specific implementations
- API layer knows about internal buffer details
- Storage format tied to specific algorithms

## Target Architecture

### 1. Layered Architecture
```
┌─────────────────────────────────────────────────┐
│                API Layer                        │ ← Python bindings
├─────────────────────────────────────────────────┤
│               Business Logic                    │ ← Core operations
├─────────────────────────────────────────────────┤
│               Storage Layer                     │ ← Data persistence
├─────────────────────────────────────────────────┤
│              Algorithm Layer                    │ ← Indexing algorithms
└─────────────────────────────────────────────────┘
```

### 2. Trait-Based Components

#### Core Traits
```mojo
trait VectorIndex:
    fn add(mut self, id: String, vector: List[Float32]) raises -> Bool
    fn search(mut self, query: List[Float32], k: Int) -> List[SearchResult]
    fn delete(mut self, id: String) raises -> Bool
    fn size(self) -> Int
    fn get_stats(self) -> IndexStats

trait VectorStorage:
    fn store(mut self, id: String, vector: List[Float32]) raises
    fn retrieve(self, id: String) raises -> List[Float32]
    fn delete(mut self, id: String) raises -> Bool
    fn checkpoint(mut self) raises -> Bool

trait MemoryTracker:
    fn track_allocation(mut self, component: String, bytes: Int)
    fn track_deallocation(mut self, component: String, bytes: Int)
    fn get_stats(self) -> MemoryStats
```

#### Concrete Implementations
```mojo
struct SparseDiskANNIndex(VectorIndex):
    # Current implementation, cleaned up

struct QuantizedVectorStorage(VectorStorage):
    # Quantized storage with configurable compression

struct DetailedMemoryTracker(MemoryTracker):
    # Working memory tracking with component breakdown
```

### 3. Configuration System
```mojo
@value
struct DatabaseConfig:
    var buffer_size: Int
    var quantization_type: QuantizationType
    var index_type: IndexType
    var storage_type: StorageType
    var memory_tracking: Bool

struct VectorDatabase[T: VectorIndex, S: VectorStorage]:
    var index: T
    var storage: S
    var config: DatabaseConfig
    var memory_tracker: DetailedMemoryTracker
```

## Refactoring Steps

### Phase 1: Extract Core Interfaces
**Target**: Define traits and basic structure

1. **Create trait definitions** - `core/traits.mojo`
2. **Extract configuration** - `core/config.mojo`
3. **Create result types** - `core/types.mojo`

### Phase 2: Modularize Native.mojo
**Target**: Break monolithic file into focused modules

1. **Split by layer**:
   - `api/vector_database.mojo` - Main API surface
   - `storage/vector_store.mojo` - Storage management
   - `memory/tracking.mojo` - Memory instrumentation
   - `config/database_config.mojo` - Configuration management

2. **Extract utilities**:
   - `utils/simd_ops.mojo` - SIMD operations
   - `utils/metrics.mojo` - Performance metrics
   - `utils/validation.mojo` - Input validation

### Phase 3: Implement Dependency Injection
**Target**: Remove global state and tight coupling

```mojo
struct VectorDatabase:
    var index: Box[VectorIndex]
    var storage: Box[VectorStorage] 
    var memory_tracker: Box[MemoryTracker]
    
    fn __init__(
        out self,
        owned index: Box[VectorIndex],
        owned storage: Box[VectorStorage],
        owned tracker: Box[MemoryTracker]
    ):
        self.index = index^
        self.storage = storage^
        self.memory_tracker = tracker^
```

### Phase 4: Factory Pattern for Configuration
**Target**: Clean instantiation and testing

```mojo
struct DatabaseFactory:
    @staticmethod
    fn create_sparse_database(config: DatabaseConfig) -> VectorDatabase:
        var index = Box(SparseDiskANNIndex(config.dimension, config.R))
        var storage = Box(QuantizedVectorStorage(config.quantization_type))
        var tracker = Box(DetailedMemoryTracker())
        return VectorDatabase(index^, storage^, tracker^)
    
    @staticmethod
    fn create_test_database() -> VectorDatabase:
        var test_config = DatabaseConfig(
            buffer_size=100,
            quantization_type=QuantizationType.NONE,
            # ... other test settings
        )
        return create_sparse_database(test_config)
```

## Benefits

### Development Velocity
- **Parallel development**: Multiple developers on different layers
- **Easier testing**: Mock implementations for unit tests
- **Clear boundaries**: Reduced merge conflicts
- **Component isolation**: Changes don't cascade

### Code Quality
- **Single responsibility**: Each module has one job
- **Dependency inversion**: High-level modules don't depend on details
- **Open/closed principle**: Extend behavior without modifying existing code
- **Interface segregation**: Clients depend only on what they use

### Performance Benefits
- **Specialized implementations**: Optimize specific use cases
- **Memory tracking that works**: Proper instrumentation
- **Configurable tradeoffs**: Choose speed vs memory vs accuracy
- **Easier profiling**: Isolate bottlenecks by component

## Implementation Priority

### Week 1 (Critical)
1. Fix memory tracking in current implementation
2. Extract core traits and interfaces
3. Create configuration system

### Week 2-3 (High)
1. Split native.mojo by layers
2. Implement dependency injection
3. Add factory pattern

### Month 1 (Medium)
1. Add comprehensive test suite
2. Implement alternative algorithms (BruteForce, HNSW)
3. Add benchmarking infrastructure

## Migration Strategy

### Backward Compatibility
- Keep existing Python API unchanged
- Implement new architecture behind same interface
- Gradual migration of components
- Feature flags for new vs old behavior

### Testing Strategy
- Unit tests for each trait implementation
- Integration tests for component interaction
- Performance regression tests
- Memory usage validation

### Rollback Plan
- Keep old implementation as fallback
- Feature toggle between architectures
- Comprehensive monitoring during migration
- Clear rollback triggers and procedures

## Expected Outcomes

### Short Term (Week 1)
- Memory tracking works correctly
- Code is organized by responsibility
- Configuration is explicit and testable

### Medium Term (Month 1)  
- Development velocity increases 2x
- Bug rate decreases due to better isolation
- Performance optimization becomes easier
- New algorithms can be added quickly

### Long Term (Quarter 1)
- Best-in-class modular architecture
- Multiple indexing algorithms supported
- Easy to extend and customize
- Production-ready for large scale deployments

---

This refactoring aligns with modern software architecture principles while leveraging Mojo's unique capabilities for high-performance computing.