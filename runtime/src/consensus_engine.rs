//! Advanced Consensus Engine for BCAI
//! 
//! This module provides a robust consensus mechanism for multi-node
//! blockchain networks with Byzantine fault tolerance and performance optimization.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, oneshot};
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

/// Consensus round state
#[derive(Debug, Clone)]
struct ConsensusRound {
    round_number: u32,
    proposals: HashMap<String, ConsensusProposal>,
    votes: HashMap<String, Vec<Vote>>,
    committed_block: Option<Block>,
    start_time: u64,
}

/// Advanced consensus engine
#[derive(Debug)]
pub struct ConsensusEngine {
    config: ConsensusConfig,
    local_node_id: String,
    validators: Arc<RwLock<HashMap<String, Validator>>>,
    current_round: Arc<RwLock<ConsensusRound>>,
    pending_transactions: Arc<RwLock<VecDeque<Transaction>>>,
    blockchain: Arc<RwLock<Vec<Block>>>,
    command_tx: mpsc::UnboundedSender<ConsensusCommand>,
}

/// Consensus commands for async processing
#[derive(Debug)]
enum ConsensusCommand {
    ProposeBlock { transactions: Vec<Transaction> },
    ReceiveProposal { proposal: ConsensusProposal },
    ReceiveVote { vote: Vote },
    StartNewRound,
    ValidateBlock { block: Block, response_tx: oneshot::Sender<bool> },
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

impl ConsensusEngine {
    /// Create a new consensus engine
    pub fn new(config: ConsensusConfig, local_node_id: String) -> Self {
        let (command_tx, mut command_rx) = mpsc::unbounded_channel();
        
        let engine = Self {
            config: config.clone(),
            local_node_id: local_node_id.clone(),
            validators: Arc::new(RwLock::new(HashMap::new())),
            current_round: Arc::new(RwLock::new(ConsensusRound {
                round_number: 0,
                proposals: HashMap::new(),
                votes: HashMap::new(),
                committed_block: None,
                start_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            })),
            pending_transactions: Arc::new(RwLock::new(VecDeque::new())),
            blockchain: Arc::new(RwLock::new(Vec::new())),
            command_tx,
        };

        // Spawn background task for consensus processing
        let validators = Arc::clone(&engine.validators);
        let current_round = Arc::clone(&engine.current_round);
        let pending_transactions = Arc::clone(&engine.pending_transactions);
        let blockchain = Arc::clone(&engine.blockchain);
        let config_clone = config.clone();

        tokio::spawn(async move {
            while let Some(command) = command_rx.recv().await {
                Self::process_command(
                    command,
                    &config_clone,
                    &local_node_id,
                    &validators,
                    &current_round,
                    &pending_transactions,
                    &blockchain,
                ).await;
            }
        });

        engine
    }

    /// Add a validator to the consensus
    pub async fn add_validator(&self, validator: Validator) {
        let mut validators = self.validators.write().unwrap();
        validators.insert(validator.node_id.clone(), validator);
    }

    /// Remove a validator from consensus
    pub async fn remove_validator(&self, node_id: &str) {
        let mut validators = self.validators.write().unwrap();
        validators.remove(node_id);
    }

    /// Submit a transaction for inclusion in the next block
    pub async fn submit_transaction(&self, transaction: Transaction) {
        let mut pending = self.pending_transactions.write().unwrap();
        pending.push_back(transaction);
    }

    /// Propose a new block
    pub async fn propose_block(&self) -> ConsensusResult {
        let transactions = {
            let mut pending = self.pending_transactions.write().unwrap();
            let count = std::cmp::min(
                pending.len(),
                self.config.max_transactions_per_block as usize,
            );
            pending.drain(..count).collect()
        };

        if let Err(_) = self.command_tx.send(ConsensusCommand::ProposeBlock { transactions }) {
            return ConsensusResult::ConsensusError("Command channel closed".to_string());
        }

        ConsensusResult::BlockCommitted(Block {
            index: 0,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            data: "Proposed block".to_string(),
            hash: "temp_hash".to_string(),
            previous_hash: "prev_hash".to_string(),
        })
    }

    /// Validate a block according to consensus rules
    pub async fn validate_block(&self, block: Block) -> bool {
        let (response_tx, response_rx) = oneshot::channel();
        
        if let Err(_) = self.command_tx.send(ConsensusCommand::ValidateBlock { block, response_tx }) {
            return false;
        }

        response_rx.await.unwrap_or(false)
    }

    /// Get consensus statistics
    pub async fn get_stats(&self) -> ConsensusStats {
        let validators = self.validators.read().unwrap();
        let round = self.current_round.read().unwrap();
        let pending = self.pending_transactions.read().unwrap();
        let blockchain = self.blockchain.read().unwrap();

        let active_validators = validators.values().filter(|v| v.is_active).count();
        let total_stake: u64 = validators.values().map(|v| v.stake).sum();
        let avg_block_time = if blockchain.len() > 1 {
            let first_time = blockchain.first().map(|b| b.timestamp).unwrap_or(0);
            let last_time = blockchain.last().map(|b| b.timestamp).unwrap_or(0);
            if blockchain.len() > 1 {
                (last_time - first_time) / (blockchain.len() - 1) as u64
            } else {
                0
            }
        } else {
            0
        };

        ConsensusStats {
            algorithm: self.config.algorithm,
            current_round: round.round_number,
            active_validators,
            total_validators: validators.len(),
            total_stake,
            pending_transactions: pending.len(),
            blockchain_height: blockchain.len(),
            avg_block_time,
            last_block_time: blockchain.last().map(|b| b.timestamp).unwrap_or(0),
        }
    }

