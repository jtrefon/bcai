use super::session::{PeerInfo, TransferSession};
use std::time::Instant;

impl TransferSession {
    /// Add a peer to the session.
    pub fn add_peer(&mut self, peer: PeerInfo) {
        self.peers.insert(peer.node_id.clone(), peer);
        self.last_activity = Instant::now();
    }

    /// Find the best peer to download a specific chunk from.
    pub fn best_peer_for_chunk(&self, chunk_index: u32) -> Option<&PeerInfo> {
        self.peers
            .values()
            .filter(|p| p.available_chunks.contains(&chunk_index))
            .min_by(|a, b| {
                // Simple algorithm: prefer peer with higher reliability
                b.reliability.partial_cmp(&a.reliability).unwrap_or(std::cmp::Ordering::Equal)
            })
    }
} 