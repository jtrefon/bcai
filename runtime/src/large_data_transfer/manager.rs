//! Chunk Manager Implementation
//!
//! This module manages the lifecycle of data chunks including storage, retrieval, and cleanup.

use crate::large_data_transfer::{LargeDataConfig, LargeDataResult};
use crate::large_data_transfer::chunk::{ChunkId, DataChunk};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Configuration for the chunk manager
#[derive(Debug, Clone)]
pub struct ChunkManagerConfig {
    /// Maximum number of chunks to keep in memory
    pub max_memory_chunks: usize,
    
    /// Maximum memory usage in bytes
    pub max_memory_bytes: u64,
    
    /// Cleanup interval for expired chunks
    pub cleanup_interval: Duration,
    
    /// Default chunk expiration time
    pub default_expiration: Duration,
}

impl Default for ChunkManagerConfig {
    fn default() -> Self {
        Self {
            max_memory_chunks: 1000,
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            cleanup_interval: Duration::from_secs(60),
            default_expiration: Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Chunk storage entry
#[derive(Debug, Clone)]
struct ChunkEntry {
    chunk: DataChunk,
    last_accessed: Instant,
    access_count: u64,
    expiration: Option<Instant>,
}

/// Chunk manager for handling chunk storage and retrieval
#[derive(Debug)]
pub struct ChunkManager {
    config: ChunkManagerConfig,
    chunks: Arc<Mutex<HashMap<ChunkId, ChunkEntry>>>,
    memory_usage: Arc<Mutex<u64>>,
    last_cleanup: Arc<Mutex<Instant>>,
}

impl ChunkManager {
    /// Create a new chunk manager
    pub fn new(config: ChunkManagerConfig) -> Self {
        Self {
            config,
            chunks: Arc::new(Mutex::new(HashMap::new())),
            memory_usage: Arc::new(Mutex::new(0)),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(ChunkManagerConfig::default())
    }

    /// Store a chunk
    pub fn store_chunk(&self, chunk: DataChunk) -> LargeDataResult<()> {
        let chunk_id = chunk.id().clone();
        let chunk_size = chunk.size() as u64;
        
        let mut chunks = self.chunks.lock().unwrap();
        let mut memory_usage = self.memory_usage.lock().unwrap();
        
        // Check if we need to make space
        while chunks.len() >= self.config.max_memory_chunks
            || *memory_usage + chunk_size > self.config.max_memory_bytes
        {
            if !self.evict_oldest_chunk(&mut chunks, &mut memory_usage) {
                break; // No more chunks to evict
            }
        }
        
        let entry = ChunkEntry {
            chunk,
            last_accessed: Instant::now(),
            access_count: 0,
            expiration: Some(Instant::now() + self.config.default_expiration),
        };
        
        // Remove existing chunk if present
        if let Some(old_entry) = chunks.remove(&chunk_id) {
            *memory_usage -= old_entry.chunk.size() as u64;
        }
        
        chunks.insert(chunk_id, entry);
        *memory_usage += chunk_size;
        
        Ok(())
    }

    /// Retrieve a chunk
    pub fn get_chunk(&self, chunk_id: &ChunkId) -> Option<DataChunk> {
        let mut chunks = self.chunks.lock().unwrap();
        
        if let Some(entry) = chunks.get_mut(chunk_id) {
            // Check if expired
            if let Some(expiration) = entry.expiration {
                if Instant::now() > expiration {
                    return None;
                }
            }
            
            // Update access info
            entry.last_accessed = Instant::now();
            entry.access_count += 1;
            
            Some(entry.chunk.clone())
        } else {
            None
        }
    }

    /// Check if a chunk exists
    pub fn has_chunk(&self, chunk_id: &ChunkId) -> bool {
        let chunks = self.chunks.lock().unwrap();
        chunks.contains_key(chunk_id)
    }

    /// Remove a chunk
    pub fn remove_chunk(&self, chunk_id: &ChunkId) -> bool {
        let mut chunks = self.chunks.lock().unwrap();
        let mut memory_usage = self.memory_usage.lock().unwrap();
        
        if let Some(entry) = chunks.remove(chunk_id) {
            *memory_usage -= entry.chunk.size() as u64;
            true
        } else {
            false
        }
    }

    /// Get chunk count
    pub fn chunk_count(&self) -> usize {
        let chunks = self.chunks.lock().unwrap();
        chunks.len()
    }

    /// Get memory usage in bytes
    pub fn memory_usage(&self) -> u64 {
        let memory_usage = self.memory_usage.lock().unwrap();
        *memory_usage
    }

    /// Perform cleanup of expired chunks
    pub fn cleanup(&self) -> usize {
        let mut chunks = self.chunks.lock().unwrap();
        let mut memory_usage = self.memory_usage.lock().unwrap();
        let mut last_cleanup = self.last_cleanup.lock().unwrap();
        
        let now = Instant::now();
        let mut removed_count = 0;
        
        // Only cleanup if enough time has passed
        if now.duration_since(*last_cleanup) < self.config.cleanup_interval {
            return removed_count;
        }
        
        // Remove expired chunks
        chunks.retain(|_, entry| {
            if let Some(expiration) = entry.expiration {
                if now > expiration {
                    *memory_usage -= entry.chunk.size() as u64;
                    removed_count += 1;
                    false
                } else {
                    true
                }
            } else {
                true // No expiration
            }
        });
        
        *last_cleanup = now;
        removed_count
    }

    /// Force cleanup to make space
    pub fn force_cleanup(&self, target_chunks: usize, target_memory: u64) -> usize {
        let mut chunks = self.chunks.lock().unwrap();
        let mut memory_usage = self.memory_usage.lock().unwrap();
        let mut removed_count = 0;
        
        // Remove chunks until we reach targets
        while chunks.len() > target_chunks || *memory_usage > target_memory {
            if !self.evict_oldest_chunk(&mut chunks, &mut memory_usage) {
                break; // No more chunks to evict
            }
            removed_count += 1;
        }
        
        removed_count
    }

    /// Evict the oldest chunk (LRU)
    fn evict_oldest_chunk(
        &self,
        chunks: &mut HashMap<ChunkId, ChunkEntry>,
        memory_usage: &mut u64,
    ) -> bool {
        if chunks.is_empty() {
            return false;
        }
        
        // Find the chunk with the oldest last_accessed time
        let oldest_id = chunks
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(id, _)| id.clone());
        
        if let Some(id) = oldest_id {
            if let Some(entry) = chunks.remove(&id) {
                *memory_usage -= entry.chunk.size() as u64;
                return true;
            }
        }
        
        false
    }

    /// Get statistics
    pub fn stats(&self) -> ChunkManagerStats {
        let chunks = self.chunks.lock().unwrap();
        let memory_usage = self.memory_usage.lock().unwrap();
        
        let total_access_count: u64 = chunks.values().map(|e| e.access_count).sum();
        let avg_access_count = if chunks.is_empty() {
            0.0
        } else {
            total_access_count as f64 / chunks.len() as f64
        };
        
        ChunkManagerStats {
            chunk_count: chunks.len(),
            memory_usage: *memory_usage,
            total_access_count,
            average_access_count: avg_access_count,
            memory_utilization: *memory_usage as f64 / self.config.max_memory_bytes as f64,
        }
    }

    /// Clear all chunks
    pub fn clear(&self) -> usize {
        let mut chunks = self.chunks.lock().unwrap();
        let mut memory_usage = self.memory_usage.lock().unwrap();
        
        let count = chunks.len();
        chunks.clear();
        *memory_usage = 0;
        
        count
    }

    /// Get all chunk IDs
    pub fn chunk_ids(&self) -> Vec<ChunkId> {
        let chunks = self.chunks.lock().unwrap();
        chunks.keys().cloned().collect()
    }

    /// Prefetch chunks (mark as high priority)
    pub fn prefetch_chunks(&self, chunk_ids: &[ChunkId]) {
        let mut chunks = self.chunks.lock().unwrap();
        let now = Instant::now();
        
        for chunk_id in chunk_ids {
            if let Some(entry) = chunks.get_mut(chunk_id) {
                entry.last_accessed = now;
                // Extend expiration for prefetched chunks
                entry.expiration = Some(now + self.config.default_expiration * 2);
            }
        }
    }
}

/// Chunk manager statistics
#[derive(Debug, Clone)]
pub struct ChunkManagerStats {
    pub chunk_count: usize,
    pub memory_usage: u64,
    pub total_access_count: u64,
    pub average_access_count: f64,
    pub memory_utilization: f64,
}

impl From<LargeDataConfig> for ChunkManagerConfig {
    fn from(config: LargeDataConfig) -> Self {
        Self {
            max_memory_chunks: 1000,
            max_memory_bytes: config.cache_config.max_size / 10, // 10% of cache for memory
            cleanup_interval: Duration::from_secs(60),
            default_expiration: Duration::from_secs(3600),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::large_data_transfer::CompressionAlgorithm;

    #[test]
    fn test_chunk_manager_creation() {
        let manager = ChunkManager::default();
        assert_eq!(manager.chunk_count(), 0);
        assert_eq!(manager.memory_usage(), 0);
    }

    #[test]
    fn test_store_and_retrieve_chunk() {
        let manager = ChunkManager::default();
        let data = b"test data".to_vec();
        let chunk = DataChunk::new(data, 0, CompressionAlgorithm::None).unwrap();
        let chunk_id = chunk.id().clone();
        
        // Store chunk
        manager.store_chunk(chunk.clone()).unwrap();
        assert_eq!(manager.chunk_count(), 1);
        assert!(manager.memory_usage() > 0);
        
        // Retrieve chunk
        let retrieved = manager.get_chunk(&chunk_id).unwrap();
        assert_eq!(retrieved.id(), chunk.id());
        
        // Check if chunk exists
        assert!(manager.has_chunk(&chunk_id));
    }

    #[test]
    fn test_chunk_removal() {
        let manager = ChunkManager::default();
        let data = b"test data".to_vec();
        let chunk = DataChunk::new(data, 0, CompressionAlgorithm::None).unwrap();
        let chunk_id = chunk.id().clone();
        
        manager.store_chunk(chunk).unwrap();
        assert!(manager.has_chunk(&chunk_id));
        
        // Remove chunk
        assert!(manager.remove_chunk(&chunk_id));
        assert!(!manager.has_chunk(&chunk_id));
        assert_eq!(manager.chunk_count(), 0);
        assert_eq!(manager.memory_usage(), 0);
    }

    #[test]
    fn test_memory_limit_eviction() {
        let config = ChunkManagerConfig {
            max_memory_chunks: 2,
            max_memory_bytes: 1000,
            ..Default::default()
        };
        let manager = ChunkManager::new(config);
        
        // Store chunks that exceed memory limit
        for i in 0..3 {
            let data = vec![b'A'; 400]; // Each chunk ~400 bytes
            let chunk = DataChunk::new(data, i, CompressionAlgorithm::None).unwrap();
            manager.store_chunk(chunk).unwrap();
        }
        
        // Should have evicted the oldest chunk
        assert!(manager.chunk_count() <= 2);
        assert!(manager.memory_usage() <= 1000);
    }

    #[test]
    fn test_stats() {
        let manager = ChunkManager::default();
        let data = b"test data".to_vec();
        let chunk = DataChunk::new(data, 0, CompressionAlgorithm::None).unwrap();
        let chunk_id = chunk.id().clone();
        
        manager.store_chunk(chunk).unwrap();
        
        // Access the chunk multiple times
        for _ in 0..5 {
            manager.get_chunk(&chunk_id);
        }
        
        let stats = manager.stats();
        assert_eq!(stats.chunk_count, 1);
        assert!(stats.memory_usage > 0);
        assert_eq!(stats.total_access_count, 5);
        assert_eq!(stats.average_access_count, 5.0);
    }

    #[test]
    fn test_clear() {
        let manager = ChunkManager::default();
        
        // Store multiple chunks with different data
        for i in 0..3 {
            let mut data = vec![b'A'; 100];
            data.push(i as u8); // Make each chunk unique
            let chunk = DataChunk::new(data, i, CompressionAlgorithm::None).unwrap();
            manager.store_chunk(chunk).unwrap();
        }
        
        assert_eq!(manager.chunk_count(), 3);
        
        // Clear all chunks
        let cleared = manager.clear();
        assert_eq!(cleared, 3);
        assert_eq!(manager.chunk_count(), 0);
        assert_eq!(manager.memory_usage(), 0);
    }
} 