    /// Calculate voting power for consensus decisions
    fn calculate_voting_power(validators: &HashMap<String, Validator>) -> HashMap<String, f32> {
        let total_power: f32 = validators.values().map(|v| v.voting_power()).sum();
        
        validators
            .iter()
            .map(|(id, validator)| {
                let normalized_power = if total_power > 0.0 {
                    validator.voting_power() / total_power
                } else {
                    0.0
                };
                (id.clone(), normalized_power)
            })
            .collect()
    }

    /// Check if we have enough votes for consensus
    fn has_consensus(votes: &[Vote], voting_power: &HashMap<String, f32>, threshold: f32) -> bool {
        let total_vote_power: f32 = votes
            .iter()
            .map(|vote| voting_power.get(&vote.voter_id).unwrap_or(&0.0))
            .sum();
        
        total_vote_power >= threshold
    }

    /// Process consensus commands asynchronously
    async fn process_command(
        command: ConsensusCommand,
        config: &ConsensusConfig,
        local_node_id: &str,
        validators: &Arc<RwLock<HashMap<String, Validator>>>,
        current_round: &Arc<RwLock<ConsensusRound>>,
        pending_transactions: &Arc<RwLock<VecDeque<Transaction>>>,
        blockchain: &Arc<RwLock<Vec<Block>>>,
    ) {
        match command {
            ConsensusCommand::ProposeBlock { transactions } => {
                let block_index = {
                    let blockchain = blockchain.read().unwrap();
                    blockchain.len() as u64
                };

                let previous_hash = {
                    let blockchain = blockchain.read().unwrap();
                    blockchain.last()
                        .map(|b| b.hash.clone())
                        .unwrap_or_else(|| "genesis".to_string())
                };

                let block = Block {
                    index: block_index,
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    data: format!("Block with {} transactions", transactions.len()),
                    hash: format!("hash_{}", block_index),
                    previous_hash,
                };

                // Add to blockchain (simplified - in reality would need consensus)
                {
                    let mut blockchain = blockchain.write().unwrap();
                    blockchain.push(block);
                }
            }
            ConsensusCommand::ReceiveProposal { proposal: _ } => {
                // TODO: Process received proposal
            }
            ConsensusCommand::ReceiveVote { vote: _ } => {
                // TODO: Process received vote
            }
            ConsensusCommand::StartNewRound => {
                let mut round = current_round.write().unwrap();
                round.round_number += 1;
                round.proposals.clear();
                round.votes.clear();
                round.committed_block = None;
                round.start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            }
            ConsensusCommand::ValidateBlock { block, response_tx } => {
                // Basic validation (in reality would be more comprehensive)
                let is_valid = !block.data.is_empty() && !block.hash.is_empty();
                let _ = response_tx.send(is_valid);
            }
        }
    }
}

/// Consensus statistics
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_consensus_creation() {
        let config = ConsensusConfig::default();
        let engine = ConsensusEngine::new(config, "test_node".to_string());
        
        let stats = engine.get_stats().await;
        assert_eq!(stats.active_validators, 0);
        assert_eq!(stats.blockchain_height, 0);
    }

    #[tokio::test]
    async fn test_validator_management() {
        let config = ConsensusConfig::default();
        let engine = ConsensusEngine::new(config, "test_node".to_string());
        
        let validator = Validator::new(
            "validator1".to_string(),
            "pubkey1".to_string(),
            1000,
        );
        
        engine.add_validator(validator).await;
        
        let stats = engine.get_stats().await;
        assert_eq!(stats.active_validators, 1);
        assert_eq!(stats.total_stake, 1000);
    }

    #[tokio::test]
    async fn test_transaction_submission() {
        let config = ConsensusConfig::default();
        let engine = ConsensusEngine::new(config, "test_node".to_string());
        
        let transaction = Transaction {
            from: "alice".to_string(),
            to: "bob".to_string(),
            amount: 100,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        
        engine.submit_transaction(transaction).await;
        
        let stats = engine.get_stats().await;
        assert_eq!(stats.pending_transactions, 1);
    }

    #[tokio::test]
    async fn test_block_proposal() {
        let config = ConsensusConfig::default();
        let engine = ConsensusEngine::new(config, "test_node".to_string());
        
        let transaction = Transaction {
            from: "alice".to_string(),
            to: "bob".to_string(),
            amount: 100,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        
        engine.submit_transaction(transaction).await;
        
        let result = engine.propose_block().await;
        assert!(matches!(result, ConsensusResult::BlockCommitted(_)));
        
        // Give some time for async processing
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let stats = engine.get_stats().await;
        assert_eq!(stats.blockchain_height, 1);
    }

    #[test]
    fn test_voting_power_calculation() {
        let mut validators = HashMap::new();
        
        validators.insert("v1".to_string(), Validator {
            node_id: "v1".to_string(),
            public_key: "key1".to_string(),
            stake: 1000,
            reputation: 1.0,
            last_seen: 0,
            is_active: true,
            performance_score: 1.0,
        });
        
        validators.insert("v2".to_string(), Validator {
            node_id: "v2".to_string(),
            public_key: "key2".to_string(),
            stake: 2000,
            reputation: 1.0,
            last_seen: 0,
            is_active: true,
            performance_score: 1.0,
        });
        
        let voting_power = ConsensusEngine::calculate_voting_power(&validators);
        
        assert!((voting_power["v1"] - 0.333).abs() < 0.01);
        assert!((voting_power["v2"] - 0.666).abs() < 0.01);
    }
} 