use ignore::{WalkBuilder, WalkState};
use lazy_static::lazy_static;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

use crate::cache::{Cache, CacheError};
use crate::error::{RkitError, RkitResult};

lazy_static! {
    static ref CACHE: Cache = Cache::new();
}

#[derive(Debug)]
struct PerformanceMetrics {
    total_duration: Duration,
    repo_count: usize,
    scanned_dirs: usize,
}

#[derive(Debug, Clone)]
pub struct WalkerConfig {
    pub max_depth: Option<usize>,
    pub follow_links: bool,
    pub same_file_system: bool,
    pub threads: usize,
    pub max_repos: Option<usize>,
    pub stop_at_git: bool,
}

impl Default for WalkerConfig {
    fn default() -> Self {
        Self {
            max_depth: Some(4),
            follow_links: false,
            same_file_system: true,
            threads: num_cpus::get(),
            max_repos: None,
            stop_at_git: true,
        }
    }
}

fn retry_operation<F, T>(f: F, max_retries: u32) -> Option<T>
where
    F: Fn() -> Option<T>,
{
    for attempt in 0..max_retries {
        if let Some(result) = f() {
            return Some(result);
        }
        if attempt < max_retries - 1 {
            log::debug!("Retry attempt {} of {}", attempt + 1, max_retries);
            thread::sleep(Duration::from_millis(100 * (attempt + 1) as u64));
        }
    }
    None
}

pub fn list_repos(
    project_root: &PathBuf,
    full: bool,
    config: Option<WalkerConfig>,
) -> RkitResult<()> {
    let config = config.unwrap_or_default();
    let start = Instant::now();

    // Validate and update cache before checking
    if let Err(e) = CACHE.validate_and_update() {
        match e {
            CacheError::LockError(msg) => log::warn!("Failed to acquire cache lock: {}", msg),
            CacheError::DirectoryError(e) => log::warn!("Failed to access cache directory: {}", e),
            CacheError::IoError(e) => log::warn!("Failed to write cache: {}", e),
            e => log::warn!("Failed to update cache: {}", e),
        }
    }

    // Check cache first with retry
    if let Some(cached_entry) = retry_operation(
        || {
            CACHE
                .get(project_root)
                .filter(|entry| Cache::validate_entry(entry, CACHE.ttl_seconds()))
        },
        3,
    ) {
        if full {
            println!("{}", cached_entry.path.display());
        } else if let Ok(relative_path) = cached_entry.path.strip_prefix(project_root) {
            println!("{}", relative_path.display());
        }
        return Ok(());
    }

    // Build parallel walker using configured threads
    let walker = WalkBuilder::new(project_root)
        .max_depth(config.max_depth)
        .follow_links(config.follow_links)
        .same_file_system(config.same_file_system)
        .threads(config.threads)
        .build_parallel();

    let repo_count = Arc::new(AtomicUsize::new(0));
    let scanned_dirs = Arc::new(AtomicUsize::new(0));
    let discovered_repos = Arc::new(Mutex::new(Vec::new()));

    walker.run(|| {
        let repo_count = Arc::clone(&repo_count);
        let scanned_dirs = Arc::clone(&scanned_dirs);
        let discovered_repos = Arc::clone(&discovered_repos);
        Box::new(move |result| {
            scanned_dirs.fetch_add(1, Ordering::SeqCst);
            match result {
                Ok(entry) => {
                    if entry.path().join(".git").exists() {
                        repo_count.fetch_add(1, Ordering::SeqCst);
                        let path = entry.path().to_path_buf();
                        {
                            discovered_repos.lock().unwrap().push(path.clone());
                        }

                        if full {
                            println!("{}", path.display());
                        } else if let Ok(relative_path) = path.strip_prefix(project_root) {
                            println!("{}", relative_path.display());
                        }

                        if let Some(max_repos) = config.max_repos {
                            if repo_count.load(Ordering::SeqCst) >= max_repos {
                                log::info!(
                                    "Reached maximum number of repositories ({})",
                                    max_repos
                                );
                                return WalkState::Quit;
                            }
                        }

                        if config.stop_at_git
                            && entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                        {
                            return WalkState::Skip;
                        }
                    }
                }
                Err(e) => log::error!("Error walking directory: {}", e),
            }
            WalkState::Continue
        })
    });

    let repo_count = repo_count.load(Ordering::SeqCst);
    let scanned_dirs = scanned_dirs.load(Ordering::SeqCst);
    let discovered_repos = Arc::try_unwrap(discovered_repos)
        .unwrap()
        .into_inner()
        .unwrap();
    // Flush stdout to ensure all output is written
    io::stdout().flush().map_err(RkitError::IoError)?;

    // Cache all discovered repositories
    if !discovered_repos.is_empty() {
        if let Err(e) = CACHE.update_and_save_many(&discovered_repos) {
            match e {
                CacheError::LockError(msg) => log::warn!("Failed to acquire cache lock: {}", msg),
                CacheError::DirectoryError(e) => {
                    log::warn!("Failed to access cache directory: {}", e)
                }
                CacheError::IoError(e) => log::warn!("Failed to write cache: {}", e),
                e => log::warn!("Failed to save discovered repositories to cache: {}", e),
            }
        }
    }

    let metrics = PerformanceMetrics {
        total_duration: start.elapsed(),
        repo_count,
        scanned_dirs,
    };

    log::info!(
        "Scanned {} directories, found {} repositories in {:?}",
        metrics.scanned_dirs,
        metrics.repo_count,
        metrics.total_duration
    );

    Ok(())
}
