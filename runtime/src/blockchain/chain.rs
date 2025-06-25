use crate::blockchain::{
    block::Block,
    config::BlockchainConfig,
    state::BlockchainState,
    transaction::Transaction,
    validation,
    genesis::GenesisCreator,
    block_processor::BlockProcessor,
    account_manager::AccountManager,
};
use std::collections::HashMap;

/// The main Blockchain struct, representing the distributed ledger.
pub use crate::blockchain::error::BlockchainError;

pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub state: BlockchainState,
    /// Mapping from public key (hex string) to the next valid nonce.
    pub account_nonces: HashMap<String, u64>,
    pub config: BlockchainConfig,
    /// Pending transactions awaiting inclusion in a block.
    pub pending_transactions: Vec<Transaction>,
}

impl Blockchain {
    /// Creates a new blockchain, complete with a genesis block.
    pub fn new(config: BlockchainConfig) -> Self {
        let mut blockchain = Self {
            blocks: Vec::new(),
            state: BlockchainState::new(),
            account_nonces: HashMap::new(),
            config,
            pending_transactions: Vec::new(),
        };
        blockchain.create_genesis_block();
        blockchain
    }

    /// Creates the very first block in the chain.
    fn create_genesis_block(&mut self) {
        let genesis_block = GenesisCreator::create_genesis_block();
        GenesisCreator::initialize_genesis_state(&mut self.state, &mut self.account_nonces);
        self.blocks.push(genesis_block);
    }

    /// Adds a new block to the chain, validating it and applying all its transactions to the state.
    pub fn add_block(&mut self, block: Block) -> Result<(), BlockchainError> {
        let prev_block = self.blocks.last().expect("Blockchain must have a genesis block");
        BlockProcessor::process_block(&block, prev_block, &mut self.state)?;
        self.blocks.push(block);
        Ok(())
    }

    /// Validates a single transaction against the current confirmed state of the blockchain.
    /// This is used to check if a transaction is valid for inclusion in the mempool.
    pub fn validate_transaction(&self, tx: &Transaction) -> Result<(), BlockchainError> {
        validation::validate_transaction_with_state(tx, &self.state.balances, &self.account_nonces)
    }

    pub fn get_nonce(&self, pubkey_hex: &str) -> u64 {
        AccountManager::get_nonce(&self.account_nonces, pubkey_hex)
    }

    pub fn get_balance(&self, pubkey_hex: &str) -> u64 {
        AccountManager::get_balance(&self.state, pubkey_hex)
    }

    /// Returns the latest block (the chain tip).
    pub fn get_tip(&self) -> &Block {
        self.blocks.last().expect("chain must have genesis block")
    }

    /// Current block height.
    pub fn height(&self) -> u64 {
        self.blocks.len() as u64
    }

    /// Adds a transaction to the pending list after validation.
    pub fn add_transaction(&mut self, tx: Transaction) -> Result<(), BlockchainError> {
        self.validate_transaction(&tx)?;
        self.pending_transactions.push(tx);
        Ok(())
    }

    /// Returns up to `limit` pending transactions.
    pub fn get_pending_transactions(&self, limit: usize) -> Vec<Transaction> {
        self.pending_transactions.iter().take(limit).cloned().collect()
    }

    /// Adaptive difficulty based on recent PoUW metrics.
    pub fn calculate_next_difficulty(&self) -> u32 {
        let prev_difficulty = self.get_tip().difficulty;
        let target_time = 60; // target solve time in seconds
        let avg_time_ms = if self.state.pouw_metrics.is_empty() {
            target_time * 1000
        } else {
            self.state
                .pouw_metrics
                .iter()
                .map(|m| m.1)
                .sum::<u64>()
                / self.state.pouw_metrics.len() as u64
        };
        crate::pouw::calculate_adaptive_difficulty(
            prev_difficulty,
            target_time,
            avg_time_ms / 1000,
        )
    }

    /// Simple blockchain statistics.
    pub fn get_stats(&self) -> BlockchainStats {
        BlockchainStats {
            height: self.height(),
            pending_txs: self.pending_transactions.len(),
        }
    }
}

/// Lightweight chain metrics.
#[derive(Debug, Clone)]
pub struct BlockchainStats {
    pub height: u64,
    pub pending_txs: usize,
}
