use super::core::ProtocolHandler;
use crate::large_data_transfer::{
    protocol::{error::TransferError, message::TransferMessage},
    LargeDataResult,
};

impl ProtocolHandler {
    pub(super) fn handle_chunk_request_internal(
        &mut self,
        content_hash: String,
        chunk_indices: Vec<u32>,
        _sequence_id: u64,
    ) -> LargeDataResult<Vec<TransferMessage>> {
        let session = self
            .sessions
            .get_mut(&content_hash)
            .ok_or_else(|| TransferError::TransferNotFound(content_hash.clone()))?;

        println!(
            "Received request for {} chunks from {}",
            chunk_indices.len(),
            session.content_hash
        );

        let descriptor = session
            .descriptor
            .as_ref()
            .ok_or_else(|| TransferError::StateError("missing descriptor".into()))?;

        let mut responses = Vec::new();
        for index in chunk_indices {
            let chunk_hash = descriptor.chunk_hashes.get(index as usize).ok_or_else(|| {
                TransferError::StateError(format!("invalid chunk index {}", index))
            })?;
            let chunk_id = crate::large_data_transfer::chunk::ChunkId::from_hex(chunk_hash)?;
            let chunk = self.chunk_manager.get_chunk(&chunk_id).ok_or_else(|| {
                TransferError::StateError(format!("chunk {} not found", chunk_id))
            })?;

            responses.push(TransferMessage::ChunkData {
                content_hash: content_hash.clone(),
                chunk,
                sequence_id: _sequence_id,
                sender_id: self.node_id.clone(),
            });
        }

        Ok(responses)
    }
}
