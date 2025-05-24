use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RkitError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Failed to create directory: {path}")]
    DirectoryCreationError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to read file: {path}")]
    FileReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write file: {path}")]
    FileWriteError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse YAML: {0}")]
    YamlParseError(#[from] serde_yaml::Error),

    #[error("Invalid repository URL: {0}")]
    InvalidRepoUrl(String),

    #[error("Git operation failed: {0}")]
    GitError(String),

    #[error("Shell command execution failed: {command}")]
    ShellCommandError {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Environment variable not found: {0}")]
    EnvVarError(String),

    #[error("Invalid path: {0}")]
    InvalidPathError(String),

    #[error("Repository not found: {0}")]
    RepoNotFoundError(PathBuf),

    #[error("Permission denied: {0}")]
    PermissionError(String),
}

// Type alias for Result type using our custom error
pub type RkitResult<T> = Result<T, RkitError>; 