use anyhow::{anyhow, Result};
use bytes::Bytes;
use flate2::read::GzDecoder;
use regex::Regex;
use reqwest::Client;
use std::io::Read;
use std::path::Path;
use tar::Archive;
use tracing::{error, info, warn};
use url::Url;

use crate::config::Config;
use crate::repository::{Repository, RepositoryType};

pub async fn mirror_packages(
    source: &str,
    zip_path: Option<&str>,
    source_type: &str,
    is_local_file: bool,
    target_type: RepositoryType,
    target_path: &str,
    config: &Config,
) -> Result<()> {
    let mut repository = Repository::new(target_type, target_path.to_string());
    let client = build_client(config)?;

    // Handle different source types
    match source_type {
        "zip" | "zip-url" => {
            info!(
                "Processing ZIP file source: {} (type: {})",
                source, source_type
            );
            let zip_path_str = zip_path.unwrap_or("");
            return mirror_from_zip(
                &client,
                source,
                zip_path_str,
                is_local_file,
                &mut repository,
                config,
            )
            .await;
        }
        "tgz" | "tgz-url" => {
            info!(
                "Processing tarball source: {} (type: {})",
                source, source_type
            );
            return mirror_from_tarball(&client, source, is_local_file, &mut repository, config)
                .await;
        }
        "local" | "url" => {
            info!(
                "Starting mirroring of single package: {} (type: {})",
                source, source_type
            );
            match mirror_single_package(&client, source, is_local_file, &mut repository, config)
                .await
            {
                Ok(_) => {
                    info!("Finalizing repository structure and generating metadata");
                    repository.finalize_repository().await?;
                    info!("Mirroring completed successfully");
                    Ok(())
                }
                Err(e) => {
                    error!("Error mirroring package: {}", e);
                    Err(e)
                }
            }
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported source type: {}. Must be one of: zip, zip-url, local, url, tgz, tgz-url",
                source_type
            ));
        }
    }
}

fn build_client(config: &Config) -> Result<Client> {
    let mut builder =
        Client::builder().timeout(std::time::Duration::from_secs(config.timeout_seconds));

    if let Some(token) = &config.github_token {
        let mut headers = reqwest::header::HeaderMap::new();
        let auth_value = format!("token {}", token);
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&auth_value)?,
        );
        builder = builder.default_headers(headers);
    }

    Ok(builder.build()?)
}

async fn mirror_single_package(
    client: &Client,
    source: &str,
    is_local_file: bool,
    repository: &mut Repository,
    config: &Config,
) -> Result<()> {
    info!("Mirroring package from: {}", source);

    // Get package content (either from URL or local file)
    let content = if is_local_file {
        info!("Reading local file: {}", source);
        let file_bytes = std::fs::read(source)
            .map_err(|e| anyhow!("Failed to read local file '{}': {}", source, e))?;
        info!(
            "Successfully read {} bytes from local file",
            file_bytes.len()
        );
        Bytes::from(file_bytes)
    } else {
        download_package(client, source, config).await?
    };

    // Extract package name from URL
    let package_name = extract_package_name(source)?;

    // Upload to target repository
    repository.upload_package(&package_name, content).await?;

    info!("Successfully mirrored: {}", package_name);
    Ok(())
}

async fn download_package(client: &Client, url: &str, config: &Config) -> Result<Bytes> {
    // Check if it's a local file path or file:// URL
    if url.starts_with("file://") || (!url.starts_with("http://") && !url.starts_with("https://")) {
        return download_local_file(url).await;
    }

    let mut attempts = 0;
    let max_attempts = config.retry_attempts;

    loop {
        attempts += 1;
        info!(
            "Downloading from {} (attempt {}/{})",
            url, attempts, max_attempts
        );

        match client.get(url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let content = response.bytes().await?;
                    info!("Successfully downloaded {} bytes", content.len());
                    return Ok(content);
                } else {
                    let status = response.status();
                    if attempts >= max_attempts {
                        return Err(anyhow!("Failed to download: HTTP {}", status));
                    }
                    warn!("Download failed with status {}, retrying...", status);
                }
            }
            Err(e) => {
                if attempts >= max_attempts {
                    return Err(anyhow!("Failed to download: {}", e));
                }
                warn!("Download error: {}, retrying...", e);
            }
        }

        // Wait before retrying
        tokio::time::sleep(std::time::Duration::from_secs(2_u64.pow(attempts - 1))).await;
    }
}

