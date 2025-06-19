use super::super::{models::NetworkTransferMessage, error::NetworkError};
use super::super::coordinator::NetworkTransferCoordinator;
use crate::large_data_transfer::chunk::ChunkId;
use crate::large_data_transfer::LargeDataResult;

impl NetworkTransferCoordinator {
    /// Announce locally available chunks to the network.
    pub async fn announce_chunks(&self, chunk_ids: Vec<ChunkId>) -> LargeDataResult<()> {
        println!("📢 Announcing {} chunks to network", chunk_ids.len());

        let message = NetworkTransferMessage::ChunkAnnouncement {
            peer_id: self.local_peer_id.clone(),
            available_chunks: chunk_ids,
        };

        self.broadcast_message(message).await
    }
} 