#!/usr/bin/env nu
# Conda package operations script for meso-forge-mirror
# This script provides comprehensive conda package building, testing, and publishing operations

# Configuration record type
const default_config = {
    build_type: "release",
    platforms: ["linux-64", "osx-64", "osx-arm64", "win-64"],
    output_dir: "conda-packages",
    rattler_output_dir: "rattler-packages",
    recipe_dir: "conda-recipe",
    test_env_prefix: "test-meso-forge",
    channel: "meso-forge",
    verbose: false
}

# Logging functions with colors
def log_info [message: string] {
    print $"(ansi blue)[INFO](ansi reset) ($message)"
}

def log_success [message: string] {
    print $"(ansi green)[SUCCESS](ansi reset) ($message)"
}

def log_warning [message: string] {
    print $"(ansi yellow)[WARNING](ansi reset) ($message)"
}

def log_error [message: string] {
    print $"(ansi red)[ERROR](ansi reset) ($message)"
}

def log_step [step: string] {
    print $"(ansi cyan)[STEP](ansi reset) ($step)"
}

# Help function
def show_help [] {
    print "Usage: nu conda-ops.nu <command> [options]

Conda package operations for meso-forge-mirror

COMMANDS:
    build [platforms...]     Build conda packages (default: all platforms)
    build-rattler           Build with rattler-build
    test [platform]         Test conda package installation
    verify [package]        Verify conda package integrity
    publish [channel]       Publish to anaconda channel
    publish-prefix          Publish to prefix.dev
    clean                   Clean build artifacts
    list-packages          List built packages
    info [package]         Show package information
    help                   Show this help

OPTIONS:
    --verbose              Verbose output
    --debug                Build in debug mode
    --output DIR           Output directory
    --recipe DIR           Recipe directory

EXAMPLES:
    nu conda-ops.nu build                    # Build for all platforms
    nu conda-ops.nu build linux-64 osx-64   # Build specific platforms
    nu conda-ops.nu build-rattler           # Build with rattler-build
    nu conda-ops.nu test linux-64           # Test Linux package
    nu conda-ops.nu publish meso-forge      # Publish to channel
    nu conda-ops.nu clean                   # Clean artifacts"
}

# Check if required tools are available
def check_tools []: nothing -> record {
    {
        conda_build: (not (which conda-build | is-empty)),
        rattler_build: (not (which rattler-build | is-empty)),
        anaconda: (not (which anaconda | is-empty)),
        conda: (not (which conda | is-empty))
    }
}

# Validate environment and tools
def validate_environment []: nothing -> bool {
    let tools = (check_tools)

    if not $tools.conda {
        log_error "conda is not available. Please install conda/mamba."
        return false
    }

    if not ("Cargo.toml" | path exists) {
        log_error "Cargo.toml not found. Please run from project root."
        return false
    }

    if not ($default_config.recipe_dir | path exists) {
        log_error $"Recipe directory not found: ($default_config.recipe_dir)"
        return false
    }

    return true
}

# Build Rust binary for specific platform
def build_rust_binary [platform: string, build_type: string] {
    let rust_target = match $platform {
        "linux-64" => "x86_64-unknown-linux-gnu",
        "osx-64" => "x86_64-apple-darwin",
        "osx-arm64" => "aarch64-apple-darwin",
        "win-64" => "x86_64-pc-windows-gnu",
        _ => {
            log_error $"Unsupported platform: ($platform)"
            return false
        }
    }

    log_info $"Building Rust binary for ($platform) (target: ($rust_target))"

    # Install target if not available
    let installed_targets = (rustup target list --installed | lines)
    if not ($rust_target in $installed_targets) {
        log_info $"Installing target: ($rust_target)"
        rustup target add $rust_target
    }

    # Build command
    mut cargo_cmd = ["cargo", "build", "--target", $rust_target, "--locked"]

    if $build_type == "release" {
        $cargo_cmd = ($cargo_cmd | append "--release")
    }

    try {
        run-external ...$cargo_cmd
        return true
    } catch {
        log_error $"Failed to build Rust binary for ($platform)"
        return false
    }
}

# Build conda package for specific platform
def build_conda_package [platform: string, config: record] {
    log_step $"Building conda package for ($platform)"

    # Ensure output directory exists
    mkdir ($config.output_dir | path join $platform)

    # Build Rust binary first
    if not (build_rust_binary $platform $config.build_type) {
        return false
    }

    # Set up conda-build command
    mut conda_build_cmd = [
        "conda-build",
        $config.recipe_dir,
        "--output-folder", $config.output_dir,
        "--variants", $"{'target_platform': ['($platform)']}"
    ]

    if $config.verbose {
        $conda_build_cmd = ($conda_build_cmd | append "--verbose")
    }

    # Set environment variables for cross-compilation
    let rust_target = match $platform {
        "linux-64" => "x86_64-unknown-linux-gnu",
        "osx-64" => "x86_64-apple-darwin",
        "osx-arm64" => "aarch64-apple-darwin",
        "win-64" => "x86_64-pc-windows-gnu",
        _ => ""
    }

    let final_conda_build_cmd = $conda_build_cmd

    with-env {
        CARGO_BUILD_TARGET: $rust_target,
        CONDA_BUILD_CROSS_COMPILATION: "1"
    } {
        try {
            log_info $"Executing: ($final_conda_build_cmd | str join ' ')"
            run-external ...$final_conda_build_cmd
            log_success $"Successfully built conda package for ($platform)"
            return true
        } catch {
            log_error $"Failed to build conda package for ($platform)"
            return false
        }
    }
}

