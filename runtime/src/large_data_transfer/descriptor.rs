//! Large Data Descriptor Implementation
//!
//! This module provides the descriptor structure for large data transfers,
//! including metadata, chunk organization, and Merkle tree verification.

use crate::large_data_transfer::{CompressionAlgorithm, EncryptionAlgorithm};
use crate::large_data_transfer::chunk::DataChunk;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use super::{metadata::TransferMetadata, redundancy::RedundancyConfig};
use crate::large_data_transfer::merkle::{calculate_content_hash, build_merkle_root};
use crate::large_data_transfer::validator::validate_descriptor;

/// Descriptor for large data transfers with content addressing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeDataDescriptor {
    /// Unique content hash of the entire data
    pub content_hash: String,
    
    /// Total size of original (uncompressed) data
    pub total_size: u64,
    
    /// Total number of chunks
    pub chunk_count: u32,
    
    /// Ordered list of chunk IDs (forms Merkle tree)
    pub chunk_hashes: Vec<String>,
    
    /// Default chunk size used for splitting
    pub chunk_size: u32,
    
    /// Compression algorithm used
    pub compression: CompressionAlgorithm,
    
    /// Encryption algorithm used
    pub encryption: EncryptionAlgorithm,
    
    /// Transfer metadata
    pub metadata: TransferMetadata,
    
    /// Redundancy configuration
    pub redundancy: RedundancyConfig,
    
    /// Merkle tree root for chunk verification
    pub merkle_root: String,
}

impl LargeDataDescriptor {
    /// Create a new descriptor from data chunks
    pub fn from_chunks(
        chunks: &[DataChunk],
        compression: CompressionAlgorithm,
        encryption: EncryptionAlgorithm,
        metadata: TransferMetadata,
    ) -> Self {
        let chunk_hashes: Vec<String> = chunks
            .iter()
            .map(|c| c.id().as_str().to_string())
            .collect();

        let total_size = chunks.iter().map(|c| c.info.original_size as u64).sum();
        let chunk_count = chunks.len() as u32;
        
        // Calculate content hash from ordered chunk hashes
        let content_hash = Self::calculate_content_hash(&chunk_hashes);
        
        // Build Merkle tree
        let merkle_root = Self::build_merkle_tree(&chunk_hashes);
        
        // Determine chunk size (use first chunk size)
        let chunk_size = if chunks.is_empty() {
            0
        } else {
            chunks[0].info.original_size
        };

        Self {
            content_hash,
            total_size,
            chunk_count,
            chunk_hashes,
            chunk_size,
            compression,
            encryption,
            metadata,
            redundancy: RedundancyConfig::default(),
            merkle_root,
        }
    }

    /// Validate descriptor consistency
    pub fn validate(&self) -> Result<(), String> {
        validate_descriptor(self)
    }
} 