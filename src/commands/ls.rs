use rayon::prelude::*;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use walkdir::WalkDir;

use crate::error::RkitResult;

pub fn list_repos(project_root: &PathBuf, full: bool) -> RkitResult<()> {
    log::debug!(
        "Listing repositories in workspace: {}",
        project_root.display()
    );

    // Create a channel for collecting results
    let (tx, rx) = mpsc::channel();

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

    // Use WalkDir with parallel iterator
    WalkDir::new(project_root)
        .into_iter()
        .par_bridge() // Convert to parallel iterator
        .filter_map(|e| e.ok())
        .filter(|entry| entry.file_name() == ".git")
        .for_each(|entry| {
            if let Some(repo_path) = entry.path().parent() {
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

    // Drop the sender to signal completion
    drop(tx);

    // Wait for the printing thread to finish
    handle.join().unwrap()?;

    Ok(())
}
