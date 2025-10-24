//! Test script to debug platform detection issues
//!
//! This example demonstrates how to properly extract platform information
//! from conda packages using rattler tools.

use anyhow::Result;
use bytes::Bytes;
use rattler_conda_types::Platform;
use rattler_package_streaming::fs::extract_conda;
use std::fs;
use std::io::Cursor;

#[tokio::main]
async fn main() -> Result<()> {
    // Test packages from the user's cache
    let test_packages = vec![
        "/var/home/phreed/.cache/rattler/cache/coreos-installer-0.25.0-he48fb7a_0.conda",
        "/var/home/phreed/.cache/rattler/cache/okd-install-4.19.15-h2b58dbe_0.conda",
        "/var/home/phreed/.cache/rattler/cache/rb-asciidoctor-revealjs-5.2.0-h1d6dcf3_0.conda",
    ];

    for package_path in test_packages {
        println!("\n=== Testing package: {} ===", package_path);

        if !std::path::Path::new(package_path).exists() {
            println!("Package not found, skipping");
            continue;
        }

        // Read the package file
        let content = fs::read(package_path)?;
        let bytes = Bytes::from(content);

        // Test platform detection
        match extract_platform_info(&bytes, package_path).await {
            Ok((platform, subdir, arch)) => {
                println!("✅ Successfully extracted metadata:");
                println!("   Platform: {:?}", platform);
                println!("   Subdir: {:?}", subdir);
                println!("   Arch: {:?}", arch);

                // Show what directory this should go into
                let expected_dir = subdir.unwrap_or_else(|| match platform {
                    Some(Platform::Linux64) => "linux-64".to_string(),
                    Some(Platform::NoArch) => "noarch".to_string(),
                    Some(p) => {
                        println!("   Other platform: {:?}", p);
                        "unknown".to_string()
                    }
                    None => "unknown".to_string(),
                });

                println!("   → Should be placed in: {}/", expected_dir);
            }
            Err(e) => {
                println!("❌ Failed to extract metadata: {}", e);

                // Fall back to filename-based detection
                println!("   Falling back to filename parsing...");
                if let Some(platform_hint) = extract_platform_from_filename(package_path) {
                    println!("   Filename suggests platform: {}", platform_hint);
                } else {
                    println!("   No platform info in filename");
                }
            }
        }
    }

    Ok(())
}

/// Extract platform information from a conda package using rattler
async fn extract_platform_info(
    content: &Bytes,
    filename: &str,
) -> Result<(Option<Platform>, Option<String>, Option<String>)> {
    println!("Attempting to extract metadata from conda package...");

    // Create cursor from bytes
    let _cursor = Cursor::new(content.as_ref());

    // Write content to temp file for rattler to process
    let temp_file = tempfile::NamedTempFile::new()?;
    std::fs::write(temp_file.path(), content)?;

    // Try to extract conda info using rattler_package_streaming
    // Create a temporary directory for extraction
    let temp_dir = tempfile::tempdir()?;
    let _extract_result = extract_conda(temp_file.path(), temp_dir.path())?;

    // Try to read the index.json file from the extracted contents
    let index_json_path = temp_dir.path().join("info").join("index.json");
    if index_json_path.exists() {
        let index_json_content = std::fs::read_to_string(&index_json_path)?;
        println!("Found conda package metadata");

        // Parse the JSON metadata
        let metadata: serde_json::Value = serde_json::from_str(&index_json_content)?;

        // Extract relevant fields
        let platform_str = metadata
            .get("platform")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let subdir = metadata
            .get("subdir")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let arch = metadata
            .get("arch")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Convert platform string to Platform enum
        let platform = if let Some(subdir_val) = &subdir {
            match subdir_val.as_str() {
                "linux-64" => Some(Platform::Linux64),
                "linux-32" => Some(Platform::Linux32),
                "linux-aarch64" => Some(Platform::LinuxAarch64),
                "linux-armv6l" => Some(Platform::LinuxArmV6l),
                "linux-armv7l" => Some(Platform::LinuxArmV7l),
                "linux-ppc64le" => Some(Platform::LinuxPpc64le),
                "linux-s390x" => Some(Platform::LinuxS390X),
                "osx-64" => Some(Platform::Osx64),
                "osx-arm64" => Some(Platform::OsxArm64),
                "win-32" => Some(Platform::Win32),
                "win-64" => Some(Platform::Win64),
                "noarch" => Some(Platform::NoArch),
                _ => {
                    println!("Unknown subdir: {}, trying platform field", subdir_val);
                    platform_str.as_ref().and_then(|p| p.parse().ok())
                }
            }
        } else {
            platform_str.as_ref().and_then(|p| p.parse().ok())
        };

        println!("Raw metadata extracted:");
        println!("  platform field: {:?}", platform_str);
        println!("  subdir field: {:?}", subdir);
        println!("  arch field: {:?}", arch);

        return Ok((platform, subdir, arch));
    }

    Err(anyhow::anyhow!(
        "No index.json found in conda package {}",
        filename
    ))
}

/// Extract platform hint from filename (fallback method)
fn extract_platform_from_filename(filename: &str) -> Option<&str> {
    let name = filename
        .strip_suffix(".conda")
        .or_else(|| filename.strip_suffix(".tar.bz2"))?;

    let parts: Vec<&str> = name.split('-').collect();
    if parts.len() >= 2 {
        // Check last part
        let last_part = parts.last()?;
        if is_platform_string(last_part) {
            return Some(last_part);
        }

        // Check second-to-last + last parts (for cases like "linux-64")
        if parts.len() >= 2 {
            let potential_platform =
                format!("{}-{}", parts[parts.len() - 2], parts[parts.len() - 1]);
            if is_platform_string(&potential_platform) {
                return Some(parts[parts.len() - 2]);
            }
        }
    }

    None
}

/// Check if a string looks like a conda platform
fn is_platform_string(s: &str) -> bool {
    matches!(
        s,
        "linux-64"
            | "linux-32"
            | "linux-aarch64"
            | "linux-armv6l"
            | "linux-armv7l"
            | "linux-ppc64le"
            | "linux-s390x"
            | "osx-64"
            | "osx-arm64"
            | "win-32"
            | "win-64"
            | "noarch"
            | "64" // For cases where platform is split like "linux" "64"
    )
}
