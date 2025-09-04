# Pixi Package Manager for Mojo Projects

*Current as of Mojo v25.4 (nightly) - Updated December 2024*

## Overview

[Pixi](https://pixi.sh/latest/) is the recommended package manager for Mojo development projects. It provides conda-based environment management with cross-platform support and reproducible builds.

## Installation

### Install Pixi

```bash
# Install pixi (Linux/macOS)
curl -fsSL https://pixi.sh/install.sh | sh

# Verify installation
pixi --version
```

### Install Shell Integration

```bash
# Add to your shell profile (bash/zsh)
echo 'eval "$(pixi completion --shell bash)"' >> ~/.bashrc
# or for zsh
echo 'eval "$(pixi completion --shell zsh)"' >> ~/.zshrc
```

## Project Setup

### Create New Mojo Project

```bash
# Create new project with Modular channel
pixi init runebase \
  -c https://conda.modular.com/max-nightly -c conda-forge \
  --platform osx-arm64 \
  && cd runebase

# Install Modular Platform (includes Mojo)
pixi add modular

# Verify Mojo installation
pixi run mojo --version
```

### Project Structure

```
runebase/
├── pixi.toml              # Project manifest
├── pixi.lock              # Lock file (auto-generated)
├── .pixi/                 # Virtual environment
├── .gitignore             # Pre-configured
├── .gitattributes         # Pre-configured
├── src/                   # Source code
├── tests/                 # Test files
└── examples/              # Example code
```

## Configuration

### pixi.toml Configuration

```toml
[workspace]
name = "runebase"
version = "0.0.1"
authors = ["Your Name <your.email@example.com>"]
channels = ["https://conda.modular.com/max-nightly", "conda-forge"]
platforms = ["osx-arm64"]  # Start with your platform, add others as verified

[dependencies]
modular = ">=25.5.0.dev2025061905,<26"  # Use specific working versions
python = ">=3.11,<3.14"

[build-dependencies]
cmake = "*"
ninja = "*"
llvm = "*"

[test-dependencies]
pytest = "*"
benchmark = "*"

[dev-dependencies]
pre-commit = "*"
black = "*"
mypy = "*"

# Feature flags
[feature.gpu]
dependencies = { cuda-toolkit = "*" }

[feature.profiling]
dependencies = { 
    valgrind = "*",
    perf = "*",
    flamegraph = "*"
}

[feature.docs]
dependencies = {
    sphinx = "*",
    sphinx-rtd-theme = "*"
}

# Tasks (custom commands) - Add only verified tasks
[tasks]
build = "mojo build src/main.mojo -o build/runebase"
test = "mojo test tests/"
format = "mojo format src/ tests/ examples/"
clean = "rm -rf build/ __pycache__ .pytest_cache"

[tasks.build-debug]
cmd = "mojo build src/main.mojo -g -o build/runebase-debug"
description = "Build debug version"

[tasks.build-release]
cmd = "mojo build src/main.mojo -O3 --march=native -o build/runebase"
description = "Build optimized release version"

[tasks.test-integration]
cmd = "mojo test tests/integration/ --parallel=false"
description = "Run integration tests"

[tasks.profile]
cmd = "perf record -g ./build/runebase < workloads/standard.sql"
description = "Profile with perf"
depends-on = ["build-release"]

[tasks.install-hooks]
cmd = "pre-commit install"
description = "Install git pre-commit hooks"

# Environment variables
[environments]
[environments.development]
features = ["gpu", "profiling", "docs"]
tasks = ["build-debug", "test", "format"]

[environments.production]
features = []
tasks = ["build-release"]

[environments.ci]
features = ["profiling"]
tasks = ["test", "lint", "build-release"]
```

## Common Commands

### Environment Management

```bash
# Activate project environment
pixi shell

# Run command in environment
pixi run mojo --version
pixi run python --version

# Exit environment
exit

# Install new dependencies
pixi add sqlite-dev
pixi add "numpy>=1.20"
pixi add --dev pytest

# Remove dependencies
pixi remove sqlite-dev

# Update dependencies
pixi update
pixi update modular  # Update specific package
```

### Task Execution
### Common Commands

```bash
# Run Mojo commands directly
pixi run mojo --version
pixi run mojo build src/main.mojo -o build/app
pixi run mojo test tests/
pixi run mojo format src/

# Run predefined tasks (if configured)
pixi run build
pixi run test
pixi run format

# Run with specific environment
pixi run --environment development build-debug
pixi run --environment production build-release
```

### Feature Management

```bash
# Install with features
pixi install --environment development  # Includes gpu, profiling, docs

# Add feature-specific dependencies
pixi add --feature gpu cuda-toolkit
pixi add --feature profiling valgrind
```

## Development Workflow

### Initial Setup

```bash
# Clone and setup project
git clone <repository>
cd runebase

# Install dependencies and activate environment
pixi install
pixi shell

# Verify setup
pixi run mojo --version
pixi run build
pixi run test
```

### Daily Development

```bash
# Start development session
pixi shell

# Build and test
pixi run build-debug
pixi run test

# Format and lint
pixi run format
pixi run lint

# Run specific tests
mojo test tests/unit/test_btree.mojo
mojo test tests/integration/

# Profile performance
pixi run profile
```

### Cross-Platform Development

```bash
# Build for different platforms
pixi run --environment production build-release

# Test on different architectures
pixi add --platform linux-aarch64 modular
pixi add --platform osx-arm64 modular
```

## Advanced Configuration

### Lock File Management

```bash
# Generate/update lock file
pixi update

# Install exact versions from lock file
pixi install --locked

# Check for updates
pixi outdated
```

### Custom Channels

```bash
# Add custom conda channel
pixi project channel add https://custom.conda.org/channel

# Add local channel
pixi project channel add file:///path/to/local/channel
```

### Environment Variables

```toml
# In pixi.toml
[tasks.debug]
cmd = "mojo build src/main.mojo -g -o build/runebase-debug"
env = { MOJO_DEBUG = "1", RUST_BACKTRACE = "full" }

[tasks.benchmark-gpu]
cmd = "mojo run benchmarks/gpu_benchmark.mojo"
env = { CUDA_VISIBLE_DEVICES = "0" }
```

## Integration with IDEs

### Visual Studio Code

**.vscode/settings.json:**
```json
{
    "python.defaultInterpreterPath": "./.pixi/envs/default/bin/python",
    "mojo.compiler.path": "./.pixi/envs/default/bin/mojo",
    "terminal.integrated.env.linux": {
        "PIXI_PROJECT_ROOT": "${workspaceFolder}"
    },
    "tasks.version": "2.0.0"
}
```

**.vscode/tasks.json:**
```json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "Pixi Build",
            "type": "shell",
            "command": "pixi",
            "args": ["run", "build"],
            "group": {
                "kind": "build",
                "isDefault": true
            }
        },
        {
            "label": "Pixi Test",
            "type": "shell",
            "command": "pixi",
            "args": ["run", "test"],
            "group": "test"
        }
    ]
}
```

### Launch Configuration

**.vscode/launch.json:**
```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "name": "Debug Runebase",
            "type": "lldb",
            "request": "launch",
            "program": "${workspaceFolder}/build/runebase-debug",
            "args": ["--database", "test.db"],
            "cwd": "${workspaceFolder}",
            "preLaunchTask": "Pixi Build Debug"
        }
    ]
}
```

## CI/CD Integration

### GitHub Actions

**.github/workflows/ci.yml:**
```yaml
name: CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Pixi
      uses: prefix-dev/setup-pixi@v0.4.1
      with:
        pixi-version: latest
        cache: true
    
    - name: Install dependencies
      run: pixi install --environment ci
    
    - name: Run tests
      run: pixi run --environment ci test
    
    - name: Run linting
      run: pixi run --environment ci lint
    
    - name: Build release
      run: pixi run --environment ci build-release
```

### Docker Integration

**Dockerfile:**
```dockerfile
FROM mambaorg/micromamba:latest

# Copy project files
COPY --chown=$MAMBA_USER:$MAMBA_USER . /app
WORKDIR /app

# Install pixi
RUN curl -fsSL https://pixi.sh/install.sh | sh
ENV PATH="/home/mambauser/.local/bin:$PATH"

# Install dependencies
RUN pixi install --environment production

# Build application
RUN pixi run build-release

# Set entrypoint
ENTRYPOINT ["pixi", "run", "--environment", "production"]
CMD ["./build/runebase"]
```

## Troubleshooting

### Common Issues

**Environment activation problems:**
```bash
# Reset environment
pixi clean
pixi install

# Check environment
pixi info
pixi list
```

**Dependency conflicts:**
```bash
# Check for conflicts
pixi tree
pixi update --dry-run

# Force resolve
pixi install --force-reinstall
```

**Lock file issues:**
```bash
# Regenerate lock file
rm pixi.lock
pixi install
```

### Performance Optimization

**Faster installations:**
```bash
# Use parallel downloads
export CONDA_MAX_THREADS=8

# Use local mirror
pixi project channel add file:///local/conda/mirror
```

**Cache management:**
```bash
# Clear cache
pixi clean cache

# Show cache info
pixi cache info
```

### Best Practices

### Project Organization

1. **Keep pixi.toml minimal** - Only include necessary dependencies
2. **Use specific version constraints** - Avoid conflicts with known working versions
3. **Start with single platform** - Add others as verified to work
4. **Minimal task configuration** - Only add tasks that are tested and working
5. **Use `modular` package** - Current standard, not `max`
6. **Document working configurations** - Keep track of what versions work together

### Dependency Management

```toml
# Good: Specific working constraints
[dependencies]
modular = ">=25.5.0.dev2025061905,<26"
python = ">=3.11,<3.14"

# Acceptable: Slightly looser but still constrained  
[dependencies]
modular = ">=25.5.0,<26"
python = ">=3.11,<3.14"

# Bad: Too loose - can cause dependency conflicts
[dependencies]
modular = "*"
python = "*"
```

### Task Definition

```toml
# Good: Clear, focused tasks
[tasks.test-unit]
cmd = "mojo test tests/unit/ --parallel"
description = "Run unit tests in parallel"

[tasks.test-integration]
cmd = "mojo test tests/integration/ --parallel=false"
description = "Run integration tests sequentially"

# Bad: Monolithic tasks
[tasks.test]
cmd = "mojo test tests/unit/ --parallel && mojo test tests/integration/ --parallel=false && mojo test tests/benchmarks/"
```

### Environment Configuration

```toml
# Development environment with all tools
[environments.dev]
features = ["gpu", "profiling", "docs"]
solve-group = "dev"

# Minimal production environment
[environments.prod]
features = []
solve-group = "prod"

# CI environment with testing tools
[environments.ci]
features = ["profiling"]
solve-group = "ci"
```

## Migration from Other Tools

### From conda/mamba

```bash
# Export existing environment
conda env export > environment.yml

# Convert to pixi.toml (manual process)
# Add dependencies to [dependencies] section
```

### From pip

```bash
# Export requirements
pip freeze > requirements.txt

# Add to pixi.toml
# Many packages available through conda-forge
```

### From Magic (deprecated)
### Migration from Magic (deprecated)

Pixi is the successor to Magic for Mojo projects. Migration steps:

1. Create new `pixi.toml` based on existing `magic.toml`
2. Update channel from `magic` to `conda.modular.com/max-nightly`
3. Replace `magic run` commands with `pixi run`
4. Update from `max` to `modular` package
5. Use `[workspace]` instead of `[project]` section
6. Update CI/CD scripts to use `pixi` instead of `magic`
7. Start with minimal platform support and add as verified

## Resources

- [Pixi Documentation](https://pixi.sh/latest/)
- [Conda Forge Packages](https://conda-forge.org/packages/)
- [Modular Conda Channel](https://conda.modular.com/max-nightly/)
- [Mojo Package Examples](https://github.com/modular/modular/tree/main/examples/mojo)