use crate::large_data_transfer::chunk::{id::ChunkId, info::ChunkInfo};
use serde::{Deserialize, Serialize};

/// A self-contained slice of data identified by a `ChunkId`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataChunk {
    pub id: ChunkId,
    pub data: Vec<u8>,
    pub info: ChunkInfo,
}

impl DataChunk {
    /// Construct directly from id, raw bytes and pre-computed info.
    pub fn new(id: ChunkId, data: Vec<u8>, info: ChunkInfo) -> Self { Self { id, data, info } }

    /// Size of stored bytes (compressed if applicable).
    pub fn len(&self) -> usize { self.data.len() }
    pub fn is_empty(&self) -> bool { self.data.is_empty() }

    /// In-memory identifier reference.
    pub fn id(&self) -> &ChunkId { &self.id }
    /// Same as `len` (alias for readability).
    pub fn size(&self) -> usize { self.data.len() }
} 