use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

use crate::error::RkitError;

/// Configuration options for the cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Time-to-live for cache entries in seconds
    pub ttl_seconds: u64,
    /// Maximum number of entries in the cache (None for unlimited)
    pub max_entries: Option<usize>,
    /// Custom cache file path (None for default)
    pub cache_path: Option<PathBuf>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            ttl_seconds: 24 * 60 * 60, // 24 hours
            max_entries: None,
            cache_path: None,
        }
    }
}

/// Represents errors that can occur in the cache module
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Failed to acquire lock: {0}")]
    LockError(String),

    #[error("Cache entry expired: {0}")]
    EntryExpired(PathBuf),

    #[error("Invalid cache version: {0}")]
    InvalidVersion(u32),

    #[error("Failed to read/write cache file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse cache: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Failed to get system time: {0}")]
    TimeError(String),

    #[error("Failed to create cache directory: {0}")]
    DirectoryError(#[from] RkitError),

    #[error("Cache is full: {0} entries")]
    CacheFull(usize),

    #[error("Invalid cache entry: {0}")]
    InvalidEntryError(String),
}

// Type alias for cache-specific results
type CacheResult<T> = Result<T, CacheError>;

/// Represents a cache entry for a git repository
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheEntry {
    /// The path to the repository
    pub path: PathBuf,
    /// Last modification time of the repository
    pub last_modified: u64,
    /// Last time the cache entry was validated
    pub last_checked: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheData {
    entries: HashMap<PathBuf, CacheEntry>,
    version: u32,
}

pub struct Cache {
    entries: RwLock<HashMap<PathBuf, CacheEntry>>,
    cache_path: PathBuf,
    pub config: CacheConfig,
}

impl Cache {
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    pub fn with_config(config: CacheConfig) -> Self {
        let cache_path = config
            .cache_path
            .clone()
            .unwrap_or_else(|| match get_cache_path() {
                Ok(path) => path,
                Err(e) => {
                    log::warn!("Failed to get cache path: {}", e);
                    let temp_path = env::temp_dir().join("rkit").join("cache.json");
                    if let Err(e) = validate_cache_path(&temp_path) {
                        log::error!("Failed to validate temp cache path: {}", e);
                    }
                    temp_path
                }
            });

        let entries = match load_cache(&cache_path) {
            Ok(entries) => entries,
            Err(e) => {
                log::warn!("Failed to load cache: {}", e);
                HashMap::new()
            }
        };
        Self {
            entries: RwLock::new(entries),
            cache_path,
            config,
        }
    }

    pub fn get(&self, path: &Path) -> Option<CacheEntry> {
        self.entries
            .read()
            .ok()
            .and_then(|entries| entries.get(path).cloned())
    }

    pub fn insert(&self, path: PathBuf, entry: CacheEntry) -> CacheResult<()> {
        // Validate entry before insertion
        if !Self::validate_entry(&entry, self.config.ttl_seconds) {
            return Err(CacheError::InvalidEntryError(format!(
                "Invalid cache entry for path: {}",
                path.display()
            )));
        }

        let mut entries = self
            .entries
            .write()
            .map_err(|_| CacheError::LockError("Failed to acquire cache write lock".to_string()))?;

        // Check if we need to enforce max entries
        if let Some(max_entries) = self.config.max_entries {
            if entries.len() >= max_entries {
                return Err(CacheError::CacheFull(max_entries));
            }
        }

        log::debug!("Inserting cache entry for path: {}", path.display());
        entries.insert(path, entry);
        log::debug!("Current cache size: {} entries", entries.len());

        // Save without acquiring another lock
        self.save_with_entries(&entries)?;
        Ok(())
    }

    pub fn validate_and_update(&self) -> CacheResult<()> {
        let mut entries = self
            .entries
            .write()
            .map_err(|_| CacheError::LockError("Failed to acquire cache write lock".to_string()))?;

        // Use retain for in-place filtering
        entries.retain(|path, entry| {
            let is_valid = Self::validate_entry(entry, self.config.ttl_seconds);
            if !is_valid {
                log::debug!("Removing invalid cache entry: {}", path.display());
            }
            is_valid
        });

        // Save the updated entries
        self.save_with_entries(&entries)?;

        Ok(())
    }

    /// Validates multiple paths in a single operation
    /// Returns a vector of paths that have valid cache entries
    pub fn validate_entries(&self, paths: &[PathBuf]) -> CacheResult<Vec<PathBuf>> {
        let entries = self
            .entries
            .read()
            .map_err(|_| CacheError::LockError("Failed to acquire cache read lock".to_string()))?;

        Ok(paths
            .iter()
            .filter(|path| {
                entries
                    .get(*path)
                    .map(|entry| Self::validate_entry(entry, self.config.ttl_seconds))
                    .unwrap_or(false)
            })
            .cloned()
            .collect())
    }

    pub fn save(&self) -> CacheResult<()> {
        let entries = self
            .entries
            .read()
            .map_err(|_| CacheError::LockError("Failed to acquire cache read lock".to_string()))?;

        self.save_with_entries(&entries)
    }

    fn save_with_entries(&self, entries: &HashMap<PathBuf, CacheEntry>) -> CacheResult<()> {
        if let Some(parent) = self.cache_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                CacheError::DirectoryError(RkitError::DirectoryCreationError {
                    path: parent.to_path_buf(),
                    source: e,
                })
            })?;
        }

        let cache_data = CacheData {
            entries: entries
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            version: 1,
        };

        log::debug!("Saving cache to: {}", self.cache_path.display());
        log::debug!("Cache entries to save: {}", entries.len());

        // Create a temporary file for atomic write
        let temp_path = self.cache_path.with_extension("tmp");
        let json = serde_json::to_string_pretty(&cache_data)?;

        // Write to temp file
        if let Err(e) = fs::write(&temp_path, json) {
            // Clean up temp file on error
            let _ = fs::remove_file(&temp_path);
            return Err(CacheError::IoError(e));
        }

        // Atomic rename
        if let Err(e) = fs::rename(&temp_path, &self.cache_path) {
            // Clean up temp file on error
            let _ = fs::remove_file(&temp_path);
            return Err(CacheError::IoError(e));
        }

        Ok(())
    }

    pub fn validate_entry(entry: &CacheEntry, ttl_seconds: u64) -> bool {
        let now = match get_current_time() {
            Ok(time) => time,
            Err(e) => {
                log::error!("Failed to get current time: {}", e);
                return false;
            }
        };

        // Check if the entry has expired
        if now - entry.last_checked > ttl_seconds {
            log::debug!("Cache entry expired for path: {}", entry.path.display());
            return false;
        }

        // Check if the path exists and is a git repository
        let path_exists = entry.path.exists();
        let is_git_repo = entry.path.join(".git").exists();

        if !path_exists {
            log::debug!("Cache entry path does not exist: {}", entry.path.display());
        }
        if !is_git_repo {
            log::debug!(
                "Cache entry is not a git repository: {}",
                entry.path.display()
            );
        }

        path_exists && is_git_repo
    }

    pub fn update_entry(path: &Path) -> CacheEntry {
        let now = get_current_time().unwrap_or_else(|e| {
            log::error!("Failed to get current time: {}", e);
            0
        });

        let metadata = fs::metadata(path).ok();
        let last_modified = metadata
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(now);

        CacheEntry {
            path: path.to_path_buf(),
            last_modified,
            last_checked: now,
        }
    }

    /// Updates an existing entry and saves it to the cache
    pub fn update_and_save(&self, path: &Path) -> CacheResult<()> {
        log::debug!(
            "Updating and saving cache entry for path: {}",
            path.display()
        );
        let entry = Self::update_entry(path);
        self.insert(path.to_path_buf(), entry)
    }

    /// Updates multiple entries and saves them to the cache
    pub fn update_and_save_many(&self, paths: &[PathBuf]) -> CacheResult<()> {
        log::debug!("Updating and saving {} cache entries", paths.len());
        let mut entries = self
            .entries
            .write()
            .map_err(|_| CacheError::LockError("Failed to acquire cache write lock".to_string()))?;

        for path in paths {
            let entry = Self::update_entry(path);
            entries.insert(path.clone(), entry);
        }

        self.save_with_entries(&entries)
    }

    /// Get the TTL in seconds for cache entries
    pub fn ttl_seconds(&self) -> u64 {
        self.config.ttl_seconds
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

fn get_current_time() -> CacheResult<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| CacheError::TimeError(format!("Failed to get system time: {}", e)))
        .map(|d| d.as_secs())
}

