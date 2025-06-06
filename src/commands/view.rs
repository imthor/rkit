use std::path::PathBuf;
use std::process::Command;

use crate::cache::{Cache, CacheError};
use crate::error::{RkitError, RkitResult};
use lazy_static::lazy_static;

lazy_static! {
    static ref CACHE: Cache = Cache::new();
}

pub fn view_repo(project_root: &PathBuf) -> RkitResult<()> {
    // Validate and update cache before checking
    if let Err(e) = Cache::new().validate_and_update() {
        match e {
            CacheError::LockError(msg) => log::warn!("Failed to acquire cache lock: {}", msg),
            CacheError::DirectoryError(e) => log::warn!("Failed to access cache directory: {}", e),
            CacheError::IoError(e) => log::warn!("Failed to write cache: {}", e),
            e => log::warn!("Failed to update cache: {}", e),
        }
    }

    // Check if the directory is a git repository
    if !project_root.join(".git").exists() {
        return Err(RkitError::InvalidPathError(format!(
            "{} is not a git repository",
            project_root.display()
        )));
    }

    // Get the git remote URL
    let output = Command::new("git")
        .current_dir(project_root)
        .args(["remote", "get-url", "origin"])
        .output()
        .map_err(RkitError::IoError)?;

    if !output.status.success() {
        return Err(RkitError::GitError(format!(
            "Failed to get git remote URL: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let remote_url = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Open the repository in the default browser
    let url = if remote_url.starts_with("git@") {
        // Convert SSH URL to HTTPS URL
        remote_url
            .replace("git@", "https://")
            .replace(":", "/")
            .replace(".git", "")
    } else {
        remote_url.replace(".git", "")
    };

    // Use the appropriate command based on the OS
    #[cfg(target_os = "macos")]
    let status = Command::new("open")
        .arg(&url)
        .status()
        .map_err(RkitError::IoError)?;

    #[cfg(target_os = "linux")]
    let status = Command::new("xdg-open")
        .arg(&url)
        .status()
        .map_err(RkitError::IoError)?;

    #[cfg(target_os = "windows")]
    let status = Command::new("cmd")
        .args(["/C", "start", &url])
        .status()
        .map_err(RkitError::IoError)?;

    if !status.success() {
        return Err(RkitError::ShellCommandError {
            command: format!("Failed to open browser for URL: {}", url),
            source: std::io::Error::other("Browser command failed"),
        });
    }

    Ok(())
}
