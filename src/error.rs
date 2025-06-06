// Remove unused imports

#[derive(Debug, thiserror::Error)]
pub enum RkitError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Failed to create directory: {path}")]
    DirectoryCreationError {
        path: std::path::PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to write file: {path}")]
    FileWriteError {
        path: std::path::PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to read file: {path}")]
    FileReadError {
        path: std::path::PathBuf,
        source: std::io::Error,
    },

    #[error("Permission denied: {0}")]
    PermissionError(String),

    #[error("Invalid repository URL: {0}")]
    InvalidRepoUrl(String),

    #[error("Git operation failed: {0}")]
    GitError(String),

    #[error("Shell command execution failed: {command}")]
    ShellCommandError {
        command: String,
        source: std::io::Error,
    },

    #[error("Repository not found: {0}")]
    RepoNotFoundError(std::path::PathBuf),

    #[error("Invalid path: {0}")]
    InvalidPathError(String),

    #[error("Environment variable not found: {0}")]
    EnvVarError(String),
}

// Type alias for Result type using our custom error
pub type RkitResult<T> = Result<T, RkitError>;
