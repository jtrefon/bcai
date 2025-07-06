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

    /// Returns the number of chunks currently stored.
    pub fn chunk_count(&self) -> usize {
        match self.chunks.lock() {
            Ok(guard) => guard.len(),
            Err(_) => 0, // Or handle poison error more gracefully
        }
    }

    /// Returns the total memory usage in bytes of all stored chunks.
    pub fn current_memory_usage(&self) -> u64 {
        match self.memory_usage.lock() {
            Ok(guard) => *guard,
            Err(_) => 0, // Or handle poison error
        }
    }

    /// Returns statistics about the chunk manager.
    pub fn stats(&self) -> super::stats::ChunkManagerStats {
        super::stats::ChunkManagerStats {
            chunks_in_cache: self.chunk_count(),
            bytes_in_cache: self.current_memory_usage(),
            evictions: 0, // Placeholder, eviction count not tracked here yet
        }
    }
} 