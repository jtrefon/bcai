//! Enhanced P2P Service for Large Data Transfer
//!
//! This module provides an enhanced P2P service that integrates
//! the large data transfer system with network coordination.

use crate::large_data_transfer::{
    network_integration::{NetworkTransferCoordinator, NetworkPeerInfo, PeerCapabilities, PeerTransferStats},
    manager::{ChunkManager, ChunkManagerConfig},
    chunk::ChunkId,
    LargeDataConfig,
};
use crate::{NetworkCoordinator, NetworkMessage, NodeCapability, UnifiedNode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};

/// Enhanced P2P service errors
#[derive(Debug, Error)]
pub enum EnhancedP2PError {
    #[error("Network connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Large data transfer failed: {0}")]
    LargeDataTransfer(String),
    #[error("Peer not found: {0}")]
    PeerNotFound(String),
    #[error("Service not started")]
    ServiceNotStarted,
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// Enhanced P2P service configuration
#[derive(Debug, Clone)]
pub struct EnhancedP2PConfig {
    pub listen_port: u16,
    pub bootstrap_peers: Vec<String>,
    pub max_peers: usize,
    pub heartbeat_interval: Duration,
    pub large_data_config: LargeDataConfig,
    pub enable_large_data_transfer: bool,
    pub peer_discovery_interval: Duration,
}

impl Default for EnhancedP2PConfig {
    fn default() -> Self {
        Self {
            listen_port: 4001,
            bootstrap_peers: vec![],
            max_peers: 50,
            heartbeat_interval: Duration::from_secs(30),
            large_data_config: LargeDataConfig::default(),
            enable_large_data_transfer: true,
            peer_discovery_interval: Duration::from_secs(60),
        }
    }
}

/// Enhanced P2P service with large data transfer capabilities
pub struct EnhancedP2PService {
    config: EnhancedP2PConfig,
    local_peer_id: String,
    network_coordinator: NetworkCoordinator,
    transfer_coordinator: Option<NetworkTransferCoordinator>,
    chunk_manager: Arc<ChunkManager>,
    peers: Arc<RwLock<HashMap<String, EnhancedPeerInfo>>>,
    stats: Arc<RwLock<EnhancedP2PStats>>,
    message_sender: mpsc::UnboundedSender<EnhancedP2PMessage>,
    message_receiver: Arc<RwLock<mpsc::UnboundedReceiver<EnhancedP2PMessage>>>,
    is_running: Arc<RwLock<bool>>,
}

/// Enhanced peer information
#[derive(Debug, Clone)]
pub struct EnhancedPeerInfo {
    pub peer_id: String,
    pub addresses: Vec<String>,
    pub node_capability: NodeCapability,
    pub transfer_capability: PeerCapabilities,
    pub last_seen: Instant,
    pub connection_quality: f32,
    pub is_large_data_enabled: bool,
}

/// Enhanced P2P service statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedP2PStats {
    pub peer_count: usize,
    pub active_large_transfers: usize,
    pub total_data_transferred: u64,
    pub average_transfer_speed: f32,
    pub network_stats: crate::large_data_transfer::NetworkStats,
    pub uptime: Duration,
    pub chunk_cache_hit_rate: f32,
}

/// Enhanced P2P messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnhancedP2PMessage {
    /// Standard network message
    NetworkMessage(NetworkMessage),
    /// Large data transfer request
    LargeDataRequest {
        content_hash: String,
        requester_id: String,
        target_peers: Vec<String>,
    },
    /// Large data transfer response
    LargeDataResponse {
        content_hash: String,
        success: bool,
        error: Option<String>,
    },
    /// Peer capability announcement
    PeerCapabilityAnnouncement {
        peer_id: String,
        capabilities: PeerCapabilities,
        node_capability: NodeCapability,
    },
    /// Chunk availability announcement
    ChunkAvailabilityAnnouncement {
        peer_id: String,
        available_chunks: Vec<ChunkId>,
    },
    /// Performance metrics update
    PerformanceUpdate {
        peer_id: String,
        transfer_speed: f32,
        success_rate: f32,
    },
}

