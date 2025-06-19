use crate::blockchain::{
    block::Block,
    constants::DEV_GENESIS_PUBKEY,
    state::BlockchainState,
};
use crate::pouw::{PoUWTask, PoUWSolution};
use std::collections::HashMap;

pub struct GenesisCreator;

impl GenesisCreator {
    /// Creates the very first block in the chain
    pub fn create_genesis_block() -> Block {
        let genesis_task = PoUWTask {
            model_id: "genesis_model".to_string(),
            dataset_id: "genesis_data".to_string(),
            epochs: 0,
        };
        let genesis_solution = PoUWSolution {
            trained_model_hash: "0".repeat(64),
            accuracy: 10000,
            nonce: 0,
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
            .insert(DEV_GENESIS_PUBKEY.to_string(), 1_000_000_000);
        account_nonces
            .insert(DEV_GENESIS_PUBKEY.to_string(), 0);
    }
} 