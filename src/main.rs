use clap::{Parser, Subcommand};
use std::path::PathBuf;

use rkit::commands;
use rkit::commands::ls::WalkerConfig;
use rkit::config;
use rkit::error::RkitResult;

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
    let args = Cli::parse();

    // Initialize logger with minimal output by default
    let log_level = match args.verbose {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level.as_str()))
        .format_timestamp(None)
        .format_module_path(false)
        .format_target(false)
        .init();

    // Get project root from config or use default
    let project_root = config::Config::load_or_create()?.expand_project_root()?;

    match args.command {
        Commands::Clone { url } => {
            log::info!("Cloning repository: {}", url);
            commands::clone::clone(&url, &project_root)
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
                stop_at_git: true,
            };
            commands::ls::list_repos(&project_root, full, Some(config))
        }
        Commands::View { path } => {
            log::info!("Viewing repository: {}", path.display());
            let repo_path = if path.is_absolute() {
                path
            } else {
                project_root.join(path)
            };
            commands::view::view_repo(&repo_path, None)
        }
    }
}
