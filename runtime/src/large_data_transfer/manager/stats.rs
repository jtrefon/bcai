//! Statistics for the `ChunkManager`.

use serde::{Serialize, Deserialize};

/// Runtime statistics for the in-memory chunk manager.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChunkManagerStats {
    /// Total number of chunks currently stored in memory.
    pub chunks_in_cache: usize,
    /// Total bytes currently stored in memory.
    pub bytes_in_cache: u64,
    /// Total number of cache evictions since startup.
    pub evictions: u64,
}

impl ChunkManagerStats {
    /// Create a new stats struct with all counters set to zero.
    pub fn new() -> Self { Self::default() }
} 