//! Statistics collected during a single data transfer session.

use serde::{Deserialize, Serialize};

/// Aggregated statistics for a transfer session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransferStats {
    /// Total number of chunks transferred.
    pub chunks_transferred: u64,
    /// Total bytes sent.
    pub bytes_sent: u64,
    /// Total bytes received.
    pub bytes_received: u64,
    /// Total time (in milliseconds) the transfer took.
    pub duration_ms: u64,
}

impl TransferStats {
    pub fn new() -> Self {
        Self::default()
    }
} 