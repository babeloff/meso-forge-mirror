#!/bin/bash
# Linux activation script for meso-forge-mirror
# Ensures proper conda package directory structure and platform-specific package placement

set -euo pipefail

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOCAL_CACHE_DIR="${PROJECT_ROOT}/local-conda-cache"
CONDA_PACKAGES_DIR="${PROJECT_ROOT}/conda-packages"
RATTLER_PACKAGES_DIR="${PROJECT_ROOT}/rattler-packages"

# Logging functions
log_info() {
    echo -e "\033[34m[INFO]\033[0m $1"
}

log_success() {
    echo -e "\033[32m[SUCCESS]\033[0m $1"
}

log_warning() {
    echo -e "\033[33m[WARNING]\033[0m $1"
}

log_error() {
    echo -e "\033[31m[ERROR]\033[0m $1"
}

log_step() {
    echo -e "\033[36m[STEP]\033[0m $1"
}

# Create proper conda channel directory structure
create_conda_directories() {
    log_step "Creating conda package directory structure"

    local base_dirs=("$LOCAL_CACHE_DIR" "$CONDA_PACKAGES_DIR" "$RATTLER_PACKAGES_DIR")
    local platforms=("linux-64" "linux-aarch64" "osx-64" "osx-arm64" "win-64" "noarch")

    for base_dir in "${base_dirs[@]}"; do
        log_info "Setting up directory structure for: $base_dir"
        mkdir -p "$base_dir"

        for platform in "${platforms[@]}"; do
            local platform_dir="$base_dir/$platform"
            mkdir -p "$platform_dir"

            # Create basic repodata structure if it doesn't exist
            if [[ ! -f "$platform_dir/repodata.json" ]]; then
                cat > "$platform_dir/repodata.json" << EOF
{
    "info": {
        "subdir": "$platform"
    },
    "packages": {},
    "packages.conda": {},
    "removed": [],
    "repodata_version": 1
}
EOF
            fi
        done

        # Create channel metadata
        if [[ ! -f "$base_dir/channeldata.json" ]]; then
            cat > "$base_dir/channeldata.json" << EOF
{
    "channeldata_version": 1,
    "subdirs": ["linux-64", "linux-aarch64", "osx-64", "osx-arm64", "win-64", "noarch"],
    "packages": {}
}
EOF
        fi

        log_success "Directory structure created for: $base_dir"
    done
}

