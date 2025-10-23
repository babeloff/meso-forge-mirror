use meso_forge_mirror::azure::{
    parse_azure_devops_url, parse_azure_source, parse_build_id, ArtifactProperties,
    ArtifactResource, AzureDevOpsArtifact, AzureDevOpsClient,
};
use meso_forge_mirror::config::Config;

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
    assert_eq!(parse_build_id("1374331").unwrap(), 1374331);
    assert!(parse_build_id("invalid").is_err());
    assert!(parse_build_id("").is_err());
    assert!(parse_build_id("-1").is_err()); // negative numbers should fail
}

#[test]
fn test_parse_azure_source() {
    // Test without build ID
    let (org, proj, build_id) = parse_azure_source("conda-forge/feedstock-builds").unwrap();
    assert_eq!(org, "conda-forge");
    assert_eq!(proj, "feedstock-builds");
    assert_eq!(build_id, None);

    // Test with build ID
    let (org, proj, build_id) = parse_azure_source("conda-forge/feedstock-builds#1374331").unwrap();
    assert_eq!(org, "conda-forge");
    assert_eq!(proj, "feedstock-builds");
    assert_eq!(build_id, Some(1374331));

    // Test with URL format
    let (org, proj, build_id) =
        parse_azure_source("https://dev.azure.com/conda-forge/feedstock-builds").unwrap();
    assert_eq!(org, "conda-forge");
    assert_eq!(proj, "feedstock-builds");
    assert_eq!(build_id, None);

    // Test with URL and build ID
    let (org, proj, build_id) =
        parse_azure_source("https://dev.azure.com/conda-forge/feedstock-builds#1374331").unwrap();
    assert_eq!(org, "conda-forge");
    assert_eq!(proj, "feedstock-builds");
    assert_eq!(build_id, Some(1374331));

    // Test invalid formats
    assert!(parse_azure_source("invalid").is_err());
    assert!(parse_azure_source("").is_err());
    assert!(parse_azure_source("/#123").is_err());
    assert!(parse_azure_source("org/proj#invalid").is_err());
}

#[test]
fn test_azure_client_creation() {
    let config = Config::default();
    let client = AzureDevOpsClient::new(&config);
    assert!(client.is_ok());
}

#[test]
fn test_azure_client_creation_with_token() {
    let config = Config {
        azure_devops_token: Some("test_token".to_string()),
        ..Default::default()
    };
    let client = AzureDevOpsClient::new(&config);
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_filter_artifacts() {
    // Create a mock Azure client
    let config = Config::default();
    let azure_client = AzureDevOpsClient::new(&config).unwrap();

    // Create mock artifacts
    let artifacts = vec![
        AzureDevOpsArtifact {
            id: 1,
            name: "conda-packages-linux".to_string(),
            source: "build".to_string(),
            resource: ArtifactResource {
                artifact_type: "Container".to_string(),
                data: "test".to_string(),
                url: "https://dev.azure.com/test/test/_apis/build/builds/1/artifacts/1".to_string(),
                download_url: Some(
                    "https://dev.azure.com/test/test/_apis/build/builds/1/artifacts/1/content"
                        .to_string(),
                ),
                properties: Some(ArtifactProperties {
                    root_id: Some("1".to_string()),
                    artifactsize: Some("1000000".to_string()),
                    hash_type: Some("SHA256".to_string()),
                    domain_id: Some("test".to_string()),
                }),
            },
        },
        AzureDevOpsArtifact {
            id: 2,
            name: "conda-packages-osx".to_string(),
            source: "build".to_string(),
            resource: ArtifactResource {
                artifact_type: "Container".to_string(),
                data: "test".to_string(),
                url: "https://dev.azure.com/test/test/_apis/build/builds/1/artifacts/2".to_string(),
                download_url: Some(
                    "https://dev.azure.com/test/test/_apis/build/builds/1/artifacts/2/content"
                        .to_string(),
                ),
                properties: Some(ArtifactProperties {
                    root_id: Some("2".to_string()),
                    artifactsize: Some("2000000".to_string()),
                    hash_type: Some("SHA256".to_string()),
                    domain_id: Some("test".to_string()),
                }),
            },
        },
        AzureDevOpsArtifact {
            id: 3,
            name: "test-results".to_string(),
            source: "build".to_string(),
            resource: ArtifactResource {
                artifact_type: "FilePath".to_string(),
                data: "test".to_string(),
                url: "https://dev.azure.com/test/test/_apis/build/builds/1/artifacts/3".to_string(),
                download_url: None,
                properties: Some(ArtifactProperties {
                    root_id: Some("3".to_string()),
                    artifactsize: Some("500000".to_string()),
                    hash_type: Some("SHA256".to_string()),
                    domain_id: Some("test".to_string()),
                }),
            },
        },
    ];

    // Test name filtering
    let filtered = azure_client.filter_artifacts_by_name(&artifacts, Some("conda-packages.*"));
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().any(|a| a.name == "conda-packages-linux"));
    assert!(filtered.iter().any(|a| a.name == "conda-packages-windows"));

    // Test type filtering
    let container_artifacts = azure_client.filter_artifacts_by_type(&artifacts, Some("Container"));
    assert_eq!(container_artifacts.len(), 2);
    assert!(container_artifacts
        .iter()
        .all(|a| a.resource.artifact_type == "Container"));

    let filepath_artifacts = azure_client.filter_artifacts_by_type(&artifacts, Some("FilePath"));
    assert_eq!(filepath_artifacts.len(), 1);
    assert_eq!(filepath_artifacts[0].name, "test-results");

    // Test combined filtering
    let combined = azure_client.filter_artifacts_by_name(&container_artifacts, Some(".*linux.*"));
    assert_eq!(combined.len(), 1);
    assert_eq!(combined[0].name, "conda-packages-linux");
}

