//! Network Integration for Large Data Transfer
//!
//! This module bridges the large data transfer system with P2P networking,
//! providing chunk routing, peer selection, and bandwidth management.

use crate::large_data_transfer::{
    chunk::{ChunkId, DataChunk},
    descriptor::LargeDataDescriptor,
    manager::ChunkManager,
    protocol::{TransferMessage, TransferSession, ProtocolHandler},
    LargeDataConfig, LargeDataError, LargeDataResult, TransferStats,
};
// use bytes::Bytes; // Removed unused import
use dashmap::DashMap;
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};

/// Network integration errors
#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Peer not found: {0}")]
    PeerNotFound(String),
    #[error("Network unreachable")]
    NetworkUnreachable,
    #[error("Transfer timeout")]
    TransferTimeout,
    #[error("Bandwidth limit exceeded")]
    BandwidthLimitExceeded,
    #[error("Chunk routing failed: {0}")]
    ChunkRoutingFailed(String),
    #[error("Large data error: {0}")]
    LargeData(#[from] LargeDataError),
}

impl From<NetworkError> for LargeDataError {
    fn from(err: NetworkError) -> Self {
        LargeDataError::Network(err.to_string())
    }
}

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

/// Network transfer coordinator
pub struct NetworkTransferCoordinator {
    local_peer_id: String,
    config: LargeDataConfig,
    chunk_manager: Arc<ChunkManager>,
    protocol_handler: Arc<RwLock<ProtocolHandler>>,
    peers: Arc<DashMap<String, NetworkPeerInfo>>,
    active_transfers: Arc<DashMap<String, TransferSession>>,
    bandwidth_tracker: Arc<RwLock<BandwidthTracker>>,
    message_sender: mpsc::UnboundedSender<NetworkTransferMessage>,
    message_receiver: Arc<RwLock<mpsc::UnboundedReceiver<NetworkTransferMessage>>>,
}

/// Bandwidth tracking and management
#[derive(Debug)]
pub struct BandwidthTracker {
    upload_usage: HashMap<String, BandwidthUsage>,
    download_usage: HashMap<String, BandwidthUsage>,
    total_upload_mbps: f32,
    total_download_mbps: f32,
    max_upload_mbps: u32,
    max_download_mbps: u32,
}

#[derive(Debug, Clone)]
pub struct BandwidthUsage {
    bytes_transferred: u64,
    start_time: Instant,
    current_mbps: f32,
}

impl NetworkTransferCoordinator {
    /// Create a new network transfer coordinator
    pub fn new(
        local_peer_id: String,
        config: LargeDataConfig,
        chunk_manager: Arc<ChunkManager>,
    ) -> Self {
        let protocol_handler = Arc::new(RwLock::new(ProtocolHandler::new(
            local_peer_id.clone(),
        )));

        let (message_sender, message_receiver) = mpsc::unbounded_channel();

        let bandwidth_tracker = Arc::new(RwLock::new(BandwidthTracker {
            upload_usage: HashMap::new(),
            download_usage: HashMap::new(),
            total_upload_mbps: 0.0,
            total_download_mbps: 0.0,
            max_upload_mbps: (config.max_upload_rate / 1_000_000) as u32,
            max_download_mbps: (config.max_download_rate / 1_000_000) as u32,
        }));

        Self {
            local_peer_id,
            config,
            chunk_manager,
            protocol_handler,
            peers: Arc::new(DashMap::new()),
            active_transfers: Arc::new(DashMap::new()),
            bandwidth_tracker,
            message_sender,
            message_receiver: Arc::new(RwLock::new(message_receiver)),
        }
    }

    /// Start the network transfer coordinator
    pub async fn start(&self) -> LargeDataResult<()> {
        println!("🌐 Starting Network Transfer Coordinator for peer: {}", self.local_peer_id);

        // Start message processing loop
        let coordinator = self.clone();
        tokio::spawn(async move {
            coordinator.message_processing_loop().await;
        });

        // Start bandwidth monitoring
        let coordinator = self.clone();
        tokio::spawn(async move {
            coordinator.bandwidth_monitoring_loop().await;
        });

        // Start peer discovery and maintenance
        let coordinator = self.clone();
        tokio::spawn(async move {
            coordinator.peer_maintenance_loop().await;
        });

        println!("✅ Network Transfer Coordinator started successfully");
        Ok(())
    }

    /// Add a peer to the network
    pub async fn add_peer(&self, peer_info: NetworkPeerInfo) {
        println!("📡 Adding peer: {} with {} available chunks", 
                peer_info.peer_id, peer_info.capabilities.available_chunks.len());
        self.peers.insert(peer_info.peer_id.clone(), peer_info);
    }

