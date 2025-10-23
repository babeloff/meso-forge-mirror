#!/usr/bin/env nu
# Build script with proper conda environment setup for AWS SDK compatibility
# Supports multiple target platforms with interactive selection

# Available target platforms
const PLATFORMS = {
    "linux-64": {
        rust_target: "x86_64-unknown-linux-gnu",
        description: "Linux x86_64",
        binary_extension: ""
    },
    "osx-64": {
        rust_target: "x86_64-apple-darwin",
        description: "macOS Intel x86_64",
        binary_extension: ""
    },
    "osx-arm64": {
        rust_target: "aarch64-apple-darwin",
        description: "macOS Apple Silicon ARM64",
        binary_extension: ""
    },
    "win-64": {
        rust_target: "x86_64-pc-windows-gnu",
        description: "Windows x86_64",
        binary_extension: ".exe"
    },
    "current": {
        rust_target: "",
        description: "Current platform (native build)",
        binary_extension: ""
    },
    "debug": {
        rust_target: "",
        description: "Debug build for current platform",
        binary_extension: ""
    }
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

# Check if we're in a conda environment
def check_conda_environment [] {
    let conda_prefix = ($env.CONDA_PREFIX? | default "")
    if ($conda_prefix | is-empty) {
        log_error "CONDA_PREFIX not set. Please run this script from within a pixi/conda environment."
        exit 1
    }
    log_info $"Building meso-forge-mirror with conda environment: ($conda_prefix)"
}

# Set up conda environment variables
def setup_conda_environment [] {
    let conda_prefix = ($env.CONDA_PREFIX? | default "")

    if ($conda_prefix | is-empty) {
        log_error "CONDA_PREFIX not found in environment"
        return
    }

    # Set AWS SDK compatibility flags
    $env.AWS_LC_SYS_NO_ASM = "1"
    $env.AWS_LC_SYS_CMAKE_BUILDER = "0"
    $env.OPENSSL_NO_VENDOR = "1"

    # Set OpenSSL paths for conda environment
    $env.OPENSSL_DIR = $conda_prefix
    $env.OPENSSL_LIB_DIR = $"($conda_prefix)/lib"
    $env.OPENSSL_INCLUDE_DIR = $"($conda_prefix)/include"

    # Update PKG_CONFIG_PATH to include conda libraries
    $env.PKG_CONFIG_PATH = $"($conda_prefix)/lib/pkgconfig:($env.PKG_CONFIG_PATH? | default '')"

    # Set Rust flags for proper linking
    $env.RUSTFLAGS = $"-L ($conda_prefix)/lib -C link-arg=-Wl,-rpath,($conda_prefix)/lib"

    # Set compiler to use conda's gcc if available
    let gcc_path = $"($conda_prefix)/bin/gcc"
    let gxx_path = $"($conda_prefix)/bin/g++"

    if ($gcc_path | path exists) {
        $env.CC = $gcc_path
        $env.CXX = $gxx_path
        log_info $"Using conda GCC: ($gcc_path)"
    }
}

# Verify OpenSSL is available
def verify_openssl [] {
    let openssl_lib = ($env.OPENSSL_LIB_DIR? | default "")

    if ($openssl_lib | is-empty) {
        log_warning "OPENSSL_LIB_DIR not set"
        return
    }

    let ssl_so = $"($openssl_lib)/libssl.so"
    let ssl_dylib = $"($openssl_lib)/libssl.dylib"

    if not (($ssl_so | path exists) or ($ssl_dylib | path exists)) {
        log_warning $"OpenSSL libraries not found in ($openssl_lib)"
        log_info "Attempting to use system OpenSSL..."

        try {
            ^pkg-config --exists openssl
        } catch {
            log_error "OpenSSL not found. Please ensure OpenSSL is installed in the conda environment."
            exit 1
        }
    }
}

# Display build configuration
def display_build_config [] {
    log_info "Build configuration:"
    print $"  AWS_LC_SYS_NO_ASM: ($env.AWS_LC_SYS_NO_ASM? | default 'not set')"
    print $"  OPENSSL_DIR: ($env.OPENSSL_DIR? | default 'not set')"
    print $"  OPENSSL_LIB_DIR: ($env.OPENSSL_LIB_DIR? | default 'not set')"
    print $"  CC: ($env.CC? | default 'system default')"
    print $"  RUSTFLAGS: ($env.RUSTFLAGS? | default 'not set')"
}

# Show available platforms
def show_platforms [] {
    print "Available target platforms:"
    for platform in ($PLATFORMS | columns | sort) {
        let info = ($PLATFORMS | get $platform)
        print $"  ($platform) - ($info.description)"
    }
}

# Prompt user to select a platform
def prompt_platform_selection []: nothing -> string {
    print ""
    show_platforms
    print ""

    mut selection = ""
    while $selection not-in ($PLATFORMS | columns) {
        $selection = (input "Please select a target platform: ")

        if $selection not-in ($PLATFORMS | columns) {
            log_error $"Invalid selection: ($selection)"
            print "Please choose from the available options."
        }
    }
    $selection
}

# Install Rust target if needed
def install_rust_target [rust_target: string] {
    if ($rust_target | is-empty) {
        return
    }

    log_info $"Checking if Rust target ($rust_target) is installed..."

    let installed_targets = (^rustup target list --installed | lines)

    if not ($rust_target in $installed_targets) {
        log_info $"Installing Rust target: ($rust_target)"
        ^rustup target add $rust_target
        log_success $"Installed target: ($rust_target)"
    } else {
        log_success $"Target ($rust_target) is already installed"
    }
}

# Build the project
def build_project [platform: string] {
    let platform_info = ($PLATFORMS | get $platform)
    let rust_target = $platform_info.rust_target
    let description = $platform_info.description
    let extension = $platform_info.binary_extension

    log_info $"Building for platform: ($platform) - ($description)"

    # Install Rust target if needed
    if not ($rust_target | is-empty) {
        install_rust_target $rust_target
    }

    # Build cargo command
    let cargo_cmd = ["cargo", "build"]
        | if $platform == "debug" {
            log_info "Building in debug mode..."
            $in
        } else {
            log_info "Building in release mode..."
            $in | append "--release"
        }
        | if not ($rust_target | is-empty) {
            log_info $"Cross-compiling for target: ($rust_target)"
            $in | append ["--target", $rust_target]
        } else {
            $in
        }
        | append "--locked"
    # Execute build
    log_info $"Executing: ($cargo_cmd | str join ' ')"

    let cargo_result = (run-external ...$cargo_cmd | complete)
    if ($cargo_result.exit_code != 0) {
        log_error $"Build failed for platform: ($platform)"
    }
    log_success "Build completed successfully!"

    # Determine binary path
    let binary_name = $"meso-forge-mirror($extension)"
    let binary_path = if $platform == "debug" {
        $"target/debug/($binary_name)"
    } else if ($rust_target | is-empty) {
        $"target/release/($binary_name)"
    } else {
        $"target/($rust_target)/release/($binary_name)"
    }

    # Verify and report binary
    if ($binary_path | path exists) {
        log_success $"Binary available at: ($binary_path)"
        let size_info = (ls $binary_path | get 0 | get size)
        print $"Size: ($size_info)"

        # Test binary if it's for current platform
        if ($platform == "current") or ($platform == "debug") or ($nu.os-info.name == "linux" and $platform == "linux-64") {
            log_info "Testing binary..."
            try {
                ^($binary_path) --version | ignore
                log_success "Binary test passed"
            } catch {
                log_warning "Binary test failed (may be due to cross-compilation)"
            }
        }
    } else {
        log_warning $"Binary not found at expected location: ($binary_path)"
    }
}

# Show help message
def show_help [] {
    print "Usage: nu build.nu [TARGET_PLATFORM]

Build script with proper conda environment setup for AWS SDK compatibility.

ARGUMENTS:
    TARGET_PLATFORM    Target platform to build for (optional)

If no target platform is provided, you will be prompted to select one.

EXAMPLES:
    nu build.nu linux-64      # Build for Linux x86_64
    nu build.nu osx-arm64     # Build for macOS Apple Silicon
    nu build.nu debug         # Debug build for current platform
    nu build.nu current       # Release build for current platform
    nu build.nu               # Interactive platform selection

SUPPORTED PLATFORMS:"

    for platform in ($PLATFORMS | columns | sort) {
        let info = ($PLATFORMS | get $platform)
        print $"    ($platform) - ($info.description)"
    }
}

# Entry point - handle command line arguments
def main [...args] {
    let help_requested = ($args | any {|arg| $arg == "--help" or $arg == "-h"})

    if $help_requested {
        show_help
        return
    }

    # Ensure we're in the project root
    if not ("Cargo.toml" | path exists) {
        log_error "Cargo.toml not found. Please run this script from the project root."
        exit 1
    }

    # Check conda environment
    check_conda_environment

    # Set up conda environment variables
    setup_conda_environment

    # Verify OpenSSL availability
    verify_openssl

    # Display build configuration
    display_build_config

    # Determine target platform
    let platform = if ($args | is-empty) {
        prompt_platform_selection
    } else {
        let target_platform = ($args | get 0)
        if ($target_platform in ($PLATFORMS | columns)) {
            $target_platform
        } else {
            log_error $"Invalid target platform: ($target_platform)"
            show_platforms
            exit 1
        }
    }

    print ""
    log_info $"Selected platform: ($platform)"
    print ""

    # Build the project
    build_project $platform

    log_success "Build process completed successfully!"
}
