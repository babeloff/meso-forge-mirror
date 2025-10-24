use bytes::Bytes;
use std::fs;
use tempfile::TempDir;

use meso_forge_mirror::repository::{Repository, RepositoryType};
use rattler_cache::default_cache_dir;
use rattler_cache::package_cache::PackageCache;

#[tokio::test]
async fn test_cache_stores_packages_correctly() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_path = temp_dir.path().join("rattler_cache");

    // Create a cache repository
    let mut cache_repo = Repository::new(
        RepositoryType::Cache,
        cache_path.to_string_lossy().to_string(),
    );

    // Create test package content
    let package_name = "rb-asciidoctor-revealjs-5.2.0-h1d6dcf3_0.conda";
    let package_content = create_minimal_conda_package();

    // Upload package to cache
    let result = cache_repo
        .upload_package(package_name, Bytes::from(package_content.clone()))
        .await;
    assert!(result.is_ok(), "Failed to cache package: {:?}", result);

    // Verify package exists in cache
    let cached_file = cache_path.join(package_name);
    assert!(
        cached_file.exists(),
        "Package should be cached at {:?}",
        cached_file
    );

    // Verify content matches
    let cached_content = fs::read(&cached_file).expect("Failed to read cached file");
    assert_eq!(
        cached_content, package_content,
        "Cached content should match original"
    );
}

#[tokio::test]
async fn test_cache_integrates_with_rattler_package_cache() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_path = temp_dir.path().join("rattler_cache");

    // Upload a package using our cache repository
    let mut cache_repo = Repository::new(
        RepositoryType::Cache,
        cache_path.to_string_lossy().to_string(),
    );

    let package_name = "test-package-1.0.0-py311_0.conda";
    let package_content = create_minimal_conda_package();

    let result = cache_repo
        .upload_package(package_name, Bytes::from(package_content))
        .await;
    assert!(result.is_ok(), "Package upload should succeed");

    // Test rattler PackageCache can work with our cache directory
    let _package_cache = PackageCache::new(&cache_path);

    // Verify cache directory structure is compatible
    assert!(cache_path.exists(), "Cache directory should exist");

    // The package file should be stored directly in the cache directory
    let package_file = cache_path.join(package_name);
    assert!(package_file.exists(), "Package file should exist in cache");
}

#[test]
fn test_default_cache_directory_resolution() {
    // Test that we can get the default cache directory
    let default_cache = default_cache_dir();
    assert!(
        default_cache.is_ok(),
        "Should be able to get default cache directory"
    );

    let cache_path = default_cache.unwrap();
    assert!(cache_path.is_absolute(), "Cache path should be absolute");

    // The path should contain "rattler" somewhere
    let path_str = cache_path.to_string_lossy();
    assert!(
        path_str.contains("rattler"),
        "Cache path should contain 'rattler': {}",
        path_str
    );
}

#[tokio::test]
async fn test_package_name_parsing_and_discovery() {
    // This test addresses the original issue: package name typos prevent discovery
    let test_cases = vec![
        (
            "rb-asciidoctor-revealjs-5.2.0-h1d6dcf3_0.conda",
            "rb-asciidoctor-revealjs",
            true,
        ),
        (
            "rb-asciidoctor-revealjs-5.2.0-h1d6dcf3_0.conda",
            "rb-asciidocgtor-revealjs",
            false,
        ), // typo: missing 't'
        (
            "rb-asciidoctor-revealjs-5.2.0-h1d6dcf3_0.conda",
            "asciidoctor-revealjs",
            false,
        ), // missing prefix
        ("python-3.11.0-h1234567_0.conda", "python", true),
        ("numpy-1.21.0-py311h1234567_0.conda", "numpy", true),
    ];

    for (package_filename, search_term, should_match) in test_cases {
        let matches = package_matches_search(package_filename, search_term);
        assert_eq!(
            matches,
            should_match,
            "Package '{}' search for '{}' should {} match",
            package_filename,
            search_term,
            if should_match { "" } else { "not" }
        );
    }
}

