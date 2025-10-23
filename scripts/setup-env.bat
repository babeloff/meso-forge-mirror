@echo off
REM Environment setup script for meso-forge-mirror development
REM This script is automatically sourced by pixi when activating the environment

echo Setting up meso-forge-mirror development environment...

REM Check if cargo is available
where cargo >nul 2>nul
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
if not exist "target" mkdir target
if not exist "examples" mkdir examples
if not exist "examples\output" mkdir examples\output
if not exist "test-data" mkdir test-data

REM Set up git hooks if in a git repository
if exist ".git" (
    REM Install pre-commit hooks
    where pre-commit >nul 2>nul
    if %errorlevel% equ 0 (
        pre-commit install --install-hooks
    )
)

REM Verify cargo configuration
if exist "Cargo.toml" (
    echo ✓ Cargo.toml found
) else (
    echo ⚠ Warning: Cargo.toml not found in current directory
)

REM Function to check dependency (implemented as subroutine)
goto :main

:check_dependency
set command=%~1
set purpose=%~2

where %command% >nul 2>nul
if %errorlevel% equ 0 (
    echo ✓ %command% available
) else (
    echo ⚠ %command% not found ^(required for: %purpose%^)
)
goto :eof

:main
echo Checking system dependencies:
call :check_dependency "pkg-config" "OpenSSL linking"
call :check_dependency "git" "version control"
call :check_dependency "curl" "HTTP requests testing"

REM Rust toolchain verification
where cargo >nul 2>nul
if %errorlevel% equ 0 (
    for /f "tokens=2" %%i in ('cargo --version') do set rust_version=%%i
    echo ✓ Rust toolchain: !rust_version!

    REM Check for required targets for cross-compilation
    where rustup >nul 2>nul
    if %errorlevel% equ 0 (
        rustup target list --installed | findstr "x86_64-unknown-linux-gnu" >nul
        if %errorlevel% equ 0 (
            echo ✓ Linux target available
        )

        rustup target list --installed | findstr "x86_64-apple-darwin" >nul
        if %errorlevel% equ 0 (
            echo ✓ macOS target available
        )
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