# Fix misplaced packages by moving them to correct platform directories
fix_package_placement() {
    log_step "Checking and fixing package placement"

    local base_dirs=("$LOCAL_CACHE_DIR" "$CONDA_PACKAGES_DIR" "$RATTLER_PACKAGES_DIR")

    for base_dir in "${base_dirs[@]}"; do
        if [[ ! -d "$base_dir" ]]; then
            continue
        fi

        log_info "Checking package placement in: $base_dir"

        # Check for packages that should be moved from noarch to linux-64
        local noarch_dir="$base_dir/noarch"
        local linux64_dir="$base_dir/linux-64"

        if [[ -d "$noarch_dir" ]]; then
            # Known Linux-specific packages that should be in linux-64
            local linux_packages=(
                "coreos-installer"
                "okd-install"
                "openshift-installer"
                "docker"
                "podman"
                "containerd"
                "runc"
                "skopeo"
                "buildah"
                "cni-plugins"
                "helm"
                "kubectl"
                "oc"
                "kind"
                "minikube"
            )

            for package_pattern in "${linux_packages[@]}"; do
                # Find packages matching the pattern
                for package_file in "$noarch_dir"/${package_pattern}-*.conda "$noarch_dir"/${package_pattern}-*.tar.bz2; do
                    if [[ -f "$package_file" ]]; then
                        local filename="$(basename "$package_file")"
                        log_warning "Found Linux-specific package in noarch: $filename"

                        # Move to linux-64 directory
                        mkdir -p "$linux64_dir"
                        if mv "$package_file" "$linux64_dir/"; then
                            log_success "Moved $filename to linux-64/"

                            # Update repodata if conda-index is available
                            if command -v conda-index >/dev/null 2>&1; then
                                log_info "Updating conda index for linux-64"
                                conda-index "$linux64_dir" --no-progress >/dev/null 2>&1 || true
                                conda-index "$noarch_dir" --no-progress >/dev/null 2>&1 || true
                            fi
                        else
                            log_error "Failed to move $filename"
                        fi
                    fi
                done
            done
        fi

        # Check for packages with platform-specific suffixes in wrong directories
        for platform_dir in "$base_dir"/*; do
            if [[ ! -d "$platform_dir" ]]; then
                continue
            fi

            local platform_name="$(basename "$platform_dir")"

            # Skip non-platform directories
            case "$platform_name" in
                linux-64|linux-aarch64|osx-64|osx-arm64|win-64|noarch) ;;
                *) continue ;;
            esac

            log_info "Checking packages in $platform_name directory"

            for package_file in "$platform_dir"/*.conda "$platform_dir"/*.tar.bz2; do
                if [[ ! -f "$package_file" ]]; then
                    continue
                fi

                local filename="$(basename "$package_file")"

                # Check if filename suggests different platform
                case "$filename" in
                    *-linux_64-*|*-linux64-*)
                        if [[ "$platform_name" != "linux-64" ]]; then
                            log_warning "Package $filename appears to be linux-64 but is in $platform_name"
                            mkdir -p "$base_dir/linux-64"
                            if mv "$package_file" "$base_dir/linux-64/"; then
                                log_success "Moved $filename to linux-64/"
                            fi
                        fi
                        ;;
                    *-osx_64-*|*-osx64-*)
                        if [[ "$platform_name" != "osx-64" ]]; then
                            log_warning "Package $filename appears to be osx-64 but is in $platform_name"
                            mkdir -p "$base_dir/osx-64"
                            if mv "$package_file" "$base_dir/osx-64/"; then
                                log_success "Moved $filename to osx-64/"
                            fi
                        fi
                        ;;
                    *-win_64-*|*-win64-*)
                        if [[ "$platform_name" != "win-64" ]]; then
                            log_warning "Package $filename appears to be win-64 but is in $platform_name"
                            mkdir -p "$base_dir/win-64"
                            if mv "$package_file" "$base_dir/win-64/"; then
                                log_success "Moved $filename to win-64/"
                            fi
                        fi
                        ;;
                esac
            done
        done
    done
}

# Rebuild conda indices after package moves
rebuild_indices() {
    log_step "Rebuilding conda indices"

    if ! command -v conda-index >/dev/null 2>&1; then
        log_warning "conda-index not available, skipping index rebuild"
        log_info "To enable index rebuilding, install conda-build: pixi add conda-build"
        return 0
    fi

    local base_dirs=("$LOCAL_CACHE_DIR" "$CONDA_PACKAGES_DIR" "$RATTLER_PACKAGES_DIR")

    for base_dir in "${base_dirs[@]}"; do
        if [[ -d "$base_dir" ]]; then
            log_info "Rebuilding index for: $base_dir"
            conda-index "$base_dir" --no-progress >/dev/null 2>&1 && \
                log_success "Index rebuilt for $base_dir" || \
                log_warning "Failed to rebuild index for $base_dir"
        fi
    done
}

# Verify package placement
verify_placement() {
    log_step "Verifying package placement"

    local base_dirs=("$LOCAL_CACHE_DIR" "$CONDA_PACKAGES_DIR" "$RATTLER_PACKAGES_DIR")

    for base_dir in "${base_dirs[@]}"; do
        if [[ ! -d "$base_dir" ]]; then
            continue
        fi

        log_info "Verifying: $base_dir"

        local total_packages=0
        for platform_dir in "$base_dir"/*; do
            if [[ ! -d "$platform_dir" ]]; then
                continue
            fi

            local platform_name="$(basename "$platform_dir")"
            case "$platform_name" in
                linux-64|linux-aarch64|osx-64|osx-arm64|win-64|noarch)
                    local count=$(find "$platform_dir" -name "*.conda" -o -name "*.tar.bz2" 2>/dev/null | wc -l)
                    if [[ $count -gt 0 ]]; then
                        log_info "  $platform_name: $count packages"
                        total_packages=$((total_packages + count))
                    fi
                    ;;
            esac
        done

        if [[ $total_packages -gt 0 ]]; then
            log_success "Total packages in $base_dir: $total_packages"
        fi
    done
}

# Environment setup
setup_environment() {
    log_step "Setting up conda environment variables"

    # Set conda channel priority
    export CONDA_CHANNEL_PRIORITY=strict

    # Add local channels to conda config if available
    if command -v conda >/dev/null 2>&1; then
        for base_dir in "$LOCAL_CACHE_DIR" "$CONDA_PACKAGES_DIR" "$RATTLER_PACKAGES_DIR"; do
            if [[ -d "$base_dir" ]]; then
                local channel_url="file://$(realpath "$base_dir")"
                log_info "Adding local channel: $channel_url"
                conda config --env --add channels "$channel_url" 2>/dev/null || true
            fi
        done
    fi

    # Export environment variables for meso-forge-mirror
    export MESO_FORGE_LOCAL_CACHE="$LOCAL_CACHE_DIR"
    export MESO_FORGE_CONDA_PACKAGES="$CONDA_PACKAGES_DIR"
    export MESO_FORGE_RATTLER_PACKAGES="$RATTLER_PACKAGES_DIR"

    log_success "Environment variables set"
}

# Print helpful information
print_usage_info() {
    log_step "Usage information"

    cat << 'EOF'

Conda package directories have been set up:
  - local-conda-cache/     : Local mirror cache
  - conda-packages/        : Built conda packages
  - rattler-packages/      : Built rattler packages

Each directory contains platform-specific subdirectories:
  - linux-64/      : Linux x86_64 packages
  - linux-aarch64/ : Linux ARM64 packages
  - osx-64/        : macOS x86_64 packages
  - osx-arm64/     : macOS ARM64 packages (Apple Silicon)
  - win-64/        : Windows x86_64 packages
  - noarch/        : Platform-independent packages

Common commands:
  pixi run meso-forge-mirror --help
  pixi run conda-build-nu
  pixi run setup-local-channel

To manually fix package placement, run:
  bash scripts/linux-activation.sh

Environment variables:
  MESO_FORGE_LOCAL_CACHE     : Points to local-conda-cache/
  MESO_FORGE_CONDA_PACKAGES  : Points to conda-packages/
  MESO_FORGE_RATTLER_PACKAGES: Points to rattler-packages/

EOF
}

# Main execution
main() {
    log_info "Starting Linux activation script for meso-forge-mirror"

    create_conda_directories
    fix_package_placement
    rebuild_indices
    verify_placement
    setup_environment

    if [[ "${1:-}" != "--quiet" ]]; then
        print_usage_info
    fi

    log_success "Linux activation completed successfully"
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
