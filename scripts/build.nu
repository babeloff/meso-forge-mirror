#!/usr/bin/env nu
# Cross-platform build script for meso-forge-mirror conda packaging
# This script handles building the Rust binary for different target platforms

# Default values
let default_config = {
    target_platform: "",
    build_type: "release",
    output_dir: "target",
    verbose: false,
    clean_build: false
}

# Color codes for output (using nushell's built-in color support)
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

# Help function
def show_help [] {
    print "Usage: nu build.nu [OPTIONS]

Cross-platform build script for meso-forge-mirror

OPTIONS:
    --target PLATFORM    Target platform (linux-64, osx-64, osx-arm64, win-64, all)
    --debug              Build in debug mode (default: release)
    --output DIR         Output directory (default: target)
    --verbose            Verbose output
    --clean              Clean previous builds
    --help               Show this help message

PLATFORMS:
    linux-64             x86_64-unknown-linux-gnu
    osx-64               x86_64-apple-darwin
    osx-arm64            aarch64-apple-darwin
    win-64               x86_64-pc-windows-gnu
    all                  Build for all platforms

EXAMPLES:
    nu build.nu --target linux-64
    nu build.nu --target osx-arm64 --debug
    nu build.nu --target win-64 --output ./dist
    nu build.nu --target all --verbose"
}

# Map conda platforms to Rust targets
def get_rust_target [platform: string]: string -> string {
    match $platform {
        "linux-64" => "x86_64-unknown-linux-gnu",
        "osx-64" => "x86_64-apple-darwin",
        "osx-arm64" => "aarch64-apple-darwin",
        "win-64" => "x86_64-pc-windows-gnu",
        _ => {
            log_error $"Unsupported target platform: ($platform)"
            exit 1
        }
    }
}

# Get binary extension for platform
def get_binary_extension [platform: string]: string -> string {
    match $platform {
        "win-64" => ".exe",
        _ => ""
    }
}

# Install Rust target if not available
def install_target [rust_target: string] {
    log_info $"Checking if target ($rust_target) is installed..."

    let installed_targets = (rustup target list --installed | lines)

    if not ($rust_target in $installed_targets) {
        log_info $"Installing target: ($rust_target)"
        rustup target add $rust_target
    } else {
        log_success $"Target ($rust_target) is already installed"
    }
}

# Build for specific target
def build_target [platform: string, rust_target: string, extension: string, config: record] {
    log_info $"Building for platform: ($platform) (target: ($rust_target))"

    # Install target if needed
    install_target $rust_target

    # Build cargo command
    mut cargo_cmd = ["cargo", "build", "--target", $rust_target]

    if $config.build_type == "release" {
        $cargo_cmd = ($cargo_cmd | append "--release")
    }

    if $config.verbose {
        $cargo_cmd = ($cargo_cmd | append "--verbose")
    }

    # Add locked flag to use exact dependencies from Cargo.lock
    $cargo_cmd = ($cargo_cmd | append "--locked")

    # Execute build
    log_info $"Executing: ($cargo_cmd | str join ' ')"

    try {
        run-external ($cargo_cmd | first) ...($cargo_cmd | skip 1)
    } catch {
        log_error $"Build failed for ($platform)"
        return false
    }

    # Verify binary was created
    let binary_path = $"target/($rust_target)/($config.build_type)/meso-forge-mirror($extension)"

    if ($binary_path | path exists) {
        log_success $"Binary built successfully: ($binary_path)"

        # Show binary info
        ls $binary_path | select name size modified

        # Test that binary works (only for compatible platforms)
        if ($platform == "linux-64") or ($nu.os-info.name == "macos" and ($platform == "osx-64" or $platform == "osx-arm64")) {
            log_info "Testing binary..."
            try {
                run-external $binary_path "--version" | ignore
                log_success "Binary test passed"
            } catch {
                log_warning "Binary test failed (may be due to cross-compilation)"
            }
        }

        return true
    } else {
        log_error $"Binary not found at expected path: ($binary_path)"
        return false
    }
}

