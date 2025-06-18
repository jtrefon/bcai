//! Manages peer discovery, reputation, and maintenance for the network layer.

use super::{
    coordinator::NetworkTransferCoordinator,
    error::NetworkError,
    models::{NetworkPeerInfo, PeerTransferStats},
};
use crate::large_data_transfer::{chunk::ChunkId, LargeDataResult};
use std::time::{Duration, Instant};

impl NetworkTransferCoordinator {
    /// Add a peer to the network topology.
    pub async fn add_peer(&self, peer_info: NetworkPeerInfo) {
        println!(
            "📡 Adding peer: {} with {} available chunks",
            peer_info.peer_id,
            peer_info.capabilities.available_chunks.len()
        );
        self.peers.insert(peer_info.peer_id.clone(), peer_info);
    }

    /// Remove a peer from the network topology and cancel any active transfers.
    pub async fn remove_peer(&self, peer_id: &str) {
        println!("🗑️ Removing peer: {}", peer_id);
        if self.peers.remove(peer_id).is_some() {
            // Cancel any active transfers with this peer
            let transfers_to_cancel: Vec<String> = self
                .active_transfers
                .iter()
                .filter(|entry| entry.value().peers.contains_key(peer_id))
                .map(|entry| entry.key().clone())
                .collect();

            for transfer_id in transfers_to_cancel {
                self.active_transfers.remove(&transfer_id);
                println!("❌ Cancelled transfer: {} due to peer removal", transfer_id);
            }
        }
    }

    /// Periodically checks for stale peers and removes them.
    pub async fn peer_maintenance_loop(&self) {
        let mut interval = tokio::time::interval(self.config.peer_update_interval);

        loop {
            interval.tick().await;

            let stale_timeout = self.config.peer_timeout;
            let now = Instant::now();

            let stale_peers: Vec<String> = self
                .peers
                .iter()
                .filter_map(|entry| {
                    if now.duration_since(entry.value().last_seen) > stale_timeout {
                        Some(entry.key().clone())
                    } else {
                        None
                    }
                })
                .collect();

            if !stale_peers.is_empty() {
                println!("🧼 Pruning {} stale peers...", stale_peers.len());
                for peer_id in stale_peers {
                    self.remove_peer(&peer_id).await;
                }
            }
        }
    }

    /// Finds the best peer to request a specific chunk from based on a scoring model.
    pub(crate) async fn find_best_peer_for_chunk(
        &self,
        chunk_id: &ChunkId,
    ) -> LargeDataResult<Option<String>> {
        let mut best_peer = None;
        let mut best_score = -1.0; // Initialize with a score that any valid peer can beat

        for peer_entry in self.peers.iter() {
            let peer_info = peer_entry.value();

            if !peer_info.capabilities.available_chunks.contains(chunk_id) {
                continue;
            }

            // Simple scoring: reputation + availability. Lower is better for load.
            let load_score = peer_info.transfer_stats.active_transfers as f32;
            let score = peer_info.reputation - load_score;

            if score > best_score {
                best_score = score;
                best_peer = Some(peer_info.peer_id.clone());
            }
        }

        Ok(best_peer)
    }
} 