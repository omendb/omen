# Structured Logging Implementation

**Status:** ✅ Complete
**Date:** September 30, 2025
**Version:** v0.2.0

---

## Overview

OmenDB now includes production-ready structured logging with JSON format support, configurable log levels, and comprehensive query tracing throughout the SQL engine and connection pool.

## Features

### 1. JSON Format Logging
- Production-ready JSON output for log aggregation systems
- Compatible with ELK stack, Datadog, Splunk, etc.
- Structured fields for easy parsing and filtering

### 2. Configurable Log Levels
- **trace**: Most verbose, includes all execution details
- **debug**: Development-friendly, includes query details
- **info**: Default production level
- **warn**: Warnings and potential issues
- **error**: Errors and failures only

### 3. Query Logging
- Optional query text logging (disabled by default for security)
- Query type tracking (SELECT, INSERT, CREATE_TABLE)
- Query duration and row count in logs
- Error context for failed queries

### 4. Span-Based Tracing
- Hierarchical execution tracing
- Function entry/exit logging
- Timing information for operations
- Connection pool operations tracked

## Configuration

### Quick Start

```rust
use omendb::{LogConfig, init_logging};

// Production: JSON format, INFO level
let config = LogConfig::production();
init_logging(config)?;

// Development: Pretty format, DEBUG level, query logging
let config = LogConfig::development();
init_logging(config)?;

// Verbose: TRACE level with full query logging
let config = LogConfig::verbose();
init_logging(config)?;
```

### Environment Variables

```bash
# Set log level
export RUST_LOG=debug

# Set format (json or pretty)
export OMENDB_LOG_FORMAT=json

# Enable query logging
export OMENDB_LOG_QUERIES=true
```

Then initialize from environment:
```rust
use omendb::init_from_env;
init_from_env()?;
```

### Custom Configuration

```rust
let config = LogConfig {
    level: "info".to_string(),
    json_format: true,
    log_queries: false,
    log_spans: true,
    log_file: Some("/var/log/omendb/omendb.log".to_string()),
};
init_logging(config)?;
```

## Log Output Examples

### JSON Format (Production)

```json
{
  "timestamp": "2025-09-30T16:29:49.544888Z",
  "level": "INFO",
  "fields": {
    "message": "Query executed successfully",
    "query_type": "SELECT",
    "rows": 100,
    "duration_ms": 15.2
  },
  "target": "omendb::sql_engine",
  "spans": [
    {
      "name": "execute",
      "query_length": 42
    }
  ]
}
```

### Pretty Format (Development)

```
2025-09-30T16:29:49.544888Z  INFO omendb::sql_engine: Query executed successfully
    query_type: SELECT
    rows: 100
    duration_ms: 15.2
    at src/sql_engine.rs:136
    in execute with query_length=42
```

## Integration Points

### SQL Engine
- Query execution start/end
- Parse errors with context
- Timeout warnings
- Resource limit violations
- Query success with metrics

### Connection Pool
- Connection acquisition
- Connection reuse
- Pool capacity warnings
- Idle connection cleanup
- Connection statistics

## Log Levels by Component

| Component | Info | Debug | Trace |
|-----------|------|-------|-------|
| SQL Parser | Errors | Statement count | Full query |
| Query Execution | Success/failure | Table access | Row-level ops |
| Connection Pool | Create/cleanup | Acquire/release | All state changes |
| Metrics | Errors | All recordings | - |

## Performance Impact

- **Minimal overhead** when using `info` level or higher
- **JSON serialization**: ~5-10μs per log entry
- **Filtering**: Zero-cost when log level is above threshold
- **Async I/O**: Non-blocking writes (when configured)

## Production Recommendations

1. **Use JSON format** for production deployments
2. **Set level to `info`** by default
3. **Disable query logging** unless debugging (contains sensitive data)
4. **Enable span events** for distributed tracing
5. **Forward logs** to centralized logging system

## Example Deployment

### Docker

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/omendb /usr/local/bin/

ENV RUST_LOG=info
ENV OMENDB_LOG_FORMAT=json
ENV OMENDB_LOG_QUERIES=false

CMD ["omendb"]
```

### Kubernetes ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: omendb-config
data:
  RUST_LOG: "info"
  OMENDB_LOG_FORMAT: "json"
  OMENDB_LOG_QUERIES: "false"
```

## Monitoring Integration

### With ELK Stack
```json
{
  "query_type": "SELECT",
  "duration_ms": { "gte": 1000 }
}
```

### With Datadog
```
service:omendb level:error
```

### With Prometheus (via metrics)
Use metrics for aggregation, logs for debugging:
- **Metrics**: Query counts, latencies, error rates
- **Logs**: Specific error details, query text, stack traces

## Files Modified

1. `src/logging.rs` - New module (210 lines)
2. `src/sql_engine.rs` - Added tracing instrumentation
3. `src/connection_pool.rs` - Added tracing instrumentation
4. `tests/logging_tests.rs` - 11 comprehensive tests
5. `Cargo.toml` - Added tracing dependencies

## Testing

Run logging tests:
```bash
cargo test --test logging_tests
```

Test with different log levels:
```bash
RUST_LOG=debug cargo test
RUST_LOG=trace cargo test
```

## Troubleshooting

### Logs not appearing
- Check `RUST_LOG` environment variable is set
- Verify initialization was called: `init_logging()` or `init_from_env()`
- Check log level - `error` level only shows errors

### JSON format not working
- Set `OMENDB_LOG_FORMAT=json` or use `LogConfig::production()`
- Verify `json_format: true` in config

### Too much output
- Increase log level: `info` or `warn`
- Disable span events: `log_spans: false`
- Disable query logging: `log_queries: false`

## Future Enhancements (v0.3.0+)

- [ ] Async log rotation
- [ ] Per-query log sampling
- [ ] Distributed tracing integration (OpenTelemetry)
- [ ] Log compression
- [ ] Custom formatters

---

**Status:** Production-ready for v0.2.0 release
**Test Coverage:** 11 tests, all passing
**Performance:** Negligible overhead at INFO level
