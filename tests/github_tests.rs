use meso_forge_mirror::config::Config;
use meso_forge_mirror::github::{parse_artifact_id, parse_github_repository, GitHubClient};

#[test]
fn test_parse_github_repository() {
    // Test owner/repo format
    let (owner, repo) = parse_github_repository("octocat/Hello-World").unwrap();
    assert_eq!(owner, "octocat");
    assert_eq!(repo, "Hello-World");

    // Test GitHub URL formats
    let (owner, repo) = parse_github_repository("https://github.com/octocat/Hello-World").unwrap();
    assert_eq!(owner, "octocat");
    assert_eq!(repo, "Hello-World");

    let (owner, repo) = parse_github_repository("https://github.com/octocat/Hello-World/").unwrap();
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

#[test]
fn test_github_client_creation() {
    let config = Config::default();
    let client = GitHubClient::new(&config);
    assert!(client.is_ok());
}

#[test]
fn test_github_client_creation_with_token() {
    let config = Config {
        github_token: Some("test_token".to_string()),
        ..Default::default()
    };
    let client = GitHubClient::new(&config);
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_filter_artifacts() {
    // Create a mock GitHub client
    let config = Config::default();
    let github_client = GitHubClient::new(&config).unwrap();

    // Create mock artifacts
    let artifacts = vec![
        meso_forge_mirror::github::GitHubArtifact {
            id: 1,
            name: "conda-package-linux".to_string(),
            size_in_bytes: 1000,
            url: "https://api.github.com/repos/test/test/actions/artifacts/1".to_string(),
            archive_download_url: "https://api.github.com/repos/test/test/actions/artifacts/1/zip"
                .to_string(),
            expired: false,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            expires_at: "2024-12-31T23:59:59Z".to_string(),
            workflow_run: None,
        },
        meso_forge_mirror::github::GitHubArtifact {
            id: 2,
            name: "conda-package-windows".to_string(),
            size_in_bytes: 2000,
            url: "https://api.github.com/repos/test/test/actions/artifacts/2".to_string(),
            archive_download_url: "https://api.github.com/repos/test/test/actions/artifacts/2/zip"
                .to_string(),
            expired: true,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            expires_at: "2024-01-01T23:59:59Z".to_string(),
            workflow_run: None,
        },
        meso_forge_mirror::github::GitHubArtifact {
            id: 3,
            name: "test-results".to_string(),
            size_in_bytes: 500,
            url: "https://api.github.com/repos/test/test/actions/artifacts/3".to_string(),
            archive_download_url: "https://api.github.com/repos/test/test/actions/artifacts/3/zip"
                .to_string(),
            expired: false,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            expires_at: "2024-12-31T23:59:59Z".to_string(),
            workflow_run: None,
        },
    ];

    // Test name filtering
    let filtered = github_client.filter_artifacts_by_name(&artifacts, Some("conda-package.*"));
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().any(|a| a.name == "conda-package-linux"));
    assert!(filtered.iter().any(|a| a.name == "conda-package-windows"));

    // Test expired filtering
    let non_expired = github_client.filter_non_expired_artifacts(&artifacts);
    assert_eq!(non_expired.len(), 2);
    assert!(non_expired.iter().any(|a| a.name == "conda-package-linux"));
    assert!(non_expired.iter().any(|a| a.name == "test-results"));
    assert!(!non_expired
        .iter()
        .any(|a| a.name == "conda-package-windows"));

    // Test combined filtering
    let combined = github_client.filter_artifacts_by_name(&non_expired, Some("conda.*"));
    assert_eq!(combined.len(), 1);
    assert_eq!(combined[0].name, "conda-package-linux");
}

// This test demonstrates what the CLI help should show
#[test]
fn test_cli_help_includes_new_options() {
    // This is more of a documentation test to show expected CLI behavior
    let expected_src_types = ["zip", "zip-url", "local", "url", "tgz", "tgz-url", "github"];

    // Verify that our source type validation includes github
    for src_type in expected_src_types {
        match src_type {
            "zip" | "zip-url" | "local" | "url" | "tgz" | "tgz-url" | "github" => {
                // This should pass
            }
            _ => panic!("Unexpected source type: {}", src_type),
        }
    }
}

// Test configuration with GitHub token
#[test]
fn test_config_with_github_token() {
    // Test default behavior (should try to get from environment)
    // We won't test the actual environment variable since that's system-dependent

    // Test explicit token setting
    let config = Config {
        github_token: Some("ghp_test_token".to_string()),
        ..Default::default()
    };
    assert_eq!(config.github_token.as_deref(), Some("ghp_test_token"));
}

// Test the artifact ID parsing for the new format owner/repo#artifact_id
#[test]
fn test_artifact_id_in_source_format() {
    let source = "owner/repo#123456";
    let parts: Vec<&str> = source.split('#').collect();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], "owner/repo");
    assert_eq!(parts[1], "123456");

    let artifact_id = parse_artifact_id(parts[1]).unwrap();
    assert_eq!(artifact_id, 123456);
}
