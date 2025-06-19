use super::super::{models::NetworkTransferMessage, error::NetworkError, coordinator::NetworkTransferCoordinator};
use crate::large_data_transfer::{LargeDataResult, chunk::{ChunkId, DataChunk}};
use std::time::Instant;

impl NetworkTransferCoordinator {
    /// Process incoming messages from the network layer.
    pub async fn message_processing_loop(&self) {
        println!("🔄 Starting message processing loop");
        let mut receiver = self.message_receiver.write().await;

        while let Some(message) = receiver.recv().await {
            if let Err(e) = self.handle_network_message(message).await {
                eprintln!("❌ Error handling network message: {}", e);
            }
        }
    }

    /// Dispatch a single network message.
    pub async fn handle_network_message(&self, message: NetworkTransferMessage) -> LargeDataResult<()> {
        match message {
            NetworkTransferMessage::ChunkAnnouncement { peer_id, available_chunks } => {
                if let Some(mut peer) = self.peers.get_mut(&peer_id) {
                    println!(
                        "📬 Received chunk announcement from {}: {} chunks",
                        peer_id,
                        available_chunks.len()
                    );
                    peer.capabilities.available_chunks = available_chunks;
                    peer.last_seen = Instant::now();
                }
            }
            NetworkTransferMessage::ChunkRequest { chunk_id, requester_id } => {
                println!("📥 Received chunk request for {} from {}", chunk_id, requester_id);
                let response = match self.chunk_manager.get_chunk(&chunk_id) {
                    Some(chunk) => NetworkTransferMessage::ChunkResponse {
                        chunk_id,
                        data: Some(chunk.data),
                        error: None,
                    },
                    None => NetworkTransferMessage::ChunkResponse {
                        chunk_id,
                        data: None,
                        error: Some("Chunk not found".to_string()),
                    },
                };
                self.send_to_peer(&requester_id, response).await?;
            }
            _ => {}
        }
        Ok(())
    }
} 