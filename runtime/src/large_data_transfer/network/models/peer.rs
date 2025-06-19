use crate::large_data_transfer::chunk::ChunkId;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Capability advertisement for a remote peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerCapabilities {
    pub max_bandwidth_mbps: u32,
    pub max_concurrent_transfers: u32,
    pub supported_compression: Vec<String>,
    pub storage_capacity_gb: u64,
    pub available_chunks: Vec<ChunkId>,
}

/// Runtime statistics collected for a given peer.
#[derive(Debug, Clone)]
pub struct PeerTransferStats {
    pub bytes_transferred: u64,
    pub average_speed_mbps: f32,
    pub success_rate: f32,
    pub active_transfers: u32,
    pub last_transfer_time: Option<Instant>,
}

impl Default for PeerTransferStats {
    fn default() -> Self {
        Self {
            bytes_transferred: 0,
            average_speed_mbps: 0.0,
            success_rate: 1.0,
            active_transfers: 0,
            last_transfer_time: None,
        }
    }
}

/// Aggregated information we keep for each connected peer.
#[derive(Debug, Clone)]
pub struct NetworkPeerInfo {
    pub peer_id: String,
    pub addresses: Vec<String>,
    pub capabilities: PeerCapabilities,
    pub reputation: f32,
    pub last_seen: Instant,
    pub transfer_stats: PeerTransferStats,
} 