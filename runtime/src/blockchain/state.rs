use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainState {
    /// Mapping from public key (hex string) to token balance.
    pub balances: HashMap<String, u64>,
}

impl BlockchainState {
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
        }
    }
} 