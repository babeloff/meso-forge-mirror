use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub max_concurrent_downloads: usize,
    pub retry_attempts: u32,
    pub timeout_seconds: u64,
    pub s3_region: Option<String>,
    pub s3_endpoint: Option<String>,
    pub github_token: Option<String>,
    pub azure_devops_token: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_concurrent_downloads: 5,
            retry_attempts: 3,
            timeout_seconds: 300,
            s3_region: None,
            s3_endpoint: None,
            github_token: std::env::var("GITHUB_TOKEN").ok(),
            azure_devops_token: std::env::var("AZURE_DEVOPS_TOKEN").ok(),
        }
    }
}

impl Config {
    pub fn load_from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.max_concurrent_downloads, 5);
        assert_eq!(config.retry_attempts, 3);
        assert_eq!(config.timeout_seconds, 300);
    }

    #[test]
    fn test_config_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.json");

        let config = Config::default();
        config.save_to_file(config_path.to_str().unwrap()).unwrap();

        let loaded_config = Config::load_from_file(config_path.to_str().unwrap()).unwrap();
        assert_eq!(
            loaded_config.max_concurrent_downloads,
            config.max_concurrent_downloads
        );
        assert_eq!(loaded_config.retry_attempts, config.retry_attempts);
    }
}
