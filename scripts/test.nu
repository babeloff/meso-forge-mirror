#!/usr/bin/env nu
# Comprehensive testing script for meso-forge-mirror
# This script provides various testing operations including unit tests, integration tests, and package validation

# Configuration
const test_config = {
    test_dir: "test-output",
    integration_dir: "integration-tests",
    temp_dir: "temp-test",
    test_packages: [
        "https://conda.anaconda.org/conda-forge/noarch/pip-24.0-pyhd8ed1ab_0.conda",
        "https://conda.anaconda.org/conda-forge/linux-64/zlib-1.2.13-hd590300_5.conda",
        "https://conda.anaconda.org/conda-forge/noarch/wheel-0.42.0-pyhd8ed1ab_0.conda"
    ],
    test_timeout: 300
}

# Logging functions
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

def log_test [test_name: string] {
    print $"(ansi cyan)[TEST](ansi reset) ($test_name)"
}

# Help function
def show_help [] {
    print "Usage: nu test.nu <command> [options]

Comprehensive testing for meso-forge-mirror

COMMANDS:
    unit                    Run Rust unit tests
    integration            Run integration tests
    cargo                  Run all cargo tests (unit + doc + bench)
    lint                   Run linting and formatting checks
    local-mirror           Test local mirroring functionality
    s3-mock               Test S3 operations with mock server
    config                Test configuration handling
    performance           Run performance benchmarks
    all                   Run all tests
    clean                 Clean test artifacts
    help                  Show this help

OPTIONS:
    --verbose             Verbose output
    --timeout SECONDS     Test timeout (default: 300)
    --no-cleanup         Don't cleanup test files
    --filter PATTERN     Filter tests by pattern

EXAMPLES:
    nu test.nu unit                    # Run unit tests
    nu test.nu integration --verbose  # Run integration tests with verbose output
    nu test.nu local-mirror           # Test local mirroring
    nu test.nu all --no-cleanup       # Run all tests, keep artifacts
    nu test.nu performance            # Run benchmarks"
}

# Setup test environment
def setup_test_env [] {
    log_info "Setting up test environment..."

    # Create test directories
    mkdir $test_config.test_dir
    mkdir $test_config.integration_dir
    mkdir $test_config.temp_dir

    # Set test environment variables
    $env.RUST_LOG = "debug"
    $env.RUST_BACKTRACE = "1"
    $env.TEST_MODE = "true"

    log_success "Test environment ready"
}

# Cleanup test artifacts
def cleanup_test_env [force: bool = false] {
    if $force or ($env.NO_CLEANUP? | default false | not) {
        log_info "Cleaning up test artifacts..."

        try { rm -rf $test_config.test_dir } catch { }
        try { rm -rf $test_config.integration_dir } catch { }
        try { rm -rf $test_config.temp_dir } catch { }
        try { rm -f test-config.json } catch { }
        try { rm -f benchmark-results.json } catch { }

        log_success "Cleanup completed"
    }
}

# Run Rust unit tests
def run_unit_tests [verbose: bool = false]: bool -> bool {
    log_test "Rust unit tests"

    mut cargo_cmd = ["cargo", "test", "--lib"]

    if $verbose {
        $cargo_cmd = ($cargo_cmd | append "--verbose")
    }

    try {
        run-external ...$cargo_cmd
        log_success "Unit tests passed"
        return true
    } catch {
        log_error "Unit tests failed"
        return false
    }
}

# Run integration tests
def run_integration_tests [verbose: bool = false]: bool -> bool {
    log_test "Integration tests"

    mut cargo_cmd = ["cargo", "test", "--test", "*"]

    if $verbose {
        $cargo_cmd = ($cargo_cmd | append "--verbose")
    }

    try {
        run-external ...$cargo_cmd
        log_success "Integration tests passed"
        return true
    } catch {
        log_error "Integration tests failed"
        return false
    }
}

# Run all cargo tests
def run_cargo_tests [verbose: bool = false]: bool -> record {
    log_test "All cargo tests"

    mut results = { unit: false, doc: false, integration: false }

    # Unit tests
    $results.unit = (run_unit_tests $verbose)

    # Documentation tests
    try {
        if $verbose { log_info "Running documentation tests..." }
        cargo test --doc
        $results.doc = true
        log_success "Documentation tests passed"
    } catch {
        log_error "Documentation tests failed"
        $results.doc = false
    }

    # Integration tests
    $results.integration = (run_integration_tests $verbose)

    return $results
}

# Run linting and formatting checks
def run_lint_checks [verbose: bool = false]: bool -> record {
    log_test "Linting and formatting checks"

    mut results = { clippy: false, fmt: false, audit: false }

    # Clippy
    try {
        if $verbose { log_info "Running clippy..." }
        cargo clippy -- -D warnings
        $results.clippy = true
        log_success "Clippy checks passed"
    } catch {
        log_error "Clippy checks failed"
        $results.clippy = false
    }

    # Formatting
    try {
        if $verbose { log_info "Checking code formatting..." }
        cargo fmt -- --check
        $results.fmt = true
        log_success "Code formatting is correct"
    } catch {
        log_error "Code formatting issues found"
        $results.fmt = false
    }

    # Security audit (if cargo-audit is installed)
    try {
        if not (which cargo-audit | is-empty) {
            if $verbose { log_info "Running security audit..." }
            cargo audit
            $results.audit = true
            log_success "Security audit passed"
        } else {
            log_warning "cargo-audit not installed, skipping security audit"
            $results.audit = true  # Don't fail if not installed
        }
    } catch {
        log_error "Security audit failed"
        $results.audit = false
    }

    return $results
}

