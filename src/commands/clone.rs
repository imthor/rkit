use std::path::PathBuf;
use std::process::Command;
use url::Url;
use crate::error::{RkitError, RkitResult};

pub fn clone(url: &str, project_root: &PathBuf) -> RkitResult<()> {
    log::info!("Cloning repository: {}", url);
    let parsed_url = Url::parse(url)
        .map_err(|e| {
            log::error!("Failed to parse URL: {}", e);
            RkitError::InvalidRepoUrl(format!("Failed to parse URL: {}", e))
        })?;
    
    let domain = parsed_url.host_str()
        .ok_or_else(|| {
            log::error!("No domain found in URL: {}", url);
            RkitError::InvalidRepoUrl("No domain found in URL".to_string())
        })?;
    
    let path_segments: Vec<&str> = parsed_url.path_segments()
        .ok_or_else(|| {
            log::error!("No path segments in URL: {}", url);
            RkitError::InvalidRepoUrl("No path segments in URL".to_string())
        })?
        .collect();
    
    if path_segments.len() < 2 {
        log::error!("URL must contain organization and repository: {}", url);
        return Err(RkitError::InvalidRepoUrl("URL must contain organization and repository".to_string()));
    }
    
    let org = path_segments[0];
    let repo = path_segments[1].trim_end_matches(".git");
    
    let target_dir = project_root
        .join(domain)
        .join(org)
        .join(repo);

    if let Some(parent) = target_dir.parent() {
        if !parent.exists() {
            log::debug!("Creating parent directories: {}", parent.display());
            std::fs::create_dir_all(parent)
                .map_err(|e| RkitError::DirectoryCreationError {
                    path: parent.to_path_buf(),
                    source: e,
                })?;
        } else if !parent.read_dir().is_ok() {
            log::error!("No permission to write to directory: {}", parent.display());
            return Err(RkitError::PermissionError(format!(
                "No permission to write to directory: {}", 
                parent.display()
            )));
        }
    }
    
    log::info!("Running: git clone {} {}", url, target_dir.display());
    let status = Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(&target_dir)
        .status()
        .map_err(|e| RkitError::ShellCommandError {
            command: format!("git clone {} {}", url, target_dir.display()),
            source: e,
        })?;
    
    if !status.success() {
        log::error!("git clone failed with status: {}", status);
        return Err(RkitError::GitError(format!("git clone failed with status: {}", status)));
    }
    
    log::info!("Successfully cloned {} to {}", url, target_dir.display());
    Ok(())
} 