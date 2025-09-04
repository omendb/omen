# OmenDB Refactoring Plan - Pragmatic Approach
*Updated: August 25, 2025*

## Current Situation
- **File Size**: native.mojo is 3,136 lines (only real issue)
- **Performance**: 79K vec/s (MUST maintain) - verified 80K+ vec/s post-refactor
- **Stability**: Working, needs to be production-ready
- **Mojo Limitations**: Module-level vars not until 2026+

## Refactoring History (August 25, 2025)

### What We Attempted
1. **VectorStore Extraction** - Extracted 1,130 lines to core/database.mojo
   - **Result**: Runtime segfault after 2000 vectors during batch operations
   - **Root Cause**: Global state dependencies (`__buffer_size`, `__global_db_ptr`)
   - **Decision**: Reverted - needs proper state management module first

### What We Successfully Did
1. **Added Section Markers** - Organized native.mojo into clear sections
2. **Extracted Type Definitions** - utils/types.mojo (working)
3. **Extracted Validation** - utils/validation.mojo (working)
4. **Documented Approach** - Created this pragmatic plan

## Strategy: "Annotated Monolith"
**Keep what works, prepare for future**

### Core Principles
1. **Don't break working code** - Performance is critical
2. **Document for future extraction** - Add markers and comments
3. **Extract only zero-risk items** - Pure functions with no state
4. **Test everything** - Every change must maintain 79K vec/s

## Immediate Actions (Safe Extractions)

### ✅ Already Done (August 25, 2025)
```
utils/
├── types.mojo          # Type definitions (EXTRACTED & WORKING)
└── validation.mojo      # Validation functions (EXTRACTED & WORKING)

native.mojo:
├── SECTION 1: IMPORTS AND DEPENDENCIES
├── SECTION 2: GLOBAL STATE AND CONFIGURATION  
├── SECTION 3: CORE VECTORSTORE IMPLEMENTATION
├── SECTION 4: HELPER FUNCTIONS
├── SECTION 5: COLLECTION MANAGEMENT
├── SECTION 6: FFI PYTHON EXPORTS - Core Database
└── SECTION 7: FFI PYTHON EXPORTS - Collections API
```

### ✓ Safe to Extract Now
```
utils/
├── helpers.mojo         # Pure math/utility functions
│   ├── cosine_similarity()
│   ├── normalize_vector()
│   └── compute_hash()
└── constants.mojo       # All constants and aliases
    ├── DEFAULT_BUFFER_SIZE
    ├── MAX_VECTOR_DIM
    └── VERSION_STRING
```

### ⚠️ DO NOT Extract (Performance Risk)
```
- VectorStore struct     # Keep in native.mojo
- Global state vars      # Keep in native.mojo
- FFI exports           # Must stay in native.mojo
- Collection management  # Tightly coupled to globals
```

## Code Organization (Within native.mojo)

### Section Structure with Clear Markers
```mojo
# =============================================================================
# SECTION 1: IMPORTS AND CONSTANTS
# =============================================================================
# [All imports]
# [All constants]

# =============================================================================
# SECTION 2: GLOBAL STATE
# FUTURE: Extract to state.mojo when module-vars stabilize (2026+)
# =============================================================================
var __global_db_ptr: UnsafePointer[VectorStore]
var __buffer_size: Int = DEFAULT_BUFFER_SIZE
# [All other globals]

# =============================================================================
# SECTION 3: HELPER FUNCTIONS  
# FUTURE: Extract to utils/helpers.mojo (safe now if needed)
# =============================================================================
@always_inline
fn normalize_vector(...) -> Float32:
    # Pure function - safe to extract

# =============================================================================
# SECTION 4: VECTOR STORE
# FUTURE: Extract to core/database.mojo when globals fixed
# DEPENDENCY: Requires global state refactor first
# =============================================================================
struct VectorStore:
    # Keep here for performance

# =============================================================================
# SECTION 5: COLLECTIONS
# FUTURE: Extract to core/collections.mojo 
# DEPENDENCY: Requires VectorStore extraction first
# =============================================================================
fn get_or_create_collection(...):
    # Collection management

# =============================================================================
# SECTION 6: FFI EXPORTS
# NOTE: Must remain in native.mojo (Python bindings)
# =============================================================================
@export
fn add_vector(...):
    # FFI layer
```

