#!/bin/bash
# Script to capture conda packages from Azure DevOps artifacts and install them locally
# Usage: ./capture-azure-artifacts.sh <build_id> [organization] [project]

set -e  # Exit on error
set -u  # Exit on undefined variable

# Default values
ORGANIZATION="${2:-conda-forge}"
PROJECT="${3:-feedstock-builds}"
BUILD_ID="${1:-}"
LOCAL_CACHE_DIR="./local-conda-cache"
TEMP_DIR="./temp-artifacts"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Help function
show_help() {
    echo "Usage: $0 <build_id> [organization] [project]"
    echo ""
    echo "Capture conda packages from Azure DevOps artifacts and install them locally"
    echo ""
    echo "Arguments:"
    echo "  build_id      Azure DevOps build ID (required)"
    echo "  organization  Azure DevOps organization (default: conda-forge)"
    echo "  project       Azure DevOps project (default: feedstock-builds)"
    echo ""
    echo "Examples:"
    echo "  $0 1372241"
    echo "  $0 1372241 conda-forge feedstock-builds"
    echo ""
    echo "Environment variables:"
    echo "  AZURE_DEVOPS_EXT_PAT  Personal Access Token for Azure DevOps (optional)"
    echo ""
}

# Check if build ID is provided
if [ -z "$BUILD_ID" ]; then
    echo -e "${RED}Error: Build ID is required${NC}"
    show_help
    exit 1
fi

# Check if required tools are available
check_tools() {
    local tools=("curl" "jq" "unzip")
    for tool in "${tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            echo -e "${RED}Error: $tool is not installed${NC}"
            exit 1
        fi
    done
}

# Create necessary directories
setup_directories() {
    mkdir -p "$LOCAL_CACHE_DIR"
    mkdir -p "$TEMP_DIR"
    echo -e "${GREEN}Created directories: $LOCAL_CACHE_DIR, $TEMP_DIR${NC}"
}

# Get Azure DevOps API headers
get_api_headers() {
    if [ -n "${AZURE_DEVOPS_EXT_PAT:-}" ]; then
        echo "-H 'Authorization: Basic $(echo -n :$AZURE_DEVOPS_EXT_PAT | base64)'"
    else
        echo ""
    fi
}

# List artifacts for the build
list_artifacts() {
    local api_url="https://dev.azure.com/$ORGANIZATION/$PROJECT/_apis/build/builds/$BUILD_ID/artifacts?api-version=6.0"
    local headers=$(get_api_headers)

    echo -e "${YELLOW}Fetching artifacts for build $BUILD_ID...${NC}"

    if [ -n "$headers" ]; then
        eval curl -s $headers "$api_url"
    else
        curl -s "$api_url"
    fi
}

# Download artifact
download_artifact() {
    local artifact_name="$1"
    local download_url="$2"
    local output_file="$TEMP_DIR/${artifact_name}.zip"
    local headers=$(get_api_headers)

    echo -e "${YELLOW}Downloading artifact: $artifact_name${NC}"

    if [ -n "$headers" ]; then
        eval curl -L $headers -o "$output_file" "$download_url"
    else
        curl -L -o "$output_file" "$download_url"
    fi

    echo "$output_file"
}

# Extract conda packages from artifact
extract_conda_packages() {
    local artifact_file="$1"
    local extract_dir="$TEMP_DIR/$(basename "$artifact_file" .zip)"

    echo -e "${YELLOW}Extracting artifact: $(basename "$artifact_file")${NC}"

    unzip -q "$artifact_file" -d "$extract_dir"

    # Find conda packages (.conda or .tar.bz2 files)
    find "$extract_dir" -type f \( -name "*.conda" -o -name "*.tar.bz2" \) | while read -r package_file; do
        local package_name=$(basename "$package_file")
        local target_file="$LOCAL_CACHE_DIR/$package_name"

        echo -e "${GREEN}Found conda package: $package_name${NC}"
        cp "$package_file" "$target_file"
        echo "  → Copied to: $target_file"
    done
}

