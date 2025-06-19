use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::chains::ChainId;

/// Bridge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    pub supported_chains: Vec<ChainId>,
    pub min_confirmations: HashMap<ChainId, u32>,
    pub bridge_fee_rate: f64,
    pub validator_threshold: u32,
    pub transaction_timeout_hours: u64,
    pub max_transaction_amount: u64,
    pub emergency_pause_enabled: bool,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            supported_chains: vec![
                ChainId::Ethereum,
                ChainId::Polygon,
                ChainId::BinanceSmartChain,
            ],
            min_confirmations: HashMap::from([
                (ChainId::Ethereum, 12),
                (ChainId::Polygon, 64),
                (ChainId::BinanceSmartChain, 15),
            ]),
            bridge_fee_rate: 0.001, // 0.1%
            validator_threshold: 3,
            transaction_timeout_hours: 24,
            max_transaction_amount: 1_000_000, // $1M
            emergency_pause_enabled: false,
        }
    }
} 