async fn download_local_file(url: &str) -> Result<Bytes> {
    let file_path = if url.starts_with("file://") {
        url.strip_prefix("file://").unwrap()
    } else {
        url
    };

    info!("Reading local file: {}", file_path);

    let path = Path::new(file_path);
    if !path.exists() {
        return Err(anyhow!("Local file does not exist: {}", file_path));
    }

    let content = tokio::fs::read(path).await?;
    let bytes = Bytes::from(content);
    info!("Successfully read {} bytes from local file", bytes.len());

    Ok(bytes)
}

async fn mirror_from_zip(
    client: &Client,
    source: &str,
    zip_path: &str,
    is_local_file: bool,
    repository: &mut Repository,
    config: &Config,
) -> Result<()> {
    // Get ZIP file content (either from URL or local file)
    let zip_content = if is_local_file {
        info!("Reading local file: {}", source);
        std::fs::read(source)
            .map_err(|e| anyhow!("Failed to read local file '{}': {}", source, e))?
            .into()
    } else {
        info!("Downloading ZIP file from: {}", source);
        download_package(client, source, config).await?
    };

    info!("Extracting conda packages from ZIP file");

    // Create a cursor from the downloaded bytes
    let cursor = std::io::Cursor::new(zip_content);
    let mut archive = zip::ZipArchive::new(cursor)?;

    let mut success_count = 0;
    let mut error_count = 0;
    let mut all_file_paths = Vec::new();

    // Compile regex pattern if provided
    let path_regex = if zip_path.is_empty() {
        None
    } else {
        Some(Regex::new(zip_path)?)
    };

    let mut first_match_processed = false;

    // Iterate through files in the ZIP
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_name = file.name().to_string();

        // Collect all file paths for potential debugging
        all_file_paths.push(file_name.clone());

        // Check if this file matches the regex pattern (if any) and is a conda package
        let is_in_path = if let Some(ref regex) = path_regex {
            regex.is_match(&file_name)
        } else {
            true
        };

        let is_conda_package = file_name.ends_with(".conda") || file_name.ends_with(".tar.bz2");

        // If using regex pattern, only process the first match
        let should_process = if path_regex.is_some() {
            is_in_path && is_conda_package && !first_match_processed
        } else {
            is_in_path && is_conda_package
        };

        if should_process {
            info!("Found conda package in ZIP: {}", file_name);

            // If using regex, mark that we've processed the first match
            if path_regex.is_some() {
                first_match_processed = true;
            }

            // Read the file content
            let mut content = Vec::new();
            file.read_to_end(&mut content)?;
            let content_bytes = Bytes::from(content);

            // Extract just the filename for the package name
            let package_name = std::path::Path::new(&file_name)
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| anyhow!("Could not extract package name from: {}", file_name))?;

            // Upload to repository
            match repository.upload_package(package_name, content_bytes).await {
                Ok(_) => {
                    success_count += 1;
                    info!("Successfully extracted and mirrored: {}", package_name);
                }
                Err(e) => {
                    error_count += 1;
                    error!("Error mirroring package {}: {}", package_name, e);
                }
            }

            // If using regex, stop after processing the first match
            if path_regex.is_some() {
                break;
            }
        }
    }

    info!(
        "ZIP processing completed: {} succeeded, {} failed",
        success_count, error_count
    );

    // Finalize repository structure
    if success_count > 0 {
        info!("Finalizing repository structure and generating metadata");
        repository.finalize_repository().await?;
    }

    if error_count > 0 {
        Err(anyhow!("{} packages failed to mirror", error_count))
    } else if success_count == 0 {
        let mut error_msg = format!(
            "No conda packages found in ZIP file matching pattern: '{}'",
            if zip_path.is_empty() {
                "<root>"
            } else {
                zip_path
            }
        );

        error_msg.push_str("\n\nAll files in ZIP:");
        for (i, path) in all_file_paths.iter().enumerate() {
            error_msg.push_str(&format!("\n  {}: {}", i + 1, path));
        }

        if !zip_path.is_empty() {
            error_msg.push_str(&format!(
                "\n\nHint: File paths must match regex pattern '{}' and have .conda or .tar.bz2 extensions",
                zip_path
            ));
        } else {
            error_msg.push_str("\n\nHint: Files must have .conda or .tar.bz2 extensions");
        }

        Err(anyhow!(error_msg))
    } else {
        Ok(())
    }
}