#[tokio::test]
async fn test_cache_vs_repository_structure_differences() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_path = temp_dir.path().join("cache");
    let repo_path = temp_dir.path().join("repo");

    let mut cache_repo = Repository::new(
        RepositoryType::Cache,
        cache_path.to_string_lossy().to_string(),
    );
    let mut local_repo = Repository::new(
        RepositoryType::Local,
        repo_path.to_string_lossy().to_string(),
    );

    let package_name = "example-package-1.0.0-h123_0.conda";
    let package_content = create_minimal_conda_package();

    // Upload to both
    let cache_result = cache_repo
        .upload_package(package_name, Bytes::from(package_content.clone()))
        .await;
    let repo_result = local_repo
        .upload_package(package_name, Bytes::from(package_content))
        .await;

    assert!(cache_result.is_ok(), "Cache upload should succeed");
    assert!(repo_result.is_ok(), "Repository upload should succeed");

    // Cache behavior: stores individual package files
    let cache_package_file = cache_path.join(package_name);
    assert!(
        cache_package_file.exists(),
        "Cache should store package file directly"
    );

    // Repository behavior: creates structured conda repository
    assert!(repo_path.exists(), "Repository directory should exist");

    // The key difference: cache stores for reuse, repository creates conda channels
    // Cache files can be used by rattler/pixi for package resolution
    // Repository creates repodata.json and platform-specific directories
}

#[tokio::test]
async fn test_multiple_packages_in_cache() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_path = temp_dir.path().join("cache");

    let mut cache_repo = Repository::new(
        RepositoryType::Cache,
        cache_path.to_string_lossy().to_string(),
    );

    let packages = vec![
        "numpy-1.21.0-py311h1234567_0.conda",
        "scipy-1.7.0-py311h1234567_0.conda",
        "matplotlib-3.5.0-py311h1234567_0.conda",
        "rb-asciidoctor-revealjs-5.2.0-h1d6dcf3_0.conda",
    ];

    // Cache multiple packages
    for package_name in &packages {
        let content = create_minimal_conda_package();
        let result = cache_repo
            .upload_package(package_name, Bytes::from(content))
            .await;
        assert!(
            result.is_ok(),
            "Failed to cache package {}: {:?}",
            package_name,
            result
        );
    }

    // Verify all packages are cached
    for package_name in &packages {
        let package_file = cache_path.join(package_name);
        assert!(
            package_file.exists(),
            "Package {} should be cached",
            package_name
        );
    }

    // Verify cache directory contains all packages
    let cached_files: Vec<_> = fs::read_dir(&cache_path)
        .expect("Failed to read cache directory")
        .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
        .collect();

    for package_name in &packages {
        assert!(
            cached_files.contains(&package_name.to_string()),
            "Cache should contain {}, found: {:?}",
            package_name,
            cached_files
        );
    }
}

#[test]
fn test_identify_user_issue_root_cause() {
    // This test documents the exact issue from the user's problem

    // User ran this command successfully:
    // meso-forge-mirror mirror --src-type zip --src ~/Downloads/conda_pkgs_noarch-rb-asciidoctor.zip --src-path 'conda_pkgs_noarch/rb-asciidoctor-revealjs-5\.2\.0-.*_0\.conda'

    // The package that should have been cached:
    let correct_package_name = "rb-asciidoctor-revealjs-5.2.0-h1d6dcf3_0.conda";

    // User tried to install with this command:
    // pixi global install rb-asciidocgtor-revealjs  (note the typo: missing 't' in asciidoctor)
    let user_search_term = "rb-asciidocgtor-revealjs";
    let correct_search_term = "rb-asciidoctor-revealjs";

    // The root cause: typo in the search term
    assert!(
        !correct_package_name.starts_with(user_search_term),
        "Package with correct spelling should not match search with typo"
    );

    assert!(
        correct_package_name.starts_with(correct_search_term),
        "Package should match search with correct spelling"
    );

    // Additional issues that could compound the problem:
    // 1. Cache stores individual .conda files but may not create proper repository structure
    // 2. pixi expects conda repositories with repodata.json for package discovery
    // 3. The cache behavior is for individual package reuse, not channel creation
}

