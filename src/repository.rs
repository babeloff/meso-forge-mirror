use anyhow::{anyhow, Result};
use bytes::Bytes;
use std::path::Path;
use tracing::info;

#[derive(Debug, Clone)]
pub enum RepositoryType {
    PrefixDev,
    S3,
    Local,
}

impl RepositoryType {
    pub fn from_string(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "prefix-dev" | "prefix" => Ok(RepositoryType::PrefixDev),
            "s3" | "minio" => Ok(RepositoryType::S3),
            "local" | "file" => Ok(RepositoryType::Local),
            _ => Err(anyhow!("Unknown repository type: {}", s)),
        }
    }
}

pub struct Repository {
    pub repo_type: RepositoryType,
    pub path: String,
}

impl Clone for Repository {
    fn clone(&self) -> Self {
        Self {
            repo_type: self.repo_type.clone(),
            path: self.path.clone(),
        }
    }
}

impl Repository {
    pub fn new(repo_type: RepositoryType, path: String) -> Self {
        Self { repo_type, path }
    }

    pub async fn upload_package(&self, package_name: &str, content: Bytes) -> Result<()> {
        match &self.repo_type {
            RepositoryType::Local => self.upload_local(package_name, content).await,
            RepositoryType::S3 => self.upload_s3(package_name, content).await,
            RepositoryType::PrefixDev => self.upload_prefix_dev(package_name, content).await,
        }
    }

    async fn upload_local(&self, package_name: &str, content: Bytes) -> Result<()> {
        info!(
            "Uploading {} to local repository at {}",
            package_name, self.path
        );

        let target_dir = Path::new(&self.path);
        std::fs::create_dir_all(target_dir)?;

        let file_path = target_dir.join(package_name);
        std::fs::write(file_path, content)?;

        info!("Successfully uploaded {} to local repository", package_name);
        Ok(())
    }

    async fn upload_s3(&self, package_name: &str, content: Bytes) -> Result<()> {
        info!(
            "Uploading {} to S3 repository at {}",
            package_name, self.path
        );

        // Parse bucket and key from path
        let parts: Vec<&str> = self
            .path
            .trim_start_matches("s3://")
            .splitn(2, '/')
            .collect();
        let bucket = parts.first().ok_or_else(|| anyhow!("Invalid S3 path"))?;
        let prefix = parts.get(1).unwrap_or(&"");
        let key = if prefix.is_empty() {
            package_name.to_string()
        } else {
            format!("{}/{}", prefix, package_name)
        };

        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .load()
            .await;
        let client = aws_sdk_s3::Client::new(&config);

        client
            .put_object()
            .bucket(*bucket)
            .key(&key)
            .body(content.into())
            .send()
            .await?;

        info!("Successfully uploaded {} to S3", package_name);
        Ok(())
    }

    async fn upload_prefix_dev(&self, package_name: &str, content: Bytes) -> Result<()> {
        info!("Uploading {} to prefix.dev at {}", package_name, self.path);

        // For prefix.dev, we need to use their API
        // This is a placeholder - actual implementation would require prefix.dev API credentials
        let client = reqwest::Client::new();
        let url = format!("{}/{}", self.path.trim_end_matches('/'), package_name);

        let response = client.put(&url).body(content).send().await?;

        if response.status().is_success() {
            info!("Successfully uploaded {} to prefix.dev", package_name);
            Ok(())
        } else {
            Err(anyhow!(
                "Failed to upload to prefix.dev: {}",
                response.status()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_type_from_string() {
        assert!(matches!(
            RepositoryType::from_string("prefix-dev").unwrap(),
            RepositoryType::PrefixDev
        ));
        assert!(matches!(
            RepositoryType::from_string("prefix").unwrap(),
            RepositoryType::PrefixDev
        ));
        assert!(matches!(
            RepositoryType::from_string("s3").unwrap(),
            RepositoryType::S3
        ));
        assert!(matches!(
            RepositoryType::from_string("minio").unwrap(),
            RepositoryType::S3
        ));
        assert!(matches!(
            RepositoryType::from_string("local").unwrap(),
            RepositoryType::Local
        ));
        assert!(matches!(
            RepositoryType::from_string("file").unwrap(),
            RepositoryType::Local
        ));
        assert!(RepositoryType::from_string("invalid").is_err());
    }

    #[test]
    fn test_repository_new() {
        let repo = Repository::new(RepositoryType::Local, "/tmp/test".to_string());
        assert!(matches!(repo.repo_type, RepositoryType::Local));
        assert_eq!(repo.path, "/tmp/test");
    }
}
