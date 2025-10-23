#!/bin/bash
# Environment setup script for meso-forge-mirror development
# This script is automatically sourced by pixi when activating the environment

echo "Setting up meso-forge-mirror development environment..."

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Warning: cargo not found in PATH"
fi

# Set environment variables for development
export RUST_LOG="${RUST_LOG:-info}"
export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"

# AWS/S3 configuration for local testing (MinIO)
export AWS_ENDPOINT_URL="${AWS_ENDPOINT_URL:-http://localhost:9000}"
export AWS_ACCESS_KEY_ID="${AWS_ACCESS_KEY_ID:-minioadmin}"
export AWS_SECRET_ACCESS_KEY="${AWS_SECRET_ACCESS_KEY:-minioadmin}"

# Create necessary directories
mkdir -p target
mkdir -p examples/output
mkdir -p test-data

# Set up git hooks if in a git repository
if [ -d ".git" ]; then
    # Install pre-commit hooks
    if command -v pre-commit &> /dev/null; then
        pre-commit install --install-hooks
    fi
fi

# Verify cargo configuration
if [ -f "Cargo.toml" ]; then
    echo "✓ Cargo.toml found"
else
    echo "⚠ Warning: Cargo.toml not found in current directory"
fi

# Function to check dependency
check_dependency() {
    local command="$1"
    local purpose="$2"

    if command -v "$command" &> /dev/null; then
        echo "✓ $command available"
    else
        echo "⚠ $command not found (required for: $purpose)"
    fi
}

echo "Checking system dependencies:"
check_dependency "pkg-config" "OpenSSL linking"
check_dependency "git" "version control"
check_dependency "curl" "HTTP requests testing"

# Rust toolchain verification
if command -v cargo &> /dev/null; then
    rust_version=$(cargo --version | awk '{print $2}')
    echo "✓ Rust toolchain: $rust_version"

    # Check for required targets for cross-compilation
    if command -v rustup &> /dev/null; then
        targets=$(rustup target list --installed)

        if echo "$targets" | grep -q "x86_64-unknown-linux-gnu"; then
            echo "✓ Linux target available"
        fi

        if echo "$targets" | grep -q "x86_64-apple-darwin"; then
            echo "✓ macOS target available"
        fi
    fi
fi

# Setup completion
echo ""
echo "Environment setup complete!"
echo ""
echo "Available pixi tasks:"
echo "  pixi run build          - Build the project"
echo "  pixi run test           - Run tests"
echo "  pixi run dev-setup      - Complete development setup"
echo "  pixi run conda-build    - Build conda package"
echo "  pixi run demo-local     - Run local demo"
echo ""
echo "For a full list of tasks, run: pixi task list"