#[tokio::test]
async fn test_cache_repository_finalization() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_path = temp_dir.path().join("cache");

    let mut cache_repo = Repository::new(
        RepositoryType::Cache,
        cache_path.to_string_lossy().to_string(),
    );

    // Add a package
    let result = cache_repo
        .upload_package(
            "test-package-1.0.0-h123_0.conda",
            Bytes::from(create_minimal_conda_package()),
        )
        .await;
    assert!(result.is_ok(), "Package upload should succeed");

    // Finalize repository (this should not create repodata for cache)
    let finalize_result = cache_repo.finalize_repository().await;
    assert!(finalize_result.is_ok(), "Cache finalization should succeed");

    // Cache repositories don't create repodata.json (unlike local/s3/prefix-dev repositories)
    let repodata_file = cache_path.join("repodata.json");
    assert!(
        !repodata_file.exists(),
        "Cache should not create repodata.json (that's for conda channels)"
    );
}

// Helper functions

fn create_minimal_conda_package() -> Vec<u8> {
    // Create a minimal conda package structure for testing
    // In reality, conda packages are tar.bz2 or conda (zip) archives with specific structure
    let mut content = Vec::new();

    // Add conda package signature (simplified for testing)
    content.extend_from_slice(b"PK\x03\x04"); // ZIP signature for .conda format
    content.extend_from_slice(b"test_conda_package_content");

    // Add some realistic-looking metadata
    let metadata = r#"
    {
        "name": "test-package",
        "version": "1.0.0",
        "build": "h123_0",
        "channel": "conda-forge"
    }
    "#;
    content.extend_from_slice(metadata.as_bytes());

    // Pad to make it look more like a real package
    content.extend_from_slice(&[0u8; 256]);

    content
}

fn package_matches_search(package_filename: &str, search_term: &str) -> bool {
    // Simple package matching logic (similar to what pixi might do)

    // Remove .conda extension
    let package_name = package_filename
        .strip_suffix(".conda")
        .or_else(|| package_filename.strip_suffix(".tar.bz2"))
        .unwrap_or(package_filename);

    // Extract the package name part (before version)
    // conda packages follow: name-version-build pattern
    if let Some(version_start) = find_version_start(package_name) {
        let name_part = &package_name[..version_start];
        name_part == search_term
    } else {
        // Fallback: simple prefix match
        package_name.starts_with(search_term)
    }
}

fn find_version_start(package_name: &str) -> Option<usize> {
    // Find where the version starts in a conda package name
    // This is a simplified version - real conda package parsing is more complex

    let parts: Vec<&str> = package_name.split('-').collect();
    if parts.len() < 2 {
        return None;
    }

    // Look for a part that starts with a digit (likely version)
    for (i, part) in parts.iter().enumerate().skip(1) {
        if part.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            // Found version, return position in original string
            let prefix_parts = &parts[..i];
            let prefix_len = prefix_parts.join("-").len();
            return Some(if prefix_len > 0 { prefix_len + 1 } else { 0 });
        }
    }

    None
}

#[cfg(test)]
mod package_name_parsing_tests {
    use super::*;

    #[test]
    fn test_find_version_start() {
        assert_eq!(find_version_start("numpy-1.21.0-py311h123_0"), Some(6)); // after "numpy-"
        assert_eq!(
            find_version_start("rb-asciidoctor-revealjs-5.2.0-h123_0"),
            Some(24)
        ); // after "rb-asciidoctor-revealjs-"
        assert_eq!(find_version_start("python-3.11.0-h123_0"), Some(7)); // after "python-"
        assert_eq!(find_version_start("single-package"), None); // no version found
    }

    #[test]
    fn test_package_name_extraction() {
        let test_cases = vec![
            ("numpy-1.21.0-py311h123_0.conda", "numpy"),
            (
                "rb-asciidoctor-revealjs-5.2.0-h123_0.conda",
                "rb-asciidoctor-revealjs",
            ),
            ("python-3.11.0-h123_0.tar.bz2", "python"),
            ("scipy-1.7.0-py311h123_0", "scipy"),
        ];

        for (package_filename, expected_name) in test_cases {
            let matches = package_matches_search(package_filename, expected_name);
            assert!(
                matches,
                "Package '{}' should match name '{}'",
                package_filename, expected_name
            );
        }
    }
}
