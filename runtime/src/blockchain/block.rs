use crate::blockchain::transaction::Transaction;
use crate::pouw::{PoUWTask, PoUWSolution};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub index: u32,
    pub hash: String,
    pub prev_hash: String,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub difficulty: u32,
    pub miner: String,
    /// The Proof-of-Work challenge.
    pub task: PoUWTask,
    /// The solution to the Proof-of-Work challenge.
    pub solution: PoUWSolution,
}

impl Block {
    /// Creates a new block. The hash is calculated automatically.
    pub fn new(
        index: u32,
        prev_hash: String,
        transactions: Vec<Transaction>,
        difficulty: u32,
        miner: String,
        task: PoUWTask,
        solution: PoUWSolution,
    ) -> Self {
        let timestamp = Utc::now().timestamp();
        let mut block = Block {
            index,
            hash: String::new(),
            prev_hash,
            timestamp,
            transactions,
            difficulty,
            miner,
            task,
            solution,
        };
        block.hash = block.calculate_hash();
        block
    }

    /// Calculates the block's hash based on its contents.
    fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let tx_root = Transaction::merkle_root(&self.transactions);
        let contents = format!(
            "{}{}{}{}{}{}",
            self.index,
            self.prev_hash,
            self.timestamp,
            tx_root,
            self.difficulty,
            self.solution.trained_model_hash
        );
        hasher.update(contents);
        hex::encode(hasher.finalize())
    }
} 