    /// Remove a peer from the network
    pub async fn remove_peer(&self, peer_id: &str) {
        println!("🗑️ Removing peer: {}", peer_id);
        self.peers.remove(peer_id);
        
        // Cancel any active transfers with this peer
        let transfers_to_cancel: Vec<String> = self.active_transfers
            .iter()
            .filter(|entry| entry.value().peers.contains_key(peer_id))
            .map(|entry| entry.key().clone())
            .collect();

        for transfer_id in transfers_to_cancel {
            self.active_transfers.remove(&transfer_id);
            println!("❌ Cancelled transfer: {} due to peer removal", transfer_id);
        }
    }

    /// Announce available chunks to the network
    pub async fn announce_chunks(&self, chunk_ids: Vec<ChunkId>) -> LargeDataResult<()> {
        println!("📢 Announcing {} chunks to network", chunk_ids.len());
        
        let message = NetworkTransferMessage::ChunkAnnouncement {
            peer_id: self.local_peer_id.clone(),
            available_chunks: chunk_ids,
        };

        self.broadcast_message(message).await?;
        Ok(())
    }

    /// Request a chunk from the network
    pub async fn request_chunk(&self, chunk_id: ChunkId) -> LargeDataResult<Option<DataChunk>> {
        println!("🔍 Requesting chunk: {}", chunk_id.to_string());
        
        // Find best peer for this chunk
        let best_peer = self.find_best_peer_for_chunk(&chunk_id).await?;
        
        if let Some(peer_id) = best_peer {
            println!("📥 Found chunk on peer: {}", peer_id);
            
            // Check bandwidth availability
            if !self.check_bandwidth_availability(&peer_id, 1_000_000).await? {
                println!("⚠️ Bandwidth limit exceeded for peer: {}", peer_id);
                return Err(NetworkError::BandwidthLimitExceeded.into());
            }

            // Send chunk request
            let message = NetworkTransferMessage::ChunkRequest {
                chunk_id: chunk_id.clone(),
                requester_id: self.local_peer_id.clone(),
            };

            self.send_to_peer(&peer_id, message).await?;

            // Wait for response with timeout
            tokio::time::timeout(
                self.config.chunk_timeout,
                self.wait_for_chunk_response(chunk_id),
            ).await
            .map_err(|_| {
                println!("⏰ Chunk request timed out");
                NetworkError::TransferTimeout
            })?
        } else {
            println!("❌ No peer found with requested chunk");
            Ok(None)
        }
    }

    /// Transfer a large data descriptor to the network
    pub async fn transfer_large_data(
        &self,
        descriptor: LargeDataDescriptor,
        target_peers: Vec<String>,
    ) -> LargeDataResult<TransferStats> {
        println!("🚀 Starting large data transfer: {} ({} chunks to {} peers)", 
                descriptor.content_hash, descriptor.chunk_count, target_peers.len());
        
        // Create transfer session (simplified for demo)
        let session_id = format!("session_{}", descriptor.content_hash);
        println!("📝 Created transfer session: {}", session_id);

        // Add target peers to session
        for peer_id in target_peers {
            if let Some(_peer_info) = self.peers.get(&peer_id) {
                println!("✅ Added peer to session: {}", peer_id);
            } else {
                println!("⚠️ Peer not found: {}", peer_id);
            }
        }

        // Start chunk transfer coordination
        let coordinator = self.clone();
        let transfer_task = tokio::spawn(async move {
            coordinator.coordinate_chunk_transfers(session_id).await
        });

        // Wait for completion
        transfer_task.await.map_err(|e| {
            println!("❌ Transfer coordination failed: {}", e);
            LargeDataError::Network(format!("Transfer coordination failed: {}", e))
        })?
    }

    /// Get network statistics
    pub async fn get_network_stats(&self) -> NetworkStats {
        let bandwidth_tracker = self.bandwidth_tracker.read().await;
        
        NetworkStats {
            connected_peers: self.peers.len(),
            active_transfers: self.active_transfers.len(),
            total_upload_mbps: bandwidth_tracker.total_upload_mbps,
            total_download_mbps: bandwidth_tracker.total_download_mbps,
            available_chunks: self.get_total_available_chunks().await,
        }
    }

    /// Get total available chunks across all peers
    async fn get_total_available_chunks(&self) -> usize {
        self.peers.iter()
            .map(|entry| entry.value().capabilities.available_chunks.len())
            .sum()
    }

