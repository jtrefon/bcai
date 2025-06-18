//! The network integration layer for large data transfers.
//!
//! This module is responsible for bridging the core data transfer logic
//! with the underlying P2P network, handling peer management, chunk routing,
//! and bandwidth control.

pub mod coordinator;
pub mod error;
pub mod models;
pub mod peer_manager;
pub mod bandwidth_manager;
pub mod transfer_handler;

pub use coordinator::NetworkTransferCoordinator;
pub use error::NetworkError;
pub use models::{NetworkPeerInfo, NetworkStats, PeerCapabilities}; 