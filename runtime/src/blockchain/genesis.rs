use crate::blockchain::{
    block::Block,
    constants::DEV_PUBLIC_KEY,
    state::BlockchainState,
};
use crate::pouw::{PoUWTask}; use crate::pouw::types::PoUWSolution;
use std::collections::HashMap;

pub struct GenesisCreator;

impl GenesisCreator {
    /// Creates the very first block in the chain
    pub fn create_genesis_block() -> Block {
        let genesis_task = PoUWTask {
            model_id: "genesis_model".to_string(),
            dataset_id: "genesis_data".to_string(),
            model_hash: None,
            dataset_hash: None,
            epochs: 0,
            timestamp: 0,
            challenge: [0u8; 32],
        };
        let genesis_solution = PoUWSolution {
            trained_model_hash: "0".repeat(64),
            accuracy: 10000,
            nonce: 0,
            computation_time_ms: 0,
        };
        
        Block::new(
            0,
            "0".repeat(64),
            vec![],
            0, // difficulty
            "genesis".to_string(),
            genesis_task,
            genesis_solution,
        )
    }

    /// Initializes the genesis state with pre-funded developer account
    pub fn initialize_genesis_state(
        state: &mut BlockchainState,
        account_nonces: &mut HashMap<String, u64>,
    ) {
        state
            .balances
            .insert(DEV_PUBLIC_KEY.to_string(), 1_000_000_000);
        account_nonces
            .insert(DEV_PUBLIC_KEY.to_string(), 0);
    }
} 