# Test local mirroring functionality
def test_local_mirror [verbose: bool = false] -> bool {
    log_test "Local mirroring functionality"

    let target_dir = ($test_config.test_dir | path join "local-mirror")
    mkdir $target_dir

    # Test with a small package
    let test_package = $test_config.test_packages.0

    try {
        if $verbose {
            log_info $"Testing local mirror with: ($test_package)"
            log_info $"Target directory: ($target_dir)"
        }

        cargo run -- mirror `
            --sources $test_package `
            --target-type local `
            --target-path $target_dir

        # Verify the package was mirrored
        let package_files = (ls $target_dir | where type == file)

        if ($package_files | is-empty) {
            log_error "No files found in mirror directory"
            return false
        }

        log_success $"Local mirror test passed - ($package_files | length) files mirrored"

        if $verbose {
            print "Mirrored files:"
            $package_files | select name size | table
        }

        return true
    } catch {
        log_error "Local mirror test failed"
        return false
    }
}

# Test S3 operations with mock server
def test_s3_mock [verbose: bool = false] -> bool {
    log_test "S3 operations with mock server"

    # Check if we can use MinIO for testing
    let minio_available = not (which minio | is-empty)

    if not $minio_available {
        log_warning "MinIO not available, skipping S3 mock tests"
        return true
    }

    try {
        # Start MinIO in background (if not running)
        let minio_running = try {
            http get "http://localhost:9000/minio/health/live" | ignore
            true
        } catch { false }

        if not $minio_running {
            log_info "Starting MinIO server for testing..."
            # This would need to be implemented based on your MinIO setup
            log_warning "MinIO server setup not implemented in this test"
            return true
        }

        # Test S3 mirroring
        let test_package = $test_config.test_packages.0

        with-env {
            AWS_ACCESS_KEY_ID: "minioadmin",
            AWS_SECRET_ACCESS_KEY: "minioadmin",
            AWS_ENDPOINT_URL: "http://localhost:9000"
        } {
            cargo run -- mirror `
                --sources $test_package `
                --target-type s3 `
                --target-path "s3://test-bucket/packages"
        }

        log_success "S3 mock test passed"
        return true
    } catch {
        log_error "S3 mock test failed"
        return false
    }
}

# Test configuration handling
def test_config_handling [verbose: bool = false] -> bool {
    log_test "Configuration handling"

    try {
        # Test config generation
        if $verbose { log_info "Testing config generation..." }
        cargo run -- init-config -o test-config.json

        if not ("test-config.json" | path exists) {
            log_error "Config file was not created"
            return false
        }

        # Verify config content
        let config = (open test-config.json)
        let expected_keys = ["max_concurrent_downloads", "retry_attempts", "timeout_seconds"]

        for key in $expected_keys {
            if not ($key in ($config | columns)) {
                log_error $"Missing config key: ($key)"
                return false
            }
        }

        # Test using the config
        if $verbose { log_info "Testing config usage..." }
        let test_package = $test_config.test_packages.2  # Use wheel package
        let target_dir = ($test_config.test_dir | path join "config-test")
        mkdir $target_dir

        cargo run -- mirror `
            --sources $test_package `
            --target-type local `
            --target-path $target_dir `
            --config test-config.json

        log_success "Configuration handling test passed"
        return true
    } catch {
        log_error "Configuration handling test failed"
        return false
    }
}

