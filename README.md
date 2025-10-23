# meso-forge-mirror

A Rust application for mirroring conda packages from various sources to target repositories with full conda ecosystem integration. This tool is particularly useful when you want to use packages that are waiting to be included in conda-forge but are taking a long time to go through the process.

**Enhanced with Rattler Integration**: Version 0.2.0 introduces comprehensive conda package processing, validation, and proper repository structure management through integration with the rattler ecosystem crates.

## Features

### Enhanced Conda Package Processing (v0.2.0+)
- **Full Conda Package Validation**: Integration with rattler ecosystem for proper conda package handling
- **Metadata Extraction**: Automatic extraction of package metadata (name, version, build, dependencies)
- **Platform-Aware Organization**: Automatic organization by platform (linux-64/, osx-64/, noarch/, etc.)
- **Repository Structure**: Generates proper conda repository structure with repodata.json files
- **Integrity Verification**: MD5 and SHA256 checksum validation for all packages
- **Rattler Cache Integration**: Native support for `~/.cache/rattler/cache/pkgs/` directory structure

### Core Mirroring Features
- Mirror conda packages from URLs to different target repository types:
  - **Local** repositories with proper conda structure (including Rattler cache)
  - **S3/MinIO** repositories with platform organization
  - **prefix.dev** channels (e.g., `https://prefix.dev/channels/meso-forge`)
- Concurrent downloads with configurable parallelism
- Automatic retry with exponential backoff
- Configurable timeouts and connection settings
- **GitHub Artifacts Integration**: Download conda packages from GitHub Actions artifacts
- **Azure DevOps Integration**: Download conda packages from Azure DevOps build artifacts
- Enhanced error handling and diagnostics

### GitHub Artifacts Integration
- **Info Command**: Discover and list artifacts from GitHub repositories
- **GitHub Source Type**: Use `--src-type github` to mirror from GitHub artifacts
- **Flexible Repository Formats**: Support for `owner/repo`, GitHub URLs, and specific artifact IDs
- **Smart Filtering**: Filter artifacts by name patterns and expiration status
- **Authentication Support**: GitHub token support for private repositories and higher rate limits

### Azure DevOps Integration (New!)
- **Build and Artifact Discovery**: List builds and artifacts from Azure DevOps projects
- **Azure Source Type**: Use `--src-type azure` to mirror from Azure DevOps artifacts
- **conda-forge Support**: Direct support for conda-forge's Azure DevOps infrastructure
- **Flexible Project Formats**: Support for `org/project`, Azure DevOps URLs, and specific build IDs
- **Personal Access Token Authentication**: Secure authentication for Azure DevOps access

## Installation

### From Source

```bash
git clone https://github.com/babeloff/meso-forge-mirror.git
cd meso-forge-mirror
cargo build --release
```

The binary will be available at `target/release/meso-forge-mirror`.

## Usage

### Initialize Configuration

First, create a configuration file:

```bash
meso-forge-mirror init -o config.json
```

This creates a configuration file with default settings including GitHub and Azure DevOps token support:

```json
{
  "max_concurrent_downloads": 5,
  "retry_attempts": 3,
  "timeout_seconds": 300,
  "github_token": null,
  "azure_devops_token": null,
  "s3_region": null,
  "s3_endpoint": null
}
```

For GitHub integration, set your token:
```bash
export GITHUB_TOKEN=ghp_your_token_here
# or add it to the config file
```

For Azure DevOps integration, set your Personal Access Token:
```bash
export AZURE_DEVOPS_TOKEN=your_pat_token_here
# or add it to the config file
```

### Quick Start

#### Discover GitHub Artifacts

```bash
# List all artifacts for a repository
meso-forge-mirror info --github conda-forge/numpy

# Filter artifacts by name pattern
meso-forge-mirror info --github owner/repo --name-filter "conda.*linux.*"
```

#### Mirror from GitHub Artifacts

```bash
# Mirror from GitHub artifacts to cache
meso-forge-mirror mirror --src-type github --src owner/repository

# Mirror to local conda repository
meso-forge-mirror mirror \
  --src-type github \
  --src owner/repo \
  --tgt-type local \
  --tgt ./conda-repo

# Process specific artifact by ID
meso-forge-mirror mirror --src-type github --src owner/repo#123456789
```

