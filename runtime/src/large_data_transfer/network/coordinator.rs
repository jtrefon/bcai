//! The central coordinator for network-related large data transfer tasks.

use super::models::{
    BandwidthTracker, NetworkPeerInfo, NetworkStats, NetworkTransferMessage,
};
use crate::large_data_transfer::{
    manager::ChunkManager,
    protocol::{ProtocolHandler, TransferSession},
    LargeDataConfig, LargeDataResult,
};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Network transfer coordinator
///
/// This struct is the main entry point for the network layer. It holds the shared state
/// and is responsible for creating and starting the background tasks for message
/// processing, peer management, and bandwidth monitoring.
pub struct NetworkTransferCoordinator {
    pub(crate) local_peer_id: String,
    pub(crate) config: LargeDataConfig,
    pub(crate) chunk_manager: Arc<ChunkManager>,
    pub(crate) protocol_handler: Arc<RwLock<ProtocolHandler>>,
    pub(crate) peers: Arc<DashMap<String, NetworkPeerInfo>>,
    pub(crate) active_transfers: Arc<DashMap<String, TransferSession>>,
    pub(crate) bandwidth_tracker: Arc<RwLock<BandwidthTracker>>,
    pub(crate) message_sender: mpsc::UnboundedSender<NetworkTransferMessage>,
    pub(crate) message_receiver: Arc<RwLock<mpsc::UnboundedReceiver<NetworkTransferMessage>>>,
}

impl NetworkTransferCoordinator {
    /// Create a new network transfer coordinator.
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
            upload_usage: Default::default(),
            download_usage: Default::default(),
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

    /// Start the network transfer coordinator's background tasks.
    pub async fn start(&self) -> LargeDataResult<()> {
        println!("🌐 Starting Network Transfer Coordinator for peer: {}", self.local_peer_id);

        let self_clone = self.clone();
        tokio::spawn(async move {
            self_clone.message_processing_loop().await;
        });

        let self_clone = self.clone();
        tokio::spawn(async move {
            self_clone.bandwidth_monitoring_loop().await;
        });

        let self_clone = self.clone();
        tokio::spawn(async move {
            self_clone.peer_maintenance_loop().await;
        });

        println!("✅ Network Transfer Coordinator started successfully");
        Ok(())
    }

    /// Get current network statistics.
    pub async fn get_network_stats(&self) -> NetworkStats {
        let bandwidth_tracker = self.bandwidth_tracker.read().await;
        let total_available_chunks = self.peers.iter()
            .map(|entry| entry.value().capabilities.available_chunks.len())
            .sum();

        NetworkStats {
            connected_peers: self.peers.len(),
            active_transfers: self.active_transfers.len(),
            total_upload_mbps: bandwidth_tracker.total_upload_mbps,
            total_download_mbps: bandwidth_tracker.total_download_mbps,
            available_chunks: total_available_chunks,
        }
    }
}

impl Clone for NetworkTransferCoordinator {
    fn clone(&self) -> Self {
        Self {
            local_peer_id: self.local_peer_id.clone(),
            config: self.config.clone(),
            chunk_manager: self.chunk_manager.clone(),
            protocol_handler: self.protocol_handler.clone(),
            peers: self.peers.clone(),
            active_transfers: self.active_transfers.clone(),
            bandwidth_tracker: self.bandwidth_tracker.clone(),
            message_sender: self.message_sender.clone(),
            message_receiver: self.message_receiver.clone(),
        }
    }
}