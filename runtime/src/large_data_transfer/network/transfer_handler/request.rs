use super::super::{models::NetworkTransferMessage, error::NetworkError};
use super::super::coordinator::NetworkTransferCoordinator;
use crate::large_data_transfer::{chunk::{ChunkId, DataChunk}, LargeDataResult};
use std::time::Instant;

impl NetworkTransferCoordinator {
    /// Request a single chunk from the best available peer.
    pub async fn request_chunk(&self, chunk_id: ChunkId) -> LargeDataResult<Option<DataChunk>> {
        println!("🔍 Requesting chunk: {}", chunk_id.to_string());

        if let Some(peer_id) = self.find_best_peer_for_chunk(&chunk_id).await? {
            println!("📥 Found chunk on peer: {}", peer_id);

            let message = NetworkTransferMessage::ChunkRequest {
                chunk_id: chunk_id.clone(),
                requester_id: self.local_peer_id.clone(),
            };
            self.send_to_peer(&peer_id, message).await?;

            // Wait for response with timeout.
            tokio::time::timeout(self.config.chunk_timeout, self.wait_for_chunk_response(chunk_id))
                .await
                .map_err(|_| NetworkError::TransferTimeout.into())?
        } else {
            println!("❌ No peer found with requested chunk");
            Ok(None)
        }
    }

    // Await a response for a specific chunk (placeholder implementation).
    async fn wait_for_chunk_response(&self, chunk_id: ChunkId) -> LargeDataResult<Option<DataChunk>> {
        Ok(self.chunk_manager.get_chunk(&chunk_id))
    }
} 