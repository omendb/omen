# OmenDB Test Suite

**Test Organization**: Following Mojo ecosystem standard with `test/` directory (singular)

## 🏗️ **Test Structure**

### **Integration Tests** (`integration/`)
- **Core functionality**: File persistence, production integration
- **End-to-end workflows**: Real database operations with native module
- **Usage**: `pixi run python test/integration/test_file_persistence.py`

### **Native Module Tests** (`native_module/`)
- **Native bindings**: Direct Mojo module testing
- **Performance validation**: Vector operations, similarity calculations
- **Usage**: `pixi run python test/native_module/test_production_simple.py`

### **Unit Tests** (`unit/`)
- **Mojo components**: Core algorithms, storage, indexing
- **Python components**: API layer, error handling
- **Usage**: Individual `.mojo` files with `pixi run mojo test/unit/...`

### **Performance Tests** (`performance/`)
- **Benchmarking**: vs ChromaDB, scale validation
- **Optimization**: SIMD, RoarGraph algorithm testing
- **Usage**: `pixi run python test/performance/benchmark_vs_chroma.py`

### **Python API Tests** (`python/`)
- **API standards**: Modern Python interface compliance
- **Error handling**: Comprehensive exception testing
- **Usage**: `pixi run python -m pytest test/python/`

## 🚀 **Running Tests**

### **Quick Test** (File Persistence)
```bash
pixi run python test/integration/test_file_persistence.py
```

### **Comprehensive Test** (All Python)
```bash
pixi run python -m pytest test/python/ test/integration/ test/native_module/
```

### **Performance Benchmarks**
```bash
pixi run python test/performance/benchmark_vs_chroma.py
```

### **Individual Mojo Tests**
```bash
pixi run mojo test/unit/core/test_vector.mojo
```

## 📊 **Test Categories**

| Category | Purpose | Files | Status |
|----------|---------|-------|---------|
| **Integration** | End-to-end workflows | `integration/` | ✅ Production |
| **Native Module** | Mojo bindings | `native_module/` | ✅ Production |  
| **Unit** | Component testing | `unit/` | 🔄 Mixed |
| **Performance** | Benchmarking | `performance/` | ✅ Production |
| **Python API** | API validation | `python/` | ✅ Production |

## 🎯 **Test Philosophy**

**Mojo-First Testing**: Following `external/modular/mojo/stdlib/test/` convention:
- **Single `test/` directory** (not `tests/`)
- **Mixed Mojo + Python** test organization
- **Performance-focused** validation
- **Real operations** over mocks

**Quality Standards**:
- ✅ **Real functionality**: No stubs in production tests
- ✅ **Performance validation**: Actual benchmarks vs competitors  
- ✅ **Error handling**: Comprehensive exception testing
- ✅ **File operations**: Real persistence with .omen format