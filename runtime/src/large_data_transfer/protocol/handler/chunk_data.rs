use super::core::ProtocolHandler;
use crate::large_data_transfer::{LargeDataResult, protocol::{message::TransferMessage, error::TransferError}};
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

        println!("Received chunk {} for {}", chunk.chunk_id.to_string(), session.content_hash);
        // TODO: store chunk, update session state, maybe send ACK.
        Ok(vec![])
    }
} 