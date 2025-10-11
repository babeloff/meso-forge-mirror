#!/bin/bash
# Cross-platform build script for meso-forge-mirror conda packaging
# This script handles building the Rust binary for different target platforms

set -e

# Default values
TARGET_PLATFORM=""
BUILD_TYPE="release"
OUTPUT_DIR="target"
VERBOSE=false

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Help function
show_help() {
    cat << EOF
Usage: $0 [OPTIONS]

Cross-platform build script for meso-forge-mirror

OPTIONS:
    -t, --target PLATFORM    Target platform (linux-64, osx-64, osx-arm64, win-64)
    -d, --debug              Build in debug mode (default: release)
    -o, --output DIR         Output directory (default: target)
    -v, --verbose            Verbose output
    -h, --help               Show this help message

PLATFORMS:
    linux-64                 x86_64-unknown-linux-gnu
    osx-64                   x86_64-apple-darwin
    osx-arm64                aarch64-apple-darwin
    win-64                   x86_64-pc-windows-gnu

EXAMPLES:
    $0 --target linux-64
    $0 --target osx-arm64 --debug
    $0 --target win-64 --output ./dist

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--target)
            TARGET_PLATFORM="$2"
            shift 2
            ;;
        -d|--debug)
            BUILD_TYPE="debug"
            shift
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Map conda platforms to Rust targets
get_rust_target() {
    case "$1" in
        "linux-64")
            echo "x86_64-unknown-linux-gnu"
            ;;
        "osx-64")
            echo "x86_64-apple-darwin"
            ;;
        "osx-arm64")
            echo "aarch64-apple-darwin"
            ;;
        "win-64")
            echo "x86_64-pc-windows-gnu"
            ;;
        *)
            log_error "Unsupported target platform: $1"
            exit 1
            ;;
    esac
}

# Get binary extension for platform
get_binary_extension() {
    case "$1" in
        "win-64")
            echo ".exe"
            ;;
        *)
            echo ""
            ;;
    esac
}

# Install Rust target if not available
install_target() {
    local rust_target="$1"
    log_info "Checking if target $rust_target is installed..."

    if ! rustup target list --installed | grep -q "^$rust_target$"; then
        log_info "Installing target: $rust_target"
        rustup target add "$rust_target"
    else
        log_success "Target $rust_target is already installed"
    fi
}

# Build for specific target
build_target() {
    local platform="$1"
    local rust_target="$2"
    local extension="$3"

    log_info "Building for platform: $platform (target: $rust_target)"

    # Install target if needed
    install_target "$rust_target"

    # Set up build command
    local cargo_cmd="cargo build --target $rust_target"

    if [[ "$BUILD_TYPE" == "release" ]]; then
        cargo_cmd="$cargo_cmd --release"
    fi

    if [[ "$VERBOSE" == true ]]; then
        cargo_cmd="$cargo_cmd --verbose"
    fi

    # Add locked flag to use exact dependencies from Cargo.lock
    cargo_cmd="$cargo_cmd --locked"

    # Execute build
    log_info "Executing: $cargo_cmd"
    eval "$cargo_cmd"

    # Verify binary was created
    local binary_path="target/$rust_target/$BUILD_TYPE/meso-forge-mirror$extension"
    if [[ -f "$binary_path" ]]; then
        log_success "Binary built successfully: $binary_path"

        # Show binary info
        ls -lh "$binary_path"

        # Test that binary works
        if [[ "$platform" == "linux-64" ]] || [[ "$OSTYPE" == "darwin"* && ("$platform" == "osx-64" || "$platform" == "osx-arm64") ]]; then
            log_info "Testing binary..."
            if "$binary_path" --version > /dev/null 2>&1; then
                log_success "Binary test passed"
            else
                log_warning "Binary test failed (may be due to cross-compilation)"
            fi
        fi

        return 0
    else
        log_error "Binary not found at expected path: $binary_path"
        return 1
    fi
}

# Build all targets
build_all_targets() {
    log_info "Building for all supported platforms..."

    local platforms=("linux-64" "osx-64" "osx-arm64" "win-64")
    local failed_builds=()

    for platform in "${platforms[@]}"; do
        local rust_target=$(get_rust_target "$platform")
        local extension=$(get_binary_extension "$platform")

        log_info "Starting build for $platform..."
        if build_target "$platform" "$rust_target" "$extension"; then
            log_success "Completed build for $platform"
        else
            log_error "Failed to build for $platform"
            failed_builds+=("$platform")
        fi
        echo ""
    done

    # Report results
    if [[ ${#failed_builds[@]} -eq 0 ]]; then
        log_success "All builds completed successfully!"
    else
        log_error "Failed builds: ${failed_builds[*]}"
        exit 1
    fi
}

# Main execution
main() {
    log_info "Starting meso-forge-mirror build process..."
    log_info "Build type: $BUILD_TYPE"
    log_info "Output directory: $OUTPUT_DIR"

    # Ensure we're in the right directory
    if [[ ! -f "Cargo.toml" ]]; then
        log_error "Cargo.toml not found. Please run this script from the project root."
        exit 1
    fi

    # Clean previous builds if requested
    if [[ "$CLEAN_BUILD" == "true" ]]; then
        log_info "Cleaning previous builds..."
        cargo clean
    fi

    # Build based on target selection
    if [[ -z "$TARGET_PLATFORM" ]]; then
        # No target specified, build for current platform
        log_info "No target specified, building for current platform..."
        local cargo_cmd="cargo build"

        if [[ "$BUILD_TYPE" == "release" ]]; then
            cargo_cmd="$cargo_cmd --release"
        fi

        if [[ "$VERBOSE" == true ]]; then
            cargo_cmd="$cargo_cmd --verbose"
        fi

        cargo_cmd="$cargo_cmd --locked"

        log_info "Executing: $cargo_cmd"
        eval "$cargo_cmd"

        # Find and report the binary
        local binary_name="meso-forge-mirror"
        if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
            binary_name="$binary_name.exe"
        fi

        local binary_path="target/$BUILD_TYPE/$binary_name"
        if [[ -f "$binary_path" ]]; then
            log_success "Binary built: $binary_path"
            ls -lh "$binary_path"
        fi

    elif [[ "$TARGET_PLATFORM" == "all" ]]; then
        # Build for all targets
        build_all_targets
    else
        # Build for specific target
        local rust_target=$(get_rust_target "$TARGET_PLATFORM")
        local extension=$(get_binary_extension "$TARGET_PLATFORM")
        build_target "$TARGET_PLATFORM" "$rust_target" "$extension"
    fi

    log_success "Build process completed!"
}

# Run main function
main "$@"
