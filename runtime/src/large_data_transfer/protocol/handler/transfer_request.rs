use super::core::ProtocolHandler;
use crate::large_data_transfer::LargeDataResult;
use crate::large_data_transfer::protocol::message::TransferMessage;
use crate::large_data_transfer::protocol::state::TransferState;

impl ProtocolHandler {
    pub(super) fn handle_transfer_request_internal(
        &mut self,
        content_hash: String,
        _requester_id: String,
    ) -> LargeDataResult<Vec<TransferMessage>> {
        let session = self
            .sessions
            .entry(content_hash.clone())
            .or_insert_with(|| crate::large_data_transfer::protocol::session::TransferSession::new(content_hash.clone()));
        session.set_state(TransferState::Pending);

        let response = TransferMessage::TransferResponse {
            content_hash,
            accepted: true,
            reason: None,
            estimated_time: None,
        };
        Ok(vec![response])
    }
} 