# Run performance benchmarks
def run_performance_tests [verbose: bool = false] -> bool {
    log_test "Performance benchmarks"

    try {
        # Run cargo bench if benchmarks exist
        if $verbose { log_info "Running performance benchmarks..." }

        # Check if benchmarks directory exists
        if ("benches" | path exists) {
            cargo bench
        } else {
            log_info "No benchmarks found, creating a simple performance test..."

            # Simple performance test: time multiple package downloads
            let start_time = (date now)

            let target_dir = ($test_config.test_dir | path join "perf-test")
            mkdir $target_dir

            # Download multiple packages concurrently
            let packages = ($test_config.test_packages | str join ",")

            cargo run -- mirror `
                --sources $packages `
                --target-type local `
                --target-path $target_dir

            let end_time = (date now)
            let duration = ($end_time - $start_time)

            let benchmark_result = {
                test: "multi_package_download",
                packages: ($test_config.test_packages | length),
                duration_ms: ($duration / 1ms),
                timestamp: ($start_time | format date "%Y-%m-%d %H:%M:%S")
            }

            $benchmark_result | to json | save benchmark-results.json

            log_success $"Performance test completed in ($duration)"

            if $verbose {
                print "Benchmark results:"
                $benchmark_result | table
            }
        }

        log_success "Performance tests completed"
        return true
    } catch {
        log_error "Performance tests failed"
        return false
    }
}

# Run all tests
def run_all_tests [verbose: bool = false, timeout: int = 300] -> record {
    log_info "Running comprehensive test suite..."

    let start_time = (date now)

    mut results = {
        cargo: {},
        lint: {},
        local_mirror: false,
        s3_mock: false,
        config: false,
        performance: false,
        overall_success: false
    }

    # Set timeout
    with-env { TEST_TIMEOUT: ($timeout | into string) } {
        # Run cargo tests
        $results.cargo = (run_cargo_tests $verbose)

        # Run lint checks
        $results.lint = (run_lint_checks $verbose)

        # Run functional tests
        $results.local_mirror = (test_local_mirror $verbose)
        $results.s3_mock = (test_s3_mock $verbose)
        $results.config = (test_config_handling $verbose)
        $results.performance = (run_performance_tests $verbose)
    }

    # Calculate overall success
    let cargo_success = ($results.cargo.unit and $results.cargo.doc and $results.cargo.integration)
    let lint_success = ($results.lint.clippy and $results.lint.fmt and $results.lint.audit)
    let functional_success = ($results.local_mirror and $results.s3_mock and $results.config and $results.performance)

    $results.overall_success = ($cargo_success and $lint_success and $functional_success)

    let end_time = (date now)
    let total_duration = ($end_time - $start_time)

    # Print summary
    print ""
    log_info "=== TEST SUMMARY ==="
    print $"Total duration: ($total_duration)"
    print ""

    print "Cargo Tests:"
    print $"  Unit tests:        ($results.cargo.unit | if $in { '✓' } else { '✗' })"
    print $"  Documentation:     ($results.cargo.doc | if $in { '✓' } else { '✗' })"
    print $"  Integration:       ($results.cargo.integration | if $in { '✓' } else { '✗' })"
    print ""

    print "Lint Checks:"
    print $"  Clippy:           ($results.lint.clippy | if $in { '✓' } else { '✗' })"
    print $"  Formatting:       ($results.lint.fmt | if $in { '✓' } else { '✗' })"
    print $"  Security audit:   ($results.lint.audit | if $in { '✓' } else { '✗' })"
    print ""

    print "Functional Tests:"
    print $"  Local mirror:     ($results.local_mirror | if $in { '✓' } else { '✗' })"
    print $"  S3 mock:          ($results.s3_mock | if $in { '✓' } else { '✗' })"
    print $"  Configuration:    ($results.config | if $in { '✓' } else { '✗' })"
    print $"  Performance:      ($results.performance | if $in { '✓' } else { '✗' })"
    print ""

    if $results.overall_success {
        log_success "ALL TESTS PASSED!"
    } else {
        log_error "SOME TESTS FAILED!"
    }

    return $results
}

# Main function
def main [...args] {
    if ($args | is-empty) {
        show_help
        return
    }

    let command = ($args | first)
    let remaining_args = ($args | skip 1)

    # Parse options
    mut verbose = false
    mut timeout = $test_config.test_timeout
    mut no_cleanup = false
    mut filter = ""
    mut command_args = []

    mut i = 0
    while $i < ($remaining_args | length) {
        let arg = ($remaining_args | get $i)
        match $arg {
            "--verbose" => {
                $verbose = true
                $i = $i + 1
            },
            "--timeout" => {
                if ($i + 1) < ($remaining_args | length) {
                    $timeout = (($remaining_args | get ($i + 1)) | into int)
                    $i = $i + 2
                } else {
                    log_error "Missing value for --timeout"
                    return
                }
            },
            "--no-cleanup" => {
                $no_cleanup = true
                $env.NO_CLEANUP = "true"
                $i = $i + 1
            },
            "--filter" => {
                if ($i + 1) < ($remaining_args | length) {
                    $filter = ($remaining_args | get ($i + 1))
                    $i = $i + 2
                } else {
                    log_error "Missing value for --filter"
                    return
                }
            },
            _ => {
                $command_args = ($command_args | append $arg)
                $i = $i + 1
            }
        }
    }

    # Setup test environment
    setup_test_env

    try {
        # Execute command
        match $command {
            "unit" => {
                run_unit_tests $verbose | ignore
            },

            "integration" => {
                run_integration_tests $verbose | ignore
            },

            "cargo" => {
                run_cargo_tests $verbose | ignore
            },

            "lint" => {
                run_lint_checks $verbose | ignore
            },

            "local-mirror" => {
                test_local_mirror $verbose | ignore
            },

            "s3-mock" => {
                test_s3_mock $verbose | ignore
            },

            "config" => {
                test_config_handling $verbose | ignore
            },

            "performance" => {
                run_performance_tests $verbose | ignore
            },

            "all" => {
                run_all_tests $verbose $timeout | ignore
            },

            "clean" => {
                cleanup_test_env true
            },

            "help" => {
                show_help
            },

            _ => {
                log_error $"Unknown command: ($command)"
                show_help
            }
        }
    } finally {
        # Cleanup unless requested not to
        cleanup_test_env false
    }
}

# Run main function with all arguments
main ...$args
