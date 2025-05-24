use std::path::PathBuf;
use std::process::{Command, Stdio};
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
            
            println!("=== {} ===", cmd.label);
            
            log::debug!("Running command for {}: {}", cmd.label, command_str);
            let mut child = Command::new(parts[0])
                .args(&parts[1..])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .map_err(|e| RkitError::ShellCommandError {
                    command: cmd.command.clone(),
                    source: e,
                })?;

            // Wait for the command to complete
            let status = child.wait()
                .map_err(|e| RkitError::ShellCommandError {
                    command: cmd.command.clone(),
                    source: e,
                })?;

            if !status.success() {
                log::warn!("Command '{}' exited with status: {}", cmd.command, status);
            }
            
            println!(); // Add a blank line between commands
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
            let mut child = Command::new("ls")
                .arg("-la")
                .arg(repo_path)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .map_err(|e| RkitError::ShellCommandError {
                    command: format!("ls -la {}", repo_path.display()),
                    source: e,
                })?;

            child.wait()
                .map_err(|e| RkitError::ShellCommandError {
                    command: format!("ls -la {}", repo_path.display()),
                    source: e,
                })?;
        }
    }
    Ok(())
} 