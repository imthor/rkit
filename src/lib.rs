pub mod cache;
pub mod commands;
pub mod config;
pub mod error;

use std::sync::LazyLock;

/// Shared cache instance used by all commands
pub static CACHE: LazyLock<cache::Cache> = LazyLock::new(cache::Cache::new);
