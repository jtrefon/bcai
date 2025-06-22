//! Network Transfer Coordinator – orchestrates peers, bandwidth and protocol.

mod core;
#[cfg(feature="ldtc-loops")]
mod loops;
mod stats;

pub use core::NetworkTransferCoordinator;
pub use crate::large_data_transfer::network::models::NetworkStats; 