use super::super::{models::NetworkTransferMessage, error::NetworkError};
use super::super::coordinator::NetworkTransferCoordinator;
use crate::large_data_transfer::LargeDataResult;

impl NetworkTransferCoordinator {
    /// Placeholder unicast send – in production this hits the P2P layer.
    pub(crate) async fn send_to_peer(&self, _peer_id: &str, message: NetworkTransferMessage) -> LargeDataResult<()> {
        self.message_sender
            .send(message)
            .map_err(|_| NetworkError::NetworkUnreachable.into())
    }

    /// Placeholder broadcast – currently loops back into local channel.
    pub(crate) async fn broadcast_message(&self, message: NetworkTransferMessage) -> LargeDataResult<()> {
        self.message_sender
            .send(message)
            .map_err(|_| NetworkError::NetworkUnreachable.into())
    }
} 