#### Discover Azure DevOps Builds and Artifacts

```bash
# List recent builds for a project
meso-forge-mirror info --azure conda-forge/feedstock-builds

# List artifacts for a specific build
meso-forge-mirror info --azure conda-forge/feedstock-builds --build-id 1374331

# Filter artifacts by name pattern
meso-forge-mirror info --azure conda-forge/feedstock-builds --build-id 1374331 --name-filter "conda.*"
```

#### Mirror from Azure DevOps Artifacts

```bash
# Mirror from specific build to cache
meso-forge-mirror mirror --src-type azure --src conda-forge/feedstock-builds#1374331

# Mirror to local conda repository
meso-forge-mirror mirror \
  --src-type azure \
  --src conda-forge/feedstock-builds#1374331 \
  --tgt-type local \
  --tgt ./conda-repo

# Process latest successful build
meso-forge-mirror mirror --src-type azure --src conda-forge/feedstock-builds
```

```json
{
  "max_concurrent_downloads": 5,
  "retry_attempts": 3,
  "timeout_seconds": 300,
  "s3_region": null,
  "s3_endpoint": null,
  "github_token": null
}
```

Edit this file to customize settings, especially if you need S3 access or GitHub API access.

### Mirror Packages

#### To a Local Repository

```bash
meso-forge-mirror mirror \
  --src-type url \
  --src https://example.com/packages/my-package-1.0.0.tar.bz2 \
  --tgt-type local \
  --tgt /path/to/local/repository
```

#### To an S3/MinIO Repository

```bash
# Configure AWS credentials first
export AWS_ACCESS_KEY_ID=your_key
export AWS_SECRET_ACCESS_KEY=your_secret

meso-forge-mirror mirror \
  --src-type url \
  --src https://example.com/packages/my-package-1.0.0.tar.bz2 \
  --tgt-type s3 \
  --tgt s3://my-bucket/conda-packages \
  --config config.json
```

#### To prefix.dev

```bash
meso-forge-mirror mirror \
  --src-type url \
  --src https://example.com/packages/my-package-1.0.0.tar.bz2 \
  --tgt-type prefix-dev \
  --tgt https://prefix.dev/channels/meso-forge
```

#### Mirror Multiple Packages
### Source Types

The `--src-type` option supports different source formats:

```bash
# Remote conda package (default when using URL)
meso-forge-mirror mirror \
  --src https://example.com/pkg.tar.bz2 \
  --src-type url \
  --tgt /path/to/repository

# Local conda package
meso-forge-mirror mirror \
  --src /path/to/package.conda \
  --src-type local \
  --tgt /path/to/repository

# Remote ZIP file containing conda packages (using regex pattern)
meso-forge-mirror mirror \
  --src https://example.com/packages.zip \
  --src-type zip-url \
  --src-path "^artifacts/.*" \
  --tgt /path/to/repository

# GitHub Artifacts
meso-forge-mirror mirror \
  --src owner/repository \
  --src-type github \
  --tgt /path/to/repository

# GitHub with artifact name filtering
meso-forge-mirror mirror \
  --src owner/repository \
  --src-type github \
  --src-path "conda.*linux.*" \
  --tgt /path/to/repository

# Azure DevOps Artifacts (NEW!)
meso-forge-mirror mirror \
  --src organization/project#build_id \
  --src-type azure \
  --tgt /path/to/repository

# Azure DevOps with artifact name filtering
meso-forge-mirror mirror \
  --src conda-forge/feedstock-builds#1374331 \
  --src-type azure \
  --src-path "conda.*packages.*" \
  --tgt /path/to/repository

# Local tarball containing conda packages
meso-forge-mirror mirror \
  --src ./packages.tar.gz \
  --src-type tgz \
  --tgt /path/to/repository
```

### GitHub Artifacts Integration

The tool now supports downloading conda packages from GitHub Actions artifacts:

#### Available Commands

```bash
# Info command - discover artifacts
meso-forge-mirror info --github owner/repository
meso-forge-mirror info --github https://github.com/owner/repository
meso-forge-mirror info --github owner/repo --name-filter "conda.*" --exclude-expired true

# Mirror command - download and process artifacts
meso-forge-mirror mirror --src-type github --src owner/repository
meso-forge-mirror mirror --src-type github --src owner/repository#artifact_id
```

