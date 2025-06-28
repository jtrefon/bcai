use crate::large_data_transfer::{
    manager::ChunkManager,
    network::{models::NetworkTransferMessage, coordinator::NetworkTransferCoordinator},
    chunk::ChunkId,
};
use super::replication::ReplicationManager;
use std::sync::Arc;
use tracing::{info, error};
use tokio::time::{interval, Duration};

/// Periodically checks replica health and generates replication plans to
/// maintain the required redundancy level.
pub async fn run_auto_heal(
    chunk_manager: Arc<ChunkManager>,
    repl_manager: ReplicationManager,
    coordinator: NetworkTransferCoordinator,
    required_copies: u32,
) {
    let mut ticker = interval(Duration::from_secs(60));
    loop {
        ticker.tick().await;
        let entries: Vec<(String, Vec<String>)> = {
            let chunks = chunk_manager.chunks.lock().unwrap();
            chunks
                .iter()
                .map(|(id, entry)| (id.to_string(), entry.chunk.info.replicas.clone()))
                .collect()
        };

        for (key, replicas) in entries {
            let plan = repl_manager.plan_replication(&key, &replicas, required_copies);
            for (node_id, blk) in plan {
                if let Ok(chunk_id) = ChunkId::from_hex(&blk) {
                    if let Some(chunk) = chunk_manager.get_chunk(&chunk_id) {
                        let msg = NetworkTransferMessage::ChunkResponse {
                            chunk_id: chunk_id.clone(),
                            data: Some(chunk.data.clone()),
                            error: None,
                        };
                        if let Err(e) = coordinator.send_to_peer(&node_id, msg).await {
                            error!(%blk, ?node_id, ?e, "Failed to send replication chunk");
                        } else {
                            info!(?node_id, block = %blk, "Replication copy sent");
                        }
                    }
                }
            }
        }
    }
} 