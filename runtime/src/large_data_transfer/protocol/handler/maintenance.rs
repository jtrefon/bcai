use super::core::ProtocolHandler;
use crate::large_data_transfer::{LargeDataResult, protocol::message::TransferMessage};

impl ProtocolHandler {
    /// High-level dispatcher that routes incoming protocol messages to the
    /// specialised handlers defined in sibling modules.
    pub fn handle_message(&mut self, message: TransferMessage) -> LargeDataResult<Vec<TransferMessage>> {
        use TransferMessage::*;
        match message {
            TransferRequest { content_hash, requester_id, .. } =>
                self.handle_transfer_request_internal(content_hash, requester_id),
            ChunkRequest { content_hash, chunk_indices, sequence_id, .. } =>
                self.handle_chunk_request_internal(content_hash, chunk_indices, sequence_id),
            ChunkData { content_hash, chunk, .. } =>
                self.handle_chunk_data_internal(content_hash, chunk),
            _ => Ok(vec![]),
        }
    }
} 