//! Transfer Protocol Implementation
//!
//! This module defines the protocol messages and state management for large data transfers.

use crate::large_data_transfer::{TransferPriority, LargeDataResult};
use crate::large_data_transfer::chunk::{ChunkId, DataChunk};
use crate::large_data_transfer::descriptor::LargeDataDescriptor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Transfer protocol messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferMessage {
    /// Request to initiate a transfer
    TransferRequest {
        content_hash: String,
        priority: TransferPriority,
        bandwidth_limit: Option<u64>,
        requester_id: String,
    },

    /// Response to transfer request
    TransferResponse {
        content_hash: String,
        accepted: bool,
        reason: Option<String>,
        estimated_time: Option<Duration>,
    },

    /// Request specific chunks
    ChunkRequest {
        content_hash: String,
        chunk_indices: Vec<u32>,
        requested_by: String,
        sequence_id: u64,
    },

    /// Response with chunk data
    ChunkData {
        content_hash: String,
        chunk: DataChunk,
        sequence_id: u64,
        sender_id: String,
    },

    /// Progress update
    TransferProgress {
        content_hash: String,
        chunks_completed: u32,
        total_chunks: u32,
        bytes_transferred: u64,
        transfer_rate: f64,
        eta: Option<Duration>,
    },

    /// Transfer completed successfully
    TransferComplete {
        content_hash: String,
        verification_hash: String,
        total_time: Duration,
        final_stats: TransferStats,
    },

    /// Transfer failed or cancelled
    TransferError {
        content_hash: String,
        error_type: TransferErrorType,
        error_message: String,
        retry_after: Option<Duration>,
    },

    /// Heartbeat message
    Heartbeat {
        node_id: String,
        timestamp: u64,
        available_bandwidth: u64,
    },

    /// Descriptor announcement
    DescriptorAnnouncement {
        descriptor: LargeDataDescriptor,
        announcer_id: String,
    },
}

/// Transfer error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferErrorType {
    /// Network connectivity issues
    NetworkError,
    /// Timeout occurred
    Timeout,
    /// Data integrity check failed
    IntegrityFailure,
    /// Insufficient bandwidth
    BandwidthLimited,
    /// Storage space exhausted
    StorageFull,
    /// Authentication/authorization failed
    Unauthorized,
    /// Transfer was cancelled
    Cancelled,
    /// Unknown error
    Unknown,
}

/// Transfer state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferState {
    /// Transfer is being initiated
    Initiating,
    /// Waiting for peer response
    Pending,
    /// Transfer is active
    Active,
    /// Transfer is paused
    Paused,
    /// Transfer completed successfully
    Completed,
    /// Transfer failed
    Failed(TransferErrorType),
    /// Transfer was cancelled
    Cancelled,
}

/// Transfer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferStats {
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub bytes_transferred: u64,
    pub chunks_transferred: u32,
    pub average_speed: f64,
    pub peak_speed: f64,
    pub retry_count: u32,
    pub peer_count: u32,
    pub compression_ratio: f32,
}

impl Default for TransferStats {
    fn default() -> Self {
        Self {
            start_time: 0,
            end_time: None,
            bytes_transferred: 0,
            chunks_transferred: 0,
            average_speed: 0.0,
            peak_speed: 0.0,
            retry_count: 0,
            peer_count: 0,
            compression_ratio: 1.0,
        }
    }
}

/// Protocol errors
#[derive(Error, Debug)]
pub enum TransferError {
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
    
    #[error("Protocol violation: {0}")]
    ProtocolViolation(String),
    
    #[error("Transfer not found: {0}")]
    TransferNotFound(String),
    
    #[error("State error: {0}")]
    StateError(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
}

/// Transfer session manager
#[derive(Debug)]
pub struct TransferSession {
    pub content_hash: String,
    pub state: TransferState,
    pub descriptor: Option<LargeDataDescriptor>,
    pub stats: TransferStats,
    pub peers: HashMap<String, PeerInfo>,
    pub chunk_status: HashMap<u32, ChunkStatus>,
    pub last_activity: Instant,
    pub retry_count: u32,
}

/// Peer information for a transfer
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub node_id: String,
    pub available_chunks: Vec<u32>,
    pub bandwidth: u64,
    pub reliability: f32,
    pub last_seen: Instant,
}

/// Status of individual chunks
#[derive(Debug, Clone)]
pub enum ChunkStatus {
    /// Chunk location unknown
    Unknown,
    /// Chunk is available from specific peers
    Available(Vec<String>),
    /// Chunk transfer is in progress
    Downloading(String, Instant),
    /// Chunk transfer completed
    Complete(ChunkId),
    /// Chunk transfer failed
    Failed(String),
}