impl EnhancedP2PService {
    /// Create a new enhanced P2P service
    pub fn new(config: EnhancedP2PConfig, _local_node: UnifiedNode) -> Result<Self, EnhancedP2PError> {
        let local_peer_id = "test_peer".to_string(); // For stub implementation
        let network_coordinator = NetworkCoordinator::new("test_node".to_string());

        // Initialize chunk manager for large data transfers
        let chunk_manager_config = ChunkManagerConfig {
            max_memory_bytes: config.large_data_config.cache_config.max_size,
            max_memory_chunks: 1000,
            cleanup_interval: std::time::Duration::from_secs(300),
            default_expiration: std::time::Duration::from_secs(3600),
        };

        let chunk_manager = Arc::new(ChunkManager::new(chunk_manager_config));

        // Initialize transfer coordinator if large data transfer is enabled
        let transfer_coordinator = if config.enable_large_data_transfer {
            Some(NetworkTransferCoordinator::new(
                local_peer_id.clone(),
                config.large_data_config.clone(),
                chunk_manager.clone(),
            ))
        } else {
            None
        };

        let (message_sender, message_receiver) = mpsc::unbounded_channel();

        let stats = EnhancedP2PStats {
            peer_count: 0,
            active_large_transfers: 0,
            total_data_transferred: 0,
            average_transfer_speed: 0.0,
            network_stats: crate::large_data_transfer::NetworkStats {
                connected_peers: 0,
                active_transfers: 0,
                total_upload_mbps: 0.0,
                total_download_mbps: 0.0,
                available_chunks: 0,
            },
            uptime: Duration::from_secs(0),
            chunk_cache_hit_rate: 0.0,
        };

        Ok(Self {
            config,
            local_peer_id,
            network_coordinator,
            transfer_coordinator,
            chunk_manager,
            peers: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(stats)),
            message_sender,
            message_receiver: Arc::new(RwLock::new(message_receiver)),
            is_running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start the enhanced P2P service
    pub async fn start(&self) -> Result<(), EnhancedP2PError> {
        println!("🚀 Starting Enhanced P2P Service on port {}", self.config.listen_port);
        println!("🌐 Peer ID: {}", self.local_peer_id);
        println!("📦 Large Data Transfer: {}", 
                if self.config.enable_large_data_transfer { "✅ Enabled" } else { "❌ Disabled" });

        // Mark service as running
        *self.is_running.write().await = true;

        // Start transfer coordinator if available
        if let Some(ref coordinator) = self.transfer_coordinator {
            coordinator.start().await
                .map_err(|e| EnhancedP2PError::LargeDataTransfer(e.to_string()))?;
        }

        // Start message processing
        let service = self.clone();
        tokio::spawn(async move {
            service.message_processing_loop().await;
        });

        // Start peer discovery
        let service = self.clone();
        tokio::spawn(async move {
            service.peer_discovery_loop().await;
        });

        // Start performance monitoring
        let service = self.clone();
        tokio::spawn(async move {
            service.performance_monitoring_loop().await;
        });

        println!("✅ Enhanced P2P Service started successfully");
        Ok(())
    }

    /// Stop the enhanced P2P service
    pub async fn stop(&self) -> Result<(), EnhancedP2PError> {
        println!("🛑 Stopping Enhanced P2P Service");
        *self.is_running.write().await = false;
        Ok(())
    }

    /// Add a peer to the network
    pub async fn add_peer(&self, peer_info: EnhancedPeerInfo) -> Result<(), EnhancedP2PError> {
        println!("📡 Adding peer: {} with large data support: {}", 
                peer_info.peer_id, peer_info.is_large_data_enabled);

        // Add to local peer registry
        self.peers.write().await.insert(peer_info.peer_id.clone(), peer_info.clone());

        // Add to transfer coordinator if available
        if let Some(ref coordinator) = self.transfer_coordinator {
            if peer_info.is_large_data_enabled {
                let network_peer_info = NetworkPeerInfo {
                    peer_id: peer_info.peer_id.clone(),
                    addresses: peer_info.addresses.clone(),
                    capabilities: peer_info.transfer_capability.clone(),
                    reputation: peer_info.connection_quality,
                    last_seen: peer_info.last_seen,
                    transfer_stats: PeerTransferStats::default(),
                };
                coordinator.add_peer(network_peer_info).await;
            }
        }

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.peer_count = self.peers.read().await.len();

        Ok(())
    }

    /// Remove a peer from the network
    pub async fn remove_peer(&self, peer_id: &str) -> Result<(), EnhancedP2PError> {
        println!("🗑️ Removing peer: {}", peer_id);

        // Remove from local registry
        self.peers.write().await.remove(peer_id);

        // Remove from transfer coordinator
        if let Some(ref coordinator) = self.transfer_coordinator {
            coordinator.remove_peer(peer_id).await;
        }

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.peer_count = self.peers.read().await.len();

        Ok(())
    }

    /// Send a large data transfer request
    pub async fn request_large_data_transfer(
        &self,
        content_hash: String,
        target_peers: Vec<String>,
    ) -> Result<(), EnhancedP2PError> {
        if let Some(ref _coordinator) = self.transfer_coordinator {
            println!("🚀 Initiating large data transfer: {} to {} peers", 
                    content_hash, target_peers.len());

            let message = EnhancedP2PMessage::LargeDataRequest {
                content_hash: content_hash.clone(),
                requester_id: self.local_peer_id.clone(),
                target_peers: target_peers.clone(),
            };

            self.broadcast_message(message).await?;

            // Update active transfer count
            let mut stats = self.stats.write().await;
            stats.active_large_transfers += 1;

            Ok(())
        } else {
            Err(EnhancedP2PError::ConfigurationError(
                "Large data transfer not enabled".to_string()
            ))
        }
    }

    /// Announce available chunks to the network
    pub async fn announce_chunks(&self, chunk_ids: Vec<ChunkId>) -> Result<(), EnhancedP2PError> {
        if let Some(ref coordinator) = self.transfer_coordinator {
            coordinator.announce_chunks(chunk_ids.clone()).await
                .map_err(|e| EnhancedP2PError::LargeDataTransfer(e.to_string()))?;

            let message = EnhancedP2PMessage::ChunkAvailabilityAnnouncement {
                peer_id: self.local_peer_id.clone(),
                available_chunks: chunk_ids,
            };

            self.broadcast_message(message).await?;
            Ok(())
        } else {
            Err(EnhancedP2PError::ConfigurationError(
                "Large data transfer not enabled".to_string()
            ))
        }
    }

    /// Get enhanced P2P service statistics
    pub async fn get_stats(&self) -> EnhancedP2PStats {
        let mut stats = self.stats.write().await;
        
        // Update network stats from transfer coordinator
        if let Some(ref coordinator) = self.transfer_coordinator {
            stats.network_stats = coordinator.get_network_stats().await;
        }

        // Update cache hit rate from chunk manager (simplified)
        stats.chunk_cache_hit_rate = 0.8; // Demo value

        stats.clone()
    }

    /// Broadcast a message to all peers
    async fn broadcast_message(&self, message: EnhancedP2PMessage) -> Result<(), EnhancedP2PError> {
        self.message_sender.send(message)
            .map_err(|_| EnhancedP2PError::ConnectionFailed("Message broadcast failed".to_string()))?;
        Ok(())
    }

    /// Send a message to a specific peer
    async fn send_to_peer(&self, _peer_id: &str, message: EnhancedP2PMessage) -> Result<(), EnhancedP2PError> {
        // In a real implementation, this would route to specific peer
        self.message_sender.send(message)
            .map_err(|_| EnhancedP2PError::ConnectionFailed("Message send failed".to_string()))?;
        Ok(())
    }

    /// Message processing loop
    async fn message_processing_loop(&self) {
        println!("🔄 Starting enhanced message processing loop");
        
        while *self.is_running.read().await {
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            // Process network coordination
            // Simplified for stub implementation
            println!("📊 Network coordination active");
        }

        println!("🔄 Enhanced message processing loop stopped");
    }

    /// Peer discovery loop
    async fn peer_discovery_loop(&self) {
        let mut interval = tokio::time::interval(self.config.peer_discovery_interval);

        while *self.is_running.read().await {
            interval.tick().await;

            println!("📡 Peer discovery - {} peers known", 
                    self.peers.read().await.len());
        }
    }

    /// Performance monitoring loop
    async fn performance_monitoring_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(10));

        while *self.is_running.read().await {
            interval.tick().await;

            let stats = self.get_stats().await;
            println!("📊 Enhanced P2P Stats - Peers: {}, Large Transfers: {}, Cache Hit: {:.1}%", 
                    stats.peer_count, stats.active_large_transfers, 
                    stats.chunk_cache_hit_rate * 100.0);
        }
    }
}