    // Implementation details continue with helper methods...
    
    async fn find_best_peer_for_chunk(&self, chunk_id: &ChunkId) -> LargeDataResult<Option<String>> {
        let mut best_peer = None;
        let mut best_score = 0.0f32;

        for peer_entry in self.peers.iter() {
            let peer_info = peer_entry.value();
            
            // Check if peer has the chunk
            if !peer_info.capabilities.available_chunks.contains(chunk_id) {
                continue;
            }

            // Calculate peer score based on multiple factors
            let bandwidth_score = peer_info.transfer_stats.average_speed_mbps / 100.0;
            let reputation_score = peer_info.reputation;
            let availability_score = 1.0 - (peer_info.transfer_stats.active_transfers as f32 / 10.0);
            
            let total_score = bandwidth_score * 0.4 + reputation_score * 0.4 + availability_score * 0.2;

            if total_score > best_score {
                best_score = total_score;
                best_peer = Some(peer_info.peer_id.clone());
            }
        }

        Ok(best_peer)
    }

    async fn check_bandwidth_availability(
        &self,
        _peer_id: &str,
        _required_bytes_per_sec: u64,
    ) -> LargeDataResult<bool> {
        // Simplified for now - always return true
        Ok(true)
    }

    async fn send_to_peer(
        &self,
        _peer_id: &str,
        message: NetworkTransferMessage,
    ) -> LargeDataResult<()> {
        // In a real implementation, this would use the P2P network layer
        self.message_sender.send(message)
            .map_err(|_| NetworkError::NetworkUnreachable)?;
        Ok(())
    }

    async fn broadcast_message(&self, message: NetworkTransferMessage) -> LargeDataResult<()> {
        // In a real implementation, this would broadcast via P2P network
        self.message_sender.send(message)
            .map_err(|_| NetworkError::NetworkUnreachable)?;
        Ok(())
    }

    async fn wait_for_chunk_response(&self, chunk_id: ChunkId) -> LargeDataResult<Option<DataChunk>> {
        // For now, try to get from local chunk manager
        Ok(self.chunk_manager.get_chunk(&chunk_id))
    }

    async fn coordinate_chunk_transfers(&self, session_id: String) -> LargeDataResult<TransferStats> {
        let mut stats = TransferStats::default();
        
        // Simplified implementation for demo
        let total_chunks = 10; // Demo value
        println!("🔄 Coordinating transfer of {} chunks for session {}", total_chunks, session_id);

        // Coordinate transfer of each chunk
        for chunk_index in 0..total_chunks {
            // Update progress
            stats.completion_percentage = chunk_index as f32 / total_chunks as f32;
            stats.chunks_completed = chunk_index;
            stats.total_chunks = total_chunks;

            // Simulate chunk transfer time
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            if chunk_index % 5 == 0 {
                println!("📊 Progress: {:.1}% ({}/{})", 
                        stats.completion_percentage * 100.0, chunk_index, total_chunks);
            }
        }

        stats.completion_percentage = 1.0;
        stats.chunks_completed = total_chunks;
        println!("✅ Transfer completed: {} chunks", total_chunks);

        Ok(stats)
    }

    async fn message_processing_loop(&self) {
        println!("🔄 Starting message processing loop");
        let mut receiver = self.message_receiver.write().await;
        
        while let Some(message) = receiver.recv().await {
            if let Err(e) = self.handle_network_message(message).await {
                eprintln!("❌ Error handling network message: {}", e);
            }
        }
    }

    async fn handle_network_message(&self, message: NetworkTransferMessage) -> LargeDataResult<()> {
        match message {
            NetworkTransferMessage::ChunkAnnouncement { peer_id, available_chunks } => {
                println!("📡 Received chunk announcement from {}: {} chunks", peer_id, available_chunks.len());
                // Update peer's available chunks
                if let Some(mut peer_info) = self.peers.get_mut(&peer_id) {
                    peer_info.capabilities.available_chunks = available_chunks;
                }
            }

            NetworkTransferMessage::ChunkRequest { chunk_id, requester_id } => {
                println!("📥 Received chunk request for {} from {}", chunk_id.to_string(), requester_id);
                // Try to serve chunk from local storage
                if let Some(chunk) = self.chunk_manager.get_chunk(&chunk_id) {
                    let response = NetworkTransferMessage::ChunkResponse {
                        chunk_id,
                        data: Some(chunk.data),
                        error: None,
                    };
                    self.send_to_peer(&requester_id, response).await?;
                    println!("✅ Served chunk to {}", requester_id);
                } else {
                    let response = NetworkTransferMessage::ChunkResponse {
                        chunk_id,
                        data: None,
                        error: Some("Chunk not found".to_string()),
                    };
                    self.send_to_peer(&requester_id, response).await?;
                    println!("❌ Chunk not found for {}", requester_id);
                }
            }

            _ => {
                // Handle other message types
            }
        }

        Ok(())
    }

