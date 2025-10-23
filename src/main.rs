use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

mod conda_package;
mod config;
mod mirror;
mod repository;

use config::Config;
use mirror::mirror_packages;
use repository::RepositoryType;

#[derive(Parser)]
#[command(name = "meso-forge-mirror")]
#[command(version)]
#[command(about = "Mirror conda packages from staging PRs to target repositories", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Mirror packages from source to target repository
    Mirror {
        /// Source type: zip (local zip), zip-url (remote zip), local (local conda), url (remote conda), tgz (local tarball), tgz-url (remote tarball)
        #[arg(long, default_value = "local")]
        src_type: String,

        /// Source path or URL (local file path or remote URL)
        #[arg(long)]
        src: String,

        /// Regular expression to match file paths within ZIP file where conda packages are located (only first match processed; required when src-type is 'zip' or 'zip-url')
        #[arg(long)]
        src_path: Option<String>,

        /// Target repository type (prefix-dev, s3, or local)
        #[arg(long, default_value = "local")]
        tgt_type: String,

        /// Target repository path or URL
        #[arg(long)]
        tgt: String,

        /// Configuration file (optional)
        #[arg(short, long)]
        config: Option<String>,
    },
    /// Initialize configuration file
    Init {
        /// Output path for config file
        #[arg(short, long, default_value = "meso-forge-mirror.json")]
        output: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Mirror {
            src_type,
            src,
            src_path,
            tgt_type,
            tgt,
            config,
        } => {
            info!("Starting package mirroring");

            // Validate source type
            match src_type.as_str() {
                "zip" | "zip-url" | "local" | "url" | "tgz" | "tgz-url" => {}
                _ => {
                    return Err(anyhow::anyhow!(
                    "Invalid src-type '{}'. Must be one of: zip, zip-url, local, url, tgz, tgz-url",
                    src_type
                ))
                }
            }

            // Validate that src_path is provided for zip files
            if (src_type == "zip" || src_type == "zip-url") && src_path.is_none() {
                return Err(anyhow::anyhow!(
                    "--src-path is required when src-type is 'zip' or 'zip-url'"
                ));
            }

            // Validate regex pattern if provided
            if let Some(ref pattern) = src_path {
                if let Err(e) = regex::Regex::new(pattern) {
                    return Err(anyhow::anyhow!(
                        "Invalid regular expression in --src-path: {}",
                        e
                    ));
                }
            }

            let config = if let Some(config_path) = config {
                Config::load_from_file(&config_path)?
            } else {
                Config::default()
            };

            let repo_type = RepositoryType::from_string(&tgt_type)?;

            let is_local_file = matches!(src_type.as_str(), "zip" | "local" | "tgz");
            mirror_packages(
                &src,
                src_path.as_deref(),
                &src_type,
                is_local_file,
                repo_type,
                &tgt,
                &config,
            )
            .await?;

            info!("Mirroring completed successfully");
        }
        Commands::Init { output } => {
            info!("Initializing configuration file at: {}", output);
            let config = Config::default();
            config.save_to_file(&output)?;
            info!("Configuration file created successfully");
        }
    }

    Ok(())
}
