use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::chains::ChainId;

/// Bridge validator node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeValidator {
    pub validator_id: String,
    pub public_key: String,
    pub supported_chains: Vec<ChainId>,
    pub stake_amount: u64,
    pub reputation_score: f64,
    pub is_active: bool,
    pub last_heartbeat: DateTime<Utc>,
    pub total_validations: u64,
    pub successful_validations: u64,
} 