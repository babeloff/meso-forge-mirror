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
- GitHub integration for fetching artifacts
- Enhanced error handling and diagnostics

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
meso-forge-mirror init-config -o config.json
```

This creates a configuration file with default settings:

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
  --sources https://example.com/packages/my-package-1.0.0.tar.bz2 \
  --target-type local \
  --target-path /path/to/local/repository
```

#### To an S3/MinIO Repository

```bash
# Configure AWS credentials first
export AWS_ACCESS_KEY_ID=your_key
export AWS_SECRET_ACCESS_KEY=your_secret

meso-forge-mirror mirror \
  --sources https://example.com/packages/my-package-1.0.0.tar.bz2 \
  --target-type s3 \
  --target-path s3://my-bucket/conda-packages \
  --config config.json
```

#### To prefix.dev

```bash
meso-forge-mirror mirror \
  --sources https://example.com/packages/my-package-1.0.0.tar.bz2 \
  --target-type prefix-dev \
  --target-path https://prefix.dev/channels/meso-forge
```

#### Mirror Multiple Packages

You can specify multiple package URLs:

```bash
meso-forge-mirror mirror \
  --sources https://example.com/pkg1.tar.bz2,https://example.com/pkg2.tar.bz2 \
  --target-type local \
  --target-path /path/to/repository
```

Or use multiple `--sources` flags:

```bash
meso-forge-mirror mirror \
  --sources https://example.com/pkg1.tar.bz2 \
  --sources https://example.com/pkg2.tar.bz2 \
  --target-type local \
  --target-path /path/to/repository
```

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
  --sources <package-urls> \
  --target-type local \
  --target-path ~/.pixi/cache/packages
```

## Documentation

For comprehensive documentation, see the `docs/` directory:

- **[Operator Guide](docs/operator-guide.adoc)**: Complete installation, configuration, and usage guide
- **[Rattler Integration Summary](docs/rattler-integration-summary.adoc)**: Overview of rattler ecosystem integration benefits
- **[Changelog](docs/changelog-rattler-integration.adoc)**: Detailed changelog for version 0.2.0 improvements
- **[Documentation Index](docs/index.adoc)**: Complete documentation overview

## Development

### Building

```bash
cargo build
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
RUST_LOG=debug cargo run -- mirror --sources "..." --target-type local --target-path ./test-repo
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
