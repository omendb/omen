# ⚡ OmenDB Quick Reference

## Build & Test Commands
```bash
# Build Mojo library
cd omendb/engine
pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib

# Quick test (1K-10K vectors)
pixi run benchmark-quick

# Standard test (1K-100K vectors)  
pixi run benchmark-standard

# Core tests
make test-core

# Compression tests
make test-compression
```

## Debug Commands
```bash
# Enable stack traces
export MOJO_ENABLE_STACK_TRACE_ON_ERROR=1

# Debug build
mojo build -debug-level=line-tables native.mojo

# Debug with Mojo
mojo debug native.mojo

# Memory debugging
valgrind --leak-check=full python test.py
```

## Common Fixes
```bash
# Segfault on duplicate IDs
python -c "from omendb import DB; db = DB(); db.clear()"

# Import error
export PYTHONPATH="${PYTHONPATH}:$(pwd)/python"

# Mojo not found
source ~/.modular/bin/activate

# Build error
pixi install  # Reinstall environment
```

## Performance Profiling
```bash
# Profile Python
python -m cProfile -s cumulative benchmark.py

# Profile Mojo (limited)
mojo run --time native.mojo

# System monitoring
htop  # CPU/Memory
iotop  # Disk I/O
```

## Git Workflow
```bash
# Start feature
git checkout -b fix/async-buffer

# Commit
git add -A
git commit -m "fix: implement async buffer manager"

# Update submodules
git submodule update --init --recursive
git submodule update --remote external/agent-contexts
```

## Key File Locations
| What | Where |
|------|-------|
| **Bottleneck** | `omendb/engine/omendb/native.mojo:1850-2000` |
| **Python API** | `omendb/engine/python/omendb/api.py` |
| **Benchmarks** | `omendb/engine/benchmarks/` |
| **Tests** | `omendb/engine/tests/` |
| **Patterns** | `internal/patterns/` |

## Environment Variables
```bash
# Mojo/Pixi
export MODULAR_HOME=~/.modular
export PATH=$MODULAR_HOME/bin:$PATH

# Debug
export MOJO_ENABLE_STACK_TRACE_ON_ERROR=1
export MOJO_BACKTRACE=1

# Performance
export OMP_NUM_THREADS=8
export MKL_NUM_THREADS=8
```

## Documentation Navigation
```
CLAUDE.md          → Start here
ACTION_PLAN.md     → Current priorities
TASKS.md          → All tasks
SESSION_LOG.md    → Work history
DISCOVERIES.md    → Learnings
ERROR_FIXES.md    → Common problems
```

## Benchmark Targets
| Metric | Current | Target |
|--------|---------|--------|
| Vectors | 25K | 1M+ |
| Build rate | 10K/s | 40K/s |
| Query QPS | ? | 8000 |
| Memory/vector | 288 bytes | 288 bytes ✓ |