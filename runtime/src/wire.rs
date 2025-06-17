//! Defines the wire protocol for messages exchanged between BCAI nodes.
//!
//! This module contains the serializable message types that are sent over the P2P network.
//! Using a unified `WireMessage` enum ensures that all communication is strongly typed
//! and can be versioned and handled gracefully.

use crate::blockchain::{Block, Transaction};
use serde::{Deserialize, Serialize};

/// The top-level message envelope for all P2P communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WireMessage {
    /// A new block that has been mined or received.
    Block(Block),
    /// A new transaction submitted to the network.
    Transaction(Transaction),
    /// A request to get blocks from a certain height.
    GetBlocks { from_height: u64 },
    /// A response containing a batch of blocks.
    Blocks(Vec<Block>),
    /// A generic ping message for testing connectivity.
    Ping,
    /// A generic pong response.
    Pong,
} 