## Testing Protocol for ANY Change

### Required Tests
```bash
# 1. Build test
pixi run mojo build -I . omendb/native.mojo -o python/omendb/native.so --emit shared-lib

# 2. Performance test (MUST maintain 79K vec/s)
PYTHONPATH=python:$PYTHONPATH python benchmarks/quick_benchmark.py

# 3. Memory test
PYTHONPATH=python:$PYTHONPATH python benchmarks/test_memory_stats.py

# 4. Persistence test
PYTHONPATH=python:$PYTHONPATH python benchmarks/test_persistence.py
```

### Performance Regression = Immediate Revert
- If batch performance drops below 75K vec/s → REVERT
- If search latency increases > 10% → REVERT
- If memory usage increases > 5% → REVERT

## Focus Areas for v0.1.0-dev

### 1. Fix Critical Bugs
- [ ] Memory stats not showing graph memory
- [ ] Quantization not being applied
- [ ] Vector normalization changing user data
- [ ] Recovery from memory-mapped files

### 2. Improve Reliability
- [ ] Add comprehensive error handling
- [ ] Validate all inputs
- [ ] Add data integrity checks
- [ ] Test edge cases thoroughly

### 3. Documentation
- [ ] Document every function in native.mojo
- [ ] Add section headers and explanations
- [ ] Create API stability guarantees
- [ ] Write migration guides

## Future Refactor Plan (Post-Mojo-Stability)

### Phase 1: When Module Vars Stabilize (2026+)
```
core/
├── state.mojo           # All global state
├── config.mojo          # Configuration management
└── context.mojo         # Shared context
```

### Phase 2: When Traits/Interfaces Land
```
core/
├── database.mojo        # VectorStore implementation
├── collections.mojo     # Collection management
└── interfaces.mojo      # Common protocols
```

### Phase 3: Final Structure
```
omendb/
├── native.mojo          # Thin FFI layer only (~200 lines)
├── core/                # Core database logic
├── algorithms/          # Index algorithms
├── storage/             # Storage engines
└── utils/               # Utilities
```

## What NOT to Do Now (LEARNED FROM EXPERIENCE)

1. **Don't extract VectorStore** - Causes segfault due to global dependencies
   - Specifically: `__buffer_size` referenced across module boundaries
   - Module initialization order issues with UnsafePointer
   
2. **Don't refactor global state** - Mojo doesn't support module-level vars
   - Timeline: Listed as 2026+ in Mojo roadmap (4+ months minimum)
   - Workaround: Use private globals with `__` prefix
   
3. **Don't create complex abstractions** - Mojo lacks traits/interfaces
   - No protocol/interface support yet
   - Generic programming is limited
   
4. **Don't prioritize "clean code" over performance** 
   - 79K vec/s is the minimum acceptable performance
   - Every refactor must be tested with benchmarks
   - Immediate revert if performance degrades >5%

## Success Metrics for v0.1.0-dev

- [ ] Zero segfaults in 1M operation test
- [ ] Maintains 79K+ vec/s batch insert
- [ ] < 2ms search latency at 1M vectors
- [ ] All critical bugs fixed
- [ ] 100% test coverage for error paths
- [ ] Clear documentation for all APIs

## Key Lessons Learned

1. **Global State is the Blocker** - Not file size or complexity
   - VectorStore depends on 5+ global variables
   - Mojo's module system can't handle this yet
   
2. **Section Markers Work Well** - Improves navigation without risk
   - Makes 3000+ line file manageable
   - Prepares for future extraction
   
3. **Pure Functions Extract Safely** - Types and validation worked
   - No state dependencies = safe to extract
   - Test thoroughly after extraction

## Conclusion

**Keep the monolith, make it bulletproof.**

The current architecture works. Focus on:
1. Fixing bugs (memory stats, quantization, normalization)
2. Improving reliability (error handling, validation)
3. Documenting thoroughly (section markers, comments)
4. Preparing for future extraction (marked dependencies)

When Mojo stabilizes (2026+), we'll have a well-documented, well-tested codebase ready for proper modularization. The section markers and documentation will make extraction straightforward once language support exists.