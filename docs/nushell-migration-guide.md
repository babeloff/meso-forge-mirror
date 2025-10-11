# Nushell Migration Guide for Meso Forge Mirror

This guide explains the migration from bash scripts to nushell scripts in the meso-forge-mirror project and how to use the new nushell-based workflow.

## Overview

The project has been enhanced with nushell scripts that provide:
- Better error handling and structured data processing
- Cross-platform compatibility
- More readable and maintainable code
- Enhanced logging and output formatting
- Built-in table/record operations for better data handling

## Prerequisites

### Installing Nushell

#### Using Package Managers
```bash
# macOS with Homebrew
brew install nushell

# Linux with Snap
sudo snap install nushell

# Windows with Winget
winget install nushell

# Using Cargo (all platforms)
cargo install nu
```

#### Verify Installation
```bash
nu --version
```

### Pixi Integration

Nushell is automatically included in the pixi environment:
```bash
pixi install  # Includes nushell dependency
pixi shell    # Activates environment with nushell available
```

## Script Comparison

### Environment Setup

**Old Bash (`scripts/setup-env.sh`):**
```bash
#!/bin/bash
export RUST_LOG="${RUST_LOG:-info}"
if command -v cargo &> /dev/null; then
    echo "✓ cargo available"
fi
```

**New Nushell (`scripts/setup-env.nu`):**
```nu
#!/usr/bin/env nu
$env.RUST_LOG = ($env.RUST_LOG? | default "info")
if not (which cargo | is-empty) {
    print "✓ cargo available"
}
```

### Build Script

**Old Bash (`scripts/build.sh`):**
```bash
#!/bin/bash
case "$1" in
    "linux-64") echo "x86_64-unknown-linux-gnu" ;;
    "osx-64") echo "x86_64-apple-darwin" ;;
esac
```

**New Nushell (`scripts/build.nu`):**
```nu
#!/usr/bin/env nu
def get_rust_target [platform: string] -> string {
    match $platform {
        "linux-64" => "x86_64-unknown-linux-gnu",
        "osx-64" => "x86_64-apple-darwin"
    }
}
```

## New Nushell Scripts

### 1. Environment Setup (`scripts/setup-env.nu`)

**Features:**
- Cross-platform environment variable handling
- Structured dependency checking
- Better error messages with color coding
- Automatic directory creation

**Usage:**
```bash
nu scripts/setup-env.nu
```

### 2. Build Script (`scripts/build.nu`)

**Features:**
- Type-safe argument parsing
- Structured configuration records
- Cross-compilation support
- Enhanced error handling

**Usage:**
```bash
# Build for current platform
nu scripts/build.nu

# Build for specific platform
nu scripts/build.nu --target linux-64

# Build for all platforms
nu scripts/build.nu --target all

# Debug build with verbose output
nu scripts/build.nu --target osx-64 --debug --verbose
```

### 3. Conda Operations (`scripts/conda-ops.nu`)

**Features:**
- Comprehensive conda package management
- Multi-platform building
- Package verification and testing
- Publishing to multiple channels

**Usage:**
```bash
# Build packages for all platforms
nu scripts/conda-ops.nu build

# Build for specific platforms
nu scripts/conda-ops.nu build linux-64 osx-64

# Test package installation
nu scripts/conda-ops.nu test linux-64

# Publish to channel
nu scripts/conda-ops.nu publish meso-forge

# List built packages
nu scripts/conda-ops.nu list-packages
```

### 4. Testing Script (`scripts/test.nu`)

**Features:**
- Comprehensive test suite
- Performance benchmarking
- Integration testing
- Structured test reporting

**Usage:**
```bash
# Run all tests
nu scripts/test.nu all

# Run specific test categories
nu scripts/test.nu unit
nu scripts/test.nu integration
nu scripts/test.nu lint

# Run with verbose output
nu scripts/test.nu all --verbose

# Run without cleanup
nu scripts/test.nu all --no-cleanup
```

### 5. Usage Examples (`examples/usage.nu`)

**Features:**
- Interactive examples with colored output
- Error handling demonstrations
- Batch processing examples
- Configuration validation

**Usage:**
```bash
nu examples/usage.nu
```

## Pixi Task Integration

### Original Bash-based Tasks

```toml
conda-build-cross = { cmd = "bash scripts/build.sh --target all" }
```

### New Nushell-based Tasks

```toml
# Core nushell tasks
test-nu = "nu scripts/test.nu unit"
test-all-nu = "nu scripts/test.nu all"
conda-build-nu = "nu scripts/conda-ops.nu build"
conda-build-cross = "nu scripts/build.nu --target all"

# Task groups for easier execution
[task-groups]
build-all = ["build", "build-release", "conda-build-all-nu"]
test-comprehensive = ["test-all-nu", "test-lint-nu", "test-performance"]
```

