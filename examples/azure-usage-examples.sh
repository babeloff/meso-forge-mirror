#!/bin/bash

# Azure DevOps Integration Usage Examples for meso-forge-mirror
#
# This script demonstrates various ways to use the new Azure DevOps artifacts
# integration functionality for conda-forge and other Azure DevOps projects.
#
# Prerequisites:
# 1. Set AZURE_DEVOPS_TOKEN environment variable or use --config with token
# 2. Ensure meso-forge-mirror is built and available in PATH

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Azure DevOps Integration Examples ===${NC}"

# Check if Azure DevOps token is available
if [ -z "$AZURE_DEVOPS_TOKEN" ]; then
    echo -e "${YELLOW}Warning: AZURE_DEVOPS_TOKEN not set. Azure DevOps requires authentication.${NC}"
    echo "Set it with: export AZURE_DEVOPS_TOKEN=your_pat_token_here"
    echo "Get a PAT from: https://dev.azure.com/ → Security → Personal Access Tokens"
fi

echo -e "\n${GREEN}1. Basic Info Command Examples${NC}"
echo "=================================="

# Example 1: List recent builds for a project
echo -e "${BLUE}Example 1a: List recent builds${NC}"
echo "Command: meso-forge-mirror info --azure conda-forge/feedstock-builds"
echo "# This lists recent builds from the conda-forge feedstock-builds project"
echo

# Example 1b: List artifacts for a specific build
echo -e "${BLUE}Example 1b: List artifacts for specific build${NC}"
echo "Command: meso-forge-mirror info --azure conda-forge/feedstock-builds --build-id 1374331"
echo "# This lists all artifacts from build ID 1374331 (the PR #31205 example)"
echo

# Example 1c: Filter artifacts by name
echo -e "${BLUE}Example 1c: Filter artifacts by name${NC}"
echo "Command: meso-forge-mirror info --azure conda-forge/feedstock-builds --build-id 1374331 --name-filter 'conda.*'"
echo "# This shows only artifacts with 'conda' in the name"
echo

echo -e "\n${GREEN}2. conda-forge Specific Examples${NC}"
echo "================================="

# Example 2a: The original use case - PR #31205
echo -e "${BLUE}Example 2a: PR #31205 workflow${NC}"
cat << 'EOF'
# Step 1: Find the build (if you don't know the build ID)
meso-forge-mirror info --azure conda-forge/feedstock-builds | grep "31205"

# Step 2: List artifacts for that build
meso-forge-mirror info --azure conda-forge/feedstock-builds --build-id 1374331

# Step 3: Download conda packages from that PR
meso-forge-mirror mirror \
  --src-type azure \
  --src conda-forge/feedstock-builds#1374331 \
  --src-path "conda.*packages.*" \
  --tgt-type local \
  --tgt ./pr-31205-packages
EOF
echo

# Example 2b: Direct build access
echo -e "${BLUE}Example 2b: Direct access to specific build${NC}"
echo "Command: meso-forge-mirror mirror --src-type azure --src conda-forge/feedstock-builds#1374331"
echo "# This directly processes all artifacts from build 1374331"
echo

echo -e "\n${GREEN}3. Mirror Command Examples${NC}"
echo "=========================="

# Example 3a: Mirror to cache (default)
echo -e "${BLUE}Example 3a: Mirror to cache${NC}"
echo "Command: meso-forge-mirror mirror --src-type azure --src conda-forge/feedstock-builds#1374331"
echo "# This downloads artifacts and caches conda packages"
echo

# Example 3b: Mirror to local conda repository
echo -e "${BLUE}Example 3b: Mirror to local conda repository${NC}"
echo "Command: meso-forge-mirror mirror --src-type azure --src conda-forge/feedstock-builds#1374331 --tgt-type local --tgt ./conda-repo"
echo "# This creates a local conda repository with repodata.json"
echo

# Example 3c: Mirror to S3
echo -e "${BLUE}Example 3c: Mirror to S3${NC}"
echo "Command: meso-forge-mirror mirror --src-type azure --src conda-forge/feedstock-builds#1374331 --tgt-type s3 --tgt s3://my-bucket/conda-channel"
echo "# This uploads to S3 and creates conda repository structure"
echo

echo -e "\n${GREEN}4. Advanced Filtering Examples${NC}"
echo "=============================="

# Example 4a: Filter by artifact name pattern
echo -e "${BLUE}Example 4a: Filter conda package artifacts only${NC}"
echo "Command: meso-forge-mirror mirror --src-type azure --src conda-forge/feedstock-builds#1374331 --src-path 'conda.*packages.*'"
echo "# This only processes artifacts with 'conda' and 'packages' in the name"
echo

