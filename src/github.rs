use anyhow::{anyhow, Result};

use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, ContentArrangement, Table};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use tracing::{info, warn};

use crate::config::Config;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitHubArtifact {
    pub id: u64,
    pub name: String,
    pub size_in_bytes: u64,
    pub url: String,
    pub archive_download_url: String,
    pub expired: bool,
    pub created_at: String,
    pub updated_at: String,
    pub expires_at: String,
    pub workflow_run: Option<WorkflowRun>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowRun {
    pub id: u64,
    pub repository_id: u64,
    pub head_repository_id: Option<u64>,
    pub head_branch: String,
    pub head_sha: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubArtifactsResponse {
    pub total_count: u64,
    pub artifacts: Vec<GitHubArtifact>,
}

pub struct GitHubClient {
    client: Client,
    token: Option<String>,
}

impl GitHubClient {
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .user_agent("meso-forge-mirror/0.1.0")
            .build()?;

        Ok(Self {
            client,
            token: config.github_token.clone(),
        })
    }

    /// List all artifacts for a repository
    pub async fn list_artifacts(&self, owner: &str, repo: &str) -> Result<Vec<GitHubArtifact>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/actions/artifacts",
            owner, repo
        );

        let mut request = self.client.get(&url);

        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        request = request.header("Accept", "application/vnd.github+json");
        request = request.header("X-GitHub-Api-Version", "2022-11-28");

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to list GitHub artifacts: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        let artifacts_response: GitHubArtifactsResponse = response.json().await?;

        info!(
            "Found {} artifacts for {}/{}",
            artifacts_response.total_count, owner, repo
        );

        Ok(artifacts_response.artifacts)
    }

    /// Get a specific artifact by ID
    pub async fn get_artifact(
        &self,
        owner: &str,
        repo: &str,
        artifact_id: u64,
    ) -> Result<GitHubArtifact> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/actions/artifacts/{}",
            owner, repo, artifact_id
        );

        let mut request = self.client.get(&url);

        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        request = request.header("Accept", "application/vnd.github+json");
        request = request.header("X-GitHub-Api-Version", "2022-11-28");

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to get GitHub artifact {}: {} - {}",
                artifact_id,
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        let artifact: GitHubArtifact = response.json().await?;
        Ok(artifact)
    }

    /// Download an artifact by ID
    pub async fn download_artifact(
        &self,
        owner: &str,
        repo: &str,
        artifact_id: u64,
    ) -> Result<bytes::Bytes> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/actions/artifacts/{}/zip",
            owner, repo, artifact_id
        );

        let mut request = self.client.get(&url);

        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        request = request.header("Accept", "application/vnd.github+json");
        request = request.header("X-GitHub-Api-Version", "2022-11-28");

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to download GitHub artifact {}: {} - {}",
                artifact_id,
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        let content = response.bytes().await?;

        info!(
            "Downloaded artifact {} ({} bytes) from {}/{}",
            artifact_id,
            content.len(),
            owner,
            repo
        );

        Ok(content)
    }

    /// Filter artifacts by name pattern
    pub fn filter_artifacts_by_name(
        &self,
        artifacts: &[GitHubArtifact],
        pattern: Option<&str>,
    ) -> Vec<GitHubArtifact> {
        let artifacts = if let Some(pattern) = pattern {
            match regex::Regex::new(pattern) {
                Ok(regex) => {
                    let filtered: Vec<_> = artifacts
                        .iter()
                        .filter(|artifact| regex.is_match(&artifact.name))
                        .cloned()
                        .collect();

                    info!(
                        "Filtered {} artifacts to {} matching pattern '{}'",
                        artifacts.len(),
                        filtered.len(),
                        pattern
                    );

                    filtered
                }
                Err(e) => {
                    warn!("Invalid regex pattern '{}': {}", pattern, e);
                    artifacts.to_vec()
                }
            }
        } else {
            artifacts.to_vec()
        };

        artifacts
    }

    /// Filter artifacts to only include non-expired ones
    pub fn filter_non_expired_artifacts(
        &self,
        artifacts: &[GitHubArtifact],
    ) -> Vec<GitHubArtifact> {
        let non_expired: Vec<_> = artifacts
            .iter()
            .filter(|artifact| !artifact.expired)
            .cloned()
            .collect();

        if non_expired.len() != artifacts.len() {
            info!(
                "Filtered out {} expired artifacts, {} remaining",
                artifacts.len() - non_expired.len(),
                non_expired.len()
            );
        }

        non_expired
    }

    /// Print artifact information in a formatted way
    pub fn print_artifacts_info(&self, artifacts: &[GitHubArtifact], format: &str) -> Result<()> {
        match format.to_lowercase().as_str() {
            "yaml" => {
                // Add metadata header for better documentation
                println!("# GitHub Artifacts");
                println!("# Total artifacts found: {}", artifacts.len());
                println!("# Use --name-filter to filter artifacts by name pattern");
                println!("# Download URLs are available in archive_download_url field");
                println!();

                let yaml_output = serde_yaml::to_string(&artifacts)?;
                println!("{}", yaml_output);
            }
            "json" => {
                let json_output = serde_json::to_string_pretty(&artifacts)?;
                println!("{}", json_output);
            }
            "table" => {
                self.print_artifacts_info_table(artifacts);
            }
            _ => {
                return Err(anyhow!(
                    "Unsupported output format: {}. Supported formats: yaml, json, table",
                    format
                ));
            }
        }
        Ok(())
    }

    /// Print artifact information in table format using comfy-table
    fn print_artifacts_info_table(&self, artifacts: &[GitHubArtifact]) {
        if artifacts.is_empty() {
            println!("No artifacts found.");
            return;
        }

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("ID").add_attribute(Attribute::Bold),
                Cell::new("Name").add_attribute(Attribute::Bold),
                Cell::new("Size").add_attribute(Attribute::Bold),
                Cell::new("Created").add_attribute(Attribute::Bold),
                Cell::new("Expires").add_attribute(Attribute::Bold),
                Cell::new("Expired").add_attribute(Attribute::Bold),
            ]);

        for artifact in artifacts {
            let size_display = if artifact.size_in_bytes > 1_000_000 {
                format!("{:.1}M", artifact.size_in_bytes as f64 / 1_000_000.0)
            } else if artifact.size_in_bytes > 1_000 {
                format!("{:.1}K", artifact.size_in_bytes as f64 / 1_000.0)
            } else {
                artifact.size_in_bytes.to_string()
            };

            // Parse the created_at timestamp and format it
            let created_display = match chrono::DateTime::parse_from_rfc3339(&artifact.created_at) {
                Ok(dt) => dt.format("%Y-%m-%d %H:%M UTC").to_string(),
                Err(_) => artifact.created_at.clone(),
            };

            // Parse expires_at timestamp
            let expires_display = match chrono::DateTime::parse_from_rfc3339(&artifact.expires_at) {
                Ok(dt) => dt.format("%Y-%m-%d %H:%M UTC").to_string(),
                Err(_) => artifact.expires_at.clone(),
            };

            table.add_row(vec![
                Cell::new(artifact.id.to_string()),
                Cell::new(&artifact.name),
                Cell::new(&size_display),
                Cell::new(&created_display),
                Cell::new(&expires_display),
                Cell::new(if artifact.expired { "Yes" } else { "No" }),
            ]);
        }

        println!("\nFound {} artifacts:", artifacts.len());
        println!("{}", table);
    }
}

