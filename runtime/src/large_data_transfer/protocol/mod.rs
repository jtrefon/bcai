//! Defines the protocol for large data transfers, including messages, state, and handlers.

pub mod error;
pub mod handler;
pub mod message;
pub mod session;
pub mod stats;
pub mod state;

#[cfg(test)]
mod tests;

pub use error::TransferError;
pub use handler::ProtocolHandler;
pub use message::TransferMessage;
pub use session::{PeerInfo, TransferSession};
pub use stats::TransferStats;
pub use state::{ChunkStatus, TransferErrorType, TransferState}; 