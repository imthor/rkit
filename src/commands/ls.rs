use ignore::WalkBuilder;
use lazy_static::lazy_static;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use crate::cache::{Cache, CacheError};
use crate::error::RkitResult;

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
    for _ in 0..max_retries {
        if let Some(result) = f() {
            return Some(result);
        }
        thread::sleep(Duration::from_millis(100));
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
            e => log::warn!("Failed to update cache: {}", e),
        }
    }

    // Save the cache after validation
    if let Err(e) = CACHE.save() {
        match e {
            CacheError::LockError(msg) => log::warn!("Failed to acquire cache lock: {}", msg),
            CacheError::DirectoryError(e) => log::warn!("Failed to access cache directory: {}", e),
            e => log::warn!("Failed to save cache: {}", e),
        }
    }

    // Check cache first with retry
    if let Some(cached_entry) =
        retry_operation(|| CACHE.get(project_root).filter(|entry| Cache::validate_entry(entry, CACHE.ttl_seconds())), 3)
    {
        if full {
            println!("{}", cached_entry.path.display());
        } else if let Ok(relative_path) = cached_entry.path.strip_prefix(project_root) {
            println!("{}", relative_path.display());
        }
        return Ok(());
    }

    // Use a simpler, non-parallel walker
    let walker = WalkBuilder::new(project_root)
        .max_depth(config.max_depth)
        .follow_links(config.follow_links)
        .same_file_system(config.same_file_system)
        .build();

    let mut repo_count = 0;
    let mut scanned_dirs = 0;
    let mut discovered_repos = Vec::new();

    for result in walker {
        scanned_dirs += 1;
        match result {
            Ok(entry) => {
                if entry.path().join(".git").exists() {
                    repo_count += 1;
                    let path = entry.path().to_path_buf();
                    discovered_repos.push(path.clone());
                    
                    if full {
                        println!("{}", path.display());
                    } else if let Ok(relative_path) = path.strip_prefix(project_root) {
                        println!("{}", relative_path.display());
                    }
                }
            }
            Err(e) => log::error!("Error walking directory: {}", e),
        }
    }
    // Flush stdout to ensure all output is written
    io::stdout().flush().unwrap();

    // Cache all discovered repositories
    if !discovered_repos.is_empty() {
        if let Err(e) = CACHE.update_and_save_many(&discovered_repos) {
            match e {
                CacheError::LockError(msg) => log::warn!("Failed to acquire cache lock: {}", msg),
                CacheError::DirectoryError(e) => log::warn!("Failed to access cache directory: {}", e),
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
