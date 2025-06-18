//! Defines the `ChunkManager`, a cache for storing and retrieving data chunks.

use super::{
    config::ChunkManagerConfig,
    entry::ChunkEntry,
};
use crate::large_data_transfer::{
    chunk::{ChunkId, DataChunk},
    LargeDataResult,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Manages the lifecycle of data chunks, including in-memory storage,
/// retrieval, and eviction based on policies like LRU and TTL.
#[derive(Debug)]
pub struct ChunkManager {
    pub(super) config: ChunkManagerConfig,
    pub(super) chunks: Arc<Mutex<HashMap<ChunkId, ChunkEntry>>>,
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

    /// Stores a chunk in the cache.
    /// If the cache is full, it may evict other chunks based on the eviction policy.
    pub fn store_chunk(&self, chunk: DataChunk) -> LargeDataResult<()> {
        let chunk_id = chunk.id().clone();
        let chunk_size = chunk.size() as u64;

        let mut chunks = self.chunks.lock().unwrap();
        let mut memory_usage = self.memory_usage.lock().unwrap();

        // Evict chunks if necessary before inserting the new one.
        while self.is_over_capacity(&*chunks, &*memory_usage, chunk_size) {
            if !self.evict_one(&mut chunks, &mut memory_usage) {
                // Cannot evict any more chunks, break to avoid infinite loop.
                // This might mean the single new chunk is larger than the cache capacity.
                break;
            }
        }
        
        let entry = ChunkEntry {
            chunk,
            last_accessed: Instant::now(),
            access_count: 0,
            expiration: Some(Instant::now() + self.config.default_expiration),
        };
        
        if let Some(old_entry) = chunks.remove(&chunk_id) {
            *memory_usage -= old_entry.chunk.size() as u64;
        }

        chunks.insert(chunk_id, entry);
        *memory_usage += chunk_size;

        Ok(())
    }

    /// Retrieves a chunk from the cache.
    /// This updates the chunk's last access time for LRU eviction.
    pub fn get_chunk(&self, chunk_id: &ChunkId) -> Option<DataChunk> {
        let mut chunks = self.chunks.lock().unwrap();

        if let Some(entry) = chunks.get_mut(chunk_id) {
            // Do not return expired chunks.
            if entry.expiration.map_or(false, |exp| Instant::now() > exp) {
                return None;
            }

            entry.last_accessed = Instant::now();
            entry.access_count += 1;
            Some(entry.chunk.clone())
        } else {
            None
        }
    }

    /// Checks if a chunk exists in the cache without retrieving it.
    pub fn has_chunk(&self, chunk_id: &ChunkId) -> bool {
        self.chunks.lock().unwrap().contains_key(chunk_id)
    }

    /// Removes a specific chunk from the cache.
    pub fn remove_chunk(&self, chunk_id: &ChunkId) -> bool {
        let mut chunks = self.chunks.lock().unwrap();
        if let Some(entry) = chunks.remove(chunk_id) {
            *self.memory_usage.lock().unwrap() -= entry.chunk.size() as u64;
            true
        } else {
            false
        }
    }
} 