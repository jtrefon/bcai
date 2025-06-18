//! Defines the configuration for the devnet.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevnetConfig {
    pub node_count: u32,
    pub ai_workers: u32,
    pub initial_tokens: u64,
    pub port_base: u16,
}

impl Default for DevnetConfig {
    fn default() -> Self {
        Self {
            node_count: 3,
            ai_workers: 1,
            initial_tokens: 1000,
            port_base: 8000,
        }
    }
} 