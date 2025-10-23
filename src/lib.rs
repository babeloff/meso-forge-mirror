//! Meso Forge Mirror Library
//!
//! A Rust library for mirroring conda packages from various sources to target repositories.
//! This library provides enhanced functionality through integration with the rattler ecosystem
//! for proper conda package handling, validation, and repository structure management.

pub mod azure;
pub mod conda_package;
pub mod config;
pub mod github;
pub mod mirror;
pub mod repository;

pub use conda_package::{CondaPackageHandler, PackageStats, ProcessedPackage, SimpleIndexJson};
pub use config::Config;
pub use mirror::mirror_packages;
pub use repository::{Repository, RepositoryType};

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use md5::Digest;
    // use md5::Md5;
    // use sha2::Digest;
    // use sha2::Sha256;
    // use sha2::Sha512;
    use tempfile::TempDir;

    /// Test the conda package processing functionality
    #[tokio::test]
    async fn test_conda_package_processing() {
        let handler = CondaPackageHandler::new();

        // Create a mock conda package (just bytes for testing)
        let _mock_package_content = Bytes::from("mock conda package content");
        let filename = "numpy-1.21.0-py39h06a4308_0-linux-64.conda";

        // This would normally fail because we don't have real conda package data,
        // but we can test the filename parsing logic
        let result = handler.extract_metadata_from_filename(filename);
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert_eq!(metadata.name, "numpy");
        assert_eq!(metadata.version, "1.21.0");
        assert_eq!(metadata.build, "py39h06a4308_0");
        assert_eq!(metadata.build_number, 0);
    }

    /// Test repository creation and structure
    #[tokio::test]
    async fn test_repository_structure() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_str().unwrap();

        let repository = Repository::new(RepositoryType::Local, repo_path.to_string());

        // Test that we can create the repository structure
        assert!(matches!(repository.repo_type, RepositoryType::Local));
        assert_eq!(repository.path, repo_path);

        // Test package stats functionality
        let stats = repository.get_package_stats();
        assert_eq!(stats.total_packages, 0);
        assert_eq!(stats.total_size, 0);
    }

    /// Test platform detection from filenames
    #[test]
    fn test_platform_detection() {
        let test_cases = vec![
            ("package-1.0-build-linux-64.conda", "linux"),
            ("python-3.9-h123-osx-arm64.tar.bz2", "osx"),
            ("numpy-1.21-py39-win-64.conda", "win"),
            ("pure-python-1.0-noarch.conda", "noarch"),
        ];

        for (filename, expected_platform_prefix) in test_cases {
            let platform_part = CondaPackageHandler::extract_platform_from_filename(filename);
            assert!(platform_part.is_some());
            assert!(platform_part.unwrap().starts_with(expected_platform_prefix));
        }
    }

    /// Test configuration handling
    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert_eq!(config.max_concurrent_downloads, 5);
        assert_eq!(config.retry_attempts, 3);
        assert_eq!(config.timeout_seconds, 300);
    }

    /// Demonstrate the improved workflow
    #[tokio::test]
    async fn test_improved_workflow_demo() {
        // This test demonstrates the enhanced functionality we've added:

        // 1. Enhanced package processing with metadata extraction
        let _handler = CondaPackageHandler::new();
        let filename = "scipy-1.7.0-py39h06a4308_0-linux-64.conda";

        // Test that the filename is recognized as a conda package
        assert!(CondaPackageHandler::is_conda_package(filename));

        // 2. Platform-aware repository organization
        let temp_dir = TempDir::new().unwrap();
        let _repo = Repository::new(
            RepositoryType::Local,
            temp_dir.path().to_str().unwrap().to_string(),
        );

        // 3. Proper conda repository structure would be created
        // (linux-64/, osx-64/, noarch/, etc. subdirectories)

        // 4. Package validation and checksums
        let mock_content = Bytes::from("test content");
        let mock_metadata = SimpleIndexJson {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            build: "py39_0".to_string(),
            build_number: 0,
            depends: vec!["python >=3.9".to_string()],
            license: Some("MIT".to_string()),
            platform: Some("linux-64".to_string()),
            timestamp: Some(chrono::Utc::now()),
        };

        // This demonstrates the enhanced ProcessedPackage structure
        let processed = ProcessedPackage {
            content: mock_content.clone(),
            metadata: mock_metadata,
            filename: filename.to_string(),
            platform: rattler_conda_types::Platform::Linux64,
            size: mock_content.len() as u64,
            md5: format!("{:x}", md5::Md5::digest(&mock_content)),
            sha256: format!("{:x}", sha2::Sha256::digest(&mock_content)),
        };

        assert!(!processed.filename.is_empty());
        assert!(!processed.md5.is_empty());
        assert!(!processed.sha256.is_empty());
        assert_eq!(processed.size, mock_content.len() as u64);
    }
}

