# Dependency Management Strategy

## Philosophy

**Use semver ranges appropriately** - specify minimum required, let tooling pick maximum compatible.

## Version Specification Guidelines

### Rust Dependencies

**Major versions** for stable APIs:
```toml
# Stable crates with good semver practices
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
anyhow = "1"
thiserror = "1"
```

**Minor versions** for breaking-change-prone crates:
```toml
# Rapidly evolving async ecosystem
axum = { version = "0.7", features = ["ws", "macros"] }
tower = { version = "0.4", features = ["full"] }
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls"] }

# FFI with specific capability requirements
pyo3 = { version = "0.25", features = ["auto-initialize"] }  # Python 3.13 support
```

**Patch versions** only when necessary:
```toml
# Security fixes or critical compatibility
# openssl = "=1.0.2"  # Only when required
```

### Docker Images

**Track appropriate channels**:
```dockerfile
# Build stage - latest stable toolchain
FROM rust:1-slim as builder

# Runtime - latest LTS for stability
FROM ubuntu:lts
```

**Avoid** over-specific tags:
```dockerfile
# ❌ Too specific - locks to exact versions
FROM rust:1.75.2-slim
FROM ubuntu:22.04.3

# ✅ Appropriate tracking
FROM rust:1-slim
FROM ubuntu:lts
```

## Tool Usage

### Adding Dependencies

**Preferred approach**:
```bash
# Let Cargo pick latest compatible
cargo add serde --features derive
cargo add tokio --features full

# Specify version only when needed
cargo add pyo3@0.25 --features auto-initialize
```

**Avoid manual Cargo.toml editing** unless complex requirements.

### Updating Dependencies

**Regular updates**:
```bash
# Update within semver ranges
cargo update

# Check for newer major versions
cargo outdated  # requires cargo-outdated
```

**Major version upgrades**:
```bash
# Explicit upgrade when ready
cargo add serde@2 --features derive
```

## Rationale

### Benefits of Semver Ranges

**Security**: Automatic patch updates within ranges
**Stability**: Major version boundaries prevent breaking changes  
**Performance**: Minor updates include optimizations
**Compatibility**: Ecosystem moves together on minor updates

### Risk Management

**Conservative for critical paths**:
- Database drivers (SQLx minor versions)
- Async frameworks (Axum/Tower minor versions)  
- FFI libraries (PyO3 minor versions)

**Liberal for utilities**:
- Error handling (anyhow, thiserror major versions)
- Serialization (serde major versions)
- Logging (tracing major versions)

## Monitoring

**Check for updates monthly**:
```bash
cargo outdated
cargo audit  # Security vulnerabilities
```

**Test before production**:
```bash
cargo update    # Get latest in ranges
cargo test      # Validate compatibility
cargo clippy    # Check for new lints
```

## Exception Cases

**Pin exact versions when**:
- Security vulnerability in range
- Known regression in newer versions
- Deterministic builds required (CI/CD)
- Ecosystem incompatibilities

**Document reasoning** in Cargo.toml comments:
```toml
# Pin for security: CVE-2024-XXXX in 1.2.3-1.2.7
vulnerable-crate = "=1.2.8"
```

## Current Status

✅ **Dependencies use appropriate semver ranges**
✅ **PyO3 v0.25+ provides native Python 3.13 support**  
✅ **Docker images track stable channels**
✅ **No over-specification of patch versions**

This strategy provides **security updates** while maintaining **stability** and **compatibility**.