use super::core::NetworkTransferCoordinator;
use super::super::models::NetworkStats;

impl NetworkTransferCoordinator {
    pub async fn get_network_stats(&self) -> NetworkStats {
        let bt = self.bandwidth_tracker.read().await;
        let total_chunks: usize = self
            .peers
            .iter()
            .map(|p| p.value().capabilities.available_chunks.len())
            .sum();
        NetworkStats {
            connected_peers: self.peers.len(),
            active_transfers: self.active_transfers.len(),
            total_upload_mbps: bt.total_upload_mbps,
            total_download_mbps: bt.total_download_mbps,
            available_chunks: total_chunks,
        }
    }
} 