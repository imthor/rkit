use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod config;
mod commands;
mod error;

use error::RkitResult;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Increase output verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Commands {
    /// Smart Git clone wrapper
    Rclone {
        /// Git repository URL to clone
        url: String,
    },
    /// List Git repositories in workspace
    Rls {
        /// Show full paths instead of relative paths
        #[arg(short, long)]
        full: bool,
    },
    /// View repository information
    Rview {
        /// Path to repository
        path: PathBuf,
    },
}

fn main() -> RkitResult<()> {
    let cli = Cli::parse();

    // Set log level based on verbosity
    let log_level = match cli.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    std::env::set_var("RUST_LOG", log_level);
    env_logger::init();
    log::debug!("Logger initialized at {} level", log_level);

    let config = config::Config::load_or_create()?;
    let project_root = config.expand_project_root()?;

    match cli.command {
        Commands::Rclone { url } => {
            commands::rclone::clone(&url, &project_root)?;
        }
        Commands::Rls { full } => {
            commands::rls::list_repos(&project_root, full)?;
        }
        Commands::Rview { path } => {
            let repo_path = if path.is_absolute() {
                path
            } else {
                project_root.join(path)
            };
            commands::rview::view_repo(&repo_path, config.rview.as_deref())?;
        }
    }

    Ok(())
} 