# Install packages to pixi cache
install_to_pixi_cache() {
    echo -e "${YELLOW}Installing packages to pixi environment...${NC}"

    # Find all conda packages in local cache
    local packages=$(find "$LOCAL_CACHE_DIR" -name "*.conda" -o -name "*.tar.bz2")

    if [ -z "$packages" ]; then
        echo -e "${RED}No conda packages found in $LOCAL_CACHE_DIR${NC}"
        return 1
    fi

    for package in $packages; do
        local package_name=$(basename "$package")
        echo -e "${GREEN}Installing: $package_name${NC}"

        # Use conda/mamba to install the local package
        if command -v pixi &> /dev/null; then
            # Try to install with pixi
            echo "  Using pixi to install..."
            pixi add --local "$package" || echo "  Warning: pixi install failed, package copied to cache"
        elif command -v mamba &> /dev/null; then
            # Fall back to mamba
            echo "  Using mamba to install..."
            mamba install --use-local "$package" -y || echo "  Warning: mamba install failed"
        elif command -v conda &> /dev/null; then
            # Fall back to conda
            echo "  Using conda to install..."
            conda install --use-local "$package" -y || echo "  Warning: conda install failed"
        else
            echo "  No conda/mamba/pixi found, package only copied to cache"
        fi
    done
}

# Generate package index
generate_index() {
    echo -e "${YELLOW}Generating conda package index...${NC}"

    if command -v conda-index &> /dev/null; then
        conda-index "$LOCAL_CACHE_DIR"
        echo -e "${GREEN}Generated conda index in $LOCAL_CACHE_DIR${NC}"
    else
        echo -e "${YELLOW}Warning: conda-index not found, skipping index generation${NC}"
    fi
}

# Cleanup temporary files
cleanup() {
    if [ -d "$TEMP_DIR" ]; then
        echo -e "${YELLOW}Cleaning up temporary files...${NC}"
        rm -rf "$TEMP_DIR"
    fi
}

# Main execution
main() {
    echo -e "${GREEN}Azure DevOps Conda Package Capture Tool${NC}"
    echo "========================================"
    echo "Organization: $ORGANIZATION"
    echo "Project: $PROJECT"
    echo "Build ID: $BUILD_ID"
    echo "Local Cache: $LOCAL_CACHE_DIR"
    echo ""

    check_tools
    setup_directories

    # Get artifacts list
    local artifacts_json=$(list_artifacts)

    if echo "$artifacts_json" | jq -e . >/dev/null 2>&1; then
        # Parse artifacts
        local artifact_count=$(echo "$artifacts_json" | jq -r '.count // 0')

        if [ "$artifact_count" -eq 0 ]; then
            echo -e "${RED}No artifacts found for build $BUILD_ID${NC}"
            exit 1
        fi

        echo -e "${GREEN}Found $artifact_count artifacts${NC}"

        # Process each artifact
        echo "$artifacts_json" | jq -r '.value[] | "\(.name)|\(.resource.downloadUrl)"' | while IFS='|' read -r name download_url; do
            if [ -n "$name" ] && [ -n "$download_url" ]; then
                echo ""
                echo -e "${YELLOW}Processing artifact: $name${NC}"

                # Download artifact
                local artifact_file=$(download_artifact "$name" "$download_url")

                # Extract conda packages
                extract_conda_packages "$artifact_file"
            fi
        done

        # Install packages and generate index
        install_to_pixi_cache
        generate_index

    else
        echo -e "${RED}Failed to fetch artifacts. Response:${NC}"
        echo "$artifacts_json"
        exit 1
    fi

    cleanup

    echo ""
    echo -e "${GREEN}✅ Conda packages captured successfully!${NC}"
    echo -e "${GREEN}📦 Packages available in: $LOCAL_CACHE_DIR${NC}"
    echo ""
    echo "To use these packages:"
    echo "  1. Add to conda channels: conda config --add channels file://$(pwd)/$LOCAL_CACHE_DIR"
    echo "  2. Or use directly: conda install --use-local <package-name>"
    echo "  3. Or with pixi: pixi add --local $LOCAL_CACHE_DIR/<package-file>"
}

# Handle script interruption
trap cleanup EXIT INT TERM

# Run main function
main "$@"
