//! Defines the `ProtocolHandler` which acts as the state machine for transfers.

use super::{
    error::TransferError,
    message::TransferMessage,
    session::TransferSession,
    state::{TransferState, TransferErrorType},
};
use crate::large_data_transfer::{descriptor::LargeDataDescriptor, LargeDataResult};
use std::collections::HashMap;
use std::time::Duration;

/// Manages all active transfer sessions and processes incoming protocol messages.
pub struct ProtocolHandler {
    sessions: HashMap<String, TransferSession>,
    node_id: String,
    default_timeout: Duration,
}

impl ProtocolHandler {
    /// Create a new protocol handler.
    pub fn new(node_id: String) -> Self {
        Self {
            sessions: HashMap::new(),
            node_id,
            default_timeout: Duration::from_secs(60),
        }
    }

    /// The main entry point for processing messages. It dispatches to specific handlers.
    pub fn handle_message(
        &mut self,
        message: TransferMessage,
    ) -> LargeDataResult<Vec<TransferMessage>> {
        match message {
            TransferMessage::TransferRequest { content_hash, requester_id, .. } => {
                self.handle_transfer_request(content_hash, requester_id)
            }
            TransferMessage::ChunkRequest { content_hash, chunk_indices, sequence_id, .. } => {
                self.handle_chunk_request(content_hash, chunk_indices, sequence_id)
            }
            TransferMessage::ChunkData { content_hash, chunk, .. } => {
                self.handle_chunk_data(content_hash, chunk)
            }
            // Other message types would be handled here.
            _ => Ok(vec![]),
        }
    }

    /// Handles a request to start a new transfer.
    fn handle_transfer_request(
        &mut self,
        content_hash: String,
        _requester_id: String,
    ) -> LargeDataResult<Vec<TransferMessage>> {
        let session = self
            .sessions
            .entry(content_hash.clone())
            .or_insert_with(|| TransferSession::new(content_hash.clone()));

        session.set_state(TransferState::Pending);

        // In a real implementation, we would check if we want to accept the transfer.
        let response = TransferMessage::TransferResponse {
            content_hash,
            accepted: true,
            reason: None,
            estimated_time: None,
        };

        Ok(vec![response])
    }

    /// Handles a request for specific chunks from a peer.
    fn handle_chunk_request(
        &mut self,
        content_hash: String,
        chunk_indices: Vec<u32>,
        sequence_id: u64,
    ) -> LargeDataResult<Vec<TransferMessage>> {
        let session = self
            .sessions
            .get_mut(&content_hash)
            .ok_or_else(|| TransferError::TransferNotFound(content_hash.clone()))?;

        // This is a placeholder for logic that would retrieve and send the chunk data.
        // For now, we'll just send an empty response.
        println!("Received request for {} chunks from {}", chunk_indices.len(), session.content_hash);

        Ok(vec![])
    }

    /// Handles receiving chunk data from a peer.
    fn handle_chunk_data(
        &mut self,
        content_hash: String,
        chunk: crate::large_data_transfer::chunk::DataChunk,
    ) -> LargeDataResult<Vec<TransferMessage>> {
         let session = self
            .sessions
            .get_mut(&content_hash)
            .ok_or_else(|| TransferError::TransferNotFound(content_hash.clone()))?;

        println!("Received chunk {} for {}", chunk.chunk_id.to_string(), session.content_hash);
        // Here we would update the chunk status in the session.
        Ok(vec![])
    }

    /// Initiates a download for a large data object.
    pub fn start_download(
        &mut self,
        descriptor: LargeDataDescriptor,
    ) -> LargeDataResult<()> {
        let content_hash = descriptor.id.clone();
        let mut session = TransferSession::new(content_hash);
        session.descriptor = Some(descriptor);
        session.set_state(TransferState::Active);
        self.sessions.insert(session.content_hash.clone(), session);
        Ok(())
    }
    
    /// Retrieves an immutable reference to a transfer session.
    pub fn get_session(&self, content_hash: &str) -> Option<&TransferSession> {
        self.sessions.get(content_hash)
    }

    /// Retrieves a mutable reference to a transfer session.
    pub fn get_session_mut(&mut self, content_hash: &str) -> Option<&mut TransferSession> {
        self.sessions.get_mut(content_hash)
    }

    /// Cleans up sessions that have timed out.
    pub fn cleanup_timed_out(&mut self) {
        self.sessions
            .retain(|_, session| !session.is_timed_out(self.default_timeout));
    }

    /// Returns a list of all active sessions.
    pub fn active_sessions(&self) -> Vec<&TransferSession> {
        self.sessions.values().collect()
    }
} 