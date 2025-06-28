use super::super::coordinator::NetworkTransferCoordinator;
use crate::large_data_transfer::{descriptor::LargeDataDescriptor, LargeDataResult, TransferStats};

impl NetworkTransferCoordinator {
    /// Break a large data object into chunks and distribute them among peers.
    pub async fn transfer_large_data(
        &self,
        descriptor: LargeDataDescriptor,
        _target_peers: Vec<String>,
    ) -> LargeDataResult<TransferStats> {
        println!(
            "🚀 Starting large data transfer: {} ({} chunks)",
            descriptor.id,
            descriptor.chunk_hashes.len(),
        );

        let session_id = descriptor.id.clone();
        let coordinator = self.clone();
        tokio::spawn(async move { coordinator.coordinate_chunk_transfers(session_id).await })
            .await
            .map_err(|e| {
                crate::large_data_transfer::error::LargeDataError::Network(format!(
                    "Transfer coordination failed: {}",
                    e
                ))
            })?
    }

    /// Basic coordination loop that sequentially requests missing chunks until the
    /// transfer completes.
    async fn coordinate_chunk_transfers(
        &self,
        session_id: String,
    ) -> LargeDataResult<TransferStats> {
        println!("🔄 Coordinating transfer for session {}", session_id);

        loop {
            let (descriptor, pending) = {
                let mut entry = self.active_transfers.get_mut(&session_id).ok_or_else(|| {
                    crate::large_data_transfer::error::LargeDataError::Network(
                        "transfer not found".into(),
                    )
                })?;

                let desc = entry.descriptor.clone().ok_or_else(|| {
                    crate::large_data_transfer::error::LargeDataError::Network(
                        "missing descriptor".into(),
                    )
                })?;

                let pending = entry.pending_chunks();
                if pending.is_empty() {
                    let proto_stats = entry.stats.clone();
                    let result = TransferStats {
                        bytes_transferred: proto_stats.bytes_received,
                        transfer_rate: 0.0,
                        chunks_completed: proto_stats.chunks_transferred as u32,
                        total_chunks: desc.chunk_hashes.len() as u32,
                        completion_percentage: 1.0,
                        eta: None,
                        retry_count: entry.retry_count,
                        active_connections: entry.peers.len() as u32,
                        compression_ratio: 1.0,
                        cache_hit_rate: 0.0,
                    };
                    self.active_transfers.remove(&session_id);
                    return Ok(result);
                }

                (desc, pending)
            };

            for index in pending {
                let chunk_hash = descriptor.chunk_hashes[index as usize].clone();
                let chunk_id = crate::large_data_transfer::chunk::ChunkId::from_hex(&chunk_hash)?;
                if let Some(chunk) = self.request_chunk(chunk_id.clone()).await? {
                    self.chunk_manager.store_chunk(chunk.clone())?;
                    if let Some(mut entry) = self.active_transfers.get_mut(&session_id) {
                        entry.set_chunk_status(
                            index,
                            crate::large_data_transfer::protocol::ChunkStatus::Complete(
                                chunk.id.clone(),
                            ),
                        );
                        entry.stats.chunks_transferred += 1;
                        entry.stats.bytes_received += chunk.len() as u64;
                    }
                }
            }
        }
    }
}
