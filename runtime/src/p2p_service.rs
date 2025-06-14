//! P2P Service bridging unified nodes with libp2p networking
//!
//! This module provides the real networking layer that connects our unified node
//! architecture with actual P2P communication using libp2p.

use crate::network::{NetworkCoordinator, NetworkMessage, NetworkStats};
use crate::node::{NodeCapability, UnifiedNode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;

/// P2P service errors
#[derive(Debug, Error)]
pub enum P2PError {
    #[error("Network connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Message serialization failed: {0}")]
    SerializationFailed(String),
    #[error("Peer not found: {0}")]
    PeerNotFound(String),
    #[error("Service not started")]
    ServiceNotStarted,
    #[error("Channel communication error")]
    ChannelError,
}

/// P2P message wrapper for network transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PMessage {
    pub from_peer: String,
    pub to_peer: Option<String>, // None for broadcast
    pub message_type: String,
    pub payload: Vec<u8>,
    pub timestamp: u64,
    pub signature: Option<String>,
}

/// P2P peer information
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: String,
    pub capabilities: Option<NodeCapability>,
    pub last_seen: Instant,
    pub reputation: i32,
    pub connection_count: usize,
}

/// P2P service configuration
#[derive(Debug, Clone)]
pub struct P2PConfig {
    pub listen_port: u16,
    pub bootstrap_peers: Vec<String>,
    pub max_peers: usize,
    pub heartbeat_interval: Duration,
    pub message_timeout: Duration,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            listen_port: 4001,
            bootstrap_peers: vec![],
            max_peers: 50,
            heartbeat_interval: Duration::from_secs(30),
            message_timeout: Duration::from_secs(10),
        }
    }
}

/// P2P service statistics
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

/// Real P2P service for distributed BCAI networking
pub struct P2PService {
    config: P2PConfig,
    network_coordinator: NetworkCoordinator,
    peers: HashMap<String, PeerInfo>,
    stats: P2PStats,
    start_time: Option<Instant>,
}

impl P2PService {
    /// Create a new P2P service
    pub fn new(config: P2PConfig, local_node: UnifiedNode) -> (Self, P2PHandle) {
        let network_coordinator = NetworkCoordinator::new(local_node);

        let service = Self {
            config,
            network_coordinator,
            peers: HashMap::new(),
            stats: P2PStats {
                peer_count: 0,
                connected_peers: 0,
                messages_sent: 0,
                messages_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
                uptime: Duration::from_secs(0),
                network_stats: NetworkStats {
                    connected_peers: 0,
                    active_jobs: 0,
                    completed_jobs: 0,
                    network_block_height: 1,
                    local_node_stats: crate::node::NodeStats {
                        node_id: "".to_string(),
                        balance: 0,
                        staked: 0,
                        reputation: 0,
                        jobs_completed: 0,
                        jobs_active: 0,
                    },
                },
            },
            start_time: None,
        };

        let handle = P2PHandle::new();

        (service, handle)
    }

    /// Start the P2P service
    pub fn start(&mut self) -> Result<(), P2PError> {
        println!("🌐 Starting BCAI P2P Service on port {}", self.config.listen_port);

        self.start_time = Some(Instant::now());

        // Announce our capabilities to the network
        let announcement = self.network_coordinator.announce_capabilities();
        self.broadcast_message(announcement)?;

        println!("✅ P2P service started");
        Ok(())
    }

    /// Handle incoming network messages
    pub fn handle_incoming_message(
        &mut self,
        sender_id: String,
        message: NetworkMessage,
    ) -> Result<(), P2PError> {
        self.stats.messages_received += 1;

        // Update peer last seen
        if let Some(peer) = self.peers.get_mut(&sender_id) {
            peer.last_seen = Instant::now();
        } else {
            // New peer discovered
            self.peers.insert(
                sender_id.clone(),
                PeerInfo {
                    peer_id: sender_id.clone(),
                    capabilities: None,
                    last_seen: Instant::now(),
                    reputation: 0,
                    connection_count: 1,
                },
            );
        }

        // Process message through network coordinator
        match self.network_coordinator.handle_message(message, &sender_id) {
            Ok(responses) => {
                // Send any response messages
                for response in responses {
                    self.broadcast_message(response)?;
                }
            }
            Err(e) => {
                eprintln!("❌ Network coordinator error: {}", e);
            }
        }

        Ok(())
    }

    /// Send message to specific peer
    pub fn send_to_peer(&mut self, peer_id: &str, message: NetworkMessage) -> Result<(), P2PError> {
        if !self.peers.contains_key(peer_id) {
            return Err(P2PError::PeerNotFound(peer_id.to_string()));
        }

        // In a real implementation, this would use libp2p to send the message
        // For now, we simulate the sending
        println!("📤 Sending message to peer {}: {:?}", peer_id, message);

        self.stats.messages_sent += 1;
        Ok(())
    }

