use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod config;
mod error;

use commands::ls::WalkerConfig;
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
    Clone {
        /// Git repository URL to clone
        url: String,
    },
    /// List Git repositories in workspace
    Ls {
        /// Show full paths instead of relative paths
        #[arg(short, long)]
        full: bool,
        /// Maximum depth to search for repositories [default: 10]
        #[arg(long)]
        max_depth: Option<usize>,
        /// Follow symbolic links [default: false]
        #[arg(long)]
        follow_links: bool,
        /// Stay on the same filesystem [default: true]
        #[arg(long)]
        same_file_system: bool,
        /// Number of threads to use for searching [default: number of CPU cores]
        #[arg(long)]
        threads: Option<usize>,
        /// Maximum number of repositories to find [default: no limit]
        #[arg(long)]
        max_repos: Option<usize>,
    },
    /// View repository information
    View {
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
        Commands::Clone { url } => {
            commands::clone::clone(&url, &project_root)?;
        }
        Commands::Ls {
            full,
            max_depth,
            follow_links,
            same_file_system,
            threads,
            max_repos,
        } => {
            let config = WalkerConfig {
                max_depth,
                follow_links,
                same_file_system,
                threads: threads.unwrap_or_else(num_cpus::get),
                max_repos,
            };
            commands::ls::list_repos(&project_root, full, Some(config))?;
        }
        Commands::View { path } => {
            let repo_path = if path.is_absolute() {
                path
            } else {
                project_root.join(path)
            };
            commands::view::view_repo(&repo_path, config.rview.as_deref())?;
        }
    }

    Ok(())
}
