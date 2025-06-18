use crate::blockchain::Block;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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