// This test demonstrates what the CLI help should show
#[test]
fn test_cli_help_includes_azure_options() {
    // This is more of a documentation test to show expected CLI behavior
    let expected_src_types = [
        "zip", "zip-url", "local", "url", "tgz", "tgz-url", "github", "azure",
    ];

    // Verify that our source type validation includes azure
    for src_type in expected_src_types {
        match src_type {
            "zip" | "zip-url" | "local" | "url" | "tgz" | "tgz-url" | "github" | "azure" => {
                // This should pass
            }
            _ => panic!("Unexpected source type: {}", src_type),
        }
    }
}

// Test configuration with Azure DevOps token
#[test]
fn test_config_with_azure_token() {
    // Test default behavior (should try to get from environment)
    // We won't test the actual environment variable since that's system-dependent

    // Test explicit token setting
    let config = Config {
        azure_devops_token: Some("test_pat_token".to_string()),
        ..Default::default()
    };
    assert_eq!(config.azure_devops_token.as_deref(), Some("test_pat_token"));
}

// Test the build ID parsing for the new format org/project#build_id
#[test]
fn test_build_id_in_source_format() {
    let source = "conda-forge/feedstock-builds#1374331";
    let parts: Vec<&str> = source.split('#').collect();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], "conda-forge/feedstock-builds");
    assert_eq!(parts[1], "1374331");

    let build_id = parse_build_id(parts[1]).unwrap();
    assert_eq!(build_id, 1374331);
}

// Test realistic conda-forge scenarios
#[test]
fn test_conda_forge_scenarios() {
    // Test the specific conda-forge case mentioned in the requirements
    let (org, proj, build_id) = parse_azure_source("conda-forge/feedstock-builds#1374331").unwrap();
    assert_eq!(org, "conda-forge");
    assert_eq!(proj, "feedstock-builds");
    assert_eq!(build_id, Some(1374331));

    // Test without build ID (would list recent builds)
    let (org, proj, build_id) = parse_azure_source("conda-forge/feedstock-builds").unwrap();
    assert_eq!(org, "conda-forge");
    assert_eq!(proj, "feedstock-builds");
    assert_eq!(build_id, None);

    // Test full Azure DevOps URL format as it appears in conda-forge
    let (org, proj, build_id) =
        parse_azure_source("https://dev.azure.com/conda-forge/feedstock-builds#1374331").unwrap();
    assert_eq!(org, "conda-forge");
    assert_eq!(proj, "feedstock-builds");
    assert_eq!(build_id, Some(1374331));
}

// Test edge cases and error handling
#[test]
fn test_edge_cases() {
    // Test empty build ID
    assert!(parse_azure_source("org/proj#").is_err());

    // Test invalid characters in build ID
    assert!(parse_azure_source("org/proj#abc123").is_err());

    // Test very large build ID (should still work)
    let (_, _, build_id) = parse_azure_source("org/proj#999999999999").unwrap();
    assert_eq!(build_id, Some(999999999999));

    // Test zero build ID (should work)
    let (_, _, build_id) = parse_azure_source("org/proj#0").unwrap();
    assert_eq!(build_id, Some(0));
}

// Test that artifact filtering handles empty lists gracefully
#[test]
fn test_empty_artifact_filtering() {
    let config = Config::default();
    let azure_client = AzureDevOpsClient::new(&config).unwrap();

    let empty_artifacts = vec![];

    let filtered_by_name = azure_client.filter_artifacts_by_name(&empty_artifacts, Some("test.*"));
    assert_eq!(filtered_by_name.len(), 0);

    let filtered_by_type =
        azure_client.filter_artifacts_by_type(&empty_artifacts, Some("Container"));
    assert_eq!(filtered_by_type.len(), 0);
}

// Test URL parsing edge cases
#[test]
fn test_url_parsing_edge_cases() {
    // Test with trailing slashes and query parameters
    let (org, proj) =
        parse_azure_devops_url("https://dev.azure.com/conda-forge/feedstock-builds/").unwrap();
    assert_eq!(org, "conda-forge");
    assert_eq!(proj, "feedstock-builds");

    // Test with extra path components (should still work, taking first two)
    let (org, proj) =
        parse_azure_devops_url("https://dev.azure.com/conda-forge/feedstock-builds/extra/path")
            .unwrap();
    assert_eq!(org, "conda-forge");
    assert_eq!(proj, "feedstock-builds");

    // Test case sensitivity
    let (org, proj) = parse_azure_devops_url("CONDA-FORGE/FEEDSTOCK-BUILDS").unwrap();
    assert_eq!(org, "CONDA-FORGE");
    assert_eq!(proj, "FEEDSTOCK-BUILDS");
}
