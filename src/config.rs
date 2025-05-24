use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::error::{RkitError, RkitResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct RViewCmd {
    pub command: String,
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub project_root: String,
    pub rview: Option<Vec<RViewCmd>>,
}

impl Config {
    fn get_default_config() -> RkitResult<Config> {
        let default_config_path = if cfg!(windows) {
            "etc/default_config_windows.yaml"
        } else {
            "etc/default_config_linux.yaml"
        };

        let config_str = fs::read_to_string(default_config_path)
            .map_err(|e| RkitError::FileReadError {
                path: PathBuf::from(default_config_path),
                source: e,
            })?;
        
        let config: Config = serde_yaml::from_str(&config_str)?;
        Ok(config)
    }

    pub fn load_or_create() -> RkitResult<Self> {
        // Use platform-specific config directory
        let config_dir = if cfg!(windows) {
            // On Windows, use %APPDATA%\rkit
            dirs::config_dir()
                .ok_or_else(|| RkitError::ConfigError("Could not find config directory".to_string()))?
                .join("rkit")
        } else {
            // On Unix-like systems, use ~/.config/rkit
            PathBuf::from(shellexpand::tilde("~/.config/rkit").as_ref())
        };

        fs::create_dir_all(&config_dir)
            .map_err(|e| RkitError::DirectoryCreationError {
                path: config_dir.clone(),
                source: e,
            })?;

        let config_path = config_dir.join("config.yaml");

        if !config_path.exists() {
            let default_config = Self::get_default_config()?;
            let yaml = serde_yaml::to_string(&default_config)?;
            fs::write(&config_path, yaml)
                .map_err(|e| RkitError::FileWriteError {
                    path: config_path.clone(),
                    source: e,
                })?;
        }

        let config_str = fs::read_to_string(&config_path)
            .map_err(|e| RkitError::FileReadError {
                path: config_path,
                source: e,
            })?;
        let config: Config = serde_yaml::from_str(&config_str)?;

        Ok(config)
    }

    pub fn expand_project_root(&self) -> RkitResult<PathBuf> {
        let expanded = if cfg!(windows) {
            // On Windows, expand %USERPROFILE% environment variable
            std::env::var("USERPROFILE")
                .map_err(|_| RkitError::EnvVarError("USERPROFILE".to_string()))?
                + &self.project_root.replace("%USERPROFILE%", "")
        } else {
            // On Unix-like systems, expand ~
            shellexpand::tilde(&self.project_root).to_string()
        };
        Ok(PathBuf::from(expanded))
    }
} 