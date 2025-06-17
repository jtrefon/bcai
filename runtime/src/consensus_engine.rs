//! Advanced Consensus Engine for BCAI
//! 
//! This module provides a robust consensus mechanism for multi-node
//! blockchain networks with Byzantine fault tolerance and performance optimization.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::{Block, Transaction};

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

/// Consensus proposal for a new block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusProposal {
    pub proposer_id: String,
    pub block: Block,
    pub timestamp: u64,
    pub round: u32,
    pub votes: HashMap<String, Vote>,
    pub signature: String,
}

/// Vote on a consensus proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter_id: String,
    pub block_hash: String,
    pub vote_type: VoteType,
    pub timestamp: u64,
    pub signature: String,
}

/// Types of votes in consensus
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VoteType {
    Prevote,
    Precommit,
    Commit,
}

/// Consensus results
#[derive(Debug, Clone)]
pub enum ConsensusResult {
    BlockCommitted(Block),
    ProposalRejected(String),
    RoundTimeout,
    InsufficientValidators,
    ConsensusError(String),
}

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

// NOTE: Removed placeholder implementation structs:
// - ConsensusEngine
// - ConsensusRound
// This file now only defines the data models for the consensus engine. 