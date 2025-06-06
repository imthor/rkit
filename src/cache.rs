use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

use crate::error::RkitError;

/// Represents errors that can occur in the cache module
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Failed to acquire lock: {0}")]
    LockError(String),

    #[error("Cache entry expired: {0}")]
    EntryExpired(PathBuf),

    #[error("Invalid cache version: {0}")]
    InvalidVersion(u32),

    #[error("Failed to read cache file: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse cache: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Failed to get system time: {0}")]
    TimeError(String),

    #[error("Failed to create cache directory: {0}")]
    DirectoryError(#[from] RkitError),
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
}

impl Cache {
    pub fn new() -> Self {
        let entries = match load_cache() {
            Ok(entries) => entries,
            Err(e) => {
                log::warn!("Failed to load cache: {}", e);
                HashMap::new()
            }
        };
        Self {
            entries: RwLock::new(entries),
        }
    }

    pub fn get(&self, path: &Path) -> Option<CacheEntry> {
        self.entries
            .read()
            .ok()
            .and_then(|entries| entries.get(path).cloned())
    }

    pub fn insert(&self, path: PathBuf, entry: CacheEntry) -> CacheResult<()> {
        let mut entries = self.entries.write().map_err(|_| {
            CacheError::LockError("Failed to acquire cache write lock".to_string())
        })?;
        entries.insert(path, entry);
        self.save()?;
        Ok(())
    }

    pub fn validate_and_update(&self) -> CacheResult<()> {
        let mut entries = self.entries.write().map_err(|_| {
            CacheError::LockError("Failed to acquire cache write lock".to_string())
        })?;

        // Create a new HashMap to store valid entries
        let mut valid_entries = HashMap::new();
        for (path, entry) in entries.iter() {
            if Self::validate_entry(entry) {
                valid_entries.insert(path.clone(), entry.clone());
            }
        }

        // Replace the cache with valid entries
        *entries = valid_entries;

        Ok(())
    }

    pub fn save(&self) -> CacheResult<()> {
        let entries = self.entries.read().map_err(|_| {
            CacheError::LockError("Failed to acquire cache read lock".to_string())
        })?;

        let cache_path = get_cache_path()?;
        
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent).map_err(|e| RkitError::DirectoryCreationError {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        let cache_data = CacheData {
            entries: entries.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            version: 1,
        };

        let json = serde_json::to_string_pretty(&cache_data)?;
        fs::write(&cache_path, json)?;

        Ok(())
    }

    pub fn validate_entry(entry: &CacheEntry) -> bool {
        let now = match get_current_time() {
            Ok(time) => time,
            Err(e) => {
                log::error!("Failed to get current time: {}", e);
                return false;
            }
        };

        // Check if the entry has expired
        if now - entry.last_checked > CACHE_TTL_SECONDS {
            return false;
        }

        // Check if the path exists and is a git repository
        entry.path.exists() && entry.path.join(".git").exists()
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
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

const CACHE_TTL_SECONDS: u64 = 24 * 60 * 60; // 24 hours

fn get_current_time() -> CacheResult<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| CacheError::TimeError(format!("Failed to get system time: {}", e)))
        .map(|d| d.as_secs())
}

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
    
    Ok(config_dir.join("rkit").join("cache.json"))
}

fn load_cache() -> CacheResult<HashMap<PathBuf, CacheEntry>> {
    let cache_path = get_cache_path()?;
    
    if !cache_path.exists() {
        return Ok(HashMap::new());
    }

    let contents = fs::read_to_string(&cache_path)?;
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