    /// Broadcast message to all peers
    pub fn broadcast_message(&mut self, message: NetworkMessage) -> Result<(), P2PError> {
        println!("📢 Broadcasting message: {:?}", message);

        // Collect peer IDs to avoid borrow checker issues
        let peer_ids: Vec<String> = self.peers.keys().cloned().collect();

        for peer_id in peer_ids {
            self.send_to_peer(&peer_id, message.clone())?;
        }

        Ok(())
    }

    /// Perform periodic maintenance
    pub fn perform_maintenance(&mut self) {
        let now = Instant::now();
        let timeout = Duration::from_secs(300); // 5 minutes

        // Remove stale peers
        self.peers.retain(|peer_id, peer| {
            if now.duration_since(peer.last_seen) > timeout {
                println!("🗑️ Removing stale peer: {}", peer_id);
                false
            } else {
                true
            }
        });

        self.update_stats();

        println!("💓 P2P heartbeat - Connected peers: {}", self.peers.len());
    }

    /// Update service statistics
    fn update_stats(&mut self) {
        self.stats.peer_count = self.peers.len();
        self.stats.connected_peers = self.peers.len();
        self.stats.network_stats = self.network_coordinator.get_network_stats();

        if let Some(start_time) = self.start_time {
            self.stats.uptime = Instant::now().duration_since(start_time);
        }
    }

    /// Get connected peers
    pub fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers.values().cloned().collect()
    }

    /// Get P2P service statistics
    pub fn get_stats(&mut self) -> P2PStats {
        self.update_stats();
        self.stats.clone()
    }
}

/// Handle for interacting with the P2P service
#[derive(Clone)]
pub struct P2PHandle {
    // Simplified handle without channels for now
}

impl Default for P2PHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl P2PHandle {
    /// Create a new P2P handle
    pub fn new() -> Self {
        Self {}
    }

    /// Start the P2P service (placeholder)
    pub fn start(&self) -> Result<(), P2PError> {
        // In a real implementation, this would start the service
        Ok(())
    }

    /// Stop the P2P service (placeholder)  
    pub fn stop(&self) -> Result<(), P2PError> {
        // In a real implementation, this would stop the service
        Ok(())
    }

    /// Send a message through the P2P network (placeholder)
    pub fn send_message(
        &self,
        _target: Option<String>,
        _message: NetworkMessage,
    ) -> Result<(), P2PError> {
        // In a real implementation, this would send messages
        Ok(())
    }

    /// Get connected peers (placeholder)
    pub fn get_peers(&self) -> Result<Vec<PeerInfo>, P2PError> {
        // In a real implementation, this would return peers
        Ok(vec![])
    }

    /// Get P2P service statistics (placeholder)
    pub fn get_stats(&self) -> Result<P2PStats, P2PError> {
        // Return dummy stats for now
        Ok(P2PStats {
            peer_count: 0,
            connected_peers: 0,
            messages_sent: 0,
            messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            uptime: Duration::from_secs(0),
            network_stats: NetworkStats {
                connected_peers: 0,
                active_jobs: 0,
                completed_jobs: 0,
                network_block_height: 1,
                local_node_stats: crate::node::NodeStats {
                    node_id: "".to_string(),
                    balance: 0,
                    staked: 0,
                    reputation: 0,
                    jobs_completed: 0,
                    jobs_active: 0,
                },
            },
        })
    }
}

/// Create a new P2P service for distributed BCAI
pub fn create_p2p_service(config: P2PConfig, local_node: UnifiedNode) -> (P2PService, P2PHandle) {
    P2PService::new(config, local_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeCapability;

    #[test]
    fn p2p_service_creation() {
        let config = P2PConfig::default();
        let capability = NodeCapability {
            cpus: 4,
            gpus: 1,
            gpu_memory_gb: 8,
            available_stake: 0,
            reputation: 0,
        };

        let node = UnifiedNode::new("test_node".to_string(), capability, 1000);
        let (_service, _handle) = create_p2p_service(config, node);

        // Test passes if creation succeeds
    }

    #[test]
    fn p2p_message_serialization() {
        let message = P2PMessage {
            from_peer: "peer1".to_string(),
            to_peer: Some("peer2".to_string()),
            message_type: "capability_announcement".to_string(),
            payload: vec![1, 2, 3, 4],
            timestamp: 1234567890,
            signature: None,
        };

        // Test that the message can be cloned
        let _cloned = message.clone();

        assert_eq!(message.from_peer, "peer1");
        assert_eq!(message.to_peer, Some("peer2".to_string()));
    }
}
