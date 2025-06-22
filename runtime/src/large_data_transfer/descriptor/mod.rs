//! Placeholder implementation for LargeDataDescriptor until full module is restored.

use serde::{Serialize, Deserialize};

/// Lightweight metadata describing a large data object being transferred.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeDataDescriptor {
    /// Unique identifier for the overall content – usually a root Merkle hash.
    pub id: String,
    pub content_hash: String,
    pub size_bytes: u64,
    /// Ordered list of chunk hashes that compose the content.
    pub chunk_hashes: Vec<String>,
}

impl LargeDataDescriptor {
    pub fn new(id: String, content_hash: String, size_bytes: u64, chunk_hashes: Vec<String>) -> Self {
        Self { id, content_hash, size_bytes, chunk_hashes }
    }
}