# Build all targets
def build_all_targets [config: record]: record -> bool {
    log_info "Building for all supported platforms..."

    let platforms = ["linux-64", "osx-64", "osx-arm64", "win-64"]
    mut failed_builds = []

    for platform in $platforms {
        let rust_target = (get_rust_target $platform)
        let extension = (get_binary_extension $platform)

        log_info $"Starting build for ($platform)..."

        if (build_target $platform $rust_target $extension $config) {
            log_success $"Completed build for ($platform)"
        } else {
            log_error $"Failed to build for ($platform)"
            $failed_builds = ($failed_builds | append $platform)
        }
        print ""
    }

    # Report results
    if ($failed_builds | is-empty) {
        log_success "All builds completed successfully!"
        return true
    } else {
        log_error $"Failed builds: ($failed_builds | str join ', ')"
        return false
    }
}

# Build for current platform
def build_current_platform [config: record]: record -> bool {
    log_info "No target specified, building for current platform..."

    mut cargo_cmd = ["cargo", "build"]

    if $config.build_type == "release" {
        $cargo_cmd = ($cargo_cmd | append "--release")
    }

    if $config.verbose {
        $cargo_cmd = ($cargo_cmd | append "--verbose")
    }

    $cargo_cmd = ($cargo_cmd | append "--locked")

    log_info $"Executing: ($cargo_cmd | str join ' ')"

    try {
        run-external ($cargo_cmd | first) ...($cargo_cmd | skip 1)
    } catch {
        log_error "Build failed"
        return false
    }

    # Find and report the binary
    let binary_name = if $nu.os-info.name == "windows" { "meso-forge-mirror.exe" } else { "meso-forge-mirror" }
    let binary_path = $"target/($config.build_type)/($binary_name)"

    if ($binary_path | path exists) {
        log_success $"Binary built: ($binary_path)"
        ls $binary_path | select name size modified
        return true
    } else {
        log_error $"Binary not found at expected path: ($binary_path)"
        return false
    }
}

# Main function
def main [...args] {
    # Parse arguments
    mut config = $default_config
    mut i = 0

    while $i < ($args | length) {
        let arg = ($args | get $i)
        match $arg {
            "--target" | "-t" => {
                if ($i + 1) < ($args | length) {
                    $config.target_platform = ($args | get ($i + 1))
                    $i = $i + 2
                } else {
                    log_error "Missing value for --target"
                    exit 1
                }
            },
            "--debug" | "-d" => {
                $config.build_type = "debug"
                $i = $i + 1
            },
            "--output" | "-o" => {
                if ($i + 1) < ($args | length) {
                    $config.output_dir = ($args | get ($i + 1))
                    $i = $i + 2
                } else {
                    log_error "Missing value for --output"
                    exit 1
                }
            },
            "--verbose" | "-v" => {
                $config.verbose = true
                $i = $i + 1
            },
            "--clean" => {
                $config.clean_build = true
                $i = $i + 1
            },
            "--help" | "-h" => {
                show_help
                exit 0
            },
            _ => {
                log_error $"Unknown option: ($arg)"
                show_help
                exit 1
            }
        }
    }

    log_info "Starting meso-forge-mirror build process..."
    log_info $"Build type: ($config.build_type)"
    log_info $"Output directory: ($config.output_dir)"

    # Ensure we're in the right directory
    if not ("Cargo.toml" | path exists) {
        log_error "Cargo.toml not found. Please run this script from the project root."
        exit 1
    }

    # Clean previous builds if requested
    if $config.clean_build {
        log_info "Cleaning previous builds..."
        cargo clean
    }

    # Build based on target selection
    let success = if ($config.target_platform | is-empty) {
        # No target specified, build for current platform
        build_current_platform $config
    } else if $config.target_platform == "all" {
        # Build for all targets
        build_all_targets $config
    } else {
        # Build for specific target
        let rust_target = (get_rust_target $config.target_platform)
        let extension = (get_binary_extension $config.target_platform)
        build_target $config.target_platform $rust_target $extension $config
    }

    if $success {
        log_success "Build process completed!"
        exit 0
    } else {
        log_error "Build process failed!"
        exit 1
    }
}

# Run main function with all arguments
# main ...$args
