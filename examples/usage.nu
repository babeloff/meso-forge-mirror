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
        --sources "https://conda.anaconda.org/conda-forge/linux-64/zlib-1.2.13-hd590300_5.conda" `
        --target-type local `
        --target-path ./local-mirror
}

# Example 2: Mirror multiple packages to local repository
run_example "2: Mirror multiple packages" {
    cargo run -- mirror `
        --sources "https://conda.anaconda.org/conda-forge/linux-64/zlib-1.2.13-hd590300_5.conda" `
        --sources "https://conda.anaconda.org/conda-forge/linux-64/bzip2-1.0.8-hd590300_5.conda" `
        --target-type local `
        --target-path ./local-mirror
}

# Example 3: Mirror packages to S3
run_example "3: Mirror to S3" {
    # Set AWS environment variables
    $env.AWS_ACCESS_KEY_ID = "your_access_key"
    $env.AWS_SECRET_ACCESS_KEY = "your_secret_key"

    cargo run -- mirror `
        --sources "https://conda.anaconda.org/conda-forge/linux-64/zlib-1.2.13-hd590300_5.conda" `
        --target-type s3 `
        --target-path "s3://my-conda-bucket/packages" `
        --config examples/config.json
}

# Example 4: Mirror to MinIO (S3-compatible)
run_example "4: Mirror to MinIO" {
    # Set MinIO environment variables
    $env.AWS_ACCESS_KEY_ID = "minio_access_key"
    $env.AWS_SECRET_ACCESS_KEY = "minio_secret_key"

    cargo run -- mirror `
        --sources "https://conda.anaconda.org/conda-forge/linux-64/zlib-1.2.13-hd590300_5.conda" `
        --target-type s3 `
        --target-path "s3://conda-packages/linux-64" `
        --config examples/config-minio.json
}

# Example 5: Mirror to prefix.dev channel
run_example "5: Mirror to prefix.dev" {
    cargo run -- mirror `
        --sources "https://conda.anaconda.org/conda-forge/linux-64/zlib-1.2.13-hd590300_5.conda" `
        --target-type prefix-dev `
        --target-path "https://prefix.dev/channels/meso-forge"
}

# Example 6: Initialize a configuration file
run_example "6: Initialize configuration" {
    cargo run -- init-config -o my-config.json
}

# Example 7: Use comma-separated URLs
run_example "7: Comma-separated sources" {
    let sources = [
        "https://conda.anaconda.org/conda-forge/linux-64/zlib-1.2.13-hd590300_5.conda",
        "https://conda.anaconda.org/conda-forge/linux-64/bzip2-1.0.8-hd590300_5.conda"
    ] | str join ","

    cargo run -- mirror `
        --sources $sources `
        --target-type local `
        --target-path ./local-mirror
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
            --sources $url `
            --target-type local `
            --target-path ./advanced-mirror
    }
}

# Example 9: Test configuration and validate setup
run_example "9: Configuration validation" {
    # Generate a test config
    cargo run -- init-config -o test-config.json

    # Verify the config file was created and has expected content
    if ("test-config.json" | path exists) {
        let config = (open test-config.json)
        print "Configuration created successfully:"
        print ($config | to yaml)

        # Test with a small package
        cargo run -- mirror `
            --sources "https://conda.anaconda.org/conda-forge/noarch/wheel-0.42.0-pyhd8ed1ab_0.conda" `
            --target-type local `
            --target-path ./config-test `
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
                --sources $package.url `
                --target-type local `
                --target-path $target_dir

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

print "(ansi blue)All examples completed!(ansi reset)"
print "Check the created directories:"
print "  ./local-mirror"
print "  ./advanced-mirror"
print "  ./config-test"
print "  ./batch-mirror"
