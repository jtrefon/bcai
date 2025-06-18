//! Handles the logic for chunk announcements, requests, and full data transfers.

use super::{
    coordinator::NetworkTransferCoordinator,
    error::NetworkError,
    models::{NetworkTransferMessage, PeerCapabilities, NetworkPeerInfo},
};
use crate::large_data_transfer::{
    chunk::{ChunkId, DataChunk},
    descriptor::LargeDataDescriptor,
    protocol::TransferMessage,
    LargeDataResult, TransferStats,
};
use std::time::Instant;


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

    /// Request a chunk from the network, finding the best peer for it.
    pub async fn request_chunk(&self, chunk_id: ChunkId) -> LargeDataResult<Option<DataChunk>> {
        println!("🔍 Requesting chunk: {}", chunk_id.to_string());

        if let Some(peer_id) = self.find_best_peer_for_chunk(&chunk_id).await? {
            println!("📥 Found chunk on peer: {}", peer_id);

            // In a real scenario, you would check bandwidth and other constraints here.
            
            let message = NetworkTransferMessage::ChunkRequest {
                chunk_id: chunk_id.clone(),
                requester_id: self.local_peer_id.clone(),
            };

            self.send_to_peer(&peer_id, message).await?;

            // The wait_for_chunk_response logic would be more complex, likely involving
            // a temporary listener for the response message.
            tokio::time::timeout(
                self.config.chunk_timeout,
                self.wait_for_chunk_response(chunk_id),
            )
            .await
            .map_err(|_| NetworkError::TransferTimeout.into())?
        } else {
            println!("❌ No peer found with requested chunk");
            Ok(None)
        }
    }

    /// Orchestrates the transfer of a large data object, breaking it into chunks.
    pub async fn transfer_large_data(
        &self,
        descriptor: LargeDataDescriptor,
        target_peers: Vec<String>,
    ) -> LargeDataResult<TransferStats> {
        println!(
            "🚀 Starting large data transfer: {} ({} chunks to {} peers)",
            descriptor.id,
            descriptor.chunk_hashes.len(),
            target_peers.len()
        );

        let session_id = descriptor.id.clone();
        // Further session creation logic would go here.

        let coordinator = self.clone();
        tokio::spawn(async move {
            coordinator
                .coordinate_chunk_transfers(session_id)
                .await
        })
        .await
        .map_err(|e| LargeDataError::Network(format!("Transfer coordination failed: {}", e)))?
    }

    /// The main loop for processing incoming network messages.
    pub async fn message_processing_loop(&self) {
        println!("🔄 Starting message processing loop");
        let mut receiver = self.message_receiver.write().await;

        while let Some(message) = receiver.recv().await {
            if let Err(e) = self.handle_network_message(message).await {
                eprintln!("❌ Error handling network message: {}", e);
            }
        }
    }

    /// The main handler that dispatches different network message types.
    pub async fn handle_network_message(&self, message: NetworkTransferMessage) -> LargeDataResult<()> {
        match message {
            NetworkTransferMessage::ChunkAnnouncement { peer_id, available_chunks } => {
                if let Some(mut peer) = self.peers.get_mut(&peer_id) {
                    println!("📬 Received chunk announcement from {}: {} chunks", peer_id, available_chunks.len());
                    peer.capabilities.available_chunks = available_chunks;
                    peer.last_seen = Instant::now();
                }
            }
            NetworkTransferMessage::ChunkRequest { chunk_id, requester_id } => {
                println!("📥 Received chunk request for {} from {}", chunk_id.to_string(), requester_id);
                let response = match self.chunk_manager.get_chunk(&chunk_id) {
                    Some(chunk) => {
                        println!("✅ Served chunk {} to {}", chunk_id.to_string(), requester_id);
                        NetworkTransferMessage::ChunkResponse {
                            chunk_id,
                            data: Some(chunk.data),
                            error: None,
                        }
                    }
                    None => {
                        println!("❌ Chunk {} not found for {}", chunk_id.to_string(), requester_id);
                        NetworkTransferMessage::ChunkResponse {
                            chunk_id,
                            data: None,
                            error: Some("Chunk not found".to_string()),
                        }
                    }
                };
                self.send_to_peer(&requester_id, response).await?;
            }
            // Other message handlers would go here.
            _ => {}
        }
        Ok(())
    }

    /// Sends a message to a specific peer (placeholder).
    pub(crate) async fn send_to_peer(
        &self,
        _peer_id: &str,
        message: NetworkTransferMessage,
    ) -> LargeDataResult<()> {
        // In a real implementation, this would use the P2P network layer's unicast send.
        // For this refactoring, we simulate it by sending to our own message channel.
        self.message_sender
            .send(message)
            .map_err(|e| NetworkError::NetworkUnreachable.into())
    }

    /// Broadcasts a message to all peers (placeholder).
    pub(crate) async fn broadcast_message(&self, message: NetworkTransferMessage) -> LargeDataResult<()> {
        // In a real implementation, this would use the P2P network layer's broadcast.
        // For this refactoring, we simulate it by sending to our own message channel.
         self.message_sender
            .send(message)
            .map_err(|e| NetworkError::NetworkUnreachable.into())
    }

    /// Awaits a response for a specific chunk request (placeholder).
    async fn wait_for_chunk_response(
        &self,
        chunk_id: ChunkId,
    ) -> LargeDataResult<Option<DataChunk>> {
        // This is a placeholder. A real implementation would need a mechanism to
        // correlate requests and responses, perhaps with a map of waiting futures.
        // For now, we just check if we have it locally.
        Ok(self.chunk_manager.get_chunk(&chunk_id))
    }

    /// Coordinates the series of chunk transfers for a large data object (placeholder).
    async fn coordinate_chunk_transfers(&self, session_id: String) -> LargeDataResult<TransferStats> {
        // This is a placeholder. A real implementation would involve requesting chunks
        // from peers, assembling them, and managing the state of the transfer.
        println!("🔄 Coordinating transfer for session {}", session_id);
        Ok(TransferStats::default())
    }
} 