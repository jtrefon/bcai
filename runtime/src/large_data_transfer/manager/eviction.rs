//! Implements the eviction and cleanup logic for the `ChunkManager`.

use super::{
    entry::ChunkEntry,
    manager::ChunkManager,
};
use crate::large_data_transfer::chunk::ChunkId;
use std::collections::HashMap;
use std::sync::MutexGuard;
use std::time::Instant;

impl ChunkManager {
    /// Clears all chunks from the cache. Returns the number of chunks removed.
    pub fn clear(&self) -> usize {
        let mut chunks = self.chunks.lock().unwrap();
        let count = chunks.len();
        chunks.clear();
        *self.memory_usage.lock().unwrap() = 0;
        count
    }

    /// Performs cleanup of expired chunks based on their TTL.
    pub fn cleanup(&self) -> usize {
        let mut last_cleanup = self.last_cleanup.lock().unwrap();
        if last_cleanup.elapsed() < self.config.cleanup_interval {
            return 0; // Not time to clean up yet.
        }

        let mut chunks = self.chunks.lock().unwrap();
        let mut memory_usage = self.memory_usage.lock().unwrap();
        let now = Instant::now();
        let mut removed_count = 0;

        chunks.retain(|_, entry| {
            if entry.expiration.map_or(false, |exp| now > exp) {
                *memory_usage -= entry.chunk.size() as u64;
                removed_count += 1;
                false // Remove the entry
            } else {
                true // Keep the entry
            }
        });

        *last_cleanup = now;
        removed_count
    }

    /// Checks if adding a new chunk would exceed the cache's capacity limits.
    pub(super) fn is_over_capacity(
        &self,
        chunks: &HashMap<ChunkId, ChunkEntry>,
        mem_usage: &u64,
        new_chunk_size: u64,
    ) -> bool {
        (chunks.len() >= self.config.max_memory_chunks)
            || (*mem_usage + new_chunk_size > self.config.max_memory_bytes)
    }

    /// The core eviction logic that removes one chunk based on an LRU policy.
    /// This is intended for internal use by `store_chunk`.
    pub(super) fn evict_one(
        &self,
        chunks: &mut MutexGuard<HashMap<ChunkId, ChunkEntry>>,
        memory_usage: &mut MutexGuard<u64>,
    ) -> bool {
        if chunks.is_empty() {
            return false;
        }

        // Find the chunk with the oldest last_accessed time (LRU).
        let oldest_id = chunks
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(id, _)| id.clone());

        if let Some(id) = oldest_id {
            if let Some(entry) = chunks.remove(&id) {
                **memory_usage -= entry.chunk.size() as u64;
                return true;
            }
        }
        false
    }
} 