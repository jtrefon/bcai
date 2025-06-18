//! Configuration for the P2P service.

use serde::{Deserialize, Serialize};

/// Basic configuration options for P2P networking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    /// TCP port to listen on.
    pub listen_port: u16,
    /// Optional external address to advertise.
    pub external_address: Option<String>,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            listen_port: 0, // Let the OS pick a free port.
            external_address: None,
        }
    }
} 