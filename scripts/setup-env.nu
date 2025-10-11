#!/usr/bin/env nu
# Environment setup script for meso-forge-mirror development
# This script is automatically sourced by pixi when activating the environment

print "Setting up meso-forge-mirror development environment..."

# Check if cargo is available
if (which cargo | is-empty) {
    print "Warning: cargo not found in PATH"
}

# Set environment variables for development
$env.RUST_LOG = ($env.RUST_LOG? | default "info")
$env.RUST_BACKTRACE = ($env.RUST_BACKTRACE? | default "1")

# AWS/S3 configuration for local testing (MinIO)
$env.AWS_ENDPOINT_URL = ($env.AWS_ENDPOINT_URL? | default "http://localhost:9000")
$env.AWS_ACCESS_KEY_ID = ($env.AWS_ACCESS_KEY_ID? | default "minioadmin")
$env.AWS_SECRET_ACCESS_KEY = ($env.AWS_SECRET_ACCESS_KEY? | default "minioadmin")

# Create necessary directories
mkdir target
mkdir examples/output
mkdir test-data

# Set up git hooks if in a git repository
if (".git" | path exists) {
    # Install pre-commit hooks
    if not (which pre-commit | is-empty) {
        pre-commit install --install-hooks
    }
}

# Verify cargo configuration
if ("Cargo.toml" | path exists) {
    print "✓ Cargo.toml found"
} else {
    print "⚠ Warning: Cargo.toml not found in current directory"
}

# Function to check dependency
def check_dependency [command: string, purpose: string] {
    if not (which $command | is-empty) {
        print $"✓ ($command) available"
    } else {
        print $"⚠ ($command) not found (required for: ($purpose))"
    }
}

print "Checking system dependencies:"
check_dependency "pkg-config" "OpenSSL linking"
check_dependency "git" "version control"
check_dependency "curl" "HTTP requests testing"

# Rust toolchain verification
if not (which cargo | is-empty) {
    let rust_version = (cargo --version | split words | get 1)
    print $"✓ Rust toolchain: ($rust_version)"

    # Check for required targets for cross-compilation
    let targets = (rustup target list --installed | lines)

    if ("x86_64-unknown-linux-gnu" in $targets) {
        print "✓ Linux target available"
    }

    if ("x86_64-apple-darwin" in $targets) {
        print "✓ macOS target available"
    }
}

# Setup completion
print ""
print "Environment setup complete!"
print ""
print "Available pixi tasks:"
print "  pixi run build          - Build the project"
print "  pixi run test           - Run tests"
print "  pixi run dev-setup      - Complete development setup"
print "  pixi run conda-build    - Build conda package"
print "  pixi run demo-local     - Run local demo"
print ""
print "For a full list of tasks, run: pixi task list"
