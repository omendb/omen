# OmenDB Known Limitations
## As of September 27, 2025

## 1. RMI (Recursive Model Index) Limitations

### Extreme Data Clustering
**Issue**: The learned index fails when data has extreme clustering with large gaps (e.g., 1M+ gap between clusters).

**Impact**:
- 0% recall on clustered datasets with gaps > 100K
- Affects time-series with irregular patterns
- Makes index unsuitable for sparse data

**Root Cause**:
Linear models in RMI cannot accurately predict positions when data distribution is highly non-uniform. The root model's predictions become wildly inaccurate.

**Workaround**:
- Use B-tree fallback for clustered data
- Pre-process data to reduce gaps
- Consider hybrid approach for sparse regions

**Test Coverage**:
- `test_worst_case_distribution` - IGNORED
- `test_time_series_pattern` - IGNORED

---

## 2. Scale Limitations

### Tested Scale
- **Reliable**: Up to 1M keys
- **Untested**: 10M+ keys in production scenarios
- **Unknown**: Behavior at 1B+ keys

### Memory Usage
- No memory profiling done
- Potential memory leaks not investigated
- No OOM handling

---

## 3. Concurrency Limitations

### Current Implementation
- Basic RwLock protection
- No lock-free structures
- No parallel index construction
- Sequential bulk insertion

### Performance Impact
- All writes are serialized
- Read concurrency limited by lock contention
- No MVCC or snapshot isolation

---

## 4. Durability Limitations

### WAL Implementation
- Basic implementation, not battle-tested
- No compression
- No parallel recovery
- Recovery time scales linearly with log size

### Crash Recovery
- Not tested under real crash scenarios
- No partial write handling
- No torn page detection

---

## 5. Query Limitations

### Unsupported Operations
- No JOIN support
- No GROUP BY
- No complex predicates
- No full-text search
- No geospatial queries

### Range Query Performance
- Linear scan within ranges
- No range index optimization
- Poor performance on large ranges

---

## 6. Operational Limitations

### Monitoring
- **NONE** - Complete blind spot

### Security
- **NONE** - Wide open

### Backup/Restore
- Manual only
- No incremental backup
- No point-in-time recovery

### Configuration
- No dynamic configuration
- No performance tuning knobs
- No resource limits

---

## 7. Data Type Limitations

### Supported Types
- Only i64 keys
- Only simple values
- No composite keys
- No variable-length data

### Schema
- Fixed schema only
- No schema evolution
- No ALTER TABLE equivalent

---

## 8. Platform Limitations

### OS Support
- Tested only on macOS
- Linux assumed to work
- Windows unknown

### Architecture
- x86_64 only
- ARM64 untested
- No SIMD optimizations

---

## 9. Network/Protocol Limitations

### Current State
- No network protocol
- No client libraries
- No connection pooling
- No load balancing

---

## 10. Compliance Limitations

### Certifications
- No SOC2
- No HIPAA
- No GDPR compliance
- No encryption at rest
- No audit logging

---

## Risk Matrix

| Limitation | Severity | Likelihood | Risk Level |
|-----------|----------|------------|------------|
| RMI clustering failure | HIGH | MEDIUM | **HIGH** |
| No monitoring | CRITICAL | CERTAIN | **CRITICAL** |
| No security | CRITICAL | CERTAIN | **CRITICAL** |
| Scale untested | HIGH | HIGH | **HIGH** |
| No crash recovery testing | HIGH | MEDIUM | **HIGH** |
| Memory leaks | MEDIUM | HIGH | **HIGH** |
| No backup/restore | HIGH | LOW | **MEDIUM** |
| Platform limitations | LOW | LOW | **LOW** |

---

## Recommendation

**These limitations make OmenDB unsuitable for production use.**

Priority fixes needed:
1. Security implementation (authentication, encryption)
2. Monitoring and observability
3. Scale validation to 100M+ keys
4. Crash recovery testing
5. RMI clustering workaround

Estimated time to address critical limitations: **6-8 weeks**

---

*This document will be updated as limitations are addressed or new ones are discovered.*