#!/usr/bin/env nu
# Example usage scripts for meso-forge-mirror using nushell

# Function to run example with logging
def run_example [name: string, command: closure] {
    print $"(ansi green)Example ($name):(ansi reset)"
    do $command
    print ""
}

# Example 1: Mirror a single package to a local repository
run_example "1: Mirror to local repository" {
    cargo run -- mirror `
        --src "https://conda.anaconda.org/conda-forge/linux-64/zlib-1.2.13-hd590300_5.conda" `
        --src-type url `
        --tgt-type local `
        --tgt ./local-mirror
}

# Example 2: Mirror multiple packages to local repository
run_example "2: Mirror multiple packages" {
    cargo run -- mirror `
        --src "https://conda.anaconda.org/conda-forge/linux-64/zlib-1.2.13-hd590300_5.conda" `
        --src-type url `
        --tgt-type local `
        --tgt ./multi-mirror

    cargo run -- mirror `
        --src "https://conda.anaconda.org/conda-forge/linux-64/bzip2-1.0.8-hd590300_5.conda" `
        --src-type url `
        --tgt-type local `
        --tgt ./multi-mirror
}

# Example 3: Mirror packages to S3
run_example "3: Mirror to S3" {
    # Set AWS environment variables
    $env.AWS_ACCESS_KEY_ID = "your_access_key"
    $env.AWS_SECRET_ACCESS_KEY = "your_secret_key"

    cargo run -- mirror `
        --src "https://conda.anaconda.org/conda-forge/linux-64/zlib-1.2.13-hd590300_5.conda" `
        --src-type url `
        --tgt-type s3 `
        --tgt "s3://my-conda-bucket/packages" `
        --config examples/config-s3.json
}

# Example 4: Mirror to MinIO (S3-compatible)
run_example "4: Mirror to MinIO" {
    # Set MinIO environment variables
    $env.AWS_ACCESS_KEY_ID = "minio_access_key"
    $env.AWS_SECRET_ACCESS_KEY = "minio_secret_key"

    cargo run -- mirror `
        --src "https://conda.anaconda.org/conda-forge/linux-64/zlib-1.2.13-hd590300_5.conda" `
        --src-type url `
        --tgt-type s3 `
        --tgt "s3://conda-packages/linux-64" `
        --config examples/config-s3.json
}

# Example 5: Mirror to prefix.dev channel
run_example "5: Mirror to prefix.dev" {
    cargo run -- mirror `
        --src "https://conda.anaconda.org/conda-forge/linux-64/zlib-1.2.13-hd590300_5.conda" `
        --src-type url `
        --tgt-type prefix-dev `
        --tgt "https://prefix.dev/channels/meso-forge"
}

# Example 6: Initialize a configuration file
run_example "6: Initialize configuration" {
    cargo run -- init -o my-config.json
}

# Example 7: Use comma-separated URLs
run_example "7: Comma-separated sources" {
    let packages = [
        "https://conda.anaconda.org/conda-forge/linux-64/zlib-1.2.13-hd590300_5.conda",
        "https://conda.anaconda.org/conda-forge/linux-64/bzip2-1.0.8-hd590300_5.conda"
    ]

    # Mirror each package individually
    for package in $packages {
        cargo run -- mirror `
            --src $package `
            --src-type url `
            --tgt-type local `
            --tgt ./local-mirror
    }
}

# Example 8: Advanced usage with multiple packages from different sources
run_example "8: Advanced multi-source mirroring" {
    let package_urls = [
        "https://conda.anaconda.org/conda-forge/linux-64/openssl-3.1.4-hd590300_0.conda",
        "https://conda.anaconda.org/conda-forge/linux-64/ca-certificates-2023.11.17-hbcca054_0.conda",
        "https://conda.anaconda.org/conda-forge/noarch/pip-23.3.1-pyhd8ed1ab_0.conda"
    ]

    # Mirror each package individually with logging
    for url in $package_urls {
        print $"Mirroring: (ansi blue)($url)(ansi reset)"
        cargo run -- mirror `
            --src $url `
            --src-type url `
            --tgt-type local `
            --tgt ./advanced-mirror
    }
}

