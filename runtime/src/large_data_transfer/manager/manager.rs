//! Defines the `ChunkManager`, a cache for storing and retrieving data chunks.

use super::{
    config::ChunkManagerConfig,
    entry::ChunkEntry,
};
use crate::large_data_transfer::chunk::ChunkId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Manages the lifecycle of data chunks, including in-memory storage,
/// retrieval, and eviction based on policies like LRU and TTL.
#[derive(Debug)]
pub struct ChunkManager {
    pub(super) config: ChunkManagerConfig,
    pub(crate) chunks: Arc<Mutex<HashMap<ChunkId, ChunkEntry>>>,
    pub(super) memory_usage: Arc<Mutex<u64>>,
    pub(super) last_cleanup: Arc<Mutex<Instant>>,
}

impl ChunkManager {
    /// Create a new chunk manager with the given configuration.
    pub fn new(config: ChunkManagerConfig) -> Self {
        Self {
            config,
            chunks: Arc::new(Mutex::new(HashMap::new())),
            memory_usage: Arc::new(Mutex::new(0)),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Create a new chunk manager with a default configuration.
    pub fn default() -> Self {
        Self::new(ChunkManagerConfig::default())
    }
} 