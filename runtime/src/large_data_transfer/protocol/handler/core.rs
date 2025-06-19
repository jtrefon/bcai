use std::collections::HashMap;
use std::time::Duration;
use crate::large_data_transfer::{descriptor::LargeDataDescriptor, LargeDataResult};
use super::super::{message::TransferMessage, session::TransferSession, state::TransferState};
use super::super::error::TransferError;

/// Central state-machine orchestrating all active transfer sessions.
///
/// Most message-specific logic lives in sibling modules that extend this struct
/// via `impl` blocks, keeping each file concise.
pub struct ProtocolHandler {
    pub(super) sessions: HashMap<String, TransferSession>,
    pub node_id: String,
    default_timeout: Duration,
}

impl ProtocolHandler {
    /// Create a new handler for the local node.
    pub fn new(node_id: String) -> Self {
        Self { sessions: HashMap::new(), node_id, default_timeout: Duration::from_secs(60) }
    }

    /// Begin downloading a large data object by descriptor.
    pub fn start_download(&mut self, descriptor: LargeDataDescriptor) -> LargeDataResult<()> {
        let content_hash = descriptor.id.clone();
        let mut session = TransferSession::new(content_hash);
        session.descriptor = Some(descriptor);
        session.set_state(TransferState::Active);
        self.sessions.insert(session.content_hash.clone(), session);
        Ok(())
    }

    /// Immutable access to a session.
    pub fn get_session(&self, id: &str) -> Option<&TransferSession> { self.sessions.get(id) }
    /// Mutable access to a session.
    pub fn get_session_mut(&mut self, id: &str) -> Option<&mut TransferSession> { self.sessions.get_mut(id) }
    /// Remove sessions that exceeded the default timeout.
    pub fn cleanup_timed_out(&mut self) { self.sessions.retain(|_, s| !s.is_timed_out(self.default_timeout)); }
    /// List active sessions.
    pub fn active_sessions(&self) -> Vec<&TransferSession> { self.sessions.values().collect() }
} 