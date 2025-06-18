//! Defines state-related enums for the transfer protocol.

use crate::large_data_transfer::chunk::ChunkId;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Types of errors that can occur during a transfer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransferErrorType {
    NetworkError,
    Timeout,
    IntegrityFailure,
    BandwidthLimited,
    StorageFull,
    Unauthorized,
    Cancelled,
    Unknown,
}

/// The overall state of a transfer session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransferState {
    Initiating,
    Pending,
    Active,
    Paused,
    Completed,
    Failed(TransferErrorType),
    Cancelled,
}

/// The status of an individual chunk within a transfer.
#[derive(Debug, Clone)]
pub enum ChunkStatus {
    /// Chunk location is not yet known.
    Unknown,
    /// Chunk is available from a set of peers.
    Available(Vec<String>),
    /// Chunk is currently being downloaded from a peer.
    Downloading(String, Instant),
    /// Chunk has been successfully downloaded and verified.
    Complete(ChunkId),
    /// An attempt to download the chunk failed.
    Failed(String),
} 