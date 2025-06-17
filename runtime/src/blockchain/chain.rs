use crate::blockchain::{
    block::Block,
    config::BlockchainConfig,
    constants::{BLOCK_REWARD, DEV_GENESIS_PUBKEY},
    error::BlockchainError,
    state::BlockchainState,
    transaction::Transaction,
    validation,
};
use crate::pouw::{PoUWTask, PoUWSolution};
use std::collections::HashMap;

/// The main Blockchain struct, representing the distributed ledger.
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub state: BlockchainState,
    /// Mapping from public key (hex string) to the next valid nonce.
    pub account_nonces: HashMap<String, u64>,
    pub config: BlockchainConfig,
}

impl Blockchain {
    /// Creates a new blockchain, complete with a genesis block.
    pub fn new(config: BlockchainConfig) -> Self {
        let mut blockchain = Self {
            blocks: Vec::new(),
            state: BlockchainState::new(),
            account_nonces: HashMap::new(),
            config,
        };
        blockchain.create_genesis_block();
        blockchain
    }

    /// Creates the very first block in the chain.
    fn create_genesis_block(&mut self) {
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
        let genesis_block = Block::new(
            0,
            "0".repeat(64),
            vec![],
            0, // difficulty
            "genesis".to_string(),
            genesis_task,
            genesis_solution,
        );

        // Pre-fund the developer account.
        self.state
            .balances
            .insert(DEV_GENESIS_PUBKEY.to_string(), 1_000_000_000);
        self.account_nonces
            .insert(DEV_GENESIS_PUBKEY.to_string(), 0);

        self.blocks.push(genesis_block);
    }

    /// Adds a new block to the chain, validating it and applying all its transactions to the state.
    pub fn add_block(&mut self, block: Block) -> Result<(), BlockchainError> {
        let prev_block = self.blocks.last().expect("Blockchain must have a genesis block");
        validation::validate_block(&block, prev_block, &self.state)?;

        // Apply transactions and calculate total fees
        let mut total_fees = 0;
        for tx in &block.transactions {
            self.state.apply_transaction(tx)?;
            total_fees += tx.fee;
        }

        // Reward the miner
        let miner_reward = BLOCK_REWARD
            .checked_add(total_fees)
            .ok_or(BlockchainError::TransactionValidationError(
                "Miner reward overflow".to_string(),
            ))?;

        let miner_balance = self.state.balances.entry(block.miner.clone()).or_insert(0);
        *miner_balance = miner_balance
            .checked_add(miner_reward)
            .ok_or(BlockchainError::TransactionValidationError(
                "Miner balance overflow".to_string(),
            ))?;

        // Add the block to the chain
        self.blocks.push(block);

        Ok(())
    }

    /// Validates a single transaction against the current confirmed state of the blockchain.
    /// This is used to check if a transaction is valid for inclusion in the mempool.
    pub fn validate_transaction(&self, tx: &Transaction) -> Result<(), BlockchainError> {
        validation::validate_transaction_with_state(tx, &self.state.balances, &self.account_nonces)
    }

    pub fn get_nonce(&self, pubkey_hex: &str) -> u64 {
        *self.account_nonces.get(pubkey_hex).unwrap_or(&0)
    }

    pub fn get_balance(&self, pubkey_hex: &str) -> u64 {
        *self.state.balances.get(pubkey_hex).unwrap_or(&0)
    }
} 