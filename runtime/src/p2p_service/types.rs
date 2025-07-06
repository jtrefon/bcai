//! Defines common data types used across the P2P service.

use crate::{network::NetworkStats, node::NodeCapability};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Information about a connected peer.
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: String,
    pub capabilities: Option<NodeCapability>,
    pub last_seen: Instant,
    pub reputation: i32,
    pub connection_count: usize,
}

/// Statistics about the P2P service's performance and state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PStats {
    pub peer_count: usize,
    pub connected_peers: usize,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub uptime: Duration,
    pub network_stats: NetworkStats,
}

impl Default for P2PStats {
    fn default() -> Self {
        Self {
            peer_count: 0,
            connected_peers: 0,
            messages_sent: 0,
            messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            uptime: Duration::from_secs(0),
            network_stats: Default::default(),
        }
    }
}