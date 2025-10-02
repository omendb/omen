# Contributing to OmenDB

Thank you for your interest in contributing to OmenDB! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites
- Rust 1.75+ (latest stable)
- Git
- 8GB+ RAM
- NVMe SSD recommended

### Getting Started

```bash
# Clone repository
git clone https://github.com/omendb/omendb-core.git
cd omendb-core

# Run tests
cargo test

# Run benchmarks
cargo bench

# Start development server
cargo run --bin postgres_server
```

## Project Structure

```
omendb/core/
├── src/                    # Source code
│   ├── lib.rs              # Library entry point
│   ├── index.rs            # Learned index (RMI) implementation
│   ├── redb_storage.rs     # Storage layer with learned index
│   ├── datafusion/         # DataFusion integration
│   ├── postgres/           # PostgreSQL wire protocol
│   └── rest/               # REST API
├── tests/                  # Integration tests
├── benches/                # Performance benchmarks
└── docs/                   # User documentation
```

## Development Workflow

### 1. Create a Branch
```bash
git checkout -b feature/your-feature-name
```

### 2. Make Changes
- Write clean, idiomatic Rust code
- Follow existing code style (use `cargo fmt`)
- Add tests for new functionality
- Update documentation as needed

### 3. Run Tests
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### 4. Check Code Quality
```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings

# Check compilation
cargo check
```

### 5. Commit Changes
```bash
# Follow conventional commits format
git commit -m "feat: add new feature"
git commit -m "fix: resolve bug"
git commit -m "docs: update README"
```

Commit types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Test additions or changes
- `refactor`: Code refactoring
- `chore`: Build process or tooling changes

### 6. Push and Create PR
```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub.

## Testing Requirements

All code contributions must include tests. See `internal/TESTING_REQUIREMENTS.md` for detailed testing guidelines.

### For Performance Features
If your contribution claims performance improvement, you MUST include:

1. **Implementation Verification Test** - Proves feature is actually used
2. **Baseline Comparison Test** - Measures performance with feature ON vs OFF
3. **Benchmark** - Added to `benches/` directory
4. **Documentation** - Expected performance characteristics

See `internal/TESTING_REQUIREMENTS.md` for full requirements.

### Test Coverage Guidelines
- Unit tests: Test individual functions and modules
- Integration tests: Test component interactions
- Performance tests: Validate performance claims
- All tests must pass before PR is merged

## Code Style

### Rust Style
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Use `cargo clippy` for lints
- Prefer explicit types over `auto` when it improves clarity
- Document public APIs with `///` doc comments

### Documentation
- Add doc comments for all public APIs
- Include examples in doc comments
- Update user documentation in `docs/` for user-facing changes
- Update architecture docs for significant changes

## Performance Contributions

If you're contributing a performance optimization:

1. **Measure baseline** - Run benchmarks before your changes
2. **Implement optimization** - Make your changes
3. **Measure improvement** - Run benchmarks after your changes
4. **Document results** - Include benchmark results in PR description
5. **Add regression test** - Add benchmark to prevent future regressions

Example:
```bash
# Baseline (before)
cargo bench --bench critical_path_benchmarks

# Make changes...

# After
cargo bench --bench critical_path_benchmarks

# Compare results
```

## Reporting Issues

When reporting bugs or issues, please include:

1. **OmenDB version** - Output of `omendb --version`
2. **Operating system** - OS and version
3. **Rust version** - Output of `rustc --version`
4. **Minimal reproduction** - Smallest code example that reproduces the issue
5. **Expected behavior** - What you expected to happen
6. **Actual behavior** - What actually happened
7. **Logs** - Relevant log output

## Feature Requests

Feature requests are welcome! Please include:

1. **Use case** - Why is this feature needed?
2. **Proposed solution** - How should it work?
3. **Alternatives** - What alternatives have you considered?
4. **Additional context** - Any other relevant information

## Code Review Process

All contributions go through code review:

1. **Automated checks** - CI runs tests, lints, and benchmarks
2. **Maintainer review** - A maintainer reviews your code
3. **Feedback** - Address any review comments
4. **Approval** - Maintainer approves PR
5. **Merge** - PR is merged to main branch

### Review Criteria
- Code quality and style
- Test coverage
- Documentation
- Performance impact
- Backward compatibility

## License

By contributing to OmenDB, you agree that your contributions will be licensed under the same license as the project (Proprietary - OmenDB Inc.).

## Questions?

If you have questions about contributing:

- Open an issue on GitHub
- Check existing issues and PRs
- Read the documentation in `docs/`

Thank you for contributing to OmenDB!