impl TransferSession {
    /// Create a new transfer session
    pub fn new(content_hash: String) -> Self {
        Self {
            content_hash,
            state: TransferState::Initiating,
            descriptor: None,
            stats: TransferStats::default(),
            peers: HashMap::new(),
            chunk_status: HashMap::new(),
            last_activity: Instant::now(),
            retry_count: 0,
        }
    }

    /// Update session state
    pub fn set_state(&mut self, state: TransferState) {
        self.state = state;
        self.last_activity = Instant::now();
    }

    /// Add a peer to the session
    pub fn add_peer(&mut self, peer: PeerInfo) {
        self.peers.insert(peer.node_id.clone(), peer);
        self.last_activity = Instant::now();
    }

    /// Update chunk status
    pub fn set_chunk_status(&mut self, chunk_index: u32, status: ChunkStatus) {
        self.chunk_status.insert(chunk_index, status);
        self.last_activity = Instant::now();
    }

    /// Get chunks that need to be downloaded
    pub fn pending_chunks(&self) -> Vec<u32> {
        if let Some(descriptor) = &self.descriptor {
            (0..descriptor.chunk_count)
                .filter(|&i| {
                    !matches!(
                        self.chunk_status.get(&i),
                        Some(ChunkStatus::Complete(_))
                    )
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Calculate transfer progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if let Some(descriptor) = &self.descriptor {
            if descriptor.chunk_count == 0 {
                return 1.0;
            }
            
            let completed = self.chunk_status
                .values()
                .filter(|status| matches!(status, ChunkStatus::Complete(_)))
                .count();
                
            completed as f32 / descriptor.chunk_count as f32
        } else {
            0.0
        }
    }

    /// Check if session has timed out
    pub fn is_timed_out(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }

    /// Get best peer for downloading a chunk
    pub fn best_peer_for_chunk(&self, chunk_index: u32) -> Option<&PeerInfo> {
        self.peers
            .values()
            .filter(|peer| peer.available_chunks.contains(&chunk_index))
            .max_by(|a, b| {
                // Prefer peers with higher bandwidth and reliability
                let score_a = a.bandwidth as f32 * a.reliability;
                let score_b = b.bandwidth as f32 * b.reliability;
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}

/// Protocol handler for managing transfers
#[derive(Debug)]
pub struct ProtocolHandler {
    sessions: HashMap<String, TransferSession>,
    node_id: String,
    default_timeout: Duration,
}

impl ProtocolHandler {
    /// Create a new protocol handler
    pub fn new(node_id: String) -> Self {
        Self {
            sessions: HashMap::new(),
            node_id,
            default_timeout: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Handle incoming protocol message
    pub fn handle_message(&mut self, message: TransferMessage) -> LargeDataResult<Vec<TransferMessage>> {
        match message {
            TransferMessage::TransferRequest { content_hash, priority: _, bandwidth_limit: _, requester_id } => {
                self.handle_transfer_request(content_hash, requester_id)
            }
            
            TransferMessage::ChunkRequest { content_hash, chunk_indices, requested_by: _, sequence_id } => {
                self.handle_chunk_request(content_hash, chunk_indices, sequence_id)
            }
            
            TransferMessage::ChunkData { content_hash, chunk, sequence_id: _, sender_id: _ } => {
                self.handle_chunk_data(content_hash, chunk)
            }
            
            _ => Ok(Vec::new()), // TODO: Handle other message types
        }
    }

    /// Handle transfer request
    fn handle_transfer_request(&mut self, content_hash: String, _requester_id: String) -> LargeDataResult<Vec<TransferMessage>> {
        // Create or get existing session
        if !self.sessions.contains_key(&content_hash) {
            let session = TransferSession::new(content_hash.clone());
            self.sessions.insert(content_hash.clone(), session);
        }

        // TODO: Check if we have the data and can serve it
        let response = TransferMessage::TransferResponse {
            content_hash,
            accepted: true, // For now, always accept
            reason: None,
            estimated_time: Some(Duration::from_secs(60)),
        };

        Ok(vec![response])
    }

    /// Handle chunk request
    fn handle_chunk_request(&mut self, content_hash: String, chunk_indices: Vec<u32>, sequence_id: u64) -> LargeDataResult<Vec<TransferMessage>> {
        // TODO: Look up chunks in local storage and return them
        let mut responses = Vec::new();
        
        for chunk_index in chunk_indices {
            // For now, create a dummy chunk response
            // In real implementation, would fetch from storage
            if let Ok(dummy_chunk) = crate::large_data_transfer::chunk::DataChunk::new(
                vec![0u8; 100], // Dummy data
                chunk_index,
                crate::large_data_transfer::CompressionAlgorithm::None,
            ) {
                responses.push(TransferMessage::ChunkData {
                    content_hash: content_hash.clone(),
                    chunk: dummy_chunk,
                    sequence_id,
                    sender_id: self.node_id.clone(),
                });
            }
        }

        Ok(responses)
    }

    /// Handle received chunk data
    fn handle_chunk_data(&mut self, content_hash: String, chunk: crate::large_data_transfer::chunk::DataChunk) -> LargeDataResult<Vec<TransferMessage>> {
        if let Some(session) = self.sessions.get_mut(&content_hash) {
            // Verify chunk integrity
            if chunk.verify_integrity().is_ok() {
                session.set_chunk_status(chunk.info.index, ChunkStatus::Complete(chunk.id().clone()));
                session.stats.chunks_transferred += 1;
                session.stats.bytes_transferred += chunk.info.original_size as u64;
            } else {
                session.set_chunk_status(chunk.info.index, ChunkStatus::Failed("Integrity check failed".to_string()));
            }
        }

        Ok(Vec::new())
    }

    /// Start a new download transfer
    pub fn start_download(&mut self, content_hash: String, descriptor: LargeDataDescriptor) -> LargeDataResult<()> {
        let mut session = TransferSession::new(content_hash.clone());
        session.descriptor = Some(descriptor);
        session.set_state(TransferState::Pending);
        
        // Initialize chunk status
        for i in 0..session.descriptor.as_ref().unwrap().chunk_count {
            session.set_chunk_status(i, ChunkStatus::Unknown);
        }
        
        self.sessions.insert(content_hash, session);
        Ok(())
    }

    /// Get session by content hash
    pub fn get_session(&self, content_hash: &str) -> Option<&TransferSession> {
        self.sessions.get(content_hash)
    }

    /// Get mutable session by content hash
    pub fn get_session_mut(&mut self, content_hash: &str) -> Option<&mut TransferSession> {
        self.sessions.get_mut(content_hash)
    }

    /// Clean up timed out sessions
    pub fn cleanup_timed_out(&mut self) {
        let timeout = self.default_timeout;
        self.sessions.retain(|_, session| !session.is_timed_out(timeout));
    }

    /// Get all active sessions
    pub fn active_sessions(&self) -> Vec<&TransferSession> {
        self.sessions
            .values()
            .filter(|session| matches!(session.state, TransferState::Active | TransferState::Pending))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_session_creation() {
        let session = TransferSession::new("test_hash".to_string());
        assert_eq!(session.content_hash, "test_hash");
        assert!(matches!(session.state, TransferState::Initiating));
        assert_eq!(session.progress(), 0.0);
    }

    #[test]
    fn test_session_progress_calculation() {
        let mut session = TransferSession::new("test".to_string());
        
        // No descriptor = 0% progress
        assert_eq!(session.progress(), 0.0);
        
        // Add descriptor with 4 chunks
        let data = vec![b'A'; 100];
        let chunks = crate::large_data_transfer::chunk::ChunkUtils::split_data(
            &data, 25, crate::large_data_transfer::CompressionAlgorithm::None
        ).unwrap();
        
        let metadata = crate::large_data_transfer::descriptor::TransferMetadata::new("test".to_string());
        let descriptor = crate::large_data_transfer::descriptor::LargeDataDescriptor::from_chunks(
            &chunks,
            crate::large_data_transfer::CompressionAlgorithm::None,
            crate::large_data_transfer::EncryptionAlgorithm::None,
            metadata,
        );
        
        session.descriptor = Some(descriptor);
        assert_eq!(session.progress(), 0.0); // No chunks completed yet
        
        // Complete 2 out of 4 chunks
        session.set_chunk_status(0, ChunkStatus::Complete(chunks[0].id().clone()));
        session.set_chunk_status(1, ChunkStatus::Complete(chunks[1].id().clone()));
        
        assert_eq!(session.progress(), 0.5); // 50% complete
    }

    #[test]
    fn test_protocol_handler() {
        let mut handler = ProtocolHandler::new("test_node".to_string());
        
        let request = TransferMessage::TransferRequest {
            content_hash: "test_hash".to_string(),
            priority: TransferPriority::Normal,
            bandwidth_limit: None,
            requester_id: "requester".to_string(),
        };
        
        let responses = handler.handle_message(request).unwrap();
        assert_eq!(responses.len(), 1);
        
        // Should have created a session
        assert!(handler.get_session("test_hash").is_some());
    }

    #[test]
    fn test_session_timeout() {
        let mut session = TransferSession::new("test".to_string());
        
        // Should not be timed out immediately
        assert!(!session.is_timed_out(Duration::from_secs(1)));
        
        // Simulate old activity time
        session.last_activity = Instant::now() - Duration::from_secs(10);
        assert!(session.is_timed_out(Duration::from_secs(5)));
    }
} 