# Build with rattler-build
def build_rattler_package [platforms: list<string>, config: record] {
    log_step "Building packages with rattler-build"

    let tools = (check_tools)
    if not $tools.rattler_build {
        log_error "rattler-build is not available. Please install it."
        return false
    }

    # Ensure output directory exists
    mkdir $config.rattler_output_dir

    # Build for specified platforms
    let platform_string = ($platforms | str join ",")

    mut rattler_cmd = [
        "rattler-build", "build",
        "--recipe", ($config.recipe_dir | path join "recipe.yaml"),
        "--output-dir", $config.rattler_output_dir,
        "--target-platform", $platform_string
    ]

    if $config.verbose {
        $rattler_cmd = ($rattler_cmd | append "--verbose")
    }

    try {
        log_info $"Executing: ($rattler_cmd | str join ' ')"
        run-external ...$rattler_cmd
        log_success "Successfully built packages with rattler-build"
        return true
    } catch {
        log_error "Failed to build packages with rattler-build"
        return false
    }
}

# Test conda package installation
def test_conda_package [platform: string, config: record] {
    log_step $"Testing conda package for ($platform)"

    # Find the package file
    let package_pattern = $"($config.output_dir)/($platform)/meso-forge-mirror-*.conda"
    let package_files = (glob $package_pattern)

    if ($package_files | is-empty) {
        log_error $"No package files found for ($platform)"
        return false
    }

    let package_file = ($package_files | first)
    log_info $"Testing package: ($package_file)"

    # Create test environment
    let test_env = $"($config.test_env_prefix)-($platform)"

    try {
        # Remove existing test environment if it exists
        try {
            conda env remove -n $test_env -y
        } catch {
            # Ignore if environment doesn't exist
        }

        # Create new environment and install package
        conda create -n $test_env -y
        conda install -n $test_env $package_file -y

        # Test the binary
        conda run -n $test_env meso-forge-mirror --version
        conda run -n $test_env meso-forge-mirror --help
        conda run -n $test_env meso-forge-mirror init-config -o test-config.json

        # Cleanup test files
        rm -f test-config.json

        log_success $"Package test passed for ($platform)"
        return true
    } catch {
        log_error $"Package test failed for ($platform)"
        return false
    }
}

# Verify conda package integrity
def verify_conda_package [package_file: string] {
    log_step $"Verifying package: ($package_file)"

    if not ($package_file | path exists) {
        log_error $"Package file not found: ($package_file)"
        return false
    }

    try {
        conda verify $package_file
        log_success "Package verification passed"
        return true
    } catch {
        log_error "Package verification failed"
        return false
    }
}

# Publish to anaconda channel
def publish_to_anaconda [channel: string, config: record] {
    log_step $"Publishing to anaconda channel: ($channel)"

    let tools = (check_tools)
    if not $tools.anaconda {
        log_error "anaconda client is not available. Please install anaconda-client."
        return false
    }

    # Find all package files
    let package_files = (glob $"($config.output_dir)/**/*.conda")

    if ($package_files | is-empty) {
        log_error "No package files found to publish"
        return false
    }

    mut success_count = 0
    mut total_count = ($package_files | length)

    for package_file in $package_files {
        try {
            log_info $"Uploading: ($package_file)"
            anaconda upload -u $channel --force $package_file
            $success_count = $success_count + 1
            log_success $"Uploaded: ($package_file)"
        } catch {
            log_error $"Failed to upload: ($package_file)"
        }
    }

    log_info $"Upload summary: ($success_count)/($total_count) packages uploaded successfully"
    return ($success_count == $total_count)
}

# Publish to prefix.dev
def publish_to_prefix [channel: string, config: record]  {
    log_step $"Publishing to prefix.dev channel: ($channel)"

    let tools = (check_tools)
    if not $tools.rattler_build {
        log_error "rattler-build is not available. Please install it."
        return false
    }

    # Find all rattler package files
    let package_files = (glob $"($config.rattler_output_dir)/**/*.conda")

    if ($package_files | is-empty) {
        log_error "No rattler package files found to publish"
        return false
    }

    for package_file in $package_files {
        try {
            log_info $"Uploading to prefix.dev: ($package_file)"
            rattler-build upload prefix.dev --channel $channel $package_file
            log_success $"Uploaded to prefix.dev: ($package_file)"
        } catch {
            log_error $"Failed to upload to prefix.dev: ($package_file)"
        }
    }

    return true
}

