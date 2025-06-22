use crate::large_data_transfer::{manager::ChunkManager, protocol::ProtocolHandler, LargeDataConfig};
use crate::large_data_transfer::network::models::{BandwidthTracker, NetworkPeerInfo, NetworkTransferMessage};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Central coordinator shared across network tasks.
pub struct NetworkTransferCoordinator {
    pub(crate) local_peer_id: String,
    pub(crate) config: LargeDataConfig,
    pub(crate) chunk_manager: Arc<ChunkManager>,
    pub(crate) protocol_handler: Arc<RwLock<ProtocolHandler>>,
    pub(crate) peers: Arc<DashMap<String, NetworkPeerInfo>>,
    pub(crate) active_transfers: Arc<DashMap<String, crate::large_data_transfer::protocol::TransferSession>>,
    pub(crate) bandwidth_tracker: Arc<RwLock<BandwidthTracker>>,
    pub(crate) message_sender: mpsc::UnboundedSender<NetworkTransferMessage>,
    pub(crate) message_receiver: Arc<RwLock<mpsc::UnboundedReceiver<NetworkTransferMessage>>>,
}

impl NetworkTransferCoordinator {
    pub fn new(local_peer_id: String, config: LargeDataConfig, chunk_manager: Arc<ChunkManager>) -> Self {
        let protocol_handler = Arc::new(RwLock::new(ProtocolHandler::new(local_peer_id.clone())));
        let (tx, rx) = mpsc::unbounded_channel();
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
            message_sender: tx,
            message_receiver: Arc::new(RwLock::new(rx)),
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