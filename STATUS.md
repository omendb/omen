# OmenDB Status (October 2025)

## Critical Update
**Major finding**: SoA strategy is WRONG for HNSW. Industry benchmarks prove AoS is 7x faster due to cache locality.

## Current Performance
- **Before**: 763 vec/s (broken imports)
- **Now**: 427 vec/s (working, generic SIMD)
- **Target**: 25K+ vec/s

## What Changed
1. ✅ Fixed broken SIMD imports
2. ✅ Build now succeeds
3. ⚠️ Discovered SoA will hurt performance
4. 🔄 Pivoting to cache-optimized AoS approach

## Next Steps
1. **Zero-copy FFI** - Eliminate 50% overhead
2. **Keep AoS** - Better for HNSW's random access
3. **Cache prefetching** - Predict next nodes
4. **NO SoA migration** - Would make things worse

See `internal/STATUS.md` for full details.