use crate::consensus_engine::engine::ConsensusAlgorithm;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusStats {
    pub algorithm: ConsensusAlgorithm,
    pub current_round: u32,
    pub active_validators: usize,
    pub total_validators: usize,
    pub total_stake: u64,
    pub pending_transactions: usize,
    pub blockchain_height: usize,
    pub avg_block_time: u64,
    pub last_block_time: u64,
} 