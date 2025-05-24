use std::path::PathBuf;
use std::process::Command;
use crate::config::RViewCmd;
use crate::error::{RkitError, RkitResult};

pub fn view_repo(repo_path: &PathBuf, commands: Option<&[RViewCmd]>) -> RkitResult<()> {
    // Validate repository path
    if !repo_path.exists() {
        log::error!("Repository not found: {}", repo_path.display());
        return Err(RkitError::RepoNotFoundError(repo_path.clone()));
    }

    // Check if it's a git repository
    let git_dir = repo_path.join(".git");
    if !git_dir.exists() {
        log::error!("Not a git repository: {}", repo_path.display());
        return Err(RkitError::InvalidPathError(format!("{} is not a git repository", repo_path.display())));
    }

    // Check read permissions
    if !repo_path.read_dir().is_ok() {
        log::error!("No permission to read directory: {}", repo_path.display());
        return Err(RkitError::PermissionError(format!("No permission to read directory: {}", repo_path.display())));
    }

    if let Some(cmds) = commands {
        for cmd in cmds {
            let command_str = cmd.command.replace("{REPO}", &repo_path.to_string_lossy());
            let parts: Vec<&str> = command_str.split_whitespace().collect();
            
            if parts.is_empty() {
                log::warn!("Empty command for label: {}", cmd.label);
                continue;
            }
            
            log::debug!("Running command for {}: {}", cmd.label, command_str);
            let output = Command::new(parts[0])
                .args(&parts[1..])
                .output()
                .map_err(|e| RkitError::ShellCommandError {
                    command: cmd.command.clone(),
                    source: e,
                })?;
            
            println!("=== {} ===", cmd.label);
            if !output.stdout.is_empty() {
                println!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            }
            println!();
        }
    } else {
        // Fallback behavior
        let readme_path = repo_path.join("README.md");
        if readme_path.exists() {
            log::info!("Displaying README for {}", repo_path.display());
            println!("=== README ===");
            let content = std::fs::read_to_string(&readme_path)
                .map_err(|e| RkitError::FileReadError {
                    path: readme_path,
                    source: e,
                })?;
            println!("{}", content);
        } else {
            log::info!("Listing directory contents for {}", repo_path.display());
            println!("=== Directory Listing ===");
            let output = Command::new("ls")
                .arg("-la")
                .arg(repo_path)
                .output()
                .map_err(|e| RkitError::ShellCommandError {
                    command: format!("ls -la {}", repo_path.display()),
                    source: e,
                })?;
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
    }
    Ok(())
} 