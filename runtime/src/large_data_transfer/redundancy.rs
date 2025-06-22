use serde::{Deserialize, Serialize};

/// Configuration governing redundancy & error-correction. Responsibility: keep
/// numeric policy values – no storage logic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedundancyConfig {
    pub replica_count: u32,
    pub erasure_coding: bool,
    pub min_nodes: u32,
}

impl Default for RedundancyConfig {
    fn default() -> Self {
        Self { replica_count: 1, erasure_coding: false, min_nodes: 1 }
    }
}

/// Lightweight policy type used in pricing and CLI.
#[derive(Debug, Clone, Copy)]
pub struct RedundancyPolicy {
    pub copies: u8,
    pub geo_spread: bool,
}

impl From<RedundancyConfig> for RedundancyPolicy {
    fn from(cfg: RedundancyConfig) -> Self {
        RedundancyPolicy { copies: cfg.replica_count as u8, geo_spread: false }
    }
} 