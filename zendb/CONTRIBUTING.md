# Contributing to ZenDB

Thank you for your interest in contributing to ZenDB! We embrace the zen philosophy of simplicity and harmony in both our codebase and community.

## 🧘‍♂️ Zen Principles for Contributors

1. **Simplicity First**: Start with the simplest solution that works
2. **Balance**: Consider both performance and maintainability
3. **Harmony**: Ensure your changes work well with existing code
4. **Natural Flow**: Follow the existing patterns and conventions

## 🚀 Getting Started

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes following our guidelines
4. Write or update tests as needed
5. Run the test suite (`cargo test`)
6. Commit with clear messages
7. Push to your fork and open a Pull Request

## 📝 Development Process

### Setting Up Your Environment

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/zendb.git
cd zendb

# Build the project
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings
```

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Address all clippy warnings
- Write clear, self-documenting code
- Add comments only when the "why" isn't obvious
- Use descriptive variable and function names

### Testing

- Write property-based tests using `proptest` for storage engine components
- Include unit tests for new functionality
- Add integration tests for API changes
- Ensure all tests pass before submitting PR

### Documentation

- Update relevant documentation for API changes
- Include examples in doc comments
- Keep README.md current with new features
- Update CLAUDE.md if architecture changes

## 🎯 What We're Looking For

### High Priority

- Storage engine optimizations
- Time-travel query implementation
- Real-time subscription features
- PostgreSQL protocol compatibility improvements
- Performance improvements with benchmarks

### Good First Issues

Look for issues labeled `good-first-issue` for beginner-friendly tasks.

## 📊 Performance Contributions

If you're improving performance:

1. Include benchmark results (before/after)
2. Test on different workloads
3. Consider memory usage, not just speed
4. Document any trade-offs

## 🐛 Bug Reports

Please include:

- ZenDB version
- Rust version (`rustc --version`)
- Operating system
- Minimal reproducible example
- Expected vs actual behavior
- Any relevant logs or error messages

## 💡 Feature Requests

We appreciate feature ideas! Please:

- Check existing issues first
- Explain the use case
- Describe how it fits ZenDB's philosophy
- Consider implementation complexity

## 📜 License

By contributing, you agree that your contributions will be licensed under the Elastic License 2.0.

## 🤝 Code of Conduct

We follow the Rust Code of Conduct. Be kind, respectful, and constructive.

## 🙏 Recognition

All contributors will be recognized in our releases and documentation.

---

**Find your zen in database development** 🧘‍♂️