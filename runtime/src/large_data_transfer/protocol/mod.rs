//! Defines the protocol for large data transfers, including messages, state, and handlers.

pub mod error;
pub mod handler;
pub mod message;
pub mod session;
pub mod stats;
pub mod state;
pub mod session_state;
pub mod peer_management;
pub mod chunk_tracking;
pub mod progress_metrics;

#[cfg(test)]
mod tests;

pub use error::TransferError;
pub use handler::ProtocolHandler;
pub use message::TransferMessage;
pub use session::{PeerInfo, TransferSession};
pub use stats::TransferStats;
pub use state::{ChunkStatus, TransferErrorType, TransferState}; 