### Running Nushell Tasks

```bash
# Individual tasks
pixi run test-nu
pixi run conda-build-nu
pixi run conda-build-cross

# Task groups
pixi run build-all
pixi run test-comprehensive
```

## Key Differences and Advantages

### 1. Data Structures

**Bash:**
```bash
platforms=("linux-64" "osx-64" "win-64")
for platform in "${platforms[@]}"; do
    echo "Processing $platform"
done
```

**Nushell:**
```nu
let platforms = ["linux-64", "osx-64", "win-64"]
$platforms | each { |platform|
    print $"Processing ($platform)"
}
```

### 2. Error Handling

**Bash:**
```bash
if ! cargo build; then
    echo "Build failed"
    exit 1
fi
```

**Nushell:**
```nu
try {
    cargo build
} catch {
    log_error "Build failed"
    exit 1
}
```

### 3. Configuration Management

**Bash:**
```bash
CONFIG_FILE="config.json"
MAX_DOWNLOADS=$(jq -r '.max_concurrent_downloads' "$CONFIG_FILE")
```

**Nushell:**
```nu
let config = (open config.json)
let max_downloads = $config.max_concurrent_downloads
```

### 4. Table Operations

**Nushell Advantage:**
```nu
# List packages with detailed information
def list_packages [] -> table {
    glob "conda-packages/**/*.conda" | each { |file|
        let stat = (ls $file | first)
        {
            name: ($file | path basename),
            size: $stat.size,
            modified: $stat.modified
        }
    }
}
```

## Migration Benefits

### 1. **Type Safety**
- Function parameters with type annotations
- Structured data handling
- Better error messages

### 2. **Cross-Platform Compatibility**
- Single script works on Windows, macOS, and Linux
- No need for separate `.bat` files
- Consistent behavior across platforms

### 3. **Better Data Processing**
- Built-in JSON, YAML, TOML support
- Table operations for package information
- Structured configuration handling

### 4. **Enhanced Debugging**
- Colored logging functions
- Structured error reporting
- Verbose modes with detailed output

### 5. **Maintainability**
- More readable syntax
- Better organization with functions
- Consistent error handling patterns

## Backwards Compatibility

### Existing Bash Scripts
- Original bash scripts are preserved for reference
- Pixi tasks support both bash and nushell versions
- Gradual migration path available

### Environment Variables
- All environment variables work the same way
- Same configuration files and formats
- Compatible with existing CI/CD workflows

## Best Practices

### 1. **Function Organization**
```nu
# Use clear function names with type annotations
def build_target [platform: string, config: record] -> bool {
    # Implementation
}
```

### 2. **Error Handling**
```nu
# Use try-catch blocks for robust error handling
try {
    run-external cargo build
} catch {
    log_error "Build failed"
    return false
}
```

### 3. **Configuration**
```nu
# Use structured records for configuration
const default_config = {
    build_type: "release",
    platforms: ["linux-64", "osx-64"],
    verbose: false
}
```

### 4. **Logging**
```nu
# Use consistent logging functions
def log_info [message: string] {
    print $"(ansi blue)[INFO](ansi reset) ($message)"
}
```

## Troubleshooting

### Common Issues

1. **Nushell not found**
   ```bash
   # Install nushell or activate pixi environment
   pixi shell
   ```

2. **Permission errors**
   ```bash
   # Make scripts executable
   chmod +x scripts/*.nu
   ```

3. **Path issues**
   ```bash
   # Run from project root
   cd meso-forge-mirror
   nu scripts/build.nu --help
   ```

### Getting Help

```bash
# Script-specific help
nu scripts/build.nu --help
nu scripts/conda-ops.nu help
nu scripts/test.nu help

# Pixi task list
pixi task list

# Nushell documentation
nu --help
```

## Migration Timeline

### Phase 1: ✅ Complete
- Convert core scripts to nushell
- Add nushell to pixi dependencies
- Create parallel nushell-based tasks

### Phase 2: Current
- Update documentation and examples
- Test nushell scripts in CI/CD
- Gather feedback from users

### Phase 3: Future
- Make nushell scripts the primary option
- Phase out bash scripts (keeping for compatibility)
- Enhanced features using nushell capabilities

## Contributing

When contributing new scripts or modifying existing ones:

1. **Use nushell for new scripts**
2. **Follow the established patterns** (logging, error handling, configuration)
3. **Add comprehensive help text**
4. **Include examples in docstrings**
5. **Test on multiple platforms**

## Conclusion

The migration to nushell provides a more robust, maintainable, and cross-platform scripting solution for the meso-forge-mirror project. The structured data handling, type safety, and enhanced error handling make the build and deployment processes more reliable and easier to debug.

For questions or issues with the nushell migration, please open an issue on the GitHub repository or refer to the [Nushell documentation](https://www.nushell.sh/).