use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub max_transactions_per_block: usize,
}

impl Default for BlockchainConfig {
    fn default() -> Self {
        Self {
            max_transactions_per_block: 1000,
        }
    }
} 