/// Validates a cache path and ensures its parent directory exists
fn validate_cache_path(path: &Path) -> CacheResult<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| {
                CacheError::DirectoryError(RkitError::DirectoryCreationError {
                    path: parent.to_path_buf(),
                    source: e,
                })
            })?;
        }
    }

    if path.exists() && !path.is_file() {
        return Err(CacheError::DirectoryError(RkitError::ConfigError(format!(
            "Cache path exists but is not a file: {}",
            path.display()
        ))));
    }

    Ok(())
}

/// Gets the platform-specific cache path
fn get_cache_path() -> CacheResult<PathBuf> {
    let config_dir = if cfg!(windows) {
        dirs::config_dir().ok_or_else(|| {
            CacheError::DirectoryError(RkitError::ConfigError(
                "Failed to get Windows config directory".to_string(),
            ))
        })?
    } else {
        let home = dirs::home_dir().ok_or_else(|| {
            CacheError::DirectoryError(RkitError::ConfigError(
                "Failed to get home directory".to_string(),
            ))
        })?;
        home.join(".config")
    };

    let cache_path = config_dir.join("rkit").join("cache.json");
    validate_cache_path(&cache_path)?;
    Ok(cache_path)
}

fn load_cache(cache_path: &Path) -> CacheResult<HashMap<PathBuf, CacheEntry>> {
    if !cache_path.exists() {
        return Ok(HashMap::new());
    }

    let contents = fs::read_to_string(cache_path)?;
    match serde_json::from_str::<CacheData>(&contents) {
        Ok(data) if data.version == 1 => Ok(data.entries),
        Ok(data) => {
            log::warn!("Invalid cache version: {}", data.version);
            Err(CacheError::InvalidVersion(data.version))
        }
        Err(e) => {
            log::warn!("Failed to parse cache file: {}", e);
            Err(CacheError::ParseError(e))
        }
    }
}
