use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Consensus algorithm types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConsensusAlgorithm {
    /// Proof of Useful Work (PoUW)
    ProofOfUsefulWork,
    /// Practical Byzantine Fault Tolerance
    PBFT,
    /// Delegated Proof of Stake
    DPoS,
    /// Hybrid consensus combining multiple algorithms
    Hybrid,
}

/// Consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Consensus algorithm to use
    pub algorithm: ConsensusAlgorithm,
    /// Block time target (seconds)
    pub block_time: u64,
    /// Maximum transactions per block
    pub max_transactions_per_block: u32,
    /// Minimum validators required
    pub min_validators: u32,
    /// Byzantine fault tolerance threshold (f < n/3)
    pub byzantine_threshold: f32,
    /// Timeout for consensus rounds (seconds)
    pub consensus_timeout: u64,
    /// Enable fast finality
    pub enable_fast_finality: bool,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            algorithm: ConsensusAlgorithm::ProofOfUsefulWork,
            block_time: 10, // 10 seconds
            max_transactions_per_block: 1000,
            min_validators: 3,
            byzantine_threshold: 0.33,
            consensus_timeout: 30,
            enable_fast_finality: true,
        }
    }
}

/// Validator node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub node_id: String,
    pub public_key: String,
    pub stake: u64,
    pub reputation: f32,
    pub last_seen: u64,
    pub is_active: bool,
    pub performance_score: f32,
}

impl Validator {
    pub fn new(node_id: String, public_key: String, stake: u64) -> Self {
        Self {
            node_id,
            public_key,
            stake,
            reputation: 1.0,
            last_seen: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            is_active: true,
            performance_score: 1.0,
        }
    }

    pub fn voting_power(&self) -> f32 {
        if !self.is_active {
            return 0.0;
        }
        (self.stake as f32) * self.reputation * self.performance_score
    }
} 