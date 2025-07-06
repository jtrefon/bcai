use super::*;
use crate::large_data_transfer::chunk::{ChunkId, DataChunk};
use std::time::Duration;

use crate::large_data_transfer::chunk::info::ChunkInfo; // Added
use crate::large_data_transfer::config::CompressionAlgorithm; // Added
use crc32fast::Hasher; // Added

fn create_test_chunk(id_byte: u8, size: usize) -> DataChunk {
    let data = vec![id_byte; size];
    let chunk_id = ChunkId::from_data(&data);
    let mut hasher = Hasher::new();
    hasher.update(&data);
    let checksum = hasher.finalize();
    let chunk_info = ChunkInfo {
        id: chunk_id.clone(),
        original_size: data.len() as u32,
        compressed_size: data.len() as u32, // Assuming no compression
        compression: CompressionAlgorithm::None,
        checksum,
        index: 0, // Example index, might need to be unique if tests store multiple chunks with same data
        replicas: Vec::new(),
    };
    DataChunk::new(chunk_id, data, chunk_info)
}

#[test]
fn test_chunk_manager_creation() {
    let manager = ChunkManager::default();
    assert_eq!(manager.chunk_count(), 0);
    assert_eq!(manager.current_memory_usage(), 0);
}

#[test]
fn test_store_and_retrieve_chunk() {
    let manager = ChunkManager::default();
    let chunk = create_test_chunk(1, 1024);
    let chunk_id = chunk.id().clone();

    manager.store_chunk(chunk.clone()).unwrap();
    assert_eq!(manager.chunk_count(), 1);
    assert_eq!(manager.current_memory_usage(), 1024);
    assert!(manager.has_chunk(&chunk_id));

    let retrieved_chunk = manager.get_chunk(&chunk_id).unwrap();
    assert_eq!(chunk.data, retrieved_chunk.data);
}

#[test]
fn test_chunk_removal() {
    let manager = ChunkManager::default();
    let chunk = create_test_chunk(1, 1024);
    let chunk_id = chunk.id().clone();

    manager.store_chunk(chunk).unwrap();
    assert_eq!(manager.chunk_count(), 1);

    assert!(manager.remove_chunk(&chunk_id));
    assert_eq!(manager.chunk_count(), 0);
    assert_eq!(manager.current_memory_usage(), 0);
    assert!(!manager.has_chunk(&chunk_id));
}

#[test]
fn test_memory_limit_eviction() {
    let config = ChunkManagerConfig {
        max_memory_chunks: 10,
        max_memory_bytes: 2048,
        ..Default::default()
    };
    let manager = ChunkManager::new(config);

    manager.store_chunk(create_test_chunk(1, 1024)).unwrap();
    manager.store_chunk(create_test_chunk(2, 1024)).unwrap();
    assert_eq!(manager.chunk_count(), 2);

    // This should evict the first chunk
    manager.store_chunk(create_test_chunk(3, 1024)).unwrap();
    assert_eq!(manager.chunk_count(), 2);
    assert!(!manager.has_chunk(&create_test_chunk(1, 1024).id()));
    assert!(manager.has_chunk(&create_test_chunk(2, 1024).id()));
    assert!(manager.has_chunk(&create_test_chunk(3, 1024).id()));
}

#[test]
fn test_stats() {
    let manager = ChunkManager::default();
    manager.store_chunk(create_test_chunk(1, 1024)).unwrap();
    manager.store_chunk(create_test_chunk(2, 2048)).unwrap();

    let stats = manager.stats();
    assert_eq!(stats.chunks_in_cache, 2);
    assert_eq!(stats.bytes_in_cache, 3072);
}

#[test]
fn test_clear() {
    let manager = ChunkManager::default();
    manager.store_chunk(create_test_chunk(1, 1024)).unwrap();
    manager.store_chunk(create_test_chunk(2, 1024)).unwrap();
    assert_eq!(manager.chunk_count(), 2);

    let cleared_count = manager.clear();
    assert_eq!(cleared_count, 2);
    assert_eq!(manager.chunk_count(), 0);
    assert_eq!(manager.current_memory_usage(), 0);
}

#[test]
fn test_expiration() {
    let config = ChunkManagerConfig {
        default_expiration: Duration::from_millis(50),
        ..Default::default()
    };
    let manager = ChunkManager::new(config);
    let chunk = create_test_chunk(1, 1024);
    let chunk_id = chunk.id().clone();
    manager.store_chunk(chunk).unwrap();

    assert!(manager.get_chunk(&chunk_id).is_some());
    std::thread::sleep(Duration::from_millis(100));
    assert!(manager.get_chunk(&chunk_id).is_none());
} 