#### GitHub Authentication

For private repositories and higher rate limits, set a GitHub token:

```bash
export GITHUB_TOKEN=ghp_your_token_here
# or add to config file: "github_token": "ghp_your_token_here"
```

#### Real-world GitHub Examples

```bash
# Example 1: Mirror conda-forge feedstock artifacts
meso-forge-mirror info --github conda-forge/numpy-feedstock --name-filter "conda.*"
meso-forge-mirror mirror \
  --src-type github \
  --src conda-forge/numpy-feedstock \
  --src-path "conda.*packages.*" \
  --tgt-type local \
  --tgt ./my-conda-channel

# Example 2: CI/CD integration - mirror PR build artifacts
meso-forge-mirror mirror \
  --src-type github \
  --src myorg/my-package \
  --src-path "conda.*$(git rev-parse --short HEAD).*" \
  --tgt-type s3 \
  --tgt s3://staging-conda-channel/pr-builds

# Example 3: Process specific artifact by ID
meso-forge-mirror info --github owner/repo  # Find artifact ID
meso-forge-mirror mirror --src-type github --src owner/repo#123456789

# Example 4: conda-forge Azure DevOps workflow
meso-forge-mirror info --azure conda-forge/feedstock-builds --build-id 1374331
meso-forge-mirror mirror \
  --src-type azure \
  --src conda-forge/feedstock-builds#1374331 \
  --src-path "conda.*packages.*" \
  --tgt-type local \
  --tgt ./pr-31205-packages
```

### Regular Expression Patterns

The `--src-path` parameter accepts regular expressions for flexible file matching within ZIP archives. When multiple files match the pattern, only the first match will be processed. For detailed examples and patterns, see [REGEX_EXAMPLES.md](REGEX_EXAMPLES.md).

Common patterns:
- `^artifacts/.*` - Match files in artifacts/ directory
- `^(linux-64|osx-64)/.*` - Match platform-specific directories
- `.*\.conda$` - Match only .conda files
- `^build-\d+/conda/.*` - Match numbered build directories

**Note**: Only the first conda package matching the regex pattern will be mirrored, ensuring predictable behavior.

### Configuration Options

The configuration file supports the following options:

- `max_concurrent_downloads`: Maximum number of packages to download concurrently (default: 5)
- `retry_attempts`: Number of times to retry failed downloads (default: 3)
- `timeout_seconds`: Timeout for HTTP requests in seconds (default: 300)
- `s3_region`: AWS region for S3 uploads (optional)
- `s3_endpoint`: Custom S3 endpoint for MinIO or other S3-compatible services (optional)
- `github_token`: GitHub personal access token for API access (optional, can also be set via `GITHUB_TOKEN` environment variable)

## Use Cases

### Mirroring Packages from Staged Recipes

For packages waiting in conda-forge staged-recipes:

1. Find the PR with your package: https://github.com/conda-forge/staged-recipes/pulls
2. Locate the build artifacts from the CI/CD pipeline
3. Use `meso-forge-mirror` to copy them to your target repository

Example PRs mentioned in the issue:
- https://github.com/conda-forge/staged-recipes/pulls?q=sort%3Aupdated-desc+is%3Apr+is%3Aopen+author%3Aphreed
- https://github.com/conda-forge/openshift-cli-feedstock/pull/6

### Setting Up a Local Cache

```bash
# Mirror packages to a local directory that pixi can use
meso-forge-mirror mirror \
  --src <package-url> \
  --src-type url \
  --tgt-type local \
  --tgt ~/.pixi/cache/packages
```

## Documentation

For comprehensive documentation, see the `docs/` directory and integration guides:

