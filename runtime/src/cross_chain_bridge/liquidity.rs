use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::chains::ChainId;

/// Bridge liquidity pool for each chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPool {
    pub chain_id: ChainId,
    pub token_reserves: HashMap<String, u64>,
    pub total_locked: u64,
    pub total_minted: u64,
    pub utilization_rate: f64,
    pub fee_rate: f64,
    pub last_updated: DateTime<Utc>,
} 