# Contributing to meso-forge-mirror

Thank you for your interest in contributing to meso-forge-mirror! This document provides guidelines and instructions for contributing to the project.

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)
- Git

### Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/meso-forge-mirror.git
   cd meso-forge-mirror
   ```

3. Build the project:
   ```bash
   cargo build
   ```

4. Run tests:
   ```bash
   cargo test
   ```

## Development Workflow

### Before Making Changes

1. Create a new branch for your feature or bugfix:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make sure all tests pass:
   ```bash
   cargo test
   ```

### Making Changes

1. Write your code
2. Add tests for new functionality
3. Update documentation as needed
4. Run the formatter:
   ```bash
   cargo fmt
   ```

5. Run the linter:
   ```bash
   cargo clippy -- -D warnings
   ```

6. Run tests to ensure nothing broke:
   ```bash
   cargo test
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

## Project Structure

```
meso-forge-mirror/
├── src/
│   ├── main.rs        # CLI entry point
│   ├── config.rs      # Configuration handling
│   ├── repository.rs  # Repository implementations
│   └── mirror.rs      # Mirroring logic
├── examples/          # Example configurations and usage
├── .github/
│   └── workflows/     # CI/CD workflows
└── tests/            # Integration tests (if any)
```

## Getting Help

If you have questions or need help:

1. Check existing issues and pull requests
2. Open a new issue with your question
3. Join discussions in existing issues

## Code of Conduct

Be respectful and constructive in all interactions. We aim to maintain a welcoming and inclusive community.

## License

By contributing to meso-forge-mirror, you agree that your contributions will be licensed under the GNU General Public License v3.0.
