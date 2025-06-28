//! Manages network bandwidth allocation, tracking, and limiting.

use super::coordinator::NetworkTransferCoordinator;
use crate::large_data_transfer::LargeDataResult;
use std::time::Duration;

impl NetworkTransferCoordinator {
    /// Periodically monitors and logs bandwidth usage.
    pub async fn bandwidth_monitoring_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            let stats = self.get_network_stats().await;
            println!(
                "📊 BW Stats - Upload: {:.2} Mbps, Download: {:.2} Mbps",
                stats.total_upload_mbps, stats.total_download_mbps
            );
        }
    }

    /// Checks if a new transfer is permissible based on current bandwidth limits.
    pub(crate) async fn check_bandwidth_availability(
        &self,
        peer_id: &str,
        is_upload: bool,
        required_bytes_per_sec: u64,
    ) -> LargeDataResult<bool> {
        let tracker = self.bandwidth_tracker.read().await;

        if is_upload {
            if tracker.total_upload_mbps as u64 * 125_000 + required_bytes_per_sec
                > tracker.max_upload_mbps as u64 * 125_000
            {
                return Ok(false);
            }
            if let Some(usage) = tracker.upload_usage.get(peer_id) {
                let per_peer_limit = tracker.max_upload_mbps as u64 * 125_000
                    / std::cmp::max(1, self.peers.len()) as u64;
                if usage.current_mbps as u64 * 125_000 + required_bytes_per_sec
                    > per_peer_limit
                {
                    return Ok(false);
                }
            }
        } else {
            if tracker.total_download_mbps as u64 * 125_000 + required_bytes_per_sec
                > tracker.max_download_mbps as u64 * 125_000
            {
                return Ok(false);
            }
            if let Some(usage) = tracker.download_usage.get(peer_id) {
                let per_peer_limit = tracker.max_download_mbps as u64 * 125_000
                    / std::cmp::max(1, self.peers.len()) as u64;
                if usage.current_mbps as u64 * 125_000 + required_bytes_per_sec
                    > per_peer_limit
                {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }
}