# List built packages
def list_packages [config: record]: record -> table {
    let conda_packages = if ($config.output_dir | path exists) {
        glob $"($config.output_dir)/**/*.conda" | each { |file|
            let stat = (ls $file | first)
            {
                type: "conda-build",
                name: ($file | path basename),
                path: $file,
                size: $stat.size,
                modified: $stat.modified
            }
        }
    } else { [] }

    let rattler_packages = if ($config.rattler_output_dir | path exists) {
        glob $"($config.rattler_output_dir)/**/*.conda" | each { |file|
            let stat = (ls $file | first)
            {
                type: "rattler-build",
                name: ($file | path basename),
                path: $file,
                size: $stat.size,
                modified: $stat.modified
            }
        }
    } else { [] }

    $conda_packages | append $rattler_packages
}

# Show package information
def show_package_info [package_file: string] {
    if not ($package_file | path exists) {
        log_error $"Package file not found: ($package_file)"
        return
    }

    log_info $"Package information for: ($package_file)"

    try {
        # Extract package metadata
        conda info $package_file
    } catch {
        log_error "Failed to get package information"
    }
}

# Clean build artifacts
def clean_artifacts [config: record] {
    log_step "Cleaning build artifacts"

    if ($config.output_dir | path exists) {
        log_info $"Removing: ($config.output_dir)"
        rm -rf $config.output_dir
    }

    if ($config.rattler_output_dir | path exists) {
        log_info $"Removing: ($config.rattler_output_dir)"
        rm -rf $config.rattler_output_dir
    }

    # Clean Rust build artifacts
    try {
        cargo clean
        log_info "Cleaned cargo build artifacts"
    } catch {
        log_warning "Failed to clean cargo artifacts"
    }

    log_success "Cleanup completed"
}

# Main function
def main [...args] {
    if ($args | is-empty) {
        show_help
        return
    }

    let command = ($args | first)
    let remaining_args = ($args | skip 1)

    # Parse global options
    mut config = $default_config
    mut command_args = []
    mut i = 0

    while $i < ($remaining_args | length) {
        let arg = ($remaining_args | get $i)
        match $arg {
            "--verbose" => {
                $config.verbose = true
                $i = $i + 1
            },
            "--debug" => {
                $config.build_type = "debug"
                $i = $i + 1
            },
            "--output" => {
                if ($i + 1) < ($remaining_args | length) {
                    $config.output_dir = ($remaining_args | get ($i + 1))
                    $i = $i + 2
                } else {
                    log_error "Missing value for --output"
                    return
                }
            },
            "--recipe" => {
                if ($i + 1) < ($remaining_args | length) {
                    $config.recipe_dir = ($remaining_args | get ($i + 1))
                    $i = $i + 2
                } else {
                    log_error "Missing value for --recipe"
                    return
                }
            },
            _ => {
                $command_args = ($command_args | append $arg)
                $i = $i + 1
            }
        }
    }

    # Validate environment for most commands
    if $command not-in ["help", "clean"] {
        if not (validate_environment) {
            return
        }
    }

    # Execute command
    match $command {
        "build" => {
            let platforms = if ($command_args | is-empty) {
                $config.platforms
            } else {
                $command_args
            }

            mut success_count = 0
            for platform in $platforms {
                if (build_conda_package $platform $config) {
                    $success_count = $success_count + 1
                }
            }

            log_info $"Build summary: ($success_count)/($platforms | length) platforms built successfully"
        },

        "build-rattler" => {
            let platforms = if ($command_args | is-empty) {
                $config.platforms
            } else {
                $command_args
            }

            build_rattler_package $platforms $config
        },

        "test" => {
            let platform = if ($command_args | is-empty) {
                "linux-64"
            } else {
                ($command_args | first)
            }

            test_conda_package $platform $config
        },

        "verify" => {
            if ($command_args | is-empty) {
                log_error "Please specify a package file to verify"
                return
            }

            verify_conda_package ($command_args | first)
        },

        "publish" => {
            let channel = if ($command_args | is-empty) {
                $config.channel
            } else {
                ($command_args | first)
            }

            publish_to_anaconda $channel $config
        },

        "publish-prefix" => {
            let channel = if ($command_args | is-empty) {
                $config.channel
            } else {
                ($command_args | first)
            }

            publish_to_prefix $channel $config
        },

        "list-packages" => {
            list_packages $config | table
        },

        "info" => {
            if ($command_args | is-empty) {
                log_error "Please specify a package file"
                return
            }

            show_package_info ($command_args | first)
        },

        "clean" => {
            clean_artifacts $config
        },

        "help" => {
            show_help
        },

        _ => {
            log_error $"Unknown command: ($command)"
            show_help
        }
    }
}

# Run main function with all arguments
# main ...$args
