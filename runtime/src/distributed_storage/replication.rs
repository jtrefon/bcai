use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Storage node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageNode {
    pub node_id: String,
    pub address: String,
    pub capacity: u64,
    pub used_space: u64,
    pub last_seen: u64,
    pub reliability_score: f32,
} 