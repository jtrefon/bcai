use serde::{Deserialize, Serialize};

/// Cache configuration governing on-disk storage of chunks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum cache size in bytes (default: 10 GiB)
    pub max_size: u64,
    /// Enable LRU eviction when the cache is full.
    pub lru_eviction: bool,
    /// Enable content-based deduplication.
    pub deduplication: bool,
    /// Optional explicit cache directory; uses the system temp dir when `None`.
    pub cache_dir: Option<String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 10 * 1024 * 1024 * 1024, // 10 GiB
            lru_eviction: true,
            deduplication: true,
            cache_dir: None,
        }
    }
} 