# Example 4b: Target specific platform
echo -e "${BLUE}Example 4b: Target Linux-64 packages only${NC}"
echo "Command: meso-forge-mirror mirror --src-type azure --src conda-forge/feedstock-builds#1374331 --src-path '.*linux-64.*'"
echo "# This only processes artifacts containing 'linux-64'"
echo

# Example 4c: Process latest successful build
echo -e "${BLUE}Example 4c: Process latest successful build${NC}"
echo "Command: meso-forge-mirror mirror --src-type azure --src conda-forge/feedstock-builds --src-path 'conda.*'"
echo "# This processes the most recent successful build with conda artifacts"
echo

echo -e "\n${GREEN}5. Configuration File Examples${NC}"
echo "==============================="

# Example 5a: Using configuration file
echo -e "${BLUE}Example 5a: Using configuration file${NC}"
cat << 'EOF'
# Create azure-config.json:
{
  "azure_devops_token": "your_pat_token_here",
  "max_concurrent_downloads": 5,
  "retry_attempts": 3,
  "timeout_seconds": 300
}

# Command:
meso-forge-mirror info --azure conda-forge/feedstock-builds --config azure-config.json
EOF
echo

echo -e "\n${GREEN}6. Repository Format Examples${NC}"
echo "============================="

# Different ways to specify Azure DevOps projects
echo -e "${BLUE}Repository format options:${NC}"
cat << 'EOF'
# All of these are equivalent:
meso-forge-mirror info --azure conda-forge/feedstock-builds
meso-forge-mirror info --azure https://dev.azure.com/conda-forge/feedstock-builds
meso-forge-mirror info --azure https://dev.azure.com/conda-forge/feedstock-builds/

# Specific build ID selection:
meso-forge-mirror mirror --src-type azure --src conda-forge/feedstock-builds#1374331
meso-forge-mirror mirror --src-type azure --src https://dev.azure.com/conda-forge/feedstock-builds#1374331
EOF
echo

echo -e "\n${GREEN}7. Real-world Workflow Examples${NC}"
echo "==============================="

# Example 7a: Complete conda-forge workflow
echo -e "${BLUE}Example 7a: Complete conda-forge workflow${NC}"
cat << 'EOF'
#!/bin/bash
# Complete workflow for processing conda-forge PR artifacts

set -e

# Configuration
export AZURE_DEVOPS_TOKEN="your_pat_token_here"
ORG="conda-forge"
PROJECT="feedstock-builds"
TARGET_DIR="./conda-packages"

# Step 1: Find recent builds
echo "Finding recent builds..."
meso-forge-mirror info --azure $ORG/$PROJECT

# Step 2: Get artifacts for a specific build (replace with actual build ID)
BUILD_ID="1374331"
echo "Getting artifacts for build $BUILD_ID..."
meso-forge-mirror info --azure $ORG/$PROJECT --build-id $BUILD_ID --name-filter "conda.*"

# Step 3: Mirror the packages
echo "Mirroring conda packages..."
meso-forge-mirror mirror \
  --src-type azure \
  --src "$ORG/$PROJECT#$BUILD_ID" \
  --src-path "conda.*packages.*" \
  --tgt-type local \
  --tgt "$TARGET_DIR"

echo "Conda packages available in $TARGET_DIR"
ls -la "$TARGET_DIR"
EOF
echo

# Example 7b: CI/CD integration
echo -e "${BLUE}Example 7b: CI/CD integration${NC}"
cat << 'EOF'
# In a CI/CD pipeline script:
#!/bin/bash
set -e

# Get build ID from environment or parameter
BUILD_ID="${1:-${AZURE_BUILD_ID}}"
if [ -z "$BUILD_ID" ]; then
  echo "Usage: $0 <build_id>"
  echo "Or set AZURE_BUILD_ID environment variable"
  exit 1
fi

# Mirror packages from specific Azure DevOps build to staging
meso-forge-mirror mirror \
  --src-type azure \
  --src conda-forge/feedstock-builds#$BUILD_ID \
  --src-path "conda.*" \
  --tgt-type s3 \
  --tgt s3://staging-conda-channel/builds/$BUILD_ID \
  --config ~/.azure-conda-mirror-config.json

echo "Packages available at: s3://staging-conda-channel/builds/$BUILD_ID"
EOF
echo

# Example 7c: Multi-build processing
echo -e "${BLUE}Example 7c: Multi-build aggregation${NC}"
cat << 'EOF'
# Process artifacts from multiple builds
builds=(1374331 1374330 1374329 1374328)