impl Clone for EnhancedP2PService {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            local_peer_id: self.local_peer_id.clone(),
            network_coordinator: self.network_coordinator.clone(),
            transfer_coordinator: self.transfer_coordinator.clone(),
            chunk_manager: Arc::clone(&self.chunk_manager),
            peers: Arc::clone(&self.peers),
            stats: Arc::clone(&self.stats),
            message_sender: self.message_sender.clone(),
            message_receiver: Arc::clone(&self.message_receiver),
            is_running: Arc::clone(&self.is_running),
        }
    }
}

/// Create a new enhanced P2P service
pub fn create_enhanced_p2p_service(
    config: EnhancedP2PConfig,
    local_node: UnifiedNode,
) -> Result<EnhancedP2PService, EnhancedP2PError> {
    EnhancedP2PService::new(config, local_node)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enhanced_p2p_service_creation() {
        let config = EnhancedP2PConfig::default();
        let capability = NodeCapability {
            cpus: 2,
            gpus: 0,
            gpu_memory_gb: 0,
            available_stake: 0,
            reputation: 0,
            capability_types: vec![crate::node::CapabilityType::BasicCompute],
        };

        let node = UnifiedNode::new("test_node".to_string(), capability);
        let result = create_enhanced_p2p_service(config, node);
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_service_lifecycle() {
        let config = EnhancedP2PConfig::default();
        let capability = NodeCapability {
            cpus: 2,
            gpus: 0,
            gpu_memory_gb: 0,
            available_stake: 0,
            reputation: 0,
            capability_types: vec![crate::node::CapabilityType::BasicCompute],
        };

        let node = UnifiedNode::new("test_node".to_string(), capability);
        let service = create_enhanced_p2p_service(config, node).unwrap();

        // Start service
        service.start().await.unwrap();

        // Check stats
        let stats = service.get_stats().await;
        assert_eq!(stats.peer_count, 0);

        // Wait a moment for background tasks
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
} 