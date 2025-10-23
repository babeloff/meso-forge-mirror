#!/bin/bash

# GitHub Integration Usage Examples for meso-forge-mirror
#
# This script demonstrates various ways to use the new GitHub artifacts
# integration functionality.
#
# Prerequisites:
# 1. Set GITHUB_TOKEN environment variable or use --config with token
# 2. Ensure meso-forge-mirror is built and available in PATH

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== GitHub Artifacts Integration Examples ===${NC}"

# Check if GitHub token is available
if [ -z "$GITHUB_TOKEN" ]; then
    echo -e "${YELLOW}Warning: GITHUB_TOKEN not set. Some examples may not work for private repos.${NC}"
    echo "Set it with: export GITHUB_TOKEN=ghp_your_token_here"
fi

echo -e "\n${GREEN}1. Basic Info Command Examples${NC}"
echo "=================================="

# Example 1: List all artifacts for a public repository
echo -e "${BLUE}Example 1a: List all artifacts${NC}"
echo "Command: meso-forge-mirror info --github conda-forge/numpy"
echo "# This would list all artifacts from the conda-forge numpy feedstock"
echo

# Example 1b: List artifacts with name filtering
echo -e "${BLUE}Example 1b: Filter artifacts by name${NC}"
echo "Command: meso-forge-mirror info --github owner/repo --name-filter 'conda.*linux.*'"
echo "# This filters artifacts containing 'conda' and 'linux' in the name"
echo

# Example 1c: Include expired artifacts
echo -e "${BLUE}Example 1c: Include expired artifacts${NC}"
echo "Command: meso-forge-mirror info --github owner/repo --exclude-expired false"
echo "# This includes artifacts that have expired"
echo

echo -e "\n${GREEN}2. Basic Mirror Command Examples${NC}"
echo "====================================="

# Example 2a: Mirror to cache (default)
echo -e "${BLUE}Example 2a: Mirror to cache${NC}"
echo "Command: meso-forge-mirror mirror --src-type github --src owner/repo"
echo "# This downloads artifacts and caches conda packages"
echo

# Example 2b: Mirror to local conda repository
echo -e "${BLUE}Example 2b: Mirror to local conda repository${NC}"
echo "Command: meso-forge-mirror mirror --src-type github --src owner/repo --tgt-type local --tgt ./conda-repo"
echo "# This creates a local conda repository with repodata.json"
echo

# Example 2c: Mirror to S3
echo -e "${BLUE}Example 2c: Mirror to S3${NC}"
echo "Command: meso-forge-mirror mirror --src-type github --src owner/repo --tgt-type s3 --tgt s3://my-bucket/conda-channel"
echo "# This uploads to S3 and creates conda repository structure"
echo

echo -e "\n${GREEN}3. Advanced Filtering Examples${NC}"
echo "=============================="

# Example 3a: Filter by artifact name pattern
echo -e "${BLUE}Example 3a: Filter by conda package artifacts${NC}"
echo "Command: meso-forge-mirror mirror --src-type github --src owner/repo --src-path 'conda.*packages.*'"
echo "# This only processes artifacts with 'conda' and 'packages' in the name"
echo

# Example 3b: Target specific platform
echo -e "${BLUE}Example 3b: Target Linux-64 packages only${NC}"
echo "Command: meso-forge-mirror mirror --src-type github --src owner/repo --src-path '.*linux-64.*'"
echo "# This only processes artifacts containing 'linux-64'"
echo

# Example 3c: Process specific artifact by ID
echo -e "${BLUE}Example 3c: Process specific artifact${NC}"
echo "Command: meso-forge-mirror mirror --src-type github --src owner/repo#123456789"
echo "# This processes only the artifact with ID 123456789"
echo

echo -e "\n${GREEN}4. Configuration File Examples${NC}"
echo "==============================="

# Example 4a: Using configuration file
echo -e "${BLUE}Example 4a: Using configuration file${NC}"
cat << 'EOF'
# Create config.json:
{
  "github_token": "ghp_your_token_here",
  "max_concurrent_downloads": 5,
  "retry_attempts": 3,
  "timeout_seconds": 300
}

# Command:
meso-forge-mirror info --github owner/repo --config config.json
EOF
echo

echo -e "\n${GREEN}5. Real-world Workflow Examples${NC}"
echo "==============================="

# Example 5a: Conda-forge feedstock workflow
echo -e "${BLUE}Example 5a: Conda-forge feedstock workflow${NC}"
cat << 'EOF'
# 1. Check what artifacts are available
meso-forge-mirror info --github conda-forge/python-feedstock --name-filter "conda.*"

# 2. Mirror the conda packages to local repository
meso-forge-mirror mirror \
  --src-type github \
  --src conda-forge/python-feedstock \
  --src-path "conda.*packages.*" \
  --tgt-type local \
  --tgt ./my-conda-channel

