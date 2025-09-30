# Bugs Found During Comprehensive Verification

**Context**: All code was LLM-generated. User requested comprehensive verification before open sourcing.

**Approach**: Systematic testing following VERIFICATION_PLAN.md

---

## 🐛 Critical Bugs Found (5 total)

### Bug #1: Negative Numbers Not Supported in INSERT ✅ FIXED
**Severity**: HIGH
**Found**: Edge case testing (test_negative_keys)
**Impact**: Cannot insert negative values like -50 or negative timestamps

**Root Cause**: SQL parser produces `UnaryOp { op: Minus, expr: Number("50") }` for negative numbers, but `expr_to_value()` didn't handle UnaryOp.

**Fix**: Added UnaryOp handling in `src/sql_engine.rs` expr_to_value() (lines 488-515)

```rust
Expr::UnaryOp { op, expr } => {
    match op {
        UnaryOperator::Minus => {
            let value = Self::expr_to_value(expr, expected_type)?;
            Ok(Value::Int64(-n)) // or Float64, Timestamp
        }
        ...
    }
}
```

**Verification**: test_negative_keys now passes (INSERT and SELECT with -50 to 50)

---

### Bug #2: i64::MIN Overflow in INSERT ✅ FIXED
**Severity**: HIGH
**Found**: Edge case testing (test_boundary_values)
**Impact**: Cannot insert i64::MIN (-9223372036854775808)

**Root Cause**: i64::MIN as positive number (9223372036854775808) = i64::MAX + 1, which overflows when parsing.

**Fix**: Special case in `expr_to_value()` to detect i64::MIN and return it directly:

```rust
if n == "9223372036854775808" && matches!(expected_type, DataType::Int64) {
    return Ok(Value::Int64(i64::MIN));
}
```

**Verification**: test_boundary_values now passes (INSERT and SELECT with i64::MIN and i64::MAX)

---

### Bug #3: Negative Numbers Not Supported in WHERE Clause ✅ FIXED
**Severity**: HIGH
**Found**: Edge case testing (test_boundary_values SELECT query)
**Impact**: Cannot query with negative values in WHERE: `WHERE id = -50` fails

**Root Cause**: `evaluate_value_expr()` didn't handle UnaryOp, same as Bug #1.

**Fix**: Added UnaryOp handling in `evaluate_value_expr()` with same logic as Bug #1/2.

**Verification**: test_negative_keys and test_boundary_values SELECT queries pass

---

### Bug #4: Learned Index Broken at Scale (Floating-Point Precision) ✅ FIXED
**Severity**: **CRITICAL** 🚨
**Found**: 50M keys stress test (test_50m_keys_scale)
**Impact**: **Learned index completely non-functional with large keys** - predicted wrong model (model 8 instead of model 0 for first key!)

**Root Cause**:
- Keys like 1_600_000_000_000_000 (1.6e15) cause floating-point precision loss
- Linear regression computes sum_xx = sum(key^2) ≈ 50M * (1.6e15)^2 = 1.28e38
- This causes severe precision loss in f64 arithmetic
- Root model predicted model_idx: 8 for key at position 0 (should be model 0)

**Fix**: Normalize keys to [0, 1] range during training, then denormalize:

```rust
let min_key = self.data.first().map(|(k, _)| *k as f64).unwrap_or(0.0);
let max_key = self.data.last().map(|(k, _)| *k as f64).unwrap_or(0.0);
let key_range = (max_key - min_key).max(1.0);

// Train on normalized keys
let x = (*key as f64 - min_key) / key_range;

// Denormalize slope and intercept for actual use
self.root.slope = normalized_slope / key_range;
self.root.intercept = normalized_intercept - self.root.slope * min_key;
```

**Before Fix**:
- First key search: FAILED (returned None)
- Error: "Failed to find key at position 0"
- model_idx: 8/16 (completely wrong!)

**After Fix**:
- First key search: SUCCESS
- model_idx: 0/16 (correct!)
- Average lookup: 220ns at 50M scale

**Verification**: test_50m_keys_scale now passes (was completely broken)

---

### Bug #5: Search Window Size Limit ✅ FIXED
**Severity**: MEDIUM-HIGH
**Found**: During Bug #4 investigation
**Impact**: Returns None if search window > 16 elements (breaks at scale)

**Root Cause**: `search_in_model()` had arbitrary 16-element limit:

```rust
if start < end && end - start <= 16 {  // ← BUG!
    // Binary search
} else {
    None  // Gives up if window > 16!
}
```

**Fix**: Removed size limit, always binary search:

```rust
if start < end {
    // Binary search (no size limit)
}
```

**Verification**: No performance degradation, all tests pass

---

## 📊 Bug Discovery Statistics

**Total Bugs Found**: 5
**Critical Bugs**: 2 (Bugs #4 and #5 - learned index broken)
**High Severity**: 3 (Bugs #1, #2, #3 - negative numbers)

**Detection Methods**:
- Edge case testing: 3 bugs (negative numbers, boundary values)
- Large-scale testing: 2 bugs (learned index at scale)

**Code Quality Assessment**:
- LLM-generated code had fundamental correctness issues
- Edge cases not handled (negative numbers, i64::MIN)
- Floating-point precision not considered at scale
- **Critical**: Core value proposition (learned indexes) was broken at scale

---

## ✅ Verification Status

### Completed:
- ✅ Edge case testing (13 comprehensive tests, all passing)
- ✅ Large-scale testing (50M keys, 220ns avg lookup)
- ✅ Negative number support (INSERT and WHERE)
- ✅ Boundary value support (i64::MIN, i64::MAX, f64 limits)
- ✅ Learned index correctness at scale

### Test Results:
- **150 tests passing** (100% pass rate)
- **13 edge case tests** passing
- **50M keys stress test** passing (220ns avg lookup)
- **10K row WHERE clause test** passing (22.55x speedup)

### Remaining Verification (VERIFICATION_PLAN.md):
- SQL correctness validation
- WAL recovery with corruption scenarios
- Concurrent operations and race conditions
- Data integrity verification
- Error handling audit (no panics)
- Security audit (SQL injection, path traversal)

---

## 🎯 Key Takeaways

1. **LLM Code Needs Verification**: Found 5 bugs, 2 critical, in LLM-generated code
2. **Edge Cases Matter**: Negative numbers, boundary values broke the system
3. **Scale Reveals Bugs**: Learned index completely broken at 50M keys
4. **Systematic Testing Works**: Following VERIFICATION_PLAN.md found all bugs
5. **Core Features Were Broken**: Main value prop (learned indexes) didn't work at scale

**Recommendation**: Continue systematic verification before open sourcing.

---

*Last Updated*: After fixing all 5 bugs, all 150 tests passing