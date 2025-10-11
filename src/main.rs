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
        /// Source package URLs (comma-separated or multiple arguments)
        #[arg(short, long, value_delimiter = ',')]
        sources: Vec<String>,

        /// Target repository type (prefix-dev, s3, or local)
        #[arg(short, long)]
        target_type: String,

        /// Target repository path or URL
        #[arg(short = 'p', long)]
        target_path: String,

        /// Configuration file (optional)
        #[arg(short, long)]
        config: Option<String>,
    },
    /// Initialize configuration file
    InitConfig {
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
            sources,
            target_type,
            target_path,
            config,
        } => {
            info!("Starting package mirroring");

            let config = if let Some(config_path) = config {
                Config::load_from_file(&config_path)?
            } else {
                Config::default()
            };

            let repo_type = RepositoryType::from_string(&target_type)?;

            mirror_packages(&sources, repo_type, &target_path, &config).await?;

            info!("Mirroring completed successfully");
        }
        Commands::InitConfig { output } => {
            info!("Initializing configuration file at: {}", output);
            let config = Config::default();
            config.save_to_file(&output)?;
            info!("Configuration file created successfully");
        }
    }

    Ok(())
}
