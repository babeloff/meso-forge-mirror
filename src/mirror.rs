use anyhow::{anyhow, Result};
use bytes::Bytes;
use futures::stream::{self, StreamExt};
use reqwest::Client;
use tracing::{error, info, warn};
use url::Url;

use crate::config::Config;
use crate::repository::{Repository, RepositoryType};

pub async fn mirror_packages(
    sources: &[String],
    target_type: RepositoryType,
    target_path: &str,
    config: &Config,
) -> Result<()> {
    let repository = Repository::new(target_type, target_path.to_string());
    let client = build_client(config)?;

    info!("Starting mirroring of {} packages", sources.len());

    // Process packages concurrently
    let results = stream::iter(sources)
        .map(|source| {
            let client = client.clone();
            let repository = repository.clone();
            let config = config.clone();
            async move {
                mirror_single_package(&client, source, &repository, &config).await
            }
        })
        .buffer_unordered(config.max_concurrent_downloads)
        .collect::<Vec<_>>()
        .await;

    // Check for errors
    let mut success_count = 0;
    let mut error_count = 0;

    for result in results {
        match result {
            Ok(_) => success_count += 1,
            Err(e) => {
                error_count += 1;
                error!("Error mirroring package: {}", e);
            }
        }
    }

    info!(
        "Mirroring completed: {} succeeded, {} failed",
        success_count, error_count
    );

    if error_count > 0 {
        Err(anyhow!("{} packages failed to mirror", error_count))
    } else {
        Ok(())
    }
}

fn build_client(config: &Config) -> Result<Client> {
    let mut builder = Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout_seconds));

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
    repository: &Repository,
    config: &Config,
) -> Result<()> {
    info!("Mirroring package from: {}", source);

    // Download the package
    let content = download_package(client, source, config).await?;

    // Extract package name from URL
    let package_name = extract_package_name(source)?;

    // Upload to target repository
    repository.upload_package(&package_name, content).await?;

    info!("Successfully mirrored: {}", package_name);
    Ok(())
}

async fn download_package(client: &Client, url: &str, config: &Config) -> Result<Bytes> {
    let mut attempts = 0;
    let max_attempts = config.retry_attempts;

    loop {
        attempts += 1;
        info!("Downloading from {} (attempt {}/{})", url, attempts, max_attempts);

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

fn extract_package_name(url: &str) -> Result<String> {
    let parsed_url = Url::parse(url)?;
    let path = parsed_url.path();
    
    // Get the last segment of the path
    let package_name = path
        .split('/')
        .last()
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
    let path_segments: Vec<&str> = parsed_url.path().trim_start_matches('/').split('/').collect();
    
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
