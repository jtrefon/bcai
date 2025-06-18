//! General-purpose data types for the large data transfer module.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Transfer priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TransferPriority {
    /// Low priority - background transfers
    Low = 0,
    /// Normal priority - default
    Normal = 1,
    /// High priority - urgent transfers
    High = 2,
    /// Critical priority - system-critical transfers
    Critical = 3,
}

impl Default for TransferPriority {
    fn default() -> Self {
        TransferPriority::Normal
    }
}

/// Statistics for a single data transfer operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferStats {
    /// Total bytes transferred
    pub bytes_transferred: u64,

    /// Current transfer rate in bytes per second
    pub transfer_rate: f64,

    /// Chunks successfully transferred
    pub chunks_completed: u32,

    /// Total chunks in transfer
    pub total_chunks: u32,

    /// Transfer completion percentage (0.0 to 1.0)
    pub completion_percentage: f32,

    /// Estimated time to completion
    pub eta: Option<Duration>,

    /// Number of retry attempts
    pub retry_count: u32,

    /// Current active connections
    pub active_connections: u32,

    /// Compression ratio achieved
    pub compression_ratio: f32,

    /// Cache hit rate
    pub cache_hit_rate: f32,
}

impl Default for TransferStats {
    fn default() -> Self {
        Self {
            bytes_transferred: 0,
            transfer_rate: 0.0,
            chunks_completed: 0,
            total_chunks: 0,
            completion_percentage: 0.0,
            eta: None,
            retry_count: 0,
            active_connections: 0,
            compression_ratio: 1.0,
            cache_hit_rate: 0.0,
        }
    }
} 