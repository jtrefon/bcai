//! Defines the wire protocol messages for large data transfers.

use crate::large_data_transfer::{
    chunk::{DataChunk, ChunkId},
    descriptor::LargeDataDescriptor,
    types::{TransferPriority, TransferStats},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use super::state::TransferErrorType;


/// Transfer protocol messages exchanged between peers.
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

    /// Heartbeat message to keep connections alive and share status.
    Heartbeat {
        node_id: String,
        timestamp: u64,
        available_bandwidth: u64,
    },

    /// Announce the availability of a new large data object.
    DescriptorAnnouncement {
        descriptor: LargeDataDescriptor,
        announcer_id: String,
    },
} 