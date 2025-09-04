# OmenDB Scripts

This directory contains utility scripts for development and testing workflows.

## Available Scripts

### `run-example.sh`

A convenience script for running examples from the project root directory. Due to Mojo's module resolution system, examples must be executed from the root directory to properly resolve imports.

**Usage:**
```bash
# From project root directory
./scripts/run-example.sh <example_name>
```

**Examples:**
```bash
# Run the main vector operations demo
./scripts/run-example.sh vector_operations_demo

# Run basic usage example
./scripts/run-example.sh basic_usage

# See all available examples
./scripts/run-example.sh
```

**Features:**
- Colorized output for better readability
- Lists all available examples when run without arguments
- Provides helpful error messages and usage guidance
- Validates example files exist before attempting to run them

## Adding New Scripts

When adding new utility scripts to this directory:

1. **Make them executable**: `chmod +x scripts/new_script.sh`
2. **Add documentation**: Update this README with script description and usage
3. **Follow naming convention**: Use descriptive names with underscores or hyphens
4. **Include help text**: Scripts should show usage when run with `-h` or `--help`

## Script Categories

- **Development**: Scripts for building, testing, and development workflows
- **Examples**: Scripts for running and managing examples
- **Utilities**: General-purpose helper scripts

---

**Note**: All scripts should be run from the project root directory unless otherwise specified.