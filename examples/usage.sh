#!/bin/bash
# Example usage scripts for meso-forge-mirror

# Example 1: Mirror a single package to a local repository
echo "Example 1: Mirror to local repository"
meso-forge-mirror mirror \
  --sources "https://conda.anaconda.org/conda-forge/linux-64/zlib-1.2.13-hd590300_5.conda" \
  --target-type local \
  --target-path ./local-mirror

# Example 2: Mirror multiple packages to local repository
echo "Example 2: Mirror multiple packages"
meso-forge-mirror mirror \
  --sources "https://conda.anaconda.org/conda-forge/linux-64/zlib-1.2.13-hd590300_5.conda" \
  --sources "https://conda.anaconda.org/conda-forge/linux-64/bzip2-1.0.8-hd590300_5.conda" \
  --target-type local \
  --target-path ./local-mirror

# Example 3: Mirror packages to S3
echo "Example 3: Mirror to S3"
export AWS_ACCESS_KEY_ID="your_access_key"
export AWS_SECRET_ACCESS_KEY="your_secret_key"

meso-forge-mirror mirror \
  --sources "https://conda.anaconda.org/conda-forge/linux-64/zlib-1.2.13-hd590300_5.conda" \
  --target-type s3 \
  --target-path "s3://my-conda-bucket/packages" \
  --config examples/config.json

# Example 4: Mirror to MinIO (S3-compatible)
echo "Example 4: Mirror to MinIO"
export AWS_ACCESS_KEY_ID="minio_access_key"
export AWS_SECRET_ACCESS_KEY="minio_secret_key"

meso-forge-mirror mirror \
  --sources "https://conda.anaconda.org/conda-forge/linux-64/zlib-1.2.13-hd590300_5.conda" \
  --target-type s3 \
  --target-path "s3://conda-packages/linux-64" \
  --config examples/config-minio.json

# Example 5: Mirror to prefix.dev channel
echo "Example 5: Mirror to prefix.dev"
meso-forge-mirror mirror \
  --sources "https://conda.anaconda.org/conda-forge/linux-64/zlib-1.2.13-hd590300_5.conda" \
  --target-type prefix-dev \
  --target-path "https://prefix.dev/channels/meso-forge"

# Example 6: Initialize a configuration file
echo "Example 6: Initialize configuration"
meso-forge-mirror init-config -o my-config.json

# Example 7: Use comma-separated URLs
echo "Example 7: Comma-separated sources"
meso-forge-mirror mirror \
  --sources "https://conda.anaconda.org/conda-forge/linux-64/zlib-1.2.13-hd590300_5.conda,https://conda.anaconda.org/conda-forge/linux-64/bzip2-1.0.8-hd590300_5.conda" \
  --target-type local \
  --target-path ./local-mirror
