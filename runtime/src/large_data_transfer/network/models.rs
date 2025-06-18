//! Data models for the network integration layer.

use crate::large_data_transfer::{chunk::ChunkId, protocol::TransferMessage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Peer information for network routing
#[derive(Debug, Clone)]
pub struct NetworkPeerInfo {
    pub peer_id: String,
    pub addresses: Vec<String>,
    pub capabilities: PeerCapabilities,
    pub reputation: f32,
    pub last_seen: Instant,
    pub transfer_stats: PeerTransferStats,
}

/// Peer capabilities for data transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerCapabilities {
    pub max_bandwidth_mbps: u32,
    pub max_concurrent_transfers: u32,
    pub supported_compression: Vec<String>,
    pub storage_capacity_gb: u64,
    pub available_chunks: Vec<ChunkId>,
}

/// Transfer statistics for a peer
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

/// Network message types for large data transfer
#[derive(Debug, Clone)]
pub enum NetworkTransferMessage {
    /// Announce available chunks to network
    ChunkAnnouncement {
        peer_id: String,
        available_chunks: Vec<ChunkId>,
    },
    /// Request chunk from peer
    ChunkRequest {
        chunk_id: ChunkId,
        requester_id: String,
    },
    /// Response with chunk data
    ChunkResponse {
        chunk_id: ChunkId,
        data: Option<Vec<u8>>,
        error: Option<String>,
    },
    /// Transfer session management
    TransferControl(TransferMessage),
    /// Bandwidth negotiation
    BandwidthNegotiation {
        requested_mbps: u32,
        granted_mbps: u32,
    },
    /// Peer capability update
    CapabilityUpdate {
        peer_id: String,
        capabilities: PeerCapabilities,
    },
}

/// Bandwidth tracking and management
#[derive(Debug)]
pub struct BandwidthTracker {
    pub(crate) upload_usage: HashMap<String, BandwidthUsage>,
    pub(crate) download_usage: HashMap<String, BandwidthUsage>,
    pub(crate) total_upload_mbps: f32,
    pub(crate) total_download_mbps: f32,
    pub(crate) max_upload_mbps: u32,
    pub(crate) max_download_mbps: u32,
}

#[derive(Debug, Clone)]
pub struct BandwidthUsage {
    pub bytes_transferred: u64,
    pub start_time: Instant,
    pub current_mbps: f32,
}

/// Statistics about the network state.
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkStats {
    pub connected_peers: usize,
    pub active_transfers: usize,
    pub total_upload_mbps: f32,
    pub total_download_mbps: f32,
    pub available_chunks: usize,
} 