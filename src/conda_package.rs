use anyhow::{anyhow, Result};
use bytes::Bytes;
use md5::Md5;
use rattler_conda_types::Platform;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

/// Represents a processed conda package with metadata
#[derive(Debug, Clone)]
pub struct ProcessedPackage {
    pub content: Bytes,
    pub metadata: SimpleIndexJson,
    pub filename: String,
    pub platform: Platform,
    pub size: u64,
    pub md5: String,
    pub sha256: String,
}

/// Simplified conda package metadata structure
#[derive(Debug, Clone)]
pub struct SimpleIndexJson {
    pub name: String,
    pub version: String,
    pub build: String,
    pub build_number: u64,
    pub depends: Vec<String>,
    pub license: Option<String>,
    pub platform: Option<String>,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for SimpleIndexJson {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: String::new(),
            build: String::new(),
            build_number: 0,
            depends: Vec::new(),
            license: None,
            platform: None,
            timestamp: Some(chrono::Utc::now()),
        }
    }
}

/// Handles conda package validation, metadata extraction, and organization
pub struct CondaPackageHandler {
    cache: HashMap<String, ProcessedPackage>,
}

impl Default for CondaPackageHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CondaPackageHandler {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Process a downloaded conda package and extract metadata
    pub async fn process_package(
        &mut self,
        content: Bytes,
        filename: &str,
    ) -> Result<ProcessedPackage> {
        info!("Processing conda package: {}", filename);

        // Validate that this is a conda package by checking the filename extension
        if !Self::is_conda_package(filename) {
            return Err(anyhow!("File {} is not a conda package", filename));
        }

        // Extract metadata from filename (simplified approach)
        let metadata = self.extract_metadata_from_filename(filename)?;

        // Determine platform
        let platform = Self::determine_platform(&metadata, filename)?;

        // Calculate checksums
        let md5 = format!("{:x}", Md5::digest(&content));
        let sha256 = format!("{:x}", Sha256::digest(&content));

        let size = content.len() as u64;
        let processed = ProcessedPackage {
            content,
            metadata,
            filename: filename.to_string(),
            platform,
            size,
            md5,
            sha256,
        };

        // Cache the processed package
        self.cache.insert(filename.to_string(), processed.clone());

        info!(
            "Successfully processed package: {} (platform: {}, size: {} bytes)",
            filename, processed.platform, processed.size
        );

        Ok(processed)
    }

    /// Extract metadata from filename (fallback approach)
    pub fn extract_metadata_from_filename(&self, filename: &str) -> Result<SimpleIndexJson> {
        debug!("Extracting metadata from filename: {}", filename);

        // Remove extension
        let name_without_ext = filename
            .strip_suffix(".conda")
            .or_else(|| filename.strip_suffix(".tar.bz2"))
            .ok_or_else(|| anyhow!("Invalid conda package extension"))?;

        // Parse conda package filename format: name-version-build_platform
        // Example: numpy-1.21.0-py39hd472c2d_0-linux-64.conda
        let parts: Vec<&str> = name_without_ext.split('-').collect();

        if parts.len() < 3 {
            warn!("Cannot parse filename {}, using defaults", filename);
            return Ok(SimpleIndexJson {
                name: "unknown".to_string(),
                version: "0.0.0".to_string(),
                build: "unknown".to_string(),
                ..Default::default()
            });
        }

        let name = parts[0].to_string();
        let version = parts[1].to_string();

        // Find where build info starts (after version)
        let mut build_parts = Vec::new();
        for part in parts.iter().skip(2) {
            if Self::is_platform_string(part) {
                break;
            }
            build_parts.push(*part);
        }

        let build = if build_parts.is_empty() {
            "0".to_string()
        } else {
            build_parts.join("-")
        };

        // Extract build number from build string
        let build_number = build
            .split('_')
            .next_back()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        Ok(SimpleIndexJson {
            name,
            version,
            build,
            build_number,
            depends: Vec::new(),
            license: None,
            platform: Self::extract_platform_from_filename(filename).map(|s| s.to_string()),
            timestamp: Some(chrono::Utc::now()),
        })
    }

