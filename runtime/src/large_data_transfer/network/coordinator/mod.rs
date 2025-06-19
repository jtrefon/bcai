//! Network Transfer Coordinator – orchestrates peers, bandwidth and protocol.

mod core;
mod loops;
mod stats;

pub use core::NetworkTransferCoordinator;
pub use stats::NetworkStats; 