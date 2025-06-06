use crate::error::{RkitError, RkitResult};
use std::path::Path;
use std::process::Command;
use url::Url;
use crate::cache::Cache;
use lazy_static::lazy_static;

lazy_static! {
    static ref CACHE: Cache = Cache::new();
}

#[derive(Debug)]
pub struct ParsedRepoUrl {
    pub domain: String,
    pub org: String,
    pub repo: String,
}

fn trim_git_suffix(repo: &str) -> &str {
    repo.trim_end_matches(".git")
}

fn parse_https_url(url: &str) -> RkitResult<ParsedRepoUrl> {
    let parsed_url = Url::parse(url).map_err(|e| {
        log::error!("Failed to parse HTTPS URL: {}", e);
        RkitError::InvalidRepoUrl(format!("Failed to parse HTTPS URL: {}", e))
    })?;

    let domain = parsed_url.host_str().ok_or_else(|| {
        log::error!("No domain found in HTTPS URL: {}", url);
        RkitError::InvalidRepoUrl("No domain found in HTTPS URL".to_string())
    })?;

    let path_segments: Vec<&str> = parsed_url
        .path_segments()
        .ok_or_else(|| {
            log::error!("No path segments in HTTPS URL: {}", url);
            RkitError::InvalidRepoUrl("No path segments in HTTPS URL".to_string())
        })?
        .collect();

    if path_segments.len() < 2 {
        log::error!(
            "HTTPS URL must contain organization and repository: {}",
            url
        );
        return Err(RkitError::InvalidRepoUrl(
            "HTTPS URL must contain organization and repository".to_string(),
        ));
    }

    Ok(ParsedRepoUrl {
        domain: domain.to_string(),
        org: path_segments[0].to_string(),
        repo: trim_git_suffix(path_segments[1]).to_string(),
    })
}

fn parse_ssh_url(url: &str) -> RkitResult<ParsedRepoUrl> {
    // Handle potential port number in domain
    let (domain, path) = if let Some(idx) = url.rfind(':') {
        let domain_part = &url[url.find('@').ok_or_else(|| {
            log::error!("Invalid SSH URL format (no @ symbol): {}", url);
            RkitError::InvalidRepoUrl("Invalid SSH URL format (no @ symbol)".to_string())
        })? + 1..idx];

        // Remove port number if present
        let domain = domain_part.split(':').next().ok_or_else(|| {
            log::error!("No domain found in SSH URL: {}", url);
            RkitError::InvalidRepoUrl("No domain found in SSH URL".to_string())
        })?;

        (domain, &url[idx + 1..])
    } else {
        log::error!("Invalid SSH URL format (no path separator): {}", url);
        return Err(RkitError::InvalidRepoUrl(
            "Invalid SSH URL format (no path separator)".to_string(),
        ));
    };

    let path_parts: Vec<&str> = path.split('/').collect();
    if path_parts.len() != 2 {
        log::error!("SSH URL must contain organization and repository: {}", url);
        return Err(RkitError::InvalidRepoUrl(
            "SSH URL must contain organization and repository".to_string(),
        ));
    }

    Ok(ParsedRepoUrl {
        domain: domain.to_string(),
        org: path_parts[0].to_string(),
        repo: trim_git_suffix(path_parts[1]).to_string(),
    })
}

pub fn parse_repo_url(url: &str) -> RkitResult<ParsedRepoUrl> {
    // Try parsing as HTTPS URL first
    if url.starts_with("http://") || url.starts_with("https://") {
        return parse_https_url(url);
    }

    // Try parsing as SSH URL
    if url.contains('@') {
        return parse_ssh_url(url);
    }

    Err(RkitError::InvalidRepoUrl(
        "URL must be either HTTPS or SSH format".to_string(),
    ))
}

pub fn clone(url: &str, project_root: &Path) -> RkitResult<()> {
    log::info!("Cloning repository: {}", url);

    let parsed_url = parse_repo_url(url)?;

    let target_dir = project_root
        .join(&parsed_url.domain)
        .join(&parsed_url.org)
        .join(&parsed_url.repo);

    if let Some(parent) = target_dir.parent() {
        if !parent.exists() {
            log::debug!("Creating parent directories: {}", parent.display());
            std::fs::create_dir_all(parent).map_err(|e| RkitError::DirectoryCreationError {
                path: parent.to_path_buf(),
                source: e,
            })?;
        } else if parent.read_dir().is_err() {
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
        return Err(RkitError::GitError(format!(
            "git clone failed with status: {}",
            status
        )));
    }

    // Cache the newly cloned repository
    if let Err(e) = CACHE.update_and_save(&target_dir) {
        log::warn!("Failed to cache cloned repository: {}", e);
    }

    log::info!("Successfully cloned {} to {}", url, target_dir.display());
    Ok(())
}
