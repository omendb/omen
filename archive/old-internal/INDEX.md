# Internal Documentation Index

## 🚨 Quick Actions

### Having Issues?
- **[../ERROR_FIXES.md](../ERROR_FIXES.md)** - Immediate error → fix lookup
- **[patterns/CONCURRENCY_PATTERNS.md](patterns/CONCURRENCY_PATTERNS.md)** - 25K bottleneck fix
- **[patterns/STORAGE_PATTERNS.md](patterns/STORAGE_PATTERNS.md)** - Memory/storage patterns

### Need to Know?
- **[../CLAUDE.md](../CLAUDE.md)** - AI agent navigation and context
- **Current Focus**: Fix OmenDB 25K vector bottleneck
- **Status**: ZenDB on hold, focus on OmenDB

## 📊 Current Status

### OmenDB Engine (PRIMARY FOCUS)
- **Working**: PQ compression (288 bytes/vector), DiskANN algorithm
- **Broken**: Performance at 25K+ vectors (buffer flush issue)
- **Action**: Check `patterns/CONCURRENCY_PATTERNS.md` for fix

### ZenDB (ON HOLD)
- **Status**: 61/70 tests passing, storage engine complete
- **Purpose**: Future multimodal database research
- **Patterns Extracted**: Storage and concurrency → `patterns/`

## 📁 Directory Structure

```
internal/
├── patterns/           # ✅ ACTIONABLE patterns from ZenDB
│   ├── STORAGE_PATTERNS.md      # Memory-mapped I/O, WAL
│   └── CONCURRENCY_PATTERNS.md  # Multi-writer, buffer flush
├── archive/            # 📚 Historical (reference only)
│   └── omendb-engine-investigations/  # Past debugging
├── decisions/          # 📚 Architecture decisions
├── research/           # 📚 Performance research  
├── private/            # 🔒 Business strategy
└── technical/          # 📚 Technical specs
```

## 🎯 Decision Trees

```
IF debugging_25k_bottleneck:
    → patterns/CONCURRENCY_PATTERNS.md (buffer flush fix)
    → omendb/engine/omendb/native.mojo:1850
    
ELIF adding_storage_feature:
    → patterns/STORAGE_PATTERNS.md (mmap, WAL, compression)
    → Follow ZenDB patterns
    
ELIF fixing_mojo_error:
    → ../ERROR_FIXES.md (quick lookup)
    → external/agent-contexts/languages/mojo/MOJO_PATTERNS.md
    
ELIF understanding_architecture:
    → decisions/ (why we chose X)
    → technical/ (how X works)
```

## 🛠️ Key Commands

```bash
# Debug 25K bottleneck
cd omendb/engine
pixi run profile-25k

# Test specific component
pixi run pytest tests/test_buffer.py -xvs

# Check memory usage
pixi run benchmark-memory

# Clean rebuild
pixi run clean && pixi run build
```

## 📋 Files by Purpose

### Debugging Performance
- `patterns/CONCURRENCY_PATTERNS.md` - Buffer flush, multi-writer
- `patterns/STORAGE_PATTERNS.md` - Memory-mapped I/O, caching
- `research/` - Historical performance investigations

### Understanding Design
- `decisions/` - Why DiskANN? Why Mojo? 
- `technical/` - System architecture
- `archive/zendb/` - Multimodal DB design

### Business/Strategy
- `private/business/` - Investor materials (confidential)
- `ROADMAP.md` - Product timeline
- `strategy/` - Market positioning

## ⚠️ Common Pitfalls

1. **Don't use Mojo Dict/List** - 8KB overhead per entry
2. **Always batch FFI calls** - 5x performance difference
3. **Clear DB between tests** - Global singleton issue
4. **Check buffer size** - Currently 1MB (too small)

## Next Priority Actions

1. **Fix 25K bottleneck** - See patterns/CONCURRENCY_PATTERNS.md
2. **Remove global singleton** - omendb/engine/omendb/native.mojo:78
3. **Increase memory pool** - Line 153, change 1MB → 16MB
4. **Make buffer flush async** - Line 1850-2000

---
*Updated: January 2025 - Focus on OmenDB vector database*