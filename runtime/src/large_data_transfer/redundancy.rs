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