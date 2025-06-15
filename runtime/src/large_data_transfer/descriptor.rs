//! Large Data Descriptor Implementation
//!
//! This module provides the descriptor structure for large data transfers,
//! including metadata, chunk organization, and Merkle tree verification.

use crate::large_data_transfer::{CompressionAlgorithm, EncryptionAlgorithm, TransferPriority};
use crate::large_data_transfer::chunk::DataChunk;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Metadata associated with a large data transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferMetadata {
    /// Human-readable name for the data
    pub name: String,
    
    /// Content type/MIME type
    pub content_type: String,
    
    /// Original filename (if applicable)
    pub filename: Option<String>,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Last modified timestamp
    pub modified_at: u64,
    
    /// Transfer priority
    pub priority: TransferPriority,
    
    /// Custom metadata tags
    pub tags: HashMap<String, String>,
    
    /// Source node ID
    pub source_node: Option<String>,
    
    /// Target nodes (for specific routing)
    pub target_nodes: Vec<String>,
    
    /// Transfer timeout duration in seconds
    pub timeout_seconds: Option<u64>,
}

impl Default for TransferMetadata {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        Self {
            name: "unnamed".to_string(),
            content_type: "application/octet-stream".to_string(),
            filename: None,
            created_at: now,
            modified_at: now,
            priority: TransferPriority::Normal,
            tags: HashMap::new(),
            source_node: None,
            target_nodes: Vec::new(),
            timeout_seconds: Some(3600), // 1 hour default
        }
    }
}

impl TransferMetadata {
    /// Create new metadata with name
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    /// Set content type
    pub fn with_content_type(mut self, content_type: String) -> Self {
        self.content_type = content_type;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: TransferPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Check if transfer has timed out
    pub fn is_timed_out(&self) -> bool {
        if let Some(timeout) = self.timeout_seconds {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now.saturating_sub(self.created_at) > timeout
        } else {
            false
        }
    }
}

/// Configuration for redundancy and error correction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedundancyConfig {
    /// Number of redundant copies to maintain
    pub replica_count: u32,
    
    /// Enable Reed-Solomon erasure coding
    pub erasure_coding: bool,
    
    /// Minimum number of nodes required for reconstruction
    pub min_nodes: u32,
}

impl Default for RedundancyConfig {
    fn default() -> Self {
        Self {
            replica_count: 1, // No redundancy by default
            erasure_coding: false,
            min_nodes: 1,
        }
    }
}

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

    /// Calculate content hash from chunk hashes
    fn calculate_content_hash(chunk_hashes: &[String]) -> String {
        let mut hasher = Sha256::new();
        for hash in chunk_hashes {
            hasher.update(hash.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    /// Build Merkle tree from chunk hashes
    fn build_merkle_tree(chunk_hashes: &[String]) -> String {
        if chunk_hashes.is_empty() {
            return String::new();
        }

        if chunk_hashes.len() == 1 {
            return chunk_hashes[0].clone();
        }

        let mut current_level = chunk_hashes.to_vec();
        
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            
            for pair in current_level.chunks(2) {
                let combined = if pair.len() == 2 {
                    format!("{}{}", pair[0], pair[1])
                } else {
                    // Odd number, hash with itself
                    format!("{}{}", pair[0], pair[0])
                };
                
                let hash = Sha256::digest(combined.as_bytes());
                next_level.push(format!("{:x}", hash));
            }
            
            current_level = next_level;
        }

        current_level[0].clone()
    }

    /// Validate descriptor consistency
    pub fn validate(&self) -> Result<(), String> {
        // Check chunk count consistency
        if self.chunk_hashes.len() != self.chunk_count as usize {
            return Err("Chunk count mismatch".to_string());
        }

        // Check content hash format
        if self.content_hash.len() != 64 {
            return Err("Invalid content hash length".to_string());
        }

        Ok(())
    }
} 