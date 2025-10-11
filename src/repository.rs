use anyhow::{anyhow, Result};
use bytes::Bytes;
use rattler_conda_types::Platform;
use std::path::Path;
use tracing::{info, warn};

use crate::conda_package::{CondaPackageHandler, ProcessedPackage};

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
    conda_handler: CondaPackageHandler,
}

impl Clone for Repository {
    fn clone(&self) -> Self {
        Self {
            repo_type: self.repo_type.clone(),
            path: self.path.clone(),
            conda_handler: CondaPackageHandler::new(),
        }
    }
}

impl Repository {
    pub fn new(repo_type: RepositoryType, path: String) -> Self {
        Self {
            repo_type,
            path,
            conda_handler: CondaPackageHandler::new(),
        }
    }

    pub async fn upload_package(&mut self, package_name: &str, content: Bytes) -> Result<()> {
        // Process the conda package to extract metadata and validate
        let processed_package = self
            .conda_handler
            .process_package(content, package_name)
            .await?;

        // Validate the package
        self.conda_handler.validate_package(&processed_package)?;

        match &self.repo_type {
            RepositoryType::Local => self.upload_local_structured(&processed_package).await,
            RepositoryType::S3 => self.upload_s3_structured(&processed_package).await,
            RepositoryType::PrefixDev => {
                self.upload_prefix_dev_structured(&processed_package).await
            }
        }
    }

    async fn upload_local_structured(&mut self, package: &ProcessedPackage) -> Result<()> {
        info!(
            "Uploading {} to local repository at {} (platform: {})",
            package.filename, self.path, package.platform
        );

        let base_path = Path::new(&self.path);
        let platform_dir = base_path.join(package.platform.to_string());
        std::fs::create_dir_all(&platform_dir)?;

        let file_path = platform_dir.join(&package.filename);
        std::fs::write(file_path, &package.content)?;

        // Update repodata.json for this platform
        let packages_for_platform = vec![package.clone()];
        self.conda_handler
            .create_repodata(&package.platform, &packages_for_platform, base_path)
            .await?;

        info!(
            "Successfully uploaded {} to local repository under {}/",
            package.filename, package.platform
        );
        Ok(())
    }

    async fn upload_s3_structured(&mut self, package: &ProcessedPackage) -> Result<()> {
        info!(
            "Uploading {} to S3 repository at {} (platform: {})",
            package.filename, self.path, package.platform
        );

        // Parse bucket and key from path
        let parts: Vec<&str> = self
            .path
            .trim_start_matches("s3://")
            .splitn(2, '/')
            .collect();
        let bucket = parts.first().ok_or_else(|| anyhow!("Invalid S3 path"))?;
        let prefix = parts.get(1).unwrap_or(&"");

        // Create structured path with platform subdirectory
        let structured_key = if prefix.is_empty() {
            format!("{}/{}", package.platform, package.filename)
        } else {
            format!("{}/{}/{}", prefix, package.platform, package.filename)
        };

        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .load()
            .await;
        let client = aws_sdk_s3::Client::new(&config);

        // Upload the package
        client
            .put_object()
            .bucket(*bucket)
            .key(&structured_key)
            .body(package.content.clone().into())
            .content_type("application/x-conda-package")
            .send()
            .await?;

        // Generate and upload repodata.json for this platform
        let packages_for_platform = vec![package.clone()];
        let repodata_content = self
            .generate_repodata_content(&packages_for_platform, &package.platform)
            .await?;

        let repodata_key = if prefix.is_empty() {
            format!("{}/repodata.json", package.platform)
        } else {
            format!("{}/{}/repodata.json", prefix, package.platform)
        };

        client
            .put_object()
            .bucket(*bucket)
            .key(&repodata_key)
            .body(repodata_content.into_bytes().into())
            .content_type("application/json")
            .send()
            .await?;

        info!(
            "Successfully uploaded {} to S3 under {}/",
            package.filename, package.platform
        );
        Ok(())
    }

    async fn upload_prefix_dev_structured(&mut self, package: &ProcessedPackage) -> Result<()> {
        info!(
            "Uploading {} to prefix.dev at {} (platform: {})",
            package.filename, self.path, package.platform
        );

        // For prefix.dev, we need to use their API with structured paths
        let client = reqwest::Client::new();
        let structured_url = format!(
            "{}/{}/{}",
            self.path.trim_end_matches('/'),
            package.platform,
            package.filename
        );

        let response = client
            .put(&structured_url)
            .header("Content-Type", "application/x-conda-package")
            .body(package.content.clone())
            .send()
            .await?;

        if response.status().is_success() {
            info!(
                "Successfully uploaded {} to prefix.dev under {}/",
                package.filename, package.platform
            );

            // Note: prefix.dev typically handles repodata generation automatically
            warn!("Note: Repodata generation for prefix.dev should be handled by their service");
            Ok(())
        } else {
            Err(anyhow!(
                "Failed to upload to prefix.dev: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ))
        }
    }

    /// Generate repodata.json content for a set of packages
    async fn generate_repodata_content(
        &self,
        packages: &[ProcessedPackage],
        platform: &Platform,
    ) -> Result<String> {
        use std::collections::HashMap;

        #[derive(serde::Serialize)]
        struct RepoData {
            info: RepoDataInfo,
            packages: HashMap<String, PackageRecord>,
        }

        #[derive(serde::Serialize)]
        struct RepoDataInfo {
            subdir: String,
        }

        #[derive(serde::Serialize)]
        struct PackageRecord {
            build: String,
            build_number: u64,
            depends: Vec<String>,
            license: String,
            md5: String,
            sha256: String,
            size: u64,
            subdir: String,
            name: String,
            version: String,
            timestamp: Option<chrono::DateTime<chrono::Utc>>,
        }

        let mut repodata = RepoData {
            info: RepoDataInfo {
                subdir: platform.to_string(),
            },
            packages: HashMap::new(),
        };

        for package in packages {
            let package_record = PackageRecord {
                build: package.metadata.build.clone(),
                build_number: package.metadata.build_number,
                depends: package.metadata.depends.clone(),
                license: package.metadata.license.clone().unwrap_or_default(),
                md5: package.md5.clone(),
                sha256: package.sha256.clone(),
                size: package.size,
                subdir: platform.to_string(),
                name: package.metadata.name.clone(),
                version: package.metadata.version.clone(),
                timestamp: package.metadata.timestamp,
            };

            repodata
                .packages
                .insert(package.filename.clone(), package_record);
        }

        Ok(serde_json::to_string_pretty(&repodata)?)
    }

    /// Get statistics about processed packages
    pub fn get_package_stats(&self) -> crate::conda_package::PackageStats {
        self.conda_handler.get_stats()
    }

    /// Finalize repository by updating all repodata files
    pub async fn finalize_repository(&mut self) -> Result<()> {
        info!("Finalizing repository structure");

        let organized_packages = self.conda_handler.organize_packages();

        match &self.repo_type {
            RepositoryType::Local => {
                let base_path = Path::new(&self.path);
                for (platform, packages) in organized_packages {
                    if !packages.is_empty() {
                        self.conda_handler
                            .create_repodata(&platform, &packages, base_path)
                            .await?;
                    }
                }
            }
            RepositoryType::S3 => {
                // For S3, repodata is updated per package upload
                info!("S3 repositories update repodata per upload");
            }
            RepositoryType::PrefixDev => {
                // prefix.dev handles repodata automatically
                info!("prefix.dev handles repodata generation automatically");
            }
        }

        let stats = self.get_package_stats();
        stats.print_summary();

        Ok(())
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
