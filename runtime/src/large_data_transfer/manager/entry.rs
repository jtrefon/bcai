//! Defines the `ChunkEntry` struct for internal use by the `ChunkManager`.

use crate::large_data_transfer::chunk::DataChunk;
use std::time::Instant;

/// An entry in the chunk cache, containing the chunk and its metadata.
#[derive(Debug, Clone)]
pub(crate) struct ChunkEntry {
    /// The actual data chunk.
    pub(crate) chunk: DataChunk,
    /// The last time this chunk was accessed, for LRU eviction.
    pub(super) last_accessed: Instant,
    /// How many times the chunk has been accessed.
    pub(super) access_count: u64,
    /// The time at which this chunk expires and can be cleaned up.
    pub(super) expiration: Option<Instant>,
} 