for build_id in "${builds[@]}"; do
  echo "Processing build $build_id..."

  # Check if build has conda artifacts
  if meso-forge-mirror info --azure conda-forge/feedstock-builds --build-id $build_id --name-filter "conda.*" | grep -q "Found"; then
    meso-forge-mirror mirror \
      --src-type azure \
      --src conda-forge/feedstock-builds#$build_id \
      --src-path "conda.*" \
      --tgt-type local \
      --tgt ./aggregated-builds/$build_id \
      --config azure-config.json
    echo "Build $build_id processed successfully"
  else
    echo "No conda artifacts found for build $build_id"
  fi
done

echo "All builds processed. Results in ./aggregated-builds/"
EOF
echo

echo -e "\n${GREEN}8. Troubleshooting Examples${NC}"
echo "=========================="

# Example 8a: Debug mode
echo -e "${BLUE}Example 8a: Debug mode with verbose logging${NC}"
echo "Command: RUST_LOG=debug meso-forge-mirror info --azure conda-forge/feedstock-builds"
echo "# This enables debug logging to troubleshoot issues"
echo

# Example 8b: Test specific build
echo -e "${BLUE}Example 8b: Test specific build access${NC}"
echo "Command: meso-forge-mirror info --azure conda-forge/feedstock-builds --build-id 1374331"
echo "# This tests access to a specific build"
echo

# Example 8c: Validate configuration
echo -e "${BLUE}Example 8c: Validate configuration${NC}"
echo "Command: meso-forge-mirror init --output azure-test-config.json && cat azure-test-config.json"
echo "# This creates a default config file for reference"
echo

echo -e "\n${GREEN}9. Integration with Existing Features${NC}"
echo "====================================="

echo -e "${BLUE}Example 9a: Combine with other source types${NC}"
cat << 'EOF'
# You can still use all existing source types:
meso-forge-mirror mirror --src-type zip --src ./packages.zip --src-path "*.conda"
meso-forge-mirror mirror --src-type url --src https://example.com/package.conda
meso-forge-mirror mirror --src-type github --src owner/repo
meso-forge-mirror mirror --src-type tgz --src ./packages.tar.gz

# And now also Azure DevOps:
meso-forge-mirror mirror --src-type azure --src conda-forge/feedstock-builds#1374331
EOF
echo

echo -e "\n${GREEN}10. Authentication Setup${NC}"
echo "========================"

echo -e "${BLUE}Setting up Azure DevOps authentication:${NC}"
cat << 'EOF'
# Method 1: Environment variable
export AZURE_DEVOPS_TOKEN="your_personal_access_token_here"

# Method 2: Configuration file
cat > azure-config.json << EOL
{
  "azure_devops_token": "your_personal_access_token_here",
  "max_concurrent_downloads": 5,
  "retry_attempts": 3,
  "timeout_seconds": 300
}
EOL

# To get a Personal Access Token:
# 1. Go to https://dev.azure.com/
# 2. Click your profile → Security → Personal Access Tokens
# 3. Create new token with scopes: Build (read), Artifact (read)
# 4. Copy the token and use it as AZURE_DEVOPS_TOKEN
EOF
echo

echo -e "\n${GREEN}Setup Instructions${NC}"
echo "=================="
echo "1. Build the tool:"
echo "   cd meso-forge-mirror && cargo build --release"
echo
echo "2. Set up Azure DevOps PAT:"
echo "   - Go to https://dev.azure.com/ → Security → Personal Access Tokens"
echo "   - Create token with Build (read) and Artifact (read) scopes"
echo "   - export AZURE_DEVOPS_TOKEN=your_token_here"
echo
echo "3. Try the info command:"
echo "   ./target/release/meso-forge-mirror info --azure conda-forge/feedstock-builds"
echo
echo "4. Try accessing the specific PR build:"
echo "   ./target/release/meso-forge-mirror info --azure conda-forge/feedstock-builds --build-id 1374331"
echo
echo "5. Mirror artifacts from that build:"
echo "   ./target/release/meso-forge-mirror mirror --src-type azure --src conda-forge/feedstock-builds#1374331"
echo

echo -e "\n${GREEN}For more information, see:${NC}"
echo "- docs/chapters/azure-devops-integration.adoc - Comprehensive Azure DevOps usage guide"
echo "- docs/chapters/github-integration.adoc - GitHub integration comparison"
echo "- IMPLEMENTATION_SUMMARY.md - Technical implementation details"
echo "- examples/azure-config.json - Example configuration file"

echo -e "\n${BLUE}=== End of Azure DevOps Examples ===${NC}"
