# meso-forge-mirror

A Rust application for mirroring conda packages from various sources to target repositories. This tool is particularly useful when you want to use packages that are waiting to be included in conda-forge but are taking a long time to go through the process.

## Features

- Mirror conda packages from URLs to different target repositories
- Support for multiple target repository types:
  - **prefix.dev** channels (e.g., `https://prefix.dev/channels/meso-forge`)
  - **S3/MinIO** repositories
  - **Local** repositories (where pixi caches its repositories)
- Concurrent downloads with configurable parallelism
- Automatic retry with exponential backoff
- Configurable timeouts and connection settings
- GitHub integration for fetching artifacts

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

## Environment Variables

- `GITHUB_TOKEN`: GitHub personal access token for API authentication
- `AWS_ACCESS_KEY_ID`: AWS access key for S3 operations
- `AWS_SECRET_ACCESS_KEY`: AWS secret key for S3 operations
- `RUST_LOG`: Set logging level (e.g., `RUST_LOG=debug`)

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Support

For issues and questions, please open an issue on the GitHub repository.
