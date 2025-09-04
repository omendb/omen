# Native.mojo Refactoring Plan

## Problem
`native.mojo` is 3000+ lines - violates single responsibility principle

## Mojo/Python Best Practices
- **File size**: 200-500 lines ideal, 1000 max
- **Single responsibility**: One clear purpose per module
- **Clear interfaces**: Use traits/protocols for contracts
- **Separation of concerns**: Business logic vs FFI vs storage

## Proposed Structure

### 1. Core Database Module (database.mojo) ~400 lines
```mojo
struct Database:
    """Main database interface - coordinates all operations."""
    var storage: StorageCoordinator
    var search_engine: SearchEngine
    var quantizer: QuantizationManager
    
    fn add(...)
    fn search(...)
    fn delete(...)
```

### 2. FFI Exports (native.mojo) ~200 lines
```mojo
"""Only Python FFI exports - no business logic."""

@export
fn add_vector(...) -> PythonObject:
    return get_database().add(...)

@export  
fn search_vector(...) -> PythonObject:
    return get_database().search(...)
```

### 3. Storage Coordinator (storage/coordinator.mojo) ~300 lines
```mojo
struct StorageCoordinator:
    """Manages buffer, index, and persistence."""
    var buffer: VectorBuffer
    var index: DiskANNIndex
    var persistence: PersistenceManager
    
    fn add_to_buffer(...)
    fn flush_to_index(...)
    fn checkpoint(...)
```

### 4. Operations Modules

#### operations/add.mojo ~200 lines
```mojo
struct AddOperation:
    """Handles vector addition logic."""
    fn execute(vector: List[Float32]) -> Bool
    fn validate_dimension(...) -> Bool
    fn apply_quantization(...) -> StoredVector
```

#### operations/search.mojo ~250 lines
```mojo
struct SearchOperation:
    """Handles search logic."""
    fn execute(query: List[Float32], k: Int) -> List[SearchResult]
    fn search_buffer(...) -> List[SearchResult]
    fn search_index(...) -> List[SearchResult]
```

### 5. State Management (state/global.mojo) ~150 lines
```mojo
"""Global state management with proper initialization."""

var _global_database: Optional[Database] = None

fn get_database() -> Database:
    if not _global_database:
        _global_database = Database()
    return _global_database.value()
```

### 6. Quantization (quantization/manager.mojo) ~200 lines
```mojo
struct QuantizationManager:
    """Manages all quantization strategies."""
    var scalar_enabled: Bool
    var binary_enabled: Bool
    
    fn quantize(vector: List[Float32]) -> QuantizedVector
    fn dequantize(quantized: QuantizedVector) -> List[Float32]
```

## Benefits of Refactoring

### Maintainability
- Each file has clear purpose
- Easy to find functionality
- Simpler to test individual components

### Scalability
- New features don't bloat existing files
- Clear extension points
- Better separation of concerns

### Performance
- Smaller compilation units
- Better caching
- Clearer optimization targets

### Team Development
- Multiple people can work without conflicts
- Clear ownership boundaries
- Better code review scope

## Migration Strategy

### Phase 1: Extract Core Components
1. Create `database.mojo` with main Database struct
2. Move storage logic to `storage/coordinator.mojo`
3. Keep FFI exports in `native.mojo`

### Phase 2: Extract Operations
1. Create `operations/` directory
2. Move add logic to `add.mojo`
3. Move search logic to `search.mojo`

### Phase 3: Extract Support Systems
1. Move quantization to separate module
2. Extract metrics to `metrics.mojo`
3. Create proper state management

### Phase 4: Clean Up
1. Remove dead code
2. Update imports
3. Run comprehensive tests

## Example: Refactored Add Operation

### Before (in native.mojo)
```mojo
fn add_vector(self, ...) -> Bool:
    # 200 lines of mixed logic
    # - validation
    # - quantization
    # - buffer management
    # - persistence
    # - metrics
```

### After (modular)
```mojo
# database.mojo
fn add(self, vector: List[Float32]) -> Bool:
    var op = AddOperation(self.storage, self.quantizer)
    return op.execute(vector)

# operations/add.mojo  
fn execute(self, vector: List[Float32]) -> Bool:
    if not self.validate(vector):
        return False
    
    var stored = self.quantizer.process(vector)
    return self.storage.add(stored)
```

## Conventions for Mojo

Based on Mojo stdlib and best practices:

### File Organization
- One primary struct per file
- Related utilities in same file
- Traits in separate interface files

### Naming
- Files: `snake_case.mojo`
- Structs: `PascalCase`
- Functions: `snake_case`
- Constants: `UPPER_SNAKE_CASE`

### Size Guidelines
- Simple structs: 100-200 lines
- Complex logic: 300-500 lines
- Absolute max: 1000 lines
- If larger: split into multiple files

### Import Organization
```mojo
# Standard library
from collections import List
from memory import UnsafePointer

# Internal modules  
from .storage import StorageCoordinator
from .operations import AddOperation

# Local modules
from .utils import validate_dimension
```

## Action Items

1. **Create module structure** - Set up directories
2. **Extract Database struct** - Core coordination logic
3. **Separate FFI layer** - Just exports, no logic
4. **Move operations** - Add, search, delete
5. **Test each module** - Ensure nothing breaks
6. **Update imports** - Fix all references
7. **Document interfaces** - Clear contracts

This refactoring will make the codebase much more maintainable and follows Mojo/Python best practices for code organization.