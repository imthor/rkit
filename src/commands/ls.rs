use ignore::WalkBuilder;
use lazy_static::lazy_static;
use rayon::prelude::*;
use std::collections::HashSet;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use crate::error::RkitResult;

lazy_static! {
    static ref CACHE: Mutex<HashSet<PathBuf>> = Mutex::new(HashSet::new());
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
            max_depth: Some(10),
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

    // Check cache first with retry
    if let Some(cached_path) = retry_operation(
        || {
            CACHE
                .lock()
                .ok()
                .and_then(|cache| cache.get(project_root).cloned())
        },
        3,
    ) {
        log::debug!("Cache hit for {}", project_root.display());
        if full {
            println!("{}", cached_path.display());
        } else if let Ok(relative_path) = cached_path.strip_prefix(project_root) {
            println!("{}", relative_path.display());
        }
        return Ok(());
    }

    log::debug!(
        "Listing repositories in workspace: {}",
        project_root.display()
    );

    // Create a channel for collecting results with a larger buffer
    let (tx, rx) = mpsc::sync_channel(1000); // Bounded channel with buffer

    // Spawn a thread to collect and print results
    let handle = thread::spawn(move || {
        for path in rx {
            if let Err(e) = writeln!(io::stdout(), "{}", path) {
                if e.kind() == io::ErrorKind::BrokenPipe {
                    return Ok::<(), std::io::Error>(());
                }
                return Err(e);
            }
        }
        Ok(())
    });

    // Configure walker with optimized settings
    let walker = WalkBuilder::new(project_root)
        .hidden(false) // Don't ignore hidden files
        .ignore(true) // Use .ignore files
        .git_ignore(true) // Use .gitignore
        .git_global(false) // Don't use global gitignore
        .git_exclude(false) // Don't use git exclude
        .follow_links(config.follow_links)
        .same_file_system(config.same_file_system)
        .max_depth(config.max_depth)
        .threads(config.threads)
        .build();

    let repo_count = AtomicUsize::new(0);
    let scanned_dirs = AtomicUsize::new(0);

    // Process results and stream them directly
    walker
        .into_iter()
        .par_bridge()
        .filter_map(|result| {
            scanned_dirs.fetch_add(1, Ordering::Relaxed);
            result.ok()
        })
        .filter(|entry| {
            if entry.file_name() == ".git" {
                // If stop_at_git is enabled, check if any parent directory has a .git directory
                if config.stop_at_git {
                    if let Some(parent) = entry.path().parent() {
                        // Check all parent directories up to the project root
                        let mut current = parent;
                        while let Some(parent) = current.parent() {
                            // If we've reached the project root, we're done checking
                            if parent == project_root {
                                break;
                            }
                            // If any parent directory has a .git directory, skip this entry
                            if parent.join(".git").exists() {
                                return false;
                            }
                            current = parent;
                        }
                    }
                }
                true
            } else {
                false
            }
        })
        .for_each(|entry| {
            if let Some(repo_path) = entry.path().parent() {
                let count = repo_count.fetch_add(1, Ordering::Relaxed) + 1;

                // Early exit if max_repos is reached
                if let Some(max) = config.max_repos {
                    if count > max {
                        return;
                    }
                }

                let path_str = if full {
                    repo_path.display().to_string()
                } else if let Ok(relative_path) = repo_path.strip_prefix(project_root) {
                    relative_path.display().to_string()
                } else {
                    return;
                };

                let _ = tx.send(path_str);
            }
        });

    // Update metrics before logging
    let metrics = PerformanceMetrics {
        total_duration: start.elapsed(),
        repo_count: repo_count.load(Ordering::Relaxed),
        scanned_dirs: scanned_dirs.load(Ordering::Relaxed),
    };

    log::debug!("Found {} repositories", metrics.repo_count);
    log::debug!("Scanned {} directories", metrics.scanned_dirs);
    log::debug!("Total duration: {:?}", metrics.total_duration);
    log::debug!("Performance metrics: {:?}", metrics);

    // Drop the sender to signal completion
    drop(tx);

    handle.join().unwrap()?;

    // After successful processing, add to cache with retry
    if let Some(repo_path) = project_root.parent() {
        let _ = retry_operation(
            || {
                CACHE
                    .lock()
                    .ok()
                    .map(|mut cache| cache.insert(repo_path.to_path_buf()))
            },
            3,
        );
    }

    Ok(())
}
