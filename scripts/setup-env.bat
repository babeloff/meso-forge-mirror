@echo off
REM Environment setup script for meso-forge-mirror development on Windows
REM This script is automatically sourced by pixi when activating the environment

echo Setting up meso-forge-mirror development environment for Windows...

REM Check if cargo is available
where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo Warning: cargo not found in PATH
)

REM Set environment variables for development
if not defined RUST_LOG set RUST_LOG=info
if not defined RUST_BACKTRACE set RUST_BACKTRACE=1

REM AWS/S3 configuration for local testing (MinIO)
if not defined AWS_ENDPOINT_URL set AWS_ENDPOINT_URL=http://localhost:9000
if not defined AWS_ACCESS_KEY_ID set AWS_ACCESS_KEY_ID=minioadmin
if not defined AWS_SECRET_ACCESS_KEY set AWS_SECRET_ACCESS_KEY=minioadmin

REM Create necessary directories
if not exist target mkdir target
if not exist examples\output mkdir examples\output
if not exist test-data mkdir test-data

REM Verify cargo configuration
if exist Cargo.toml (
    echo ✓ Cargo.toml found
) else (
    echo ⚠ Warning: Cargo.toml not found in current directory
)

REM Check for required system dependencies
echo Checking system dependencies:

where git >nul 2>&1
if %errorlevel% equ 0 (
    echo ✓ git available
) else (
    echo ⚠ git not found ^(required for: version control^)
)

where curl >nul 2>&1
if %errorlevel% equ 0 (
    echo ✓ curl available
) else (
    echo ⚠ curl not found ^(required for: HTTP requests testing^)
)

where pkg-config >nul 2>&1
if %errorlevel% equ 0 (
    echo ✓ pkg-config available
) else (
    echo ⚠ pkg-config not found ^(may be needed for some dependencies^)
)

REM Rust toolchain verification
where cargo >nul 2>&1
if %errorlevel% equ 0 (
    for /f "tokens=2" %%i in ('cargo --version') do set RUST_VERSION=%%i
    echo ✓ Rust toolchain: !RUST_VERSION!

    REM Check for required targets for cross-compilation
    cargo target list | findstr "x86_64-pc-windows-gnu" >nul
    if %errorlevel% equ 0 (
        echo ✓ Windows GNU target available
    )

    cargo target list | findstr "x86_64-pc-windows-msvc" >nul
    if %errorlevel% equ 0 (
        echo ✓ Windows MSVC target available
    )
)

REM Setup completion
echo.
echo Environment setup complete!
echo.
echo Available pixi tasks:
echo   pixi run build          - Build the project
echo   pixi run test           - Run tests
echo   pixi run dev-setup      - Complete development setup
echo   pixi run conda-build    - Build conda package
echo   pixi run demo-local     - Run local demo
echo.
echo For a full list of tasks, run: pixi task list
