# Pixi Tasks Guide for Meso Forge Mirror

This guide explains how to use the pixi tasks defined for the `meso-forge-mirror` project. These tasks provide a comprehensive workflow for building, testing, and packaging the application as conda packages.

## Setup and Installation

### Initial Setup

1. **Install pixi** (if not already installed):
   ```bash
   curl -fsSL https://pixi.sh/install.sh | bash
   ```

2. **Initialize the environment**:
   ```bash
   pixi install
   ```

3. **Activate the environment**:
   ```bash
   pixi shell
   ```

The activation will automatically run setup scripts that:
- Check system dependencies
- Set environment variables
- Create necessary directories
- Verify Rust toolchain

## Core Development Tasks

### Building the Project

```bash
# Basic build (debug mode)
pixi run build

# Release build (optimized)
pixi run build-release

# Check code without building
pixi run check

# Clean build artifacts
pixi run clean
```

### Testing and Code Quality

```bash
# Run all tests
pixi run test

# Run tests with verbose output
pixi run test-verbose

# Run clippy linter
pixi run clippy

# Format code
pixi run fmt

# Check formatting without modifying files
pixi run fmt-check

# Complete development setup (build + test + lint + format check)
pixi run dev-setup

# CI-style check (includes release build)
pixi run ci-check
```

### Documentation

```bash
# Generate and open documentation
pixi run doc

# Build documentation without opening
pixi run docs-build

# Build all documentation including private items
pixi run docs-all

# Serve documentation on localhost:8000
pixi run docs-serve
```

## Conda Package Building

### Traditional Conda-Build

```bash
# Build package for current platform
pixi run conda-build

# Build packages for all supported platforms
pixi run conda-build-all

# Build for specific platforms
pixi run conda-build-linux
pixi run conda-build-macos
pixi run conda-build-macos-arm
pixi run conda-build-windows

# Cross-platform build with custom script
pixi run conda-build-cross
```

### Modern Rattler-Build

```bash
# Build with rattler-build (current platform)
pixi run rattler-build

# Build for all platforms with rattler-build
pixi run rattler-build-all
```

### Package Verification

```bash
# Verify conda package integrity
pixi run conda-verify

# Install and test locally built package
pixi run conda-test-local

# Create test environment with the package
pixi run conda-create-env
```

## Publishing and Distribution

### Uploading to Conda Channels

```bash
# Upload to conda-forge (requires permissions)
pixi run publish-conda-forge

# Upload to custom channel (set CONDA_CHANNEL env var)
pixi run upload-channel

# Upload to test channel
pixi run upload-test-channel

# Upload all built packages
pixi run upload-all-packages
```

### Publishing to prefix.dev

```bash
# Upload to prefix.dev
pixi run publish-prefix-dev

# Upload all rattler-built packages
pixi run upload-all-rattler
```

## Testing with Different Backends

### Local Repository Testing

```bash
# Basic local mirror demo
pixi run demo-local

# Integration test with local mirror
pixi run test-local-mirror

# Test configuration generation
pixi run test-config-generation

# Run complete integration tests
pixi run integration-test
```

### S3/MinIO Testing

```bash
# Test with local MinIO instance
pixi run test-s3-local
```

### prefix.dev Testing

```bash
# Test uploading to prefix.dev
pixi run test-prefix-dev
```

## Cross-Compilation Tasks

```bash
# Build for specific targets
pixi run build-linux
pixi run build-macos
pixi run build-macos-arm
pixi run build-windows
```

## Maintenance and Development Tools

### Dependency Management

```bash
# Update dependencies
pixi run update

# Check for outdated dependencies
pixi run deps-outdated

# Security audit
pixi run security-audit
```

### Development Tools Installation

```bash
# Install development tools
pixi run install-dev-tools

# Complete development setup
pixi run setup-dev
```

### File Watching for Development

```bash
# Watch files and run checks/tests on changes
pixi run watch

# Watch and run tests only
pixi run watch-test
```

## Advanced Tasks

### Binary Installation

```bash
# Install release binary globally
pixi run install

# Install debug binary globally
pixi run install-debug
```

### Performance Profiling

```bash
# Profile application performance (Linux only)
pixi run profile
```

### Examples and Demos

```bash
# Generate example configuration
pixi run run-example
```

## Cleanup Tasks

```bash
# Clean all build artifacts
pixi run clean-all

# Clean specific components
pixi run clean-examples
pixi run clean-integration
pixi run clean-packages
pixi run clean-build-artifacts
```

## Release Workflow

### Complete Release Preparation

```bash
# Prepare for release (full testing and building)
pixi run prepare-release

# Pre-release checks
pixi run pre-release
```

## Environment-Specific Tasks

The pixi configuration includes environment-specific tasks:

### Development Environment
```bash
# Switch to development environment
pixi shell -e dev
```

### Testing Environment
```bash
# Switch to testing environment
pixi shell -e test
```

### Packaging Environment
```bash
# Switch to packaging environment (includes conda-build tools)
pixi shell -e packaging
```

## Platform-Specific Considerations

### Linux
- Requires `pkg-config` and `libssl-dev`
- Full cross-compilation support available

### macOS
- Requires `pkg-config` and `openssl` from Homebrew
- Supports both Intel (osx-64) and Apple Silicon (osx-arm64)

### Windows
- Uses MSVC toolchain by default
- Some tasks may require Windows Subsystem for Linux (WSL) or Git Bash

## Environment Variables

Key environment variables that affect task behavior:

- `RUST_LOG`: Set logging level (default: `info`)
- `RUST_BACKTRACE`: Enable backtraces (default: `1`)
- `CONDA_CHANNEL`: Target conda channel for uploads (default: `meso-forge`)
- `AWS_*`: AWS credentials for S3 testing
- `GITHUB_TOKEN`: GitHub API token for accessing staged recipes

## Common Workflows

### Daily Development
```bash
pixi run dev-setup    # Build, test, lint, format check
pixi run watch        # Continuous development with file watching
```

### Before Committing
```bash
pixi run ci-check     # Full CI-style verification
```

### Building Packages
```bash
pixi run conda-build-all     # Build for all platforms
pixi run conda-test-local    # Test installation
```

### Publishing Release
```bash
pixi run prepare-release     # Complete preparation
pixi run upload-all-packages # Publish to channels
```

## Troubleshooting

### Common Issues

1. **Missing system dependencies**: Run the setup script manually:
   ```bash
   bash scripts/setup-env.sh  # Linux/macOS
   scripts/setup-env.bat      # Windows
   ```

2. **Cross-compilation failures**: Ensure required Rust targets are installed:
   ```bash
   rustup target add x86_64-unknown-linux-gnu
   rustup target add x86_64-apple-darwin
   rustup target add aarch64-apple-darwin
   rustup target add x86_64-pc-windows-gnu
   ```

3. **Conda build failures**: Verify conda-build is properly installed:
   ```bash
   pixi shell -e packaging
   conda install conda-build
   ```

### Getting Help

- List all available tasks: `pixi task list`
- View task details: `pixi info`
- Check environment status: `pixi list`

## Integration with CI/CD

The pixi tasks are designed to work seamlessly with the GitHub Actions workflows:

- `ci-check` task mirrors the CI pipeline
- Conda packaging tasks generate artifacts for release workflows
- Cross-compilation tasks ensure platform compatibility

This comprehensive task system provides a complete development, testing, and deployment workflow for the meso-forge-mirror project.