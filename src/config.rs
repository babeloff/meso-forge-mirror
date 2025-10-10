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
