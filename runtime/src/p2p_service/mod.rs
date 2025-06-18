//! Provides the real networking layer that connects our unified node
//! architecture with actual P2P communication using libp2p.
//!
//! The main entry points are the `P2PService` struct, which runs the main
//! event loop, and the `P2PHandle`, which is the public API for interacting
//! with the service.

pub mod behaviour;
pub mod codec;
pub mod command;
pub mod config;
pub mod error;
pub mod service;
pub mod types;
pub mod service_event;
pub mod service_command;
pub mod service_init;

#[cfg(test)]
mod tests;

pub use command::P2PHandle;
pub use config::P2PConfig;
pub use error::P2PError;
pub use service::P2PService;
pub use types::{P2PStats, PeerInfo}; 