# 3. The result is a conda channel at ./my-conda-channel with:
#    - linux-64/repodata.json
#    - osx-64/repodata.json
#    - win-64/repodata.json
#    - noarch/repodata.json
#    - Package files in appropriate subdirectories
EOF
echo

# Example 5b: CI/CD integration
echo -e "${BLUE}Example 5b: CI/CD integration${NC}"
cat << 'EOF'
# In a CI/CD pipeline script:
#!/bin/bash
set -e

# Mirror packages from PR builds to staging channel
meso-forge-mirror mirror \
  --src-type github \
  --src myorg/my-feedstock \
  --src-path "conda.*$(git rev-parse --short HEAD).*" \
  --tgt-type s3 \
  --tgt s3://staging-conda-channel/pr-builds \
  --config ~/.conda-mirror-config.json

echo "Packages available at: https://staging-conda-channel.s3.amazonaws.com/pr-builds"
EOF
echo

# Example 5c: Multi-repository aggregation
echo -e "${BLUE}Example 5c: Multi-repository aggregation${NC}"
cat << 'EOF'
# Aggregate packages from multiple repositories
repos=("myorg/package-a" "myorg/package-b" "myorg/package-c")

for repo in "${repos[@]}"; do
  echo "Processing $repo..."
  meso-forge-mirror mirror \
    --src-type github \
    --src "$repo" \
    --src-path "conda.*" \
    --tgt-type local \
    --tgt ./aggregated-channel \
    --config config.json
done

# Finalize the channel
meso-forge-mirror finalize-channel ./aggregated-channel
EOF
echo

echo -e "\n${GREEN}6. Troubleshooting Examples${NC}"
echo "=========================="

# Example 6a: Debug mode
echo -e "${BLUE}Example 6a: Debug mode with verbose logging${NC}"
echo "Command: RUST_LOG=debug meso-forge-mirror info --github owner/repo"
echo "# This enables debug logging to troubleshoot issues"
echo

# Example 6b: Test connectivity
echo -e "${BLUE}Example 6b: Test GitHub connectivity${NC}"
echo "Command: meso-forge-mirror info --github octocat/Hello-World"
echo "# This tests against a known public repository"
echo

# Example 6c: Validate configuration
echo -e "${BLUE}Example 6c: Validate configuration${NC}"
echo "Command: meso-forge-mirror init --output test-config.json && cat test-config.json"
echo "# This creates a default config file for reference"
echo

echo -e "\n${GREEN}7. Repository Format Examples${NC}"
echo "============================="

# Different ways to specify repositories
echo -e "${BLUE}Repository format options:${NC}"
cat << 'EOF'
# All of these are equivalent:
meso-forge-mirror info --github owner/repository
meso-forge-mirror info --github https://github.com/owner/repository
meso-forge-mirror info --github https://github.com/owner/repository/

# Specific artifact selection:
meso-forge-mirror mirror --src-type github --src owner/repo#123456789
EOF
echo

echo -e "\n${GREEN}8. Integration with Existing Features${NC}"
echo "====================================="

echo -e "${BLUE}Example 8a: Combine with existing source types${NC}"
cat << 'EOF'
# You can still use all existing source types:
meso-forge-mirror mirror --src-type zip --src ./packages.zip --src-path "*.conda"
meso-forge-mirror mirror --src-type url --src https://example.com/package.conda
meso-forge-mirror mirror --src-type tgz --src ./packages.tar.gz

# And now also:
meso-forge-mirror mirror --src-type github --src owner/repo
EOF
echo

echo -e "\n${GREEN}Setup Instructions${NC}"
echo "=================="
echo "1. Build the tool:"
echo "   cd meso-forge-mirror && cargo build --release"
echo
echo "2. Set up GitHub token:"
echo "   export GITHUB_TOKEN=ghp_your_token_here"
echo
echo "3. Try the info command:"
echo "   ./target/release/meso-forge-mirror info --github octocat/Hello-World"
echo
echo "4. Create a config file:"
echo "   ./target/release/meso-forge-mirror init --output my-config.json"
echo "   # Edit my-config.json to add your GitHub token"
echo

echo -e "\n${YELLOW}Note: Due to SSL linking issues in some environments, you may need to:"
echo "- Use system Rust instead of conda/pixi Rust"
echo "- Install OpenSSL development packages"
echo "- Set appropriate environment variables for SSL libraries${NC}"

echo -e "\n${GREEN}For more information, see:${NC}"
echo "- docs/chapters/github-integration.adoc - Comprehensive usage guide"
echo "- IMPLEMENTATION_SUMMARY.md - Technical implementation details"
echo "- examples/github-config.json - Example configuration file"

echo -e "\n${BLUE}=== End of Examples ===${NC}"
