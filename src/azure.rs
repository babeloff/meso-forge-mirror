use anyhow::{anyhow, Result};
use comfy_table::presets::NOTHING;
use comfy_table::{Attribute, Cell, ContentArrangement, Table};

use reqwest::Client;
use serde::{Deserialize, Serialize};

use tracing::{info, warn};

use crate::config::Config;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AzureDevOpsArtifact {
    pub id: u64,
    pub name: String,
    pub source: String,
    pub resource: ArtifactResource,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArtifactResource {
    #[serde(rename = "type")]
    pub artifact_type: String,
    pub data: String,
    pub properties: Option<ArtifactProperties>,
    pub url: String,
    #[serde(rename = "downloadUrl")]
    pub download_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArtifactProperties {
    #[serde(rename = "RootId")]
    pub root_id: Option<String>,
    pub artifactsize: Option<String>,
    #[serde(rename = "HashType")]
    pub hash_type: Option<String>,
    #[serde(rename = "DomainId")]
    pub domain_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AzureDevOpsArtifactsResponse {
    pub count: u64,
    pub value: Vec<AzureDevOpsArtifact>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AzureDevOpsBuild {
    pub id: u64,
    pub build_number: Option<String>,
    pub status: String,
    pub result: Option<String>,
    pub queue_time: Option<String>,
    pub start_time: Option<String>,
    pub finish_time: Option<String>,
    pub url: Option<String>,
    pub definition: BuildDefinition,
    pub project: Project,
    pub source_branch: Option<String>,
    pub source_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildDefinition {
    pub id: u64,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AzureDevOpsBuildsResponse {
    pub count: u64,
    pub value: Vec<AzureDevOpsBuild>,
}

pub struct AzureDevOpsClient {
    client: Client,
    token: Option<String>,
}

impl AzureDevOpsClient {
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .user_agent("meso-forge-mirror/0.1.0")
            .build()?;

        Ok(Self {
            client,
            token: config.azure_devops_token.clone(),
        })
    }

    /// List artifacts for a specific build
    pub async fn list_artifacts(
        &self,
        organization: &str,
        project: &str,
        build_id: u64,
    ) -> Result<Vec<AzureDevOpsArtifact>> {
        let url = format!(
            "https://dev.azure.com/{}/{}/_apis/build/builds/{}/artifacts?api-version=6.0",
            organization, project, build_id
        );

        let mut request = self.client.get(&url);

        if let Some(token) = &self.token {
            // Azure DevOps uses basic auth with empty username and PAT as password
            request = request.basic_auth("", Some(token));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Failed to list Azure DevOps artifacts: {} - {}",
                status,
                error_text
            ));
        }

        // Get the response text first to provide better error messages
        let response_text = response.text().await?;

        // Try to parse as JSON, providing the raw text if it fails
        let artifacts_response: AzureDevOpsArtifactsResponse = match serde_json::from_str(
            &response_text,
        ) {
            Ok(response) => response,
            Err(e) => {
                // Log more lines of the response for better debugging
                let preview = response_text
                    .lines()
                    .take(30)
                    .collect::<Vec<_>>()
                    .join("\n");

                // Provide specific guidance based on response content
                let guidance = if response_text.contains("<html")
                    || response_text.contains("<!DOCTYPE html")
                {
                    if response_text.contains("_signin") || response_text.contains("login") {
                        "\n\nThis appears to be an authentication redirect. Azure DevOps requires a Personal Access Token (PAT).\nSolution: Create a config file with your PAT:\n  {\n    \"azure_devops_token\": \"your_pat_here\"\n  }\nGet PAT from: https://dev.azure.com/ → Security → Personal Access Tokens"
                    } else {
                        "\n\nReceived HTML instead of JSON. This usually indicates an authentication or API endpoint issue."
                    }
                } else {
                    "\n\nExpected JSON response from Azure DevOps API."
                };

                return Err(anyhow!(
                    "Failed to parse Azure DevOps artifacts response as JSON: {}\nResponse preview:\n{}\n{}",
                    e,
                    preview,
                    guidance
                ));
            }
        };

        info!(
            "Found {} artifacts for build {} in {}/{}",
            artifacts_response.count, build_id, organization, project
        );

        Ok(artifacts_response.value)
    }

    /// List recent builds for a project
    pub async fn list_builds(
        &self,
        organization: &str,
        project: &str,
        definition_id: Option<u64>,
    ) -> Result<Vec<AzureDevOpsBuild>> {
        let mut url = format!(
            "https://dev.azure.com/{}/{}/_apis/build/builds?api-version=6.0&$top=50&statusFilter=completed",
            organization, project
        );

        if let Some(def_id) = definition_id {
            url.push_str(&format!("&definitions={}", def_id));
        }

        let mut request = self.client.get(&url);

        if let Some(token) = &self.token {
            request = request.basic_auth("", Some(token));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Failed to list Azure DevOps builds: {} - {}",
                status,
                error_text
            ));
        }

        // Get the response text first to provide better error messages
        let response_text = response.text().await?;

        // Try to parse as JSON, providing the raw text if it fails
        let builds_response: AzureDevOpsBuildsResponse = match serde_json::from_str(&response_text)
        {
            Ok(response) => response,
            Err(e) => {
                // Log more lines of the response for better debugging
                let preview = response_text
                    .lines()
                    .take(30)
                    .collect::<Vec<_>>()
                    .join("\n");

                // Provide specific guidance based on response content
                let guidance = if response_text.contains("<html")
                    || response_text.contains("<!DOCTYPE html")
                {
                    if response_text.contains("_signin") || response_text.contains("login") {
                        "\n\nThis appears to be an authentication redirect. Azure DevOps requires a Personal Access Token (PAT).\nSolution: Create a config file with your PAT:\n  {\n    \"azure_devops_token\": \"your_pat_here\"\n  }\nGet PAT from: https://dev.azure.com/ → Security → Personal Access Tokens"
                    } else {
                        "\n\nReceived HTML instead of JSON. This usually indicates an authentication or API endpoint issue."
                    }
                } else {
                    "\n\nExpected JSON response from Azure DevOps API."
                };

                return Err(anyhow!(
                    "Failed to parse Azure DevOps builds response as JSON: {}\nResponse preview:\n{}\n{}",
                    e,
                    preview,
                    guidance
                ));
            }
        };

        info!(
            "Found {} builds in {}/{}",
            builds_response.count, organization, project
        );

        Ok(builds_response.value)
    }

    /// Download an artifact
    pub async fn download_artifact(
        &self,
        organization: &str,
        project: &str,
        build_id: u64,
        artifact_name: &str,
    ) -> Result<bytes::Bytes> {
        let url = format!(
            "https://dev.azure.com/{}/{}/_apis/build/builds/{}/artifacts?artifactName={}&$format=zip&api-version=6.0",
            organization, project, build_id, artifact_name
        );

        let mut request = self.client.get(&url);

        if let Some(token) = &self.token {
            request = request.basic_auth("", Some(token));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to download Azure DevOps artifact {}: {} - {}",
                artifact_name,
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        let content = response.bytes().await?;

        info!(
            "Downloaded artifact {} ({} bytes) from build {} in {}/{}",
            artifact_name,
            content.len(),
            build_id,
            organization,
            project
        );

        Ok(content)
    }

    /// Filter artifacts by name pattern
    pub fn filter_artifacts_by_name(
        &self,
        artifacts: &[AzureDevOpsArtifact],
        pattern: Option<&str>,
    ) -> Vec<AzureDevOpsArtifact> {
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

    /// Filter artifacts by type (e.g., "PipelineArtifact", "Container")
    #[allow(dead_code)]
    pub fn filter_artifacts_by_type(
        &self,
        artifacts: &[AzureDevOpsArtifact],
        artifact_type: Option<&str>,
    ) -> Vec<AzureDevOpsArtifact> {
        let artifacts = if let Some(filter_type) = artifact_type {
            let filtered: Vec<_> = artifacts
                .iter()
                .filter(|artifact| {
                    artifact
                        .resource
                        .artifact_type
                        .eq_ignore_ascii_case(filter_type)
                })
                .cloned()
                .collect();

            info!(
                "Filtered {} artifacts to {} of type '{}'",
                artifacts.len(),
                filtered.len(),
                filter_type
            );

            filtered
        } else {
            artifacts.to_vec()
        };

        artifacts
    }

    /// Filter builds by description pattern (definition name)
    pub fn filter_builds_by_description(
        &self,
        builds: &[AzureDevOpsBuild],
        pattern: &str,
    ) -> Result<Vec<AzureDevOpsBuild>> {
        let regex = regex::Regex::new(pattern)?;

        let filtered: Vec<AzureDevOpsBuild> = builds
            .iter()
            .filter(|build| regex.is_match(&build.definition.name))
            .cloned()
            .collect();

        info!(
            "Filtered {} builds to {} builds matching description pattern '{}'",
            builds.len(),
            filtered.len(),
            pattern
        );

        Ok(filtered)
    }

    /// Print artifact information in a formatted way
    pub fn print_artifacts_info(
        &self,
        artifacts: &[AzureDevOpsArtifact],
        format: &str,
    ) -> Result<()> {
        match format.to_lowercase().as_str() {
            "yaml" => {
                // Add metadata header for better documentation
                println!("# Azure DevOps Artifacts");
                println!("# Total artifacts found: {}", artifacts.len());
                println!("# Use --name-filter to filter artifacts by name pattern");
                println!("# Download URLs are available in resource.download_url field");
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
    fn print_artifacts_info_table(&self, artifacts: &[AzureDevOpsArtifact]) {
        if artifacts.is_empty() {
            println!("No artifacts found.");
            return;
        }

        let mut table = Table::new();
        table
            .load_preset(NOTHING)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("ID").add_attribute(Attribute::Bold),
                Cell::new("Name").add_attribute(Attribute::Bold),
                Cell::new("Type").add_attribute(Attribute::Bold),
                Cell::new("Size").add_attribute(Attribute::Bold),
                Cell::new("Source").add_attribute(Attribute::Bold),
                Cell::new("Download Available").add_attribute(Attribute::Bold),
            ]);

        for artifact in artifacts {
            let size_display = if let Some(ref props) = artifact.resource.properties {
                if let Some(ref size_str) = props.artifactsize {
                    if let Ok(size_bytes) = size_str.parse::<u64>() {
                        if size_bytes > 1_000_000 {
                            format!("{:.1}M", size_bytes as f64 / 1_000_000.0)
                        } else if size_bytes > 1_000 {
                            format!("{:.1}K", size_bytes as f64 / 1_000.0)
                        } else {
                            size_bytes.to_string()
                        }
                    } else {
                        size_str.clone()
                    }
                } else {
                    "Unknown".to_string()
                }
            } else {
                "Unknown".to_string()
            };

            let download_available = if artifact.resource.download_url.is_some() {
                "Yes"
            } else {
                "No"
            };

            table.add_row(vec![
                Cell::new(artifact.id.to_string()),
                Cell::new(&artifact.name),
                Cell::new(&artifact.resource.artifact_type),
                Cell::new(&size_display),
                Cell::new(&artifact.source),
                Cell::new(download_available),
            ]);
        }

        println!("\nFound {} artifacts:", artifacts.len());
        println!("{}", table);
    }

    /// Print builds information in a formatted way with mirror command examples
    pub fn print_builds_info(
        &self,
        builds: &[AzureDevOpsBuild],
        organization: &str,
        project: &str,
        format: &str,
    ) -> Result<()> {
        match format.to_lowercase().as_str() {
            "yaml" => {
                // Add metadata header for better documentation
                println!("# Azure DevOps Builds for {}/{}", organization, project);
                println!("# Total builds found: {}", builds.len());
                println!("# Use 'meso-forge-mirror mirror --src-type azure --src {}/{}#<build_id>' to mirror artifacts", organization, project);
                println!("# Filter with --description-filter to narrow results");
                println!();

                let yaml_output = serde_yaml::to_string(&builds)?;
                println!("{}", yaml_output);
            }
            "json" => {
                let json_output = serde_json::to_string_pretty(&builds)?;
                println!("{}", json_output);
            }
            "table" => {
                self.print_builds_info_table(builds, organization, project);
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

    /// Print builds information in table format using comfy-table with mirror command examples
    fn print_builds_info_table(
        &self,
        builds: &[AzureDevOpsBuild],
        organization: &str,
        project: &str,
    ) {
        if builds.is_empty() {
            println!("No builds found.");
            return;
        }

        let mut table = Table::new();
        table
            .load_preset(NOTHING)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Build ID").add_attribute(Attribute::Bold),
                Cell::new("Build Number").add_attribute(Attribute::Bold),
                Cell::new("Status").add_attribute(Attribute::Bold),
                Cell::new("Result").add_attribute(Attribute::Bold),
                Cell::new("Definition").add_attribute(Attribute::Bold),
                Cell::new("Source Branch").add_attribute(Attribute::Bold),
                Cell::new("Finish Time").add_attribute(Attribute::Bold),
                Cell::new("Mirror Source").add_attribute(Attribute::Bold),
            ]);

        for build in builds {
            let finish_time = build
                .finish_time
                .as_ref()
                .and_then(|time| chrono::DateTime::parse_from_rfc3339(time).ok())
                .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
                .unwrap_or_else(|| "In Progress".to_string());

            let result_display = build.result.as_deref().unwrap_or("N/A");
            let src_value = format!("{}/{}#{}", organization, project, build.id);
            let source_branch_display = build.source_branch.as_deref().unwrap_or("N/A");
            let build_number_display = build.build_number.as_deref().unwrap_or("N/A");

            table.add_row(vec![
                Cell::new(build.id.to_string()),
                Cell::new(build_number_display),
                Cell::new(&build.status),
                Cell::new(result_display),
                Cell::new(&build.definition.name),
                Cell::new(source_branch_display),
                Cell::new(&finish_time),
                Cell::new(&src_value),
            ]);
        }

        println!(
            "\nFound {} builds for {}/{}:",
            builds.len(),
            organization,
            project
        );
        println!("{}", table);

        // Show example mirror commands for the most recent successful builds
        let successful_builds: Vec<_> = builds
            .iter()
            .filter(|b| b.result.as_deref() == Some("succeeded") && b.status == "completed")
            .take(3)
            .collect();

        if !successful_builds.is_empty() {
            println!("\nExample mirror commands for recent successful builds:");
            println!();

            for (i, build) in successful_builds.iter().enumerate() {
                let build_desc = match &build.build_number {
                    Some(num) => format!("Build {} ({})", build.id, num),
                    None => format!("Build {}", build.id),
                };

                println!("{}. {}:", i + 1, build_desc);
                println!("   # Mirror all artifacts:");
                println!(
                    "   meso-forge-mirror mirror --src-type azure --src {}/{}#{}",
                    organization, project, build.id
                );
                println!("   # Mirror only conda packages:");
                println!("   meso-forge-mirror mirror --src-type azure --src {}/{}#{} --src-path 'conda.*'", organization, project, build.id);
                println!("   # Mirror specific platform packages:");
                println!("   meso-forge-mirror mirror --src-type azure --src {}/{}#{} --src-path '.*linux-64.*'", organization, project, build.id);
                println!();
            }
        }
    }
}

/// Parse Azure DevOps organization/project from various formats
pub fn parse_azure_devops_url(input: &str) -> Result<(String, String)> {
    // Handle Azure DevOps URLs
    if input.starts_with("https://dev.azure.com/") {
        let path = input.strip_prefix("https://dev.azure.com/").unwrap();

        let parts: Vec<&str> = path.trim_end_matches('/').split('/').collect();
        if parts.len() >= 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }

    // Handle organization/project format
    if let Some(slash_pos) = input.find('/') {
        let organization = input[..slash_pos].trim().to_string();
        let project = input[slash_pos + 1..].trim().to_string();

        if !organization.is_empty() && !project.is_empty() {
            return Ok((organization, project));
        }
    }

    Err(anyhow!(
        "Invalid Azure DevOps format. Expected 'organization/project' or 'https://dev.azure.com/organization/project'"
    ))
}

/// Parse build ID from string
pub fn parse_build_id(input: &str) -> Result<u64> {
    input
        .parse::<u64>()
        .map_err(|_| anyhow!("Invalid build ID: '{}'. Must be a number.", input))
}

/// Parse Azure DevOps source string with optional build ID
/// Formats supported:
/// - organization/project
/// - organization/project#build_id
/// - https://dev.azure.com/organization/project
/// - https://dev.azure.com/organization/project#build_id
pub fn parse_azure_source(input: &str) -> Result<(String, String, Option<u64>)> {
    let (url_part, build_id) = if let Some(hash_pos) = input.find('#') {
        let url_part = &input[..hash_pos];
        let build_id_str = &input[hash_pos + 1..];
        let build_id = parse_build_id(build_id_str)?;
        (url_part, Some(build_id))
    } else {
        (input, None)
    };

    let (organization, project) = parse_azure_devops_url(url_part)?;
    Ok((organization, project, build_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_azure_devops_url() {
        // Test organization/project format
        let (org, proj) = parse_azure_devops_url("conda-forge/feedstock-builds").unwrap();
        assert_eq!(org, "conda-forge");
        assert_eq!(proj, "feedstock-builds");

        // Test Azure DevOps URL formats
        let (org, proj) =
            parse_azure_devops_url("https://dev.azure.com/conda-forge/feedstock-builds").unwrap();
        assert_eq!(org, "conda-forge");
        assert_eq!(proj, "feedstock-builds");

        let (org, proj) =
            parse_azure_devops_url("https://dev.azure.com/conda-forge/feedstock-builds/").unwrap();
        assert_eq!(org, "conda-forge");
        assert_eq!(proj, "feedstock-builds");

        // Test invalid formats
        assert!(parse_azure_devops_url("invalid").is_err());
        assert!(parse_azure_devops_url("").is_err());
        assert!(parse_azure_devops_url("/").is_err());
    }

    #[test]
    fn test_parse_build_id() {
        assert_eq!(parse_build_id("123456").unwrap(), 123456);
        assert!(parse_build_id("invalid").is_err());
        assert!(parse_build_id("").is_err());
    }

    #[test]
    fn test_parse_azure_source() {
        // Test without build ID
        let (org, proj, build_id) = parse_azure_source("conda-forge/feedstock-builds").unwrap();
        assert_eq!(org, "conda-forge");
        assert_eq!(proj, "feedstock-builds");
        assert_eq!(build_id, None);

        // Test with build ID
        let (org, proj, build_id) =
            parse_azure_source("conda-forge/feedstock-builds#123456").unwrap();
        assert_eq!(org, "conda-forge");
        assert_eq!(proj, "feedstock-builds");
        assert_eq!(build_id, Some(123456));

        // Test with URL and build ID
        let (org, proj, build_id) =
            parse_azure_source("https://dev.azure.com/conda-forge/feedstock-builds#123456")
                .unwrap();
        assert_eq!(org, "conda-forge");
        assert_eq!(proj, "feedstock-builds");
        assert_eq!(build_id, Some(123456));
    }

    #[test]
    fn test_print_builds_info_enhanced_output() {
        // Test the enhanced print_builds_info functionality

        let client = AzureDevOpsClient {
            client: reqwest::Client::new(),
            token: None,
        };

        // Create mock builds with different statuses
        let builds = vec![
            AzureDevOpsBuild {
                id: 1374331,
                build_number: Some("PR.31205.1".to_string()),
                status: "completed".to_string(),
                result: Some("succeeded".to_string()),
                queue_time: Some("2024-10-23T10:00:00Z".to_string()),
                start_time: Some("2024-10-23T10:05:00Z".to_string()),
                finish_time: Some("2024-10-23T10:30:00Z".to_string()),
                url: Some("https://dev.azure.com/conda-forge/feedstock-builds/_build/results?buildId=1374331".to_string()),
                definition: BuildDefinition {
                    id: 1,
                    name: "feedstock-builds".to_string(),
                    url: "https://dev.azure.com/conda-forge/feedstock-builds/_definition?definitionId=1".to_string(),
                },
                project: Project {
                    id: "project-id".to_string(),
                    name: "feedstock-builds".to_string(),
                    url: "https://dev.azure.com/conda-forge/feedstock-builds".to_string(),
                },
                source_branch: Some("refs/pull/31205/merge".to_string()),
                source_version: Some("abc123def456".to_string()),
            },
            AzureDevOpsBuild {
                id: 1374330,
                build_number: None, // Test missing build_number
                status: "completed".to_string(),
                result: Some("succeeded".to_string()),
                queue_time: Some("2024-10-23T09:00:00Z".to_string()),
                start_time: Some("2024-10-23T09:05:00Z".to_string()),
                finish_time: Some("2024-10-23T09:30:00Z".to_string()),
                url: Some("https://dev.azure.com/conda-forge/feedstock-builds/_build/results?buildId=1374330".to_string()),
                definition: BuildDefinition {
                    id: 1,
                    name: "feedstock-builds".to_string(),
                    url: "https://dev.azure.com/conda-forge/feedstock-builds/_definition?definitionId=1".to_string(),
                },
                project: Project {
                    id: "project-id".to_string(),
                    name: "feedstock-builds".to_string(),
                    url: "https://dev.azure.com/conda-forge/feedstock-builds".to_string(),
                },
                source_branch: Some("refs/heads/main".to_string()),
                source_version: Some("def456ghi789".to_string()),
            },
            AzureDevOpsBuild {
                id: 1374329,
                build_number: Some("main.1".to_string()),
                status: "inProgress".to_string(),
                result: None,
                queue_time: Some("2024-10-23T11:00:00Z".to_string()),
                start_time: Some("2024-10-23T11:05:00Z".to_string()),
                finish_time: None,
                url: Some("https://dev.azure.com/conda-forge/feedstock-builds/_build/results?buildId=1374329".to_string()),
                definition: BuildDefinition {
                    id: 1,
                    name: "feedstock-builds".to_string(),
                    url: "https://dev.azure.com/conda-forge/feedstock-builds/_definition?definitionId=1".to_string(),
                },
                project: Project {
                    id: "project-id".to_string(),
                    name: "feedstock-builds".to_string(),
                    url: "https://dev.azure.com/conda-forge/feedstock-builds".to_string(),
                },
                source_branch: Some("refs/heads/main".to_string()),
                source_version: Some("ghi789jkl012".to_string()),
            },
        ];

        // This test mainly verifies that the function doesn't panic and handles optional fields correctly
        // In a real scenario, this would print to stdout, but in tests we just verify it executes
        client
            .print_builds_info(&builds, "conda-forge", "feedstock-builds", "table")
            .unwrap();

        // Test with empty builds list
        client
            .print_builds_info(&[], "conda-forge", "feedstock-builds", "table")
            .unwrap();
    }

    #[test]
    fn test_filter_builds_by_description() {
        let client = AzureDevOpsClient {
            client: reqwest::Client::new(),
            token: None,
        };

        // Create builds with different definition names
        let builds = vec![
            AzureDevOpsBuild {
                id: 1374331,
                build_number: Some("PR.31205.1".to_string()),
                status: "completed".to_string(),
                result: Some("succeeded".to_string()),
                queue_time: Some("2024-10-23T10:00:00Z".to_string()),
                start_time: Some("2024-10-23T10:05:00Z".to_string()),
                finish_time: Some("2024-10-23T10:30:00Z".to_string()),
                url: Some("https://dev.azure.com/conda-forge/feedstock-builds/_build/results?buildId=1374331".to_string()),
                definition: BuildDefinition {
                    id: 1,
                    name: "numpy-feedstock CI".to_string(),
                    url: "https://dev.azure.com/conda-forge/feedstock-builds/_definition?definitionId=1".to_string(),
                },
                project: Project {
                    id: "project-id".to_string(),
                    name: "feedstock-builds".to_string(),
                    url: "https://dev.azure.com/conda-forge/feedstock-builds".to_string(),
                },
                source_branch: Some("refs/pull/31205/merge".to_string()),
                source_version: Some("abc123def456".to_string()),
            },
            AzureDevOpsBuild {
                id: 1374330,
                build_number: Some("PR.31204.1".to_string()),
                status: "completed".to_string(),
                result: Some("succeeded".to_string()),
                queue_time: Some("2024-10-23T09:00:00Z".to_string()),
                start_time: Some("2024-10-23T09:05:00Z".to_string()),
                finish_time: Some("2024-10-23T09:30:00Z".to_string()),
                url: Some("https://dev.azure.com/conda-forge/feedstock-builds/_build/results?buildId=1374330".to_string()),
                definition: BuildDefinition {
                    id: 2,
                    name: "pandas-feedstock CI".to_string(),
                    url: "https://dev.azure.com/conda-forge/feedstock-builds/_definition?definitionId=2".to_string(),
                },
                project: Project {
                    id: "project-id".to_string(),
                    name: "feedstock-builds".to_string(),
                    url: "https://dev.azure.com/conda-forge/feedstock-builds".to_string(),
                },
                source_branch: Some("refs/heads/main".to_string()),
                source_version: Some("def456ghi789".to_string()),
            },
            AzureDevOpsBuild {
                id: 1374329,
                build_number: Some("main.1".to_string()),
                status: "completed".to_string(),
                result: Some("succeeded".to_string()),
                queue_time: Some("2024-10-23T11:00:00Z".to_string()),
                start_time: Some("2024-10-23T11:05:00Z".to_string()),
                finish_time: Some("2024-10-23T11:30:00Z".to_string()),
                url: Some("https://dev.azure.com/conda-forge/feedstock-builds/_build/results?buildId=1374329".to_string()),
                definition: BuildDefinition {
                    id: 3,
                    name: "scipy-feedstock CI".to_string(),
                    url: "https://dev.azure.com/conda-forge/feedstock-builds/_definition?definitionId=3".to_string(),
                },
                project: Project {
                    id: "project-id".to_string(),
                    name: "feedstock-builds".to_string(),
                    url: "https://dev.azure.com/conda-forge/feedstock-builds".to_string(),
                },
                source_branch: Some("refs/heads/main".to_string()),
                source_version: Some("ghi789jkl012".to_string()),
            },
        ];

        // Test filtering by exact match
        let filtered = client
            .filter_builds_by_description(&builds, "numpy-feedstock CI")
            .unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1374331);

        // Test filtering by pattern
        let filtered = client
            .filter_builds_by_description(&builds, ".*-feedstock.*")
            .unwrap();
        assert_eq!(filtered.len(), 3);

        // Test filtering with specific pattern
        let filtered = client
            .filter_builds_by_description(&builds, "^pandas.*")
            .unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].definition.name, "pandas-feedstock CI");

        // Test no matches
        let filtered = client
            .filter_builds_by_description(&builds, "nonexistent")
            .unwrap();
        assert_eq!(filtered.len(), 0);

        // Test invalid regex
        let result = client.filter_builds_by_description(&builds, "[invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_queue_time_field() {
        let client = AzureDevOpsClient {
            client: reqwest::Client::new(),
            token: None,
        };

        // Create a build with missing queue_time field to test the fix
        let builds_with_missing_queue_time = vec![AzureDevOpsBuild {
            id: 1374332,
            build_number: Some("test.1".to_string()),
            status: "completed".to_string(),
            result: Some("succeeded".to_string()),
            queue_time: None, // This should not cause deserialization to fail
            start_time: Some("2024-10-23T10:05:00Z".to_string()),
            finish_time: Some("2024-10-23T10:30:00Z".to_string()),
            url: Some(
                "https://dev.azure.com/conda-forge/feedstock-builds/_build/results?buildId=1374332"
                    .to_string(),
            ),
            definition: BuildDefinition {
                id: 1,
                name: "feedstock-builds".to_string(),
                url:
                    "https://dev.azure.com/conda-forge/feedstock-builds/_definition?definitionId=1"
                        .to_string(),
            },
            project: Project {
                id: "project-id".to_string(),
                name: "feedstock-builds".to_string(),
                url: "https://dev.azure.com/conda-forge/feedstock-builds".to_string(),
            },
            source_branch: None,  // Also test missing source_branch
            source_version: None, // Also test missing source_version
        }];

        // This should not panic with missing optional fields
        client
            .print_builds_info(
                &builds_with_missing_queue_time,
                "conda-forge",
                "feedstock-builds",
                "table",
            )
            .unwrap();
    }

    #[test]
    fn test_independent_filtering_scenarios() {
        let client = AzureDevOpsClient {
            client: reqwest::Client::new(),
            token: None,
        };

        // Create test artifacts for name filtering
        let artifacts = vec![
            AzureDevOpsArtifact {
                id: 585295,
                name: "conda_pkgs_win".to_string(),
                source: "source1".to_string(),
                resource: ArtifactResource {
                    artifact_type: "PipelineArtifact".to_string(),
                    data: "data1".to_string(),
                    properties: Some(ArtifactProperties {
                        root_id: Some("root1".to_string()),
                        artifactsize: Some("77520".to_string()),
                        hash_type: Some("SHA256".to_string()),
                        domain_id: Some("domain1".to_string()),
                    }),
                    url: "https://example.com/artifact1".to_string(),
                    download_url: Some("https://example.com/download1".to_string()),
                },
            },
            AzureDevOpsArtifact {
                id: 585296,
                name: "logs_and_metadata".to_string(),
                source: "source2".to_string(),
                resource: ArtifactResource {
                    artifact_type: "PipelineArtifact".to_string(),
                    data: "data2".to_string(),
                    properties: Some(ArtifactProperties {
                        root_id: Some("root2".to_string()),
                        artifactsize: Some("1024".to_string()),
                        hash_type: Some("SHA256".to_string()),
                        domain_id: Some("domain2".to_string()),
                    }),
                    url: "https://example.com/artifact2".to_string(),
                    download_url: Some("https://example.com/download2".to_string()),
                },
            },
        ];

        // Test independent name filtering
        let filtered_artifacts = client.filter_artifacts_by_name(&artifacts, Some("conda.*"));
        assert_eq!(filtered_artifacts.len(), 1);
        assert_eq!(filtered_artifacts[0].name, "conda_pkgs_win");

        // Create test builds for description filtering
        let builds = vec![
            AzureDevOpsBuild {
                id: 1001,
                build_number: Some("1.0".to_string()),
                status: "completed".to_string(),
                result: Some("succeeded".to_string()),
                queue_time: Some("2024-10-23T10:00:00Z".to_string()),
                start_time: Some("2024-10-23T10:05:00Z".to_string()),
                finish_time: Some("2024-10-23T10:30:00Z".to_string()),
                url: Some("https://example.com/build1001".to_string()),
                definition: BuildDefinition {
                    id: 1,
                    name: "numpy-feedstock CI".to_string(),
                    url: "https://example.com/def1".to_string(),
                },
                project: Project {
                    id: "proj1".to_string(),
                    name: "feedstock-builds".to_string(),
                    url: "https://example.com/proj".to_string(),
                },
                source_branch: Some("refs/heads/main".to_string()),
                source_version: Some("abc123".to_string()),
            },
            AzureDevOpsBuild {
                id: 1002,
                build_number: Some("2.0".to_string()),
                status: "completed".to_string(),
                result: Some("succeeded".to_string()),
                queue_time: Some("2024-10-23T11:00:00Z".to_string()),
                start_time: Some("2024-10-23T11:05:00Z".to_string()),
                finish_time: Some("2024-10-23T11:30:00Z".to_string()),
                url: Some("https://example.com/build1002".to_string()),
                definition: BuildDefinition {
                    id: 2,
                    name: "release-automation".to_string(),
                    url: "https://example.com/def2".to_string(),
                },
                project: Project {
                    id: "proj1".to_string(),
                    name: "feedstock-builds".to_string(),
                    url: "https://example.com/proj".to_string(),
                },
                source_branch: Some("refs/heads/release".to_string()),
                source_version: Some("def456".to_string()),
            },
        ];

        // Test independent description filtering
        let filtered_builds = client
            .filter_builds_by_description(&builds, ".*feedstock.*")
            .unwrap();
        assert_eq!(filtered_builds.len(), 1);
        assert_eq!(filtered_builds[0].definition.name, "numpy-feedstock CI");

        // Test both filters can work on different data independently
        let conda_artifacts = client.filter_artifacts_by_name(&artifacts, Some("conda.*"));
        let automation_builds = client
            .filter_builds_by_description(&builds, ".*automation.*")
            .unwrap();

        assert_eq!(conda_artifacts.len(), 1);
        assert_eq!(automation_builds.len(), 1);
        assert_eq!(automation_builds[0].definition.name, "release-automation");
    }

    #[test]
    fn test_output_formats() {
        let client = AzureDevOpsClient {
            client: reqwest::Client::new(),
            token: None,
        };

        // Create test data
        let builds = vec![AzureDevOpsBuild {
            id: 1001,
            build_number: Some("1.0".to_string()),
            status: "completed".to_string(),
            result: Some("succeeded".to_string()),
            queue_time: Some("2024-10-23T10:00:00Z".to_string()),
            start_time: Some("2024-10-23T10:05:00Z".to_string()),
            finish_time: Some("2024-10-23T10:30:00Z".to_string()),
            url: Some("https://example.com/build1001".to_string()),
            definition: BuildDefinition {
                id: 1,
                name: "numpy-feedstock CI".to_string(),
                url: "https://example.com/def1".to_string(),
            },
            project: Project {
                id: "proj1".to_string(),
                name: "feedstock-builds".to_string(),
                url: "https://example.com/proj".to_string(),
            },
            source_branch: Some("refs/heads/main".to_string()),
            source_version: Some("abc123".to_string()),
        }];

        let artifacts = vec![AzureDevOpsArtifact {
            id: 585295,
            name: "conda_pkgs_win".to_string(),
            source: "source1".to_string(),
            resource: ArtifactResource {
                artifact_type: "PipelineArtifact".to_string(),
                data: "data1".to_string(),
                properties: Some(ArtifactProperties {
                    root_id: Some("root1".to_string()),
                    artifactsize: Some("77520".to_string()),
                    hash_type: Some("SHA256".to_string()),
                    domain_id: Some("domain1".to_string()),
                }),
                url: "https://example.com/artifact1".to_string(),
                download_url: Some("https://example.com/download1".to_string()),
            },
        }];

        // Test table format (should not panic)
        client
            .print_builds_info(&builds, "conda-forge", "feedstock-builds", "table")
            .unwrap();
        client.print_artifacts_info(&artifacts, "table").unwrap();

        // Test YAML format (should not panic)
        client
            .print_builds_info(&builds, "conda-forge", "feedstock-builds", "yaml")
            .unwrap();
        client.print_artifacts_info(&artifacts, "yaml").unwrap();

        // Test JSON format (should not panic)
        client
            .print_builds_info(&builds, "conda-forge", "feedstock-builds", "json")
            .unwrap();
        client.print_artifacts_info(&artifacts, "json").unwrap();

        // Test invalid format (should return error)
        let result =
            client.print_builds_info(&builds, "conda-forge", "feedstock-builds", "invalid");
        assert!(result.is_err());

        let result = client.print_artifacts_info(&artifacts, "invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_comfy_table_integration() {
        let client = AzureDevOpsClient {
            client: reqwest::Client::new(),
            token: None,
        };

        // Test data with various field states to verify table formatting
        let builds = vec![
            AzureDevOpsBuild {
                id: 1001,
                build_number: Some("PR.123.1".to_string()),
                status: "completed".to_string(),
                result: Some("succeeded".to_string()),
                queue_time: Some("2024-10-23T10:00:00Z".to_string()),
                start_time: Some("2024-10-23T10:05:00Z".to_string()),
                finish_time: Some("2024-10-23T10:30:00Z".to_string()),
                url: Some("https://example.com/build1001".to_string()),
                definition: BuildDefinition {
                    id: 1,
                    name:
                        "numpy-feedstock CI with very long name that should be handled gracefully"
                            .to_string(),
                    url: "https://example.com/def1".to_string(),
                },
                project: Project {
                    id: "proj1".to_string(),
                    name: "feedstock-builds".to_string(),
                    url: "https://example.com/proj".to_string(),
                },
                source_branch: Some("refs/pull/123/merge".to_string()),
                source_version: Some("abc123".to_string()),
            },
            AzureDevOpsBuild {
                id: 1002,
                build_number: None, // Test missing build number
                status: "failed".to_string(),
                result: Some("failed".to_string()),
                queue_time: None, // Test missing queue time
                start_time: Some("2024-10-23T11:05:00Z".to_string()),
                finish_time: None, // Test missing finish time (in progress)
                url: None,         // Test missing URL
                definition: BuildDefinition {
                    id: 2,
                    name: "short-name".to_string(),
                    url: "https://example.com/def2".to_string(),
                },
                project: Project {
                    id: "proj2".to_string(),
                    name: "feedstock-builds".to_string(),
                    url: "https://example.com/proj".to_string(),
                },
                source_branch: None,  // Test missing source branch
                source_version: None, // Test missing source version
            },
        ];

        let artifacts = vec![
            AzureDevOpsArtifact {
                id: 585295,
                name: "conda_pkgs_linux_with_very_long_name_to_test_formatting".to_string(),
                source: "source1".to_string(),
                resource: ArtifactResource {
                    artifact_type: "PipelineArtifact".to_string(),
                    data: "data1".to_string(),
                    properties: Some(ArtifactProperties {
                        root_id: Some("root1".to_string()),
                        artifactsize: Some("1048576".to_string()), // 1MB
                        hash_type: Some("SHA256".to_string()),
                        domain_id: Some("domain1".to_string()),
                    }),
                    url: "https://example.com/artifact1".to_string(),
                    download_url: Some("https://example.com/download1".to_string()),
                },
            },
            AzureDevOpsArtifact {
                id: 585296,
                name: "small_artifact".to_string(),
                source: "src2".to_string(),
                resource: ArtifactResource {
                    artifact_type: "Container".to_string(),
                    data: "data2".to_string(),
                    properties: Some(ArtifactProperties {
                        root_id: Some("root2".to_string()),
                        artifactsize: Some("512".to_string()), // Small size
                        hash_type: Some("MD5".to_string()),
                        domain_id: Some("domain2".to_string()),
                    }),
                    url: "https://example.com/artifact2".to_string(),
                    download_url: None, // Test no download URL
                },
            },
        ];

        // Test table format with various field states - should not panic
        println!("Testing comfy-table integration for builds...");
        client
            .print_builds_info(&builds, "test-org", "test-project", "table")
            .unwrap();

        println!("Testing comfy-table integration for artifacts...");
        client.print_artifacts_info(&artifacts, "table").unwrap();

        // Test YAML format with metadata comments
        println!("Testing YAML output with metadata...");
        client
            .print_builds_info(&builds, "test-org", "test-project", "yaml")
            .unwrap();
        client.print_artifacts_info(&artifacts, "yaml").unwrap();

        // Verify the structures serialize to valid JSON (all fields included)
        let builds_json = serde_json::to_string_pretty(&builds).unwrap();
        assert!(builds_json.contains("build_number"));
        assert!(builds_json.contains("source_branch"));
        assert!(builds_json.contains("queue_time"));

        let artifacts_json = serde_json::to_string_pretty(&artifacts).unwrap();
        assert!(artifacts_json.contains("\"type\""));
        assert!(artifacts_json.contains("downloadUrl"));
        assert!(artifacts_json.contains("properties"));
    }
}
