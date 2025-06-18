use super::*;
use crate::large_data_transfer::config::CompressionAlgorithm;

#[test]
fn test_chunk_id_creation() {
    let data = b"hello world";
    let id = ChunkId::from_data(data);
    assert_eq!(
        id.as_str(),
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );
}

#[test]
fn test_chunk_id_from_hex() {
    let hex = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
    let id = ChunkId::from_hex(hex).unwrap();
    assert_eq!(id.as_str(), hex);
}

#[test]
fn test_chunk_id_invalid_hex() {
    assert!(ChunkId::from_hex("not a hex").is_err());
    assert!(ChunkId::from_hex("1234").is_err());
}

#[test]
fn test_data_chunk_creation() {
    let data = vec![0; 1024];
    let chunk = DataChunk::new(data, 0, CompressionAlgorithm::None).unwrap();
    assert_eq!(chunk.info.original_size, 1024);
    assert_eq!(chunk.info.compressed_size, 0);
    assert!(!chunk.info.is_compressed());
}

#[test]
fn test_data_chunk_compression() {
    let data = vec![1; 4096]; // Use compressible data
    let chunk = DataChunk::new(data, 0, CompressionAlgorithm::Lz4).unwrap();
    assert!(chunk.info.is_compressed());
    assert!(chunk.info.compressed_size > 0);
    assert!(chunk.info.compressed_size < chunk.info.original_size);
}

#[test]
fn test_chunk_integrity_verification() {
    let data = b"some data to be chunked and verified".to_vec();
    let mut chunk = DataChunk::new(data.clone(), 0, CompressionAlgorithm::Lz4).unwrap();
    assert!(chunk.verify_integrity().is_ok());

    // Tamper with the data
    chunk.data[0] ^= 0xff;
    assert!(chunk.verify_integrity().is_err());
}

#[test]
fn test_chunk_utils_split_and_reassemble() {
    let data = (0..10000).map(|i| (i % 256) as u8).collect::<Vec<_>>();
    let chunks = ChunkUtils::split_data(&data, 1024, CompressionAlgorithm::None).unwrap();
    assert_eq!(chunks.len(), 10);
    
    let reassembled = ChunkUtils::reassemble_chunks(&chunks).unwrap();
    assert_eq!(data, reassembled);
}

#[test]
fn test_compression_stats() {
    let data = vec![0; 1000];
    let chunk = DataChunk::new(data, 0, CompressionAlgorithm::Lz4).unwrap();
    let stats = chunk.compression_stats();
    assert_eq!(stats.original_size, 1000);
    assert!(stats.compressed_size < 1000);
    assert!(stats.ratio < 1.0);
}

#[test]
fn test_chunk_utils_size_calculations() {
    let data = vec![0; 2048];
    let chunks = ChunkUtils::split_data(&data, 1024, CompressionAlgorithm::None).unwrap();
    assert_eq!(ChunkUtils::total_original_size(&chunks), 2048);
} 