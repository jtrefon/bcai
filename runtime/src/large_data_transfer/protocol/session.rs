//! Defines the `TransferSession` and related structs for tracking transfer state.

use super::{
    state::{ChunkStatus, TransferState},
    stats::TransferStats,
};
use crate::large_data_transfer::descriptor::LargeDataDescriptor;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Holds all state for a single large data transfer.
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

/// Information about a peer participating in a transfer session.
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub node_id: String,
    pub available_chunks: Vec<u32>,
    pub bandwidth: u64,
    pub reliability: f32,
    pub last_seen: Instant,
}

impl TransferSession {
    /// Create a new transfer session.
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


} 