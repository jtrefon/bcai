use super::super::{models::NetworkTransferMessage, error::NetworkError};
use super::super::coordinator::NetworkTransferCoordinator;
use crate::large_data_transfer::{
    chunk::{ChunkId, DataChunk},
    LargeDataResult,
};
use tokio::sync::oneshot;

impl NetworkTransferCoordinator {
    /// Request a single chunk from the best available peer.
    pub async fn request_chunk(&self, chunk_id: ChunkId) -> LargeDataResult<Option<DataChunk>> {
        println!("🔍 Requesting chunk: {}", chunk_id.to_string());

        if let Some(peer_id) = self.find_best_peer_for_chunk(&chunk_id).await? {
            println!("📥 Found chunk on peer: {}", peer_id);

            let (tx, rx) = oneshot::channel();
            self.pending_responses.insert(chunk_id.clone(), tx);

            let message = NetworkTransferMessage::ChunkRequest {
                chunk_id: chunk_id.clone(),
                requester_id: self.local_peer_id.clone(),
            };
            self.send_to_peer(&peer_id, message).await?;

            let result = match tokio::time::timeout(self.config.chunk_timeout, rx).await {
                Ok(Ok(chunk)) => Ok(chunk),
                Ok(Err(_)) => Err(NetworkError::NetworkUnreachable.into()),
                Err(_) => Err(NetworkError::TransferTimeout.into()),
            };
            self.pending_responses.remove(&chunk_id);
            result
        } else {
            println!("❌ No peer found with requested chunk");
            Ok(None)
        }
    }
}