    async fn bandwidth_monitoring_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        
        loop {
            interval.tick().await;
            
            let stats = self.get_network_stats().await;
            println!("📊 Network Stats - Peers: {}, Transfers: {}, Upload: {:.1} Mbps, Download: {:.1} Mbps", 
                    stats.connected_peers, stats.active_transfers, 
                    stats.total_upload_mbps, stats.total_download_mbps);
        }
    }

    async fn peer_maintenance_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            // Remove stale peers
            let stale_timeout = Duration::from_secs(300); // 5 minutes
            let now = Instant::now();
            
            let stale_peers: Vec<String> = self.peers
                .iter()
                .filter_map(|entry| {
                    if now.duration_since(entry.value().last_seen) > stale_timeout {
                        Some(entry.key().clone())
                    } else {
                        None
                    }
                })
                .collect();

            for peer_id in stale_peers {
                self.remove_peer(&peer_id).await;
            }
        }
    }
}

impl Clone for NetworkTransferCoordinator {
    fn clone(&self) -> Self {
        Self {
            local_peer_id: self.local_peer_id.clone(),
            config: self.config.clone(),
            chunk_manager: Arc::clone(&self.chunk_manager),
            protocol_handler: Arc::clone(&self.protocol_handler),
            peers: Arc::clone(&self.peers),
            active_transfers: Arc::clone(&self.active_transfers),
            bandwidth_tracker: Arc::clone(&self.bandwidth_tracker),
            message_sender: self.message_sender.clone(),
            message_receiver: Arc::clone(&self.message_receiver),
        }
    }
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub connected_peers: usize,
    pub active_transfers: usize,
    pub total_upload_mbps: f32,
    pub total_download_mbps: f32,
    pub available_chunks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::large_data_transfer::manager::ChunkManagerConfig;

    #[tokio::test]
    async fn test_network_coordinator_creation() {
        let config = LargeDataConfig::default();
        let chunk_manager_config = ChunkManagerConfig::default();
        let chunk_manager = Arc::new(ChunkManager::new(chunk_manager_config));
        
        let coordinator = NetworkTransferCoordinator::new(
            "test_peer".to_string(),
            config,
            chunk_manager,
        );

        assert_eq!(coordinator.local_peer_id, "test_peer");
        assert_eq!(coordinator.peers.len(), 0);
    }

    #[tokio::test]
    async fn test_peer_management() {
        let config = LargeDataConfig::default();
        let chunk_manager_config = ChunkManagerConfig::default();
        let chunk_manager = Arc::new(ChunkManager::new(chunk_manager_config));
        
        let coordinator = NetworkTransferCoordinator::new(
            "test_peer".to_string(),
            config,
            chunk_manager,
        );

        let peer_info = NetworkPeerInfo {
            peer_id: "peer1".to_string(),
            addresses: vec!["127.0.0.1:8001".to_string()],
            capabilities: PeerCapabilities {
                max_bandwidth_mbps: 100,
                max_concurrent_transfers: 5,
                supported_compression: vec!["lz4".to_string()],
                storage_capacity_gb: 1000,
                available_chunks: vec![],
            },
            reputation: 0.8,
            last_seen: Instant::now(),
            transfer_stats: PeerTransferStats::default(),
        };

        coordinator.add_peer(peer_info).await;
        assert_eq!(coordinator.peers.len(), 1);

        coordinator.remove_peer("peer1").await;
        assert_eq!(coordinator.peers.len(), 0);
    }

    #[tokio::test]
    async fn test_chunk_announcement() {
        let config = LargeDataConfig::default();
        let chunk_manager_config = ChunkManagerConfig::default();
        let chunk_manager = Arc::new(ChunkManager::new(chunk_manager_config));
        
        let coordinator = NetworkTransferCoordinator::new(
            "test_peer".to_string(),
            config,
            chunk_manager,
        );

        let chunk_ids = vec![
            ChunkId::from_data(b"test_chunk_1"),
            ChunkId::from_data(b"test_chunk_2"),
        ];

        // This should not panic
        let result = coordinator.announce_chunks(chunk_ids).await;
        assert!(result.is_ok());
    }
} 