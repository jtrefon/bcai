//! Defines the `TransferSession` and related structs for tracking transfer state.

use super::{
    state::{ChunkStatus, TransferState},
    stats::TransferStats,
};
use crate::large_data_transfer::descriptor::LargeDataDescriptor;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Holds all state for a single large data transfer.
#[derive(Debug)]
pub struct TransferSession {
    pub content_hash: String,
    pub state: TransferState,
    pub descriptor: Option<LargeDataDescriptor>,
    pub stats: TransferStats,
    pub peers: HashMap<String, PeerInfo>,
    pub chunk_status: HashMap<u32, ChunkStatus>,
    pub last_activity: Instant,
    pub retry_count: u32,
}

/// Information about a peer participating in a transfer session.
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub node_id: String,
    pub available_chunks: Vec<u32>,
    pub bandwidth: u64,
    pub reliability: f32,
    pub last_seen: Instant,
}

impl TransferSession {
    /// Create a new transfer session.
    pub fn new(content_hash: String) -> Self {
        Self {
            content_hash,
            state: TransferState::Initiating,
            descriptor: None,
            stats: TransferStats::default(),
            peers: HashMap::new(),
            chunk_status: HashMap::new(),
            last_activity: Instant::now(),
            retry_count: 0,
        }
    }

    /// Update session state and refresh the activity timer.
    pub fn set_state(&mut self, state: TransferState) {
        self.state = state;
        self.last_activity = Instant::now();
    }

    /// Add a peer to the session.
    pub fn add_peer(&mut self, peer: PeerInfo) {
        self.peers.insert(peer.node_id.clone(), peer);
        self.last_activity = Instant::now();
    }

    /// Update the status of a specific chunk.
    pub fn set_chunk_status(&mut self, chunk_index: u32, status: ChunkStatus) {
        self.chunk_status.insert(chunk_index, status);
        self.last_activity = Instant::now();
    }

    /// Get a list of chunk indices that still need to be downloaded.
    pub fn pending_chunks(&self) -> Vec<u32> {
        if let Some(descriptor) = &self.descriptor {
            (0..descriptor.chunk_hashes.len() as u32)
                .filter(|i| !matches!(self.chunk_status.get(i), Some(ChunkStatus::Complete(_))))
                .collect()
        } else {
            vec![]
        }
    }

    /// Calculate the current progress of the transfer as a percentage.
    pub fn progress(&self) -> f32 {
        let total_chunks = self.descriptor.as_ref().map_or(0, |d| d.chunk_hashes.len());
        if total_chunks == 0 {
            return 0.0;
        }
        let completed_chunks = self
            .chunk_status
            .values()
            .filter(|s| matches!(s, ChunkStatus::Complete(_)))
            .count();
        (completed_chunks as f32 / total_chunks as f32) * 100.0
    }

    /// Check if the session has been inactive for longer than the timeout duration.
    pub fn is_timed_out(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }

    /// Find the best peer to download a specific chunk from.
    pub fn best_peer_for_chunk(&self, chunk_index: u32) -> Option<&PeerInfo> {
        self.peers
            .values()
            .filter(|p| p.available_chunks.contains(&chunk_index))
            .min_by(|a, b| {
                // Simple algorithm: prefer peer with higher reliability
                b.reliability.partial_cmp(&a.reliability).unwrap_or(std::cmp::Ordering::Equal)
            })
    }
} 