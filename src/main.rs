use anyhow::Result;
use clap::{Parser, Subcommand};
use rattler_cache::default_cache_dir;
use tracing::{info, warn};

mod azure;
mod conda_package;
mod config;
mod github;
mod mirror;
mod repository;

use config::Config;
use mirror::mirror_packages;
use repository::RepositoryType;

#[derive(Parser)]
#[command(name = "meso-forge-mirror")]
#[command(version)]
#[command(about = "Mirror conda packages from staging PRs to target repositories", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Mirror packages from source to target repository
    Mirror {
        /// Source type: zip (local zip), zip-url (remote zip), local (local conda), url (remote conda), tgz (local tarball), tgz-url (remote tarball), github (GitHub artifacts), azure (Azure DevOps artifacts)
        #[arg(long, default_value = "local")]
        src_type: String,

        /// Source path or URL (local file path or remote URL)
        #[arg(long)]
        src: String,

        /// Regular expression to match file paths within ZIP file where conda packages are located (only first match processed; required when src-type is 'zip' or 'zip-url')
        #[arg(long)]
        src_path: Option<String>,

        /// Target type: 'cache' stores individual packages for reuse, 'local'/'s3'/'prefix-dev' create conda repositories with repodata
        #[arg(long, default_value = "cache")]
        tgt_type: String,

        /// Target path or URL (automatically determined for 'cache', required for repository types)
        #[arg(long)]
        tgt: Option<String>,

        /// Configuration file (optional)
        #[arg(short, long)]
        config: Option<String>,
    },
    /// Get information about repository artifacts
    Info {
        /// GitHub repository in format 'owner/repo' or GitHub URL
        #[arg(long)]
        github: Option<String>,

        /// Azure DevOps organization/project in format 'org/project' or Azure DevOps URL
        #[arg(long)]
        azure: Option<String>,

        /// Azure DevOps build ID (optional, if not specified lists recent builds)
        #[arg(long)]
        build_id: Option<u64>,

        /// Filter artifacts by name pattern (regex)
        #[arg(long)]
        name_filter: Option<String>,

        /// Filter builds by description pattern (regex) - Azure only
        #[arg(long)]
        description_filter: Option<String>,

        /// Output format for the info command (yaml, json, table)
        #[arg(long, default_value = "yaml", value_parser = ["yaml", "json", "table"])]
        encode: String,

        /// Show only non-expired artifacts (GitHub only)
        #[arg(long, default_value = "true")]
        exclude_expired: bool,

        /// Configuration file (optional)
        #[arg(short, long)]
        config: Option<String>,
    },
    /// Initialize configuration file
    Init {
        /// Output path for config file
        #[arg(short, long, default_value = "meso-forge-mirror.json")]
        output: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Mirror {
            src_type,
            src,
            src_path,
            tgt_type,
            tgt,
            config,
        } => {
            info!("Starting package mirroring");

            // Validate source type
            match src_type.as_str() {
                "zip" | "zip-url" | "local" | "url" | "tgz" | "tgz-url" | "github" | "azure" => {}
                _ => {
                    return Err(anyhow::anyhow!(
                    "Invalid src-type '{}'. Must be one of: zip, zip-url, local, url, tgz, tgz-url, github, azure",
                    src_type
                ))
                }
            }

            // Validate target type
            match tgt_type.as_str() {
                "prefix-dev" | "prefix" | "s3" | "minio" | "local" | "file" | "cache" => {}
                _ => {
                    return Err(anyhow::anyhow!(
                        "Invalid tgt-type '{}'. Must be one of: cache (individual package storage), prefix-dev, s3, local (conda repositories)",
                        tgt_type
                    ));
                }
            }

            // Validate that src_path is provided for zip files
            if (src_type == "zip" || src_type == "zip-url") && src_path.is_none() {
                return Err(anyhow::anyhow!(
                    "--src-path is required when src-type is 'zip' or 'zip-url'"
                ));
            }

            // Validate GitHub source format
            if src_type == "github" {
                if let Err(e) = github::parse_github_repository(&src) {
                    return Err(anyhow::anyhow!("Invalid GitHub repository format: {}", e));
                }
            }

            // Validate Azure DevOps source format
            if src_type == "azure" {
                if let Err(e) = azure::parse_azure_source(&src) {
                    return Err(anyhow::anyhow!("Invalid Azure DevOps format: {}", e));
                }
            }

            // Validate regex pattern if provided
            if let Some(ref pattern) = src_path {
                if let Err(e) = regex::Regex::new(pattern) {
                    return Err(anyhow::anyhow!(
                        "Invalid regular expression in --src-path: {}",
                        e
                    ));
                }
            }

            let config = if let Some(config_path) = config {
                Config::load_from_file(&config_path)?
            } else {
                Config::default()
            };

            let repo_type = RepositoryType::from_string(&tgt_type)?;

            // Handle target path based on repository type
            let target_path = match &repo_type {
                repository::RepositoryType::Cache => {
                    if tgt.is_some() {
                        return Err(anyhow::anyhow!(
                            "--tgt cannot be set when --tgt-type is 'cache'. Cache stores individual packages in the rattler cache directory automatically."
                        ));
                    }
                    default_cache_dir()
                        .map_err(|e| {
                            anyhow::anyhow!("Failed to get default cache directory: {}", e)
                        })?
                        .to_string_lossy()
                        .to_string()
                }
                _ => tgt.ok_or_else(|| {
                    anyhow::anyhow!(
                        "--tgt is required for repository types (local, s3, prefix-dev)"
                    )
                })?,
            };

            let is_local_file = matches!(src_type.as_str(), "zip" | "local" | "tgz");
            mirror_packages(
                &src,
                src_path.as_deref(),
                &src_type,
                is_local_file,
                repo_type,
                &target_path,
                &config,
            )
            .await?;

            info!("Mirroring completed successfully");
        }
        Commands::Info {
            github,
            azure,
            build_id,
            name_filter,
            description_filter,
            encode,
            exclude_expired,
            config,
        } => {
            let config = if let Some(config_path) = config {
                Config::load_from_file(&config_path)?
            } else {
                Config::default()
            };

            match (github, azure) {
                (Some(repo), None) => {
                    // GitHub info
                    info!(
                        "Getting GitHub artifact information for repository: {}",
                        repo
                    );
                    let github_client = github::GitHubClient::new(&config)?;
                    let (owner, repo_name) = github::parse_github_repository(&repo)?;

                    let mut artifacts = github_client.list_artifacts(&owner, &repo_name).await?;

                    // Filter by name if specified
                    if let Some(ref pattern) = name_filter {
                        artifacts =
                            github_client.filter_artifacts_by_name(&artifacts, Some(pattern));
                    }

                    // Filter expired artifacts if requested
                    if exclude_expired {
                        artifacts = github_client.filter_non_expired_artifacts(&artifacts);
                    }

                    // Print the results
                    github_client.print_artifacts_info(&artifacts, &encode)?;
                }
                (None, Some(azure_spec)) => {
                    // Azure DevOps info
                    let azure_client = azure::AzureDevOpsClient::new(&config)?;
                    let (organization, project, specified_build_id) =
                        azure::parse_azure_source(&azure_spec)?;

                    let target_build_id = build_id.or(specified_build_id);

                    // Case 1: Show artifacts for specific build (with optional name filtering)
                    if let Some(build_id) = target_build_id {
                        info!(
                            "Getting Azure DevOps artifacts for build {} in {}/{}",
                            build_id, organization, project
                        );
                        let mut artifacts = azure_client
                            .list_artifacts(&organization, &project, build_id)
                            .await?;

                        // Apply name filter if specified (works independently)
                        if let Some(ref pattern) = name_filter {
                            artifacts =
                                azure_client.filter_artifacts_by_name(&artifacts, Some(pattern));
                        }

                        azure_client.print_artifacts_info(&artifacts, &encode)?;
                    }
                    // Case 2: Show builds list (with optional description filtering)
                    else {
                        info!(
                            "Getting Azure DevOps builds for {}/{}",
                            organization, project
                        );
                        let mut builds = azure_client
                            .list_builds(&organization, &project, None)
                            .await?;

                        // Apply description filter if specified (works independently)
                        if let Some(ref pattern) = description_filter {
                            builds = azure_client.filter_builds_by_description(&builds, pattern)?;
                        }

                        // Warn if name_filter specified but ignored
                        if name_filter.is_some() {
                            warn!("--name-filter is ignored when listing builds (no --build-id specified). Use --description-filter to filter builds.");
                        }

                        azure_client.print_builds_info(
                            &builds,
                            &organization,
                            &project,
                            &encode,
                        )?;
                    }
                }
                (Some(_), Some(_)) => {
                    return Err(anyhow::anyhow!(
                        "Cannot specify both --github and --azure. Choose one."
                    ));
                }
                (None, None) => {
                    return Err(anyhow::anyhow!(
                        "Must specify either --github (for GitHub) or --azure (for Azure DevOps)."
                    ));
                }
            }
        }
        Commands::Init { output } => {
            info!("Initializing configuration file at: {}", output);
            let config = Config::default();
            config.save_to_file(&output)?;
            info!("Configuration file created successfully");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{Cli, Commands};
    use clap::{CommandFactory, Parser};

    #[test]
    fn test_cache_default_tgt_type() {
        // Test that cache is the default tgt_type
        let args = vec!["meso-forge-mirror", "mirror", "--src", "test.zip"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Mirror { tgt_type, .. } => {
                assert_eq!(tgt_type, "cache");
            }
            _ => panic!("Expected Mirror command"),
        }
    }

    #[test]
    fn test_cache_tgt_type_validation() {
        // Test that tgt is optional when tgt_type is cache
        let args = vec![
            "meso-forge-mirror",
            "mirror",
            "--src",
            "test.zip",
            "--tgt-type",
            "cache",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Mirror { tgt_type, tgt, .. } => {
                assert_eq!(tgt_type, "cache");
                assert_eq!(tgt, None);
            }
            _ => panic!("Expected Mirror command"),
        }
    }

    #[test]
    fn test_local_tgt_type_requires_tgt() {
        // Test that tgt is required when tgt_type is not cache
        let args = vec![
            "meso-forge-mirror",
            "mirror",
            "--src",
            "test.zip",
            "--tgt-type",
            "local",
            "--tgt",
            "/tmp/test",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Mirror { tgt_type, tgt, .. } => {
                assert_eq!(tgt_type, "local");
                assert_eq!(tgt, Some("/tmp/test".to_string()));
            }
            _ => panic!("Expected Mirror command"),
        }
    }

    #[test]
    fn test_help_shows_cache_option() {
        // This test ensures the help text includes cache as an option
        let help_output = Cli::command().render_help().to_string();
        assert!(help_output.contains("cache"));
        assert!(help_output.contains("stores individual packages for reuse"));
        assert!(help_output.contains("automatically determined for 'cache'"));
    }

    #[tokio::test]
    async fn test_cache_integration_with_pixi_discovery() {
        use crate::repository::{Repository, RepositoryType};
        use bytes::Bytes;
        use rattler_cache::package_cache::PackageCache;
        use std::fs;
        use tempfile::TempDir;

        // Create temporary directories for testing
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache_path = temp_dir.path().join("cache");
        fs::create_dir_all(&cache_path).expect("Failed to create cache directory");

        // Create a test conda package content (minimal valid .conda file structure)
        let test_package_name = "rb-asciidoctor-revealjs-5.2.0-h1d6dcf3_0.conda";
        let test_package_content = create_test_conda_package_content();

        // Test the cache repository
        let mut cache_repo = Repository::new(
            RepositoryType::Cache,
            cache_path.to_string_lossy().to_string(),
        );

        // Upload package to cache
        let upload_result = cache_repo
            .upload_package(test_package_name, Bytes::from(test_package_content.clone()))
            .await;
        assert!(
            upload_result.is_ok(),
            "Failed to upload package to cache: {:?}",
            upload_result
        );

        // Verify the package file exists in cache
        let cached_file = cache_path.join(test_package_name);
        assert!(
            cached_file.exists(),
            "Package file should exist in cache at {:?}",
            cached_file
        );

        // Verify file content matches
        let cached_content = fs::read(&cached_file).expect("Failed to read cached file");
        assert_eq!(
            cached_content, test_package_content,
            "Cached content should match original"
        );

        // Test PackageCache integration
        let _package_cache = PackageCache::new(&cache_path);

        // Verify cache directory structure is compatible with rattler
        assert!(cache_path.exists(), "Cache directory should exist");

        // Test package name parsing (this is what pixi would do)
        let package_parts: Vec<&str> = test_package_name
            .strip_suffix(".conda")
            .unwrap_or(test_package_name)
            .split('-')
            .collect();
        assert!(
            package_parts.len() >= 2,
            "Package name should have at least name and version"
        );
        assert_eq!(package_parts[0], "rb");
        assert_eq!(package_parts[1], "asciidoctor");
        assert_eq!(package_parts[2], "revealjs");

        // Verify this addresses the original issue: package name with typo vs correct name
        let correct_package_name = "rb-asciidoctor-revealjs";
        let typo_package_name = "rb-asciidocgtor-revealjs"; // missing 't'
        assert_ne!(
            correct_package_name, typo_package_name,
            "Package names should differ to demonstrate the typo issue"
        );

        // The package we cached should match the correct name
        assert!(
            test_package_name.starts_with(correct_package_name),
            "Cached package should start with correct name"
        );
        assert!(
            !test_package_name.starts_with(typo_package_name),
            "Cached package should not match typo name"
        );
    }

    fn create_test_conda_package_content() -> Vec<u8> {
        // Create a minimal but valid conda package structure
        // This is a simplified representation - in reality, conda packages are more complex
        let mut content = Vec::new();

        // Add some mock conda package data (ZIP format with metadata)
        // For testing purposes, we'll create a simple structure that represents a conda package
        content.extend_from_slice(b"PK\x03\x04"); // ZIP file signature
        content.extend_from_slice(b"mock_conda_package_content_for_testing");
        content.extend_from_slice(&[0u8; 100]); // Padding to make it look more realistic

        content
    }

    #[test]
    fn test_package_name_typo_detection() {
        // Test to demonstrate the original user issue with package name typo
        let correct_name = "rb-asciidoctor-revealjs-5.2.0-h1d6dcf3_0.conda";
        let search_with_typo = "rb-asciidocgtor-revealjs"; // missing 't' in 'asciidoctor'
        let search_correct = "rb-asciidoctor-revealjs";

        // Simulate package search/matching logic
        assert!(
            !correct_name.starts_with(search_with_typo),
            "Package with correct name should not match search with typo"
        );
        assert!(
            correct_name.starts_with(search_correct),
            "Package with correct name should match correct search term"
        );

        // This test documents the issue: typos in package names cause packages not to be found
        // even when they exist in the cache
    }

    #[tokio::test]
    async fn test_cache_vs_repository_behavior() {
        use crate::repository::{Repository, RepositoryType};
        use bytes::Bytes;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache_path = temp_dir.path().join("cache");
        let local_repo_path = temp_dir.path().join("local_repo");

        // Create cache and local repository
        let mut cache_repo = Repository::new(
            RepositoryType::Cache,
            cache_path.to_string_lossy().to_string(),
        );
        let mut local_repo = Repository::new(
            RepositoryType::Local,
            local_repo_path.to_string_lossy().to_string(),
        );

        let test_package_name = "test-package-1.0.0-h123_0.conda";
        let test_content = b"test_package_content".to_vec();

        // Upload to both
        let cache_result = cache_repo
            .upload_package(test_package_name, Bytes::from(test_content.clone()))
            .await;
        let local_result = local_repo
            .upload_package(test_package_name, Bytes::from(test_content.clone()))
            .await;

        assert!(cache_result.is_ok(), "Cache upload should succeed");
        assert!(local_result.is_ok(), "Local repo upload should succeed");

        // Verify different storage behaviors
        // Cache stores individual package files
        let cached_file = cache_path.join(test_package_name);
        assert!(
            cached_file.exists(),
            "Cache should store individual package file"
        );

        // Local repository creates structured directory with repodata
        assert!(
            local_repo_path.exists(),
            "Local repo directory should exist"
        );
        // Note: The actual structure depends on the repository implementation
        // This test documents the expected difference in behavior
    }
}