    /// Determine the platform from metadata or filename
    fn determine_platform(metadata: &SimpleIndexJson, filename: &str) -> Result<Platform> {
        // First try to get platform from metadata
        if let Some(platform_str) = &metadata.platform {
            if let Ok(platform) = platform_str.parse() {
                return Ok(platform);
            }
        }

        // Fall back to extracting from filename
        if let Some(platform_str) = Self::extract_platform_from_filename(filename) {
            if let Ok(platform) = platform_str.parse() {
                return Ok(platform);
            }
        }

        // Default to noarch if we can't determine the platform
        warn!(
            "Could not determine platform for {}, defaulting to noarch",
            filename
        );
        Ok(Platform::NoArch)
    }

    /// Extract platform from filename
    pub fn extract_platform_from_filename(filename: &str) -> Option<&str> {
        // Remove extension
        let name = filename
            .strip_suffix(".conda")
            .or_else(|| filename.strip_suffix(".tar.bz2"))?;

        // Split by '-' and try to find a platform-like string at the end
        let parts: Vec<&str> = name.split('-').collect();
        if parts.len() >= 2 {
            // Check last part
            if Self::is_platform_string(parts.last()?) {
                return parts.last().copied();
            }

            // Check second-to-last + last parts (for cases like "linux-64")
            if parts.len() >= 2 {
                let potential_platform =
                    format!("{}-{}", parts[parts.len() - 2], parts[parts.len() - 1]);
                if Self::is_platform_string(&potential_platform) {
                    return Some(parts[parts.len() - 2]); // Return the full platform part
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

    /// Check if a file is a conda package based on extension
    pub fn is_conda_package(filename: &str) -> bool {
        filename.ends_with(".conda") || filename.ends_with(".tar.bz2")
    }

    /// Generate repository structure for packages
    pub fn organize_packages(&self) -> HashMap<Platform, Vec<ProcessedPackage>> {
        let mut organized: HashMap<Platform, Vec<ProcessedPackage>> = HashMap::new();

        for package in self.cache.values() {
            organized
                .entry(package.platform)
                .or_default()
                .push(package.clone());
        }

        organized
    }

    /// Create or update repodata.json for a platform
    pub async fn create_repodata(
        &self,
        platform: &Platform,
        packages: &[ProcessedPackage],
        base_path: &Path,
    ) -> Result<()> {
        info!("Creating repodata for platform: {}", platform);

        let platform_dir = base_path.join(platform.to_string());
        std::fs::create_dir_all(&platform_dir)?;

        let repodata_path = platform_dir.join("repodata.json");

        // Create a simple repodata structure
        let mut repodata = SimpleRepoData {
            info: SimpleRepoDataInfo {
                subdir: platform.to_string(),
            },
            packages: HashMap::new(),
        };

        // Add packages to repodata
        for package in packages {
            let package_record = SimplePackageRecord {
                build: package.metadata.build.clone(),
                build_number: package.metadata.build_number,
                depends: package.metadata.depends.clone(),
                license: package.metadata.license.clone().unwrap_or_default(),
                md5: package.md5.clone(),
                sha256: package.sha256.clone(),
                size: package.size,
                subdir: platform.to_string(),
                name: package.metadata.name.clone(),
                version: package.metadata.version.clone(),
                timestamp: package.metadata.timestamp,
            };

            repodata
                .packages
                .insert(package.filename.clone(), package_record);
        }

        // Write repodata
        let repodata_json = serde_json::to_string_pretty(&repodata)?;
        std::fs::write(&repodata_path, repodata_json)?;

        info!("Updated repodata.json with {} packages", packages.len());
        Ok(())
    }

    /// Validate package integrity
    pub fn validate_package(&self, package: &ProcessedPackage) -> Result<()> {
        // Basic validation checks
        if package.metadata.name.is_empty() {
            return Err(anyhow!("Package name is empty"));
        }

        if package.metadata.version.is_empty() {
            return Err(anyhow!("Package version is empty"));
        }

        if package.content.is_empty() {
            return Err(anyhow!("Package content is empty"));
        }

        // Verify checksums
        let calculated_md5 = format!("{:x}", Md5::digest(&package.content));
        let calculated_sha256 = format!("{:x}", Sha256::digest(&package.content));

        if calculated_md5 != package.md5 {
            return Err(anyhow!("MD5 checksum mismatch"));
        }

        if calculated_sha256 != package.sha256 {
            return Err(anyhow!("SHA256 checksum mismatch"));
        }

        debug!("Package validation passed for: {}", package.filename);
        Ok(())
    }

    /// Get statistics about processed packages
    pub fn get_stats(&self) -> PackageStats {
        let mut stats = PackageStats::default();

        for package in self.cache.values() {
            stats.total_packages += 1;
            stats.total_size += package.size;
            *stats
                .packages_by_platform
                .entry(package.platform)
                .or_insert(0) += 1;
        }

        stats
    }
}

/// Simple repodata structure for JSON serialization
#[derive(serde::Serialize)]
struct SimpleRepoData {
    info: SimpleRepoDataInfo,
    packages: HashMap<String, SimplePackageRecord>,
}

#[derive(serde::Serialize)]
struct SimpleRepoDataInfo {
    subdir: String,
}

#[derive(serde::Serialize)]
struct SimplePackageRecord {
    build: String,
    build_number: u64,
    depends: Vec<String>,
    license: String,
    md5: String,
    sha256: String,
    size: u64,
    subdir: String,
    name: String,
    version: String,
    timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

/// Statistics about processed packages
#[derive(Debug, Default)]
pub struct PackageStats {
    pub total_packages: usize,
    pub total_size: u64,
    pub packages_by_platform: HashMap<Platform, usize>,
}

impl PackageStats {
    pub fn print_summary(&self) {
        println!("Package Statistics:");
        println!("  Total packages: {}", self.total_packages);
        println!(
            "  Total size: {:.2} MB",
            self.total_size as f64 / 1_000_000.0
        );
        println!("  Packages by platform:");

        for (platform, count) in &self.packages_by_platform {
            println!("    {}: {}", platform, count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_conda_package() {
        assert!(CondaPackageHandler::is_conda_package("package.conda"));
        assert!(CondaPackageHandler::is_conda_package("package.tar.bz2"));
        assert!(!CondaPackageHandler::is_conda_package("package.txt"));
        assert!(!CondaPackageHandler::is_conda_package("package"));
    }

    #[test]
    fn test_extract_platform_from_filename() {
        assert_eq!(
            CondaPackageHandler::extract_platform_from_filename(
                "python-3.9.0-hd23f0df_0_cpython-linux-64.conda"
            ),
            Some("linux")
        );
        assert_eq!(
            CondaPackageHandler::extract_platform_from_filename(
                "numpy-1.21.0-py39h_linux-64.conda"
            ),
            Some("linux")
        );
        assert_eq!(
            CondaPackageHandler::extract_platform_from_filename("somepackage-noarch.conda"),
            Some("noarch")
        );
    }

    #[test]
    fn test_is_platform_string() {
        assert!(CondaPackageHandler::is_platform_string("linux-64"));
        assert!(CondaPackageHandler::is_platform_string("osx-arm64"));
        assert!(CondaPackageHandler::is_platform_string("win-64"));
        assert!(CondaPackageHandler::is_platform_string("noarch"));
        assert!(CondaPackageHandler::is_platform_string("64"));
        assert!(!CondaPackageHandler::is_platform_string("random-string"));
        assert!(!CondaPackageHandler::is_platform_string("123"));
    }

    #[test]
    fn test_extract_metadata_from_filename() {
        let handler = CondaPackageHandler::new();

        let metadata = handler
            .extract_metadata_from_filename("numpy-1.21.0-py39h06a4308_0-linux-64.conda")
            .unwrap();
        assert_eq!(metadata.name, "numpy");
        assert_eq!(metadata.version, "1.21.0");
        assert_eq!(metadata.build, "py39h06a4308_0");
        assert_eq!(metadata.build_number, 0);

        let metadata = handler
            .extract_metadata_from_filename("python-3.9.7-h12debd9_1-osx-64.conda")
            .unwrap();
        assert_eq!(metadata.name, "python");
        assert_eq!(metadata.version, "3.9.7");
        assert_eq!(metadata.build, "h12debd9_1");
        assert_eq!(metadata.build_number, 1);
    }
}
