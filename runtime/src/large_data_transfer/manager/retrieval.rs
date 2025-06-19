use super::manager::ChunkManager;
use crate::large_data_transfer::chunk::{ChunkId, DataChunk};
use std::time::Instant;

impl ChunkManager {
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
} 