# Example 9: Test configuration and validate setup
run_example "9: Configuration validation" {
    # Generate a test config
    cargo run -- init -o test-config.json

    # Verify the config file was created and has expected content
    if ("test-config.json" | path exists) {
        let config = (open test-config.json)
        print "Configuration created successfully:"
        print ($config | to yaml)

        # Test with a small package
        cargo run -- mirror `
            --src "https://conda.anaconda.org/conda-forge/noarch/wheel-0.42.0-pyhd8ed1ab_0.conda" `
            --src-type url `
            --tgt-type local `
            --tgt ./config-mirror `
            --config test-config.json
    } else {
        print "(ansi red)Failed to create configuration file(ansi reset)"
    }
}

# Example 10: Batch processing with error handling
run_example "10: Batch processing with error handling" {
    let packages = [
        {
            name: "zlib",
            url: "https://conda.anaconda.org/conda-forge/linux-64/zlib-1.2.13-hd590300_5.conda"
        },
        {
            name: "bzip2",
            url: "https://conda.anaconda.org/conda-forge/linux-64/bzip2-1.0.8-hd590300_5.conda"
        },
        {
            name: "xz",
            url: "https://conda.anaconda.org/conda-forge/linux-64/xz-5.2.6-h166bdaf_0.conda"
        }
    ]

    let target_dir = "./batch-mirror"
    mkdir $target_dir

    mut successful = []
    mut failed = []

    for package in $packages {
        try {
            print $"Processing ($package.name)..."
            cargo run -- mirror `
                --src $package.url `
                --src-type url `
                --tgt-type local `
                --tgt $target_dir

            $successful = ($successful | append $package.name)
            print $"(ansi green)✓ ($package.name) mirrored successfully(ansi reset)"
        } catch {
            $failed = ($failed | append $package.name)
            print $"(ansi red)✗ ($package.name) failed to mirror(ansi reset)"
        }
    }

    print $"Batch processing complete:"
    print $"  Successful: ($successful | length) packages"
    print $"  Failed: ($failed | length) packages"

    if not ($failed | is-empty) {
        print $"Failed packages: ($failed | str join ', ')"
    }
}

# Example 11: Mirror from local conda file
run_example "11: Mirror from local conda file" {
    # First download a package to demonstrate local mirroring
    print "Downloading package for local example..."
    http get "https://conda.anaconda.org/conda-forge/noarch/wheel-0.42.0-pyhd8ed1ab_0.conda" | save local-package.conda

    # Mirror the local file
    cargo run -- mirror `
        --src local-package.conda `
        --src-type local `
        --tgt-type local `
        --tgt ./local-file-mirror

    # Clean up
    rm local-package.conda
}

# Example 12: Mirror from ZIP file containing packages
run_example "12: Mirror from ZIP file (simulated)" {
    # This example shows the command structure for ZIP files
    # In practice, you would have a real ZIP file with conda packages
    print "Note: This example shows the command structure for ZIP files (first match only)"
    print "Command would be:"
    print "cargo run -- mirror --src packages.zip --src-type zip --src-path '^conda-packages/.*' --tgt ./zip-mirror"

    # Create a demonstration directory structure
    mkdir zip-mirror-demo
    print "Created demonstration directory: zip-mirror-demo"
}

# Example 13: Mirror from remote ZIP file
run_example "13: Mirror from remote ZIP file (simulated)" {
    # This example shows the command structure for remote ZIP files
    print "Note: This example shows the command structure for remote ZIP files (first match only)"
    print "Command would be:"
    print "cargo run -- mirror --src 'https://example.com/packages.zip' --src-type zip-url --src-path '^build-artifacts/.*' --tgt ./remote-zip-mirror"

    # Create a demonstration directory structure
    mkdir remote-zip-mirror-demo
    print "Created demonstration directory: remote-zip-mirror-demo"
}

# Example 14: Mirror from tarball
run_example "14: Mirror from tarball (simulated)" {
    # This example shows the command structure for tarball files
    print "Note: This example shows the command structure for tarball files"
    print "Command would be:"
    print "cargo run -- mirror --src packages.tar.gz --src-type tgz --tgt ./tarball-mirror"

    # Create a demonstration directory structure
    mkdir tarball-mirror-demo
    print "Created demonstration directory: tarball-mirror-demo"
}

# Example 15: Mirror from remote tarball
run_example "15: Mirror from remote tarball (simulated)" {
    # This example shows the command structure for remote tarball files
    print "Note: This example shows the command structure for remote tarball files"
    print "Command would be:"
    print "cargo run -- mirror --src 'https://github.com/owner/repo/archive/main.tar.gz' --src-type tgz-url --tgt ./remote-tarball-mirror"

    # Create a demonstration directory structure
    mkdir remote-tarball-mirror-demo
    print "Created demonstration directory: remote-tarball-mirror-demo"
}

# Example 16: Demonstrate source type validation
run_example "16: Source type validation" {
    print "Testing invalid source type (should fail)..."
    try {
        cargo run -- mirror `
            --src "test" `
            --src-type invalid `
            --tgt ./test-dir
    } catch {
        print "(ansi green)✓ Validation working correctly - invalid source type rejected(ansi reset)"
    }

    print "Testing missing required --src-path for zip (should fail)..."
    try {
        cargo run -- mirror `
            --src "test.zip" `
            --src-type zip `
            --tgt ./test-dir
    } catch {
        print "(ansi green)✓ Validation working correctly - missing --src-path for zip rejected(ansi reset)"
    }
}

print "(ansi blue)All examples completed!(ansi reset)"
print "Check the created directories:"
print "  ./local-mirror"
print "  ./advanced-mirror"
print "  ./config-test"
print "  ./batch-mirror"
print "  ./zip-mirror-demo"
print "  ./remote-zip-mirror-demo"
print "  ./tarball-mirror-demo"
print "  ./remote-tarball-mirror-demo"
print ""
print "(ansi cyan)New CLI Options Summary:(ansi reset)"
print "  --src <path>           Source file or URL"
print "  --src-type <type>      Source type: local, url, zip, zip-url, tgz, tgz-url"
print "  --tgt <path>           Target repository path"
print "  --tgt-type <type>      Target type: local, s3, prefix-dev"
print "  --src-path <regex>     Required for zip/zip-url: regex pattern to match paths in archive (first match only)"
print ""
print "(ansi yellow)Source Types:(ansi reset)"
print "  local     - Local conda package file"
print "  url       - Remote conda package URL"
print "  zip       - Local ZIP file containing conda packages"
print "  zip-url   - Remote ZIP file containing conda packages"
print "  tgz       - Local tarball (tar.gz) containing conda packages"
print "  tgz-url   - Remote tarball (tar.gz) containing conda packages"
