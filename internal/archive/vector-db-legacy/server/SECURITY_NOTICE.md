# Security Notice: SQLx Protocol-Level Injection Vulnerability

**Date**: July 30, 2025  
**Severity**: High (if using SQL features)  
**Status**: ✅ FIXED

## Summary

A protocol-level SQL injection vulnerability was discovered in SQLx ≤ 0.8.0 where encoding values larger than 4GiB causes the length prefix to overflow, allowing attackers to inject arbitrary protocol commands.

## Impact on OmenDB

**Current Impact**: MINIMAL
- SQLx is a dependency but **not actively used** in our codebase
- Only referenced in error handling for future database features
- No SQL queries or database connections exist yet

**Future Risk**: HIGH (when we add database features)
- Planned for tenant metadata storage
- API key management
- Usage tracking and analytics

## Resolution

✅ **Completed**: Updated SQLx from v0.7 to v0.8.2 (includes fix)

## Additional Mitigations

When we implement database features, ensure:

1. **Input Validation**:
   ```rust
   // Reject any input over 1GB (well below 4GB limit)
   const MAX_INPUT_SIZE: usize = 1_073_741_824; // 1GB
   
   if input.len() > MAX_INPUT_SIZE {
       return Err(Error::validation("Input exceeds maximum size"));
   }
   ```

2. **Request Size Limiting**:
   - Add middleware to limit request body sizes
   - Default to 100MB max for vector operations
   - Much smaller limits for metadata operations

3. **Encoding Validation**:
   - Use `Encode::size_hint()` for sanity checks
   - Be aware that JSON/Text adapters may underestimate size

## References

- [DEF CON 32 Presentation](http://web.archive.org/web/20240812130923/https://media.defcon.org/DEF%20CON%2032/DEF%20CON%2032%20presentations/DEF%20CON%2032%20-%20Paul%20Gerste%20-%20SQL%20Injection%20Isn't%20Dead%20Smuggling%20Queries%20at%20the%20Protocol%20Level.pdf)
- [SQLx GitHub Advisory](https://github.com/launchbadge/sqlx/security)

## Action Items

- [x] Update SQLx to v0.8.2+
- [ ] Add input size validation when implementing DB features
- [ ] Configure request body limits in server middleware
- [ ] Security review before enabling any SQL functionality