- **[Operator Guide](docs/operator-guide.adoc)**: Complete installation, configuration, and usage guide
- **[GitHub Integration Guide](docs/chapters/github-integration.adoc)**: Comprehensive guide for GitHub Artifacts integration
- **[Azure DevOps Integration Guide](docs/chapters/azure-devops-integration.adoc)**: Comprehensive guide for Azure DevOps integration
- **[Implementation Summary](IMPLEMENTATION_SUMMARY.md)**: Technical details of integration implementations
- **[Pixi Tasks Guide](docs/pixi-tasks-guide.adoc)**: Complete guide to development, testing, and packaging tasks
- **[Nushell Scripts Guide](docs/nushell-scripts-guide.adoc)**: Advanced cross-platform scripts for building and testing
- **[Rattler Integration Summary](docs/rattler-integration-summary.adoc)**: Overview of rattler ecosystem integration benefits
- **[Changelog](docs/changelog-rattler-integration.adoc)**: Detailed changelog for version 0.2.0 improvements
- **[Documentation Index](docs/index.adoc)**: Complete documentation overview

### Integration Quick References

- **[GitHub Config Example](examples/github-config.json)**: Example configuration with GitHub token
- **[GitHub Usage Examples](examples/github-usage-examples.sh)**: Executable script with GitHub examples
- **[Azure DevOps Config Example](examples/azure-config.json)**: Example configuration with Azure DevOps PAT
- **[Azure DevOps Usage Examples](examples/azure-usage-examples.sh)**: Executable script with Azure DevOps examples

## Development

### Quick Setup with Pixi

The recommended way to set up the development environment is using pixi:

```bash
# Install pixi (if not already installed)
curl -fsSL https://pixi.sh/install.sh | bash

# Initialize the environment
pixi install

# Activate the shell (sets up environment variables and tools)
pixi shell

# Build the project
pixi run build

# Run tests
pixi run test
```

### Building

```bash
cargo build
```

### Advanced Build Scripts

The project includes modern Nushell-based scripts for enhanced development workflows:

```bash
# Interactive platform selection and building
nu scripts/build.nu

# Build for specific platform
nu scripts/build.nu linux-64

# Comprehensive conda operations
nu scripts/conda-ops.nu build

# Advanced testing capabilities
nu scripts/test.nu integration
```

For detailed information, see the [Nushell Scripts Guide](docs/nushell-scripts-guide.adoc).

### Development Workflows

The project supports multiple development workflows:

```bash
# Traditional cargo workflow
cargo test
cargo build --release

# Pixi task workflow (recommended)
pixi run dev-setup     # Complete development setup
pixi run ci-check      # CI-style verification
pixi run conda-build   # Build conda packages

# Advanced Nushell workflows
nu scripts/test.nu all          # Comprehensive testing
nu scripts/build.nu             # Interactive building
nu scripts/conda-ops.nu build   # Conda package operations
```

### Running Tests

```bash
cargo test
```

### Linting

```bash
cargo clippy
```

### Running with Debug Logging

```bash
RUST_LOG=debug cargo run -- mirror --src "..." --src-type url --tgt-type local --tgt ./test-repo
```

## Environment Variables

- `GITHUB_TOKEN`: GitHub personal access token for API authentication
- `AWS_ACCESS_KEY_ID`: AWS access key for S3 operations
- `AWS_SECRET_ACCESS_KEY`: AWS secret key for S3 operations
- `RUST_LOG`: Set logging level (e.g., `RUST_LOG=debug`) - Enhanced with detailed conda package processing logs

## What's New in v0.2.0

The latest release introduces major improvements through rattler ecosystem integration:

### 🎯 **Enhanced Package Processing**
- Full conda package validation and metadata extraction
- Platform-aware repository organization (linux-64/, osx-64/, etc.)
- Automatic repodata.json generation for conda compatibility
- Comprehensive checksum verification (MD5/SHA256)

### 🏗️ **Proper Repository Structure**
- Conda-compliant repository layout
- Native Rattler cache integration (`~/.cache/rattler/cache/pkgs/`)
- Multi-platform support with organized subdirectories
- Seamless integration with pixi, mamba, and conda

### 🛡️ **Reliability & Validation**
- Enhanced error handling with detailed diagnostics
- Package integrity validation prevents corrupted mirrors
- Robust platform detection with fallback mechanisms
- Comprehensive logging for debugging and monitoring

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. See the [documentation](docs/index.adoc) for development guidelines and architecture overview.

## Support

For issues and questions:
1. Check the [Operator Guide troubleshooting section](docs/operator-guide.adoc#troubleshooting)
2. Review the [changelog](docs/changelog-rattler-integration.adoc) for recent changes
3. Open an issue on the GitHub repository with detailed logs (`RUST_LOG=debug`)
