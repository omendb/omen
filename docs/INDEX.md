# OmenDB Documentation Index

## Core Documentation

| File | Purpose | Last Updated |
|------|---------|--------------|
| [CLAUDE.md](../CLAUDE.md) | AI agent context and quick facts | 2025-09-01 |
| [STATUS.md](STATUS.md) | Current metrics, issues, and state | 2025-09-01 |
| [WORKFLOW.md](WORKFLOW.md) | Development processes and Linear integration | 2025-09-01 |
| [DOC_STANDARDS.md](DOC_STANDARDS.md) | General documentation standards | 2025-09-01 |

## Technical Documentation

| File | Purpose | Last Updated |
|------|---------|--------------|
| [TECH_SPEC.md](TECH_SPEC.md) | Architecture and implementation details | 2025-08-31 |
| [MOJO_PATTERNS.md](MOJO_PATTERNS.md) | Mojo-specific workarounds and patterns | 2025-08-31 |

## Investigation & Analysis

| File | Purpose | Last Updated |
|------|---------|--------------|
| [PERFORMANCE_INVESTIGATION.md](PERFORMANCE_INVESTIGATION.md) | DiskANN performance regression analysis | 2025-09-01 |
| [REGRESSION_TRACKING.md](REGRESSION_TRACKING.md) | Performance monitoring system | 2025-09-01 |
| [SCALE_ACHIEVEMENT.md](SCALE_ACHIEVEMENT.md) | 50x scale improvement documentation | 2025-09-01 |

## Information Location Guide

### Where to Find Specific Information

**Current Performance Metrics** → [STATUS.md](STATUS.md#performance-metrics)  
**Known Issues** → [STATUS.md](STATUS.md#known-issues)  
**Memory Efficiency** → [STATUS.md](STATUS.md#memory-optimization-journey)  
**Scale Achievements** → [SCALE_ACHIEVEMENT.md](SCALE_ACHIEVEMENT.md)  
**Architecture Details** → [TECH_SPEC.md](TECH_SPEC.md)  
**Mojo Workarounds** → [MOJO_PATTERNS.md](MOJO_PATTERNS.md)  
**Development Process** → [WORKFLOW.md](WORKFLOW.md)  
**Linear Issues** → [WORKFLOW.md](WORKFLOW.md#linear-issue-management)  
**Regression Tracking** → [WORKFLOW.md](WORKFLOW.md#performance-regression-tracking)  

### Where to Update Information

**Performance changes** → Update [STATUS.md](STATUS.md), add investigation to dedicated file if complex  
**New bugs found** → Update [STATUS.md](STATUS.md#known-issues), create Linear issue  
**Architecture changes** → Update [TECH_SPEC.md](TECH_SPEC.md), document decision  
**Scale improvements** → Update [STATUS.md](STATUS.md), create achievement doc if major  
**Process changes** → Update [WORKFLOW.md](WORKFLOW.md)  
**Mojo discoveries** → Update [MOJO_PATTERNS.md](MOJO_PATTERNS.md)  

## Archive

| File | Purpose | Archived Date |
|------|---------|---------------|
| [archive/TODO_ARCHIVED_2025_09_01.md](archive/TODO_ARCHIVED_2025_09_01.md) | Old TODO items | 2025-09-01 |

## Active Linear Issues (Sept 1, 2025)

### Critical (P1)
- **OMEN-27**: 🔥 DiskANN MERGE MODE Performance Regression (95% slower) - BLOCKS RELEASE
- **OMEN-7**: SIMD Optimizations (in progress)

### High Priority (P2) 
- **OMEN-21**: Comprehensive Testing Suite (blocked by OMEN-27)
- **OMEN-11**: Code Comments Cleanup

### Medium Priority (P3)
- **OMEN-26**: Segfaults at 105K vectors (may resolve with OMEN-27)
- **OMEN-19**: Documentation Updates (after testing)

### Release Planning
- **OMEN-22**: v0.1.0 Release Plan (Sept 28 target)

## Quick References

### Latest Achievements (Sept 1, 2025)
- **100K vectors**: STABLE (2.1KB/vector, 1.4ms search)
- **50x scale improvement**: From 2K → 100K stable limit
- **Memory efficiency**: 2-5x better than Chroma/Weaviate
- **Performance regression**: Found and documented (OMEN-27)

### Current Blockers
- **OMEN-27**: 95% performance regression in MERGE MODE
- **Scale testing**: Cannot test beyond 100K due to segfaults
- **1M target**: Blocked until DiskANN issues resolved

### Next Actions
1. Investigate and fix DiskANN MERGE MODE performance (OMEN-27)
2. Resolve segfaults at 105K boundary (OMEN-26) 
3. Complete comprehensive testing suite (OMEN-21)
4. Finalize v0.1.0 release (OMEN-22)

---
*Documentation index for OmenDB project. Update this file when adding/moving major documentation.*