use super::core::ProtocolHandler;
use crate::large_data_transfer::{
    LargeDataResult,
    protocol::{message::TransferMessage, error::TransferError, state::ChunkStatus},
};
use crate::large_data_transfer::chunk::DataChunk;

impl ProtocolHandler {
    pub(super) fn handle_chunk_data_internal(
        &mut self,
        content_hash: String,
        chunk: DataChunk,
    ) -> LargeDataResult<Vec<TransferMessage>> {
        let session = self
            .sessions
            .get_mut(&content_hash)
            .ok_or_else(|| TransferError::TransferNotFound(content_hash.clone()))?;

        println!("Received chunk {} for {}", chunk.id.to_string(), session.content_hash);
        self.chunk_manager.store_chunk(chunk.clone())?;
        session.set_chunk_status(chunk.info.index, ChunkStatus::Complete(chunk.id.clone()));
        session.stats.chunks_transferred += 1;
        session.stats.bytes_received += chunk.len() as u64;

        let ack = TransferMessage::TransferProgress {
            content_hash,
            chunks_completed: session
                .chunk_status
                .values()
                .filter(|s| matches!(s, ChunkStatus::Complete(_)))
                .count() as u32,
            total_chunks: session
                .descriptor
                .as_ref()
                .map_or(0, |d| d.chunk_hashes.len() as u32),
            bytes_transferred: session.stats.bytes_received,
            transfer_rate: 0.0,
            eta: None,
        };

        Ok(vec![ack])
    }
}