async fn mirror_from_tarball(
    client: &Client,
    source: &str,
    is_local_file: bool,
    repository: &mut Repository,
    config: &Config,
) -> Result<()> {
    // Get tarball content (either from URL or local file)
    let tarball_content = if is_local_file {
        info!("Reading local tarball: {}", source);
        std::fs::read(source)
            .map_err(|e| anyhow!("Failed to read local file '{}': {}", source, e))?
            .into()
    } else {
        info!("Downloading tarball from: {}", source);
        download_package(client, source, config).await?
    };

    info!("Extracting conda packages from tarball");

    // Create a cursor from the downloaded bytes
    let cursor = std::io::Cursor::new(tarball_content);
    let tar = GzDecoder::new(cursor);
    let mut archive = Archive::new(tar);

    let mut success_count = 0;
    let mut error_count = 0;
    let mut all_file_paths = Vec::new();

    // Iterate through files in the tarball
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        let file_name = path.to_string_lossy().to_string();

        // Collect all file paths for potential debugging
        all_file_paths.push(file_name.clone());

        // Check if this file is a conda package
        let is_conda_package = file_name.ends_with(".conda") || file_name.ends_with(".tar.bz2");

        if is_conda_package {
            info!("Found conda package in tarball: {}", file_name);

            // Read the file content
            let mut content = Vec::new();
            entry.read_to_end(&mut content)?;
            let content_bytes = Bytes::from(content);

            // Extract just the filename for the package name
            let package_name = std::path::Path::new(&file_name)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();

            // Upload the package
            match repository
                .upload_package(&package_name, content_bytes)
                .await
            {
                Ok(_) => {
                    info!("Successfully uploaded: {}", package_name);
                    success_count += 1;
                }
                Err(e) => {
                    error!("Failed to upload {}: {}", package_name, e);
                    error_count += 1;
                }
            }
        }
    }

    info!(
        "Tarball processing completed: {} succeeded, {} failed",
        success_count, error_count
    );

    // Finalize repository structure
    if success_count > 0 {
        repository.finalize_repository().await?;
    }

    if success_count == 0 {
        let mut error_msg = "No conda packages found in tarball".to_string();

        error_msg.push_str("\n\nAll files in tarball:");
        for (i, path) in all_file_paths.iter().enumerate() {
            error_msg.push_str(&format!("\n  {}: {}", i + 1, path));
        }

        error_msg.push_str("\n\nHint: Files must have .conda or .tar.bz2 extensions");

        Err(anyhow!(error_msg))
    } else {
        Ok(())
    }
}

fn extract_package_name(source: &str) -> Result<String> {
    // Handle local file paths
    if !source.starts_with("http://")
        && !source.starts_with("https://")
        && !source.starts_with("file://")
    {
        let path = Path::new(source);
        let package_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("Could not extract package name from file path"))?;
        return Ok(package_name.to_string());
    }

    // Handle URLs
    let parsed_url = Url::parse(source)?;
    let path = parsed_url.path();

    // Get the last segment of the path
    let package_name = path
        .split('/')
        .next_back()
        .ok_or_else(|| anyhow!("Could not extract package name from URL"))?;

    if package_name.is_empty() {
        return Err(anyhow!("Package name is empty"));
    }

    Ok(package_name.to_string())
}

// Helper function to resolve GitHub artifact URLs from PRs
#[allow(dead_code)]
pub async fn resolve_github_pr_artifacts(pr_url: &str, config: &Config) -> Result<Vec<String>> {
    info!("Resolving artifacts from PR: {}", pr_url);

    // Parse PR URL to extract owner, repo, and PR number
    let parsed_url = Url::parse(pr_url)?;
    let path_segments: Vec<&str> = parsed_url
        .path()
        .trim_start_matches('/')
        .split('/')
        .collect();

    if path_segments.len() < 4 || path_segments[2] != "pull" {
        return Err(anyhow!("Invalid GitHub PR URL format"));
    }

    let owner = path_segments[0];
    let repo = path_segments[1];
    let pr_number = path_segments[3].trim_end_matches('/');

    // Use GitHub API to get artifacts
    let _client = build_client(config)?;
    let _api_url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}/checks",
        owner, repo, pr_number
    );

    info!("Fetching PR artifacts from GitHub API");

    // Note: This is a simplified version. In practice, you'd need to:
    // 1. Get the PR details
    // 2. Find associated CI runs
    // 3. Download artifacts from those runs
    // For now, return empty list as placeholder
    warn!("GitHub artifact resolution is not fully implemented yet");
    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_package_name() {
        let url = "https://example.com/packages/my-package-1.0.0.tar.bz2";
        let name = extract_package_name(url).unwrap();
        assert_eq!(name, "my-package-1.0.0.tar.bz2");
    }

    #[test]
    fn test_extract_package_name_with_query() {
        let url = "https://example.com/packages/my-package.tar.bz2?token=abc";
        let name = extract_package_name(url).unwrap();
        assert_eq!(name, "my-package.tar.bz2");
    }

    #[test]
    fn test_extract_package_name_empty() {
        let url = "https://example.com/";
        assert!(extract_package_name(url).is_err());
    }
}