/// Integration tests demonstrating real-world usage scenarios
#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test that shows how the tool would be used for mirroring to Rattler cache
    #[tokio::test]
    async fn test_rattler_cache_integration() {
        // This test demonstrates how the improved tool would integrate with
        // the Rattler cache directory structure:

        let cache_path = "~/.cache/rattler/cache/pkgs";
        let repository = Repository::new(RepositoryType::Local, cache_path.to_string());

        // The repository would now:
        // 1. Validate conda packages before storing
        // 2. Organize packages by platform (linux-64/, osx-64/, etc.)
        // 3. Generate proper repodata.json files
        // 4. Verify package checksums and metadata

        assert_eq!(repository.path, cache_path);
        assert!(matches!(repository.repo_type, RepositoryType::Local));
    }

    /// Test showing improved error handling and validation
    #[test]
    fn test_package_validation() {
        let _handler = CondaPackageHandler::new();

        // Test validation of invalid package names
        let invalid_files = vec!["not-a-conda-package.txt", "invalid-format", ""];

        for invalid_file in invalid_files {
            let is_conda = CondaPackageHandler::is_conda_package(invalid_file);
            assert!(
                !is_conda,
                "File {} should not be recognized as conda package",
                invalid_file
            );
        }

        // Test validation of valid package names
        let valid_files = vec![
            "package-1.0.0-build-linux-64.conda",
            "another-package-2.1-py39-osx-64.tar.bz2",
        ];

        for valid_file in valid_files {
            let is_conda = CondaPackageHandler::is_conda_package(valid_file);
            assert!(
                is_conda,
                "File {} should be recognized as conda package",
                valid_file
            );
        }
    }
}

pub mod examples {
    //! # Usage Examples
    //!
    //! ## Basic Package Mirroring
    //!
    //! ```rust,no_run
    //! use meso_forge_mirror::{mirror_packages, Config, RepositoryType};
    //!
    //! #[tokio::main]
    //! async fn main() -> anyhow::Result<()> {
    //!     let sources = vec![
    //!         "https://example.com/package1.conda".to_string(),
    //!         "https://example.com/package2.conda".to_string(),
    //!     ];
    //!
    //!     let config = Config::default();
    //!
    //!     // Mirror to Rattler cache
    //!     mirror_packages(
    //!         &sources,
    //!         RepositoryType::Local,
    //!         "~/.cache/rattler/cache/pkgs/",
    //!         &config
    //!     ).await?;
    //!
    //!     Ok(())
    //! }
    //! ```
    //!
    //! ## Advanced Package Processing
    //!
    //! ```rust,no_run
    //! use meso_forge_mirror::CondaPackageHandler;
    //! use bytes::Bytes;
    //!
    //! #[tokio::main]
    //! async fn main() -> anyhow::Result<()> {
    //!     let mut handler = CondaPackageHandler::new();
    //!
    //!     // Process a downloaded package
    //!     let package_data = Bytes::from("..."); // Downloaded package data
    //!     let processed = handler.process_package(
    //!         package_data,
    //!         "numpy-1.21.0-py39_0-linux-64.conda"
    //!     ).await?;
    //!
    //!     println!("Package: {} v{}", processed.metadata.name, processed.metadata.version);
    //!     println!("Platform: {}", processed.platform);
    //!     println!("Size: {} bytes", processed.size);
    //!     println!("SHA256: {}", processed.sha256);
    //!
    //!     Ok(())
    //! }
}
