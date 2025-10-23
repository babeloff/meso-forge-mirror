# Contributing to meso-forge-mirror

Thank you for your interest in contributing to meso-forge-mirror!
This document provides guidelines and instructions for contributing to the project.

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)
- Git
- Pixi package manager (recommended)
- Nushell (optional, for advanced scripts)

### Getting Started

0. Install the pixi package manager (recommended):
   [Pixi Installation](https://pixi.sh/latest/installation/)
   ```bash
   curl -fsSL https://pixi.sh/install.sh | sh
   ```

0.1. Install Nushell for advanced scripts (optional but recommended):
   ```bash
   # Via cargo
   cargo install nu
   
   # Or via conda
   conda install -c conda-forge nushell
   
   # Or via package manager
   brew install nushell  # macOS
   ```

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/meso-forge-mirror.git
   cd meso-forge-mirror
   ```
3. Set up development environment:
   ```bash
   pixi install
   pixi shell
   ```

4. Build the project:
   ```bash
   pixi run build
   # OR use Nushell script
   nu scripts/build.nu
   ```

5. Run tests:
   ```bash
   pixi run test
   # OR use comprehensive Nushell testing
   nu scripts/test.nu all
   ```

## Development Workflow

### Before Making Changes

1. Create a new branch for your feature or bugfix:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make sure all tests pass:
   ```bash
   pixi run test
   # OR for comprehensive testing
   pixi run ci-check
   ```

### Making Changes

1. Write your code
2. Add tests for new functionality
3. Update documentation as needed
4. Run code quality checks:
   ```bash
   # Complete development setup (build + test + lint + format)
   pixi run dev-setup
   
   # OR individual commands
   pixi run fmt
   pixi run clippy
   pixi run test
   
   # OR use Nushell scripts
   nu scripts/test.nu lint
   ```

5. For complete CI-style verification:
   ```bash
   pixi run ci-check
   ```

### Commit Messages

Use clear and descriptive commit messages. Follow the conventional commits format when possible:

- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation changes
- `test:` for test additions or modifications
- `refactor:` for code refactoring
- `chore:` for maintenance tasks

Example:
```
feat: add support for authenticated S3 endpoints
```

### Submitting Changes

1. Push your branch to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

2. Create a Pull Request on GitHub
3. Wait for review and address any feedback

## Code Style

This project follows standard Rust conventions:

- Use `cargo fmt` to format code
- Use `cargo clippy` to catch common mistakes
- Follow Rust naming conventions
- Write clear, self-documenting code
- Add comments for complex logic

## Testing

- Write unit tests for new functionality
- Write integration tests for end-to-end scenarios
- Ensure all tests pass before submitting a PR
- Aim for high code coverage

### Running Tests

#### Using Pixi (Recommended)
```bash
# Run all tests
pixi run test

# Run tests with verbose output
pixi run test-verbose

# Run complete CI-style checks
pixi run ci-check

# Run integration tests
pixi run integration-test
```

#### Using Nushell Scripts (Advanced)
```bash
# Run unit tests
nu scripts/test.nu unit

# Run all test suites
nu scripts/test.nu all

# Run integration tests
nu scripts/test.nu integration

# Run linting checks
nu scripts/test.nu lint

# Test local mirroring functionality
nu scripts/test.nu local-mirror

# Performance testing
nu scripts/test.nu performance
```

#### Traditional Cargo
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

## Documentation

- Update README.md if adding new features
- Add docstrings to public functions and modules
- Update examples if behavior changes
- Update relevant documentation in `docs/` directory:
  - `docs/operator-guide.adoc` for user-facing changes
  - `docs/pixi-tasks-guide.adoc` for new pixi tasks
  - `docs/nushell-scripts-guide.adoc` for script modifications

## Project Structure

```
meso-forge-mirror/
├── src/
│   ├── main.rs        # CLI entry point
│   ├── config.rs      # Configuration handling
│   ├── repository.rs  # Repository implementations
│   └── mirror.rs      # Mirroring logic
├── scripts/           # Nushell development scripts
│   ├── build.nu       # Advanced build script
│   ├── conda-ops.nu   # Conda package operations
│   ├── test.nu        # Comprehensive testing
│   ├── setup-env.sh   # Environment setup (Linux/macOS)
│   └── setup-env.bat  # Environment setup (Windows)
├── docs/              # Documentation
│   ├── operator-guide.adoc        # User guide
│   ├── pixi-tasks-guide.adoc      # Development tasks
│   ├── nushell-scripts-guide.adoc # Script documentation
│   └── index.adoc                 # Documentation index
├── examples/          # Example configurations and usage
│   └── usage.nu       # Nushell usage examples
├── .github/
│   └── workflows/     # CI/CD workflows
├── pixi.toml          # Pixi project configuration
└── tests/            # Integration tests (if any)
```

## Development Workflows

### Recommended Workflow (Pixi + Nushell)
```bash
# Initial setup
pixi install && pixi shell

# Daily development
pixi run dev-setup        # Complete setup
pixi run watch           # Continuous development

# Before committing
pixi run ci-check        # Full verification

# Building packages
pixi run conda-build     # Build conda packages
nu scripts/conda-ops.nu build  # Advanced conda operations

# Testing
nu scripts/test.nu all   # Comprehensive testing
```

### Traditional Workflow
```bash
cargo build
cargo test
cargo clippy
cargo fmt
```

## Getting Help

If you have questions or need help:

1. Check existing issues and pull requests
2. Review the comprehensive documentation in `docs/`
3. Open a new issue with your question
4. Join discussions in existing issues

## Code of Conduct

Be respectful and constructive in all interactions. We aim to maintain a welcoming and inclusive community.

## License

By contributing to meso-forge-mirror, you agree that your contributions will be licensed under the GNU General Public License v3.0.
