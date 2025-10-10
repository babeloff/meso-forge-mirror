# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-10-10

### Added
- Initial release of meso-forge-mirror
- Support for mirroring conda packages from URLs to target repositories
- Three repository types supported:
  - Local file system repositories
  - S3/MinIO repositories
  - prefix.dev channels
- CLI interface with two commands:
  - `mirror` - Mirror packages from sources to target
  - `init-config` - Initialize configuration file
- Configuration file support with customizable settings:
  - Maximum concurrent downloads
  - Retry attempts
  - Timeout settings
  - S3 region and endpoint configuration
  - GitHub token support
- Concurrent downloads with configurable parallelism
- Automatic retry with exponential backoff
- Comprehensive error handling and logging
- Unit tests for core functionality
- GitHub Actions CI/CD pipeline
- Example configurations and usage scripts
- Comprehensive documentation

### Features
- Download packages from any HTTP(S) URL
- Upload to local directories
- Upload to S3/MinIO buckets
- Upload to prefix.dev channels
- Support for multiple package sources
- Configurable retry logic
- Progress logging with tracing

[0.1.0]: https://github.com/babeloff/meso-forge-mirror/releases/tag/v0.1.0