/// Parse GitHub repository from URL or owner/repo format
pub fn parse_github_repository(input: &str) -> Result<(String, String)> {
    // Handle GitHub URLs
    if input.starts_with("https://github.com/") || input.starts_with("http://github.com/") {
        let path = input
            .strip_prefix("https://github.com/")
            .or_else(|| input.strip_prefix("http://github.com/"))
            .unwrap();

        let parts: Vec<&str> = path.trim_end_matches('/').split('/').collect();
        if parts.len() >= 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }

    // Handle owner/repo format
    if let Some(slash_pos) = input.find('/') {
        let owner = input[..slash_pos].trim().to_string();
        let repo = input[slash_pos + 1..].trim().to_string();

        if !owner.is_empty() && !repo.is_empty() {
            return Ok((owner, repo));
        }
    }

    Err(anyhow!(
        "Invalid GitHub repository format. Expected 'owner/repo' or 'https://github.com/owner/repo'"
    ))
}

/// Parse artifact ID from string
pub fn parse_artifact_id(input: &str) -> Result<u64> {
    input
        .parse::<u64>()
        .map_err(|_| anyhow!("Invalid artifact ID: '{}'. Must be a number.", input))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_repository() {
        // Test owner/repo format
        let (owner, repo) = parse_github_repository("octocat/Hello-World").unwrap();
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "Hello-World");

        // Test GitHub URL formats
        let (owner, repo) =
            parse_github_repository("https://github.com/octocat/Hello-World").unwrap();
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "Hello-World");

        let (owner, repo) =
            parse_github_repository("https://github.com/octocat/Hello-World/").unwrap();
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "Hello-World");

        // Test invalid formats
        assert!(parse_github_repository("invalid").is_err());
        assert!(parse_github_repository("").is_err());
        assert!(parse_github_repository("/").is_err());
    }

    #[test]
    fn test_parse_artifact_id() {
        assert_eq!(parse_artifact_id("123456").unwrap(), 123456);
        assert!(parse_artifact_id("invalid").is_err());
        assert!(parse_artifact_id("").is_err());
    }
}
