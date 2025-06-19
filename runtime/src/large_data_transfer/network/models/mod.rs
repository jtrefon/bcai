//! Networking models split into sub-modules to maintain single-responsibility.

mod peer;
mod message;
mod bandwidth;
mod stats;

pub use peer::{NetworkPeerInfo, PeerCapabilities, PeerTransferStats};
pub use message::NetworkTransferMessage;
pub use bandwidth::{BandwidthTracker, BandwidthUsage};
pub use stats::NetworkStats; 