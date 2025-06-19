use super::{
    config::ChunkManagerConfig,
    entry::ChunkEntry,
    manager::ChunkManager,
};
use crate::large_data_transfer::{
    chunk::{ChunkId, DataChunk},
    LargeDataResult,
};
use std::collections::HashMap;
use std::sync::MutexGuard;
use std::time::Instant;

impl ChunkManager {
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