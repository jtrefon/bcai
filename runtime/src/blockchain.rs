//! Blockchain Implementation for BCAI
//!
//! This module provides a real blockchain with:
//! - Proof of Useful Work consensus
//! - Immutable transaction ledger
//! - Smart contract support for AI training jobs
//! - Validator network coordination

use crate::{
    node::{NodeCapability, TrainingResult},
    pouw::{Solution, Task, generate_task, verify},
    token::TokenLedger,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Blockchain errors
#[derive(Debug, Error)]
pub enum BlockchainError {
    #[error("Invalid block: {0}")]
    InvalidBlock(String),
    #[error("Block not found: {0}")]
    BlockNotFound(u64),
    #[error("Transaction invalid: {0}")]
    InvalidTransaction(String),
    #[error("Consensus failure: {reason}")]
    ConsensusFailed { reason: String },
    #[error("Insufficient stake for validation")]
    InsufficientStake,
    #[error("Fork resolution failed")]
    ForkResolutionFailed,
}

/// Transaction types in BCAI blockchain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Transaction {
    /// Token transfer between accounts
    Transfer {
        from: String,
        to: String,
        amount: u64,
        fee: u64,
    },
    /// Stake tokens for validation rights
    Stake {
        validator: String,
        amount: u64,
    },
    /// Post a new AI training job
    JobPosting {
        poster: String,
        job_id: u64,
        description: String,
        reward: u64,
        requirements: NodeCapability,
        data_hash: String,
        deadline: u64,
    },
    /// Submit training results
    TrainingSubmission {
        worker: String,
        job_id: u64,
        result_hash: String,
        pouw_solution: Solution,
        accuracy_claim: f64,
    },
    /// Validator consensus on training result
    ValidationVote {
        validator: String,
        job_id: u64,
        worker: String,
        is_valid: bool,
        stake_weight: u64,
    },
    /// Distribute rewards after successful training
    RewardDistribution {
        job_id: u64,
        workers: Vec<String>,
        amounts: Vec<u64>,
    },
}

/// Block in the BCAI blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub height: u64,
    pub previous_hash: String,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub validator: String,
    pub pouw_task: Task,
    pub pouw_solution: Solution,
    pub merkle_root: String,
    pub state_root: String,
    pub hash: String,
}

/// Blockchain state for BCAI network
#[derive(Debug, Clone)]
pub struct BlockchainState {
    pub token_ledger: TokenLedger,
    pub validator_stakes: HashMap<String, u64>,
    pub active_jobs: HashMap<u64, JobState>,
    pub completed_jobs: HashMap<u64, CompletedJob>,
    pub reputation_scores: HashMap<String, i32>,
}

/// Job state on blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobState {
    pub job_id: u64,
    pub poster: String,
    pub description: String,
    pub reward: u64,
    pub requirements: NodeCapability,
    pub data_hash: String,
    pub deadline: u64,
    pub assigned_workers: Vec<String>,
    pub submitted_results: HashMap<String, TrainingSubmission>,
    pub validator_votes: HashMap<String, ValidationVote>,
    pub status: JobStatus,
}

/// Training submission on blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingSubmission {
    pub worker: String,
    pub result_hash: String,
    pub pouw_solution: Solution,
    pub accuracy_claim: f64,
    pub timestamp: u64,
}

/// Validation vote on blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationVote {
    pub validator: String,
    pub is_valid: bool,
    pub stake_weight: u64,
    pub timestamp: u64,
}

/// Job status on blockchain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Posted,
    WorkersAssigned,
    ResultsSubmitted,
    ValidationPending,
    Completed,
    Disputed,
}

/// Completed job record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedJob {
    pub job_id: u64,
    pub final_accuracy: f64,
    pub participating_workers: Vec<String>,
    pub validator_consensus: bool,
    pub completion_block: u64,
}

/// Blockchain configuration
#[derive(Debug, Clone)]
pub struct BlockchainConfig {
    pub block_time_secs: u64,
    pub min_validator_stake: u64,
    pub consensus_threshold: f64, // Percentage of stake needed for consensus
    pub max_block_size: usize,
    pub difficulty_adjustment_blocks: u64,
    pub job_posting_fee: u64,
}

impl Default for BlockchainConfig {
    fn default() -> Self {
        Self {
            block_time_secs: 30,
            min_validator_stake: 10000,
            consensus_threshold: 0.67, // 67% stake consensus
            max_block_size: 1000,
            difficulty_adjustment_blocks: 100,
            job_posting_fee: 100,
        }
    }
}

/// BCAI Blockchain implementation
pub struct Blockchain {
    config: BlockchainConfig,
    blocks: Vec<Block>,
    pending_transactions: VecDeque<Transaction>,
    state: BlockchainState,
    validators: HashMap<String, ValidatorInfo>,
    current_difficulty: u32,
}

/// Validator information
#[derive(Debug, Clone)]
pub struct ValidatorInfo {
    pub node_id: String,
    pub stake: u64,
    pub reputation: i32,
    pub last_validation: u64,
    pub successful_validations: u64,
    pub failed_validations: u64,
}

impl Blockchain {
    /// Create a new blockchain with genesis block
    pub fn new(config: BlockchainConfig) -> Self {
        let genesis_block = Self::create_genesis_block();
        let initial_state = BlockchainState {
            token_ledger: TokenLedger::new(),
            validator_stakes: HashMap::new(),
            active_jobs: HashMap::new(),
            completed_jobs: HashMap::new(),
            reputation_scores: HashMap::new(),
        };

        Self {
            config,
            blocks: vec![genesis_block],
            pending_transactions: VecDeque::new(),
            state: initial_state,
            validators: HashMap::new(),
            current_difficulty: 0x0000ffff, // Starting difficulty
        }
    }

    /// Create genesis block
    fn create_genesis_block() -> Block {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let genesis_task = generate_task(2, 0); // Simple genesis task
        let genesis_solution = Solution {
            result: vec![vec![0, 0], vec![0, 0]],
            nonce: 0,
            computation_time: 0,
        };

        Block {
            height: 0,
            previous_hash: "0".repeat(64),
            timestamp,
            transactions: vec![],
            validator: "genesis".to_string(),
            pouw_task: genesis_task,
            pouw_solution: genesis_solution,
            merkle_root: "0".repeat(64),
            state_root: "0".repeat(64),
            hash: "0".repeat(64),
        }
    }

    /// Add transaction to pending pool
    pub fn add_transaction(&mut self, tx: Transaction) -> Result<(), BlockchainError> {
        // Validate transaction
        self.validate_transaction(&tx)?;
        
        // Add to pending pool if not full
        if self.pending_transactions.len() < self.config.max_block_size {
            self.pending_transactions.push_back(tx);
            Ok(())
        } else {
            Err(BlockchainError::InvalidTransaction("Transaction pool full".to_string()))
        }
    }

    /// Validate a transaction
    fn validate_transaction(&self, tx: &Transaction) -> Result<(), BlockchainError> {
        match tx {
            Transaction::Transfer { from, to: _, amount, fee } => {
                let balance = self.state.token_ledger.balance(from);
                if balance < amount + fee {
                    return Err(BlockchainError::InvalidTransaction(
                        "Insufficient balance".to_string()
                    ));
                }
            }
            Transaction::Stake { validator, amount } => {
                let balance = self.state.token_ledger.balance(validator);
                if balance < *amount {
                    return Err(BlockchainError::InvalidTransaction(
                        "Insufficient balance for staking".to_string()
                    ));
                }
            }
            Transaction::JobPosting { poster, reward, .. } => {
                let balance = self.state.token_ledger.balance(poster);
                if balance < reward + self.config.job_posting_fee {
                    return Err(BlockchainError::InvalidTransaction(
                        "Insufficient balance for job posting".to_string()
                    ));
                }
            }
            Transaction::ValidationVote { validator, .. } => {
                if !self.validators.contains_key(validator) {
                    return Err(BlockchainError::InvalidTransaction(
                        "Validator not registered".to_string()
                    ));
                }
            }
            _ => {} // Other transactions validated during execution
        }
        Ok(())
    }

    /// Mine a new block (for validators)
    pub fn mine_block(&mut self, validator: &str) -> Result<Block, BlockchainError> {
        // Check if validator has sufficient stake
        let validator_info = self.validators.get(validator)
            .ok_or(BlockchainError::InsufficientStake)?;

        if validator_info.stake < self.config.min_validator_stake {
            return Err(BlockchainError::InsufficientStake);
        }

        // Create new block
        let previous_block = self.blocks.last().unwrap();
        let height = previous_block.height + 1;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Collect transactions for block
        let mut transactions = Vec::new();
        while !self.pending_transactions.is_empty() && transactions.len() < self.config.max_block_size {
            if let Some(tx) = self.pending_transactions.pop_front() {
                transactions.push(tx);
            }
        }

        // Generate PoUW task for this block
        let pouw_task = generate_task(4, height);
        
        // Solve PoUW (in production, this would be done by the validator)
        let pouw_solution = self.solve_pouw_for_block(&pouw_task)?;

        // Calculate merkle root of transactions
        let merkle_root = self.calculate_merkle_root(&transactions);

        // Apply transactions to get new state root
        let mut new_state = self.state.clone();
        for tx in &transactions {
            self.apply_transaction(&mut new_state, tx)?;
        }
        let state_root = self.calculate_state_root(&new_state);

        // Create block
        let mut block = Block {
            height,
            previous_hash: previous_block.hash.clone(),
            timestamp,
            transactions,
            validator: validator.to_string(),
            pouw_task,
            pouw_solution,
            merkle_root,
            state_root,
            hash: String::new(), // Will be calculated
        };

        // Calculate block hash
        block.hash = self.calculate_block_hash(&block);

        Ok(block)
    }

    /// Add a validated block to the chain
    pub fn add_block(&mut self, block: Block) -> Result<(), BlockchainError> {
        // Validate block
        self.validate_block(&block)?;

        // Apply transactions to state
        for tx in &block.transactions {
            self.apply_transaction(&mut self.state, tx)?;
        }

        // Add block to chain
        self.blocks.push(block);

        // Update difficulty if needed
        if self.blocks.len() % self.config.difficulty_adjustment_blocks as usize == 0 {
            self.adjust_difficulty();
        }

        Ok(())
    }

    /// Validate a block
    fn validate_block(&self, block: &Block) -> Result<(), BlockchainError> {
        // Check height is sequential
        let expected_height = self.blocks.last().unwrap().height + 1;
        if block.height != expected_height {
            return Err(BlockchainError::InvalidBlock("Invalid height".to_string()));
        }

        // Check previous hash
        let previous_hash = &self.blocks.last().unwrap().hash;
        if block.previous_hash != *previous_hash {
            return Err(BlockchainError::InvalidBlock("Invalid previous hash".to_string()));
        }

        // Validate PoUW solution
        if !verify(&block.pouw_task, &block.pouw_solution, self.current_difficulty) {
            return Err(BlockchainError::InvalidBlock("Invalid PoUW solution".to_string()));
        }

        // Validate validator has sufficient stake
        if let Some(validator_info) = self.validators.get(&block.validator) {
            if validator_info.stake < self.config.min_validator_stake {
                return Err(BlockchainError::InvalidBlock("Validator insufficient stake".to_string()));
            }
        } else {
            return Err(BlockchainError::InvalidBlock("Unknown validator".to_string()));
        }

        // Validate merkle root
        let calculated_merkle = self.calculate_merkle_root(&block.transactions);
        if block.merkle_root != calculated_merkle {
            return Err(BlockchainError::InvalidBlock("Invalid merkle root".to_string()));
        }

        // Validate block hash
        let calculated_hash = self.calculate_block_hash(block);
        if block.hash != calculated_hash {
            return Err(BlockchainError::InvalidBlock("Invalid block hash".to_string()));
        }

        Ok(())
    }

    /// Apply transaction to state
    fn apply_transaction(&self, state: &mut BlockchainState, tx: &Transaction) -> Result<(), BlockchainError> {
        match tx {
            Transaction::Transfer { from, to, amount, fee } => {
                state.token_ledger.transfer(from, to, *amount)
                    .map_err(|e| BlockchainError::InvalidTransaction(e.to_string()))?;
                // Fee goes to validator (simplified)
                state.token_ledger.mint("validator_pool", *fee)
                    .map_err(|e| BlockchainError::InvalidTransaction(e.to_string()))?;
            }
            
            Transaction::Stake { validator, amount } => {
                state.token_ledger.stake(validator, *amount)
                    .map_err(|e| BlockchainError::InvalidTransaction(e.to_string()))?;
                *state.validator_stakes.entry(validator.clone()).or_insert(0) += amount;
            }
            
            Transaction::JobPosting { poster, job_id, description, reward, requirements, data_hash, deadline } => {
                // Escrow the reward
                state.token_ledger.transfer(poster, "escrow", *reward)
                    .map_err(|e| BlockchainError::InvalidTransaction(e.to_string()))?;
                
                // Create job state
                let job_state = JobState {
                    job_id: *job_id,
                    poster: poster.clone(),
                    description: description.clone(),
                    reward: *reward,
                    requirements: requirements.clone(),
                    data_hash: data_hash.clone(),
                    deadline: *deadline,
                    assigned_workers: vec![],
                    submitted_results: HashMap::new(),
                    validator_votes: HashMap::new(),
                    status: JobStatus::Posted,
                };
                
                state.active_jobs.insert(*job_id, job_state);
            }
            
            Transaction::TrainingSubmission { worker, job_id, result_hash, pouw_solution, accuracy_claim } => {
                if let Some(job) = state.active_jobs.get_mut(job_id) {
                    let submission = TrainingSubmission {
                        worker: worker.clone(),
                        result_hash: result_hash.clone(),
                        pouw_solution: pouw_solution.clone(),
                        accuracy_claim: *accuracy_claim,
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    };
                    
                    job.submitted_results.insert(worker.clone(), submission);
                    job.status = JobStatus::ResultsSubmitted;
                }
            }
            
            Transaction::ValidationVote { validator, job_id, worker, is_valid, stake_weight } => {
                if let Some(job) = state.active_jobs.get_mut(job_id) {
                    let vote = ValidationVote {
                        validator: validator.clone(),
                        is_valid: *is_valid,
                        stake_weight: *stake_weight,
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    };
                    
                    job.validator_votes.insert(format!("{}_{}", validator, worker), vote);
                    
                    // Check if consensus reached
                    self.check_validation_consensus(state, *job_id)?;
                }
            }
            
            Transaction::RewardDistribution { job_id, workers, amounts } => {
                // Distribute rewards from escrow
                for (worker, amount) in workers.iter().zip(amounts.iter()) {
                    state.token_ledger.transfer("escrow", worker, *amount)
                        .map_err(|e| BlockchainError::InvalidTransaction(e.to_string()))?;
                }
                
                // Move job to completed
                if let Some(job) = state.active_jobs.remove(job_id) {
                    let completed_job = CompletedJob {
                        job_id: *job_id,
                        final_accuracy: 0.0, // Calculate from submissions
                        participating_workers: workers.clone(),
                        validator_consensus: true,
                        completion_block: self.blocks.len() as u64,
                    };
                    state.completed_jobs.insert(*job_id, completed_job);
                }
            }
        }
        Ok(())
    }

    /// Check if validation consensus is reached for a job
    fn check_validation_consensus(&self, state: &mut BlockchainState, job_id: u64) -> Result<(), BlockchainError> {
        if let Some(job) = state.active_jobs.get_mut(&job_id) {
            let total_stake: u64 = self.validators.values().map(|v| v.stake).sum();
            let threshold_stake = (total_stake as f64 * self.config.consensus_threshold) as u64;
            
            // Count stake for each worker's validation
            let mut worker_validations: HashMap<String, (u64, u64)> = HashMap::new(); // (positive_stake, negative_stake)
            
            for vote in job.validator_votes.values() {
                let entry = worker_validations.entry(vote.validator.clone()).or_insert((0, 0));
                if vote.is_valid {
                    entry.0 += vote.stake_weight;
                } else {
                    entry.1 += vote.stake_weight;
                }
            }
            
            // Check if any worker has reached consensus
            for (worker, (positive, negative)) in worker_validations {
                if positive >= threshold_stake {
                    job.status = JobStatus::Completed;
                    break;
                } else if negative >= threshold_stake {
                    job.status = JobStatus::Disputed;
                    break;
                }
            }
        }
        Ok(())
    }

    /// Register a new validator
    pub fn register_validator(&mut self, node_id: String, initial_stake: u64) -> Result<(), BlockchainError> {
        if initial_stake < self.config.min_validator_stake {
            return Err(BlockchainError::InsufficientStake);
        }

        let validator_info = ValidatorInfo {
            node_id: node_id.clone(),
            stake: initial_stake,
            reputation: 0,
            last_validation: 0,
            successful_validations: 0,
            failed_validations: 0,
        };

        self.validators.insert(node_id, validator_info);
        Ok(())
    }

    /// Get blockchain statistics
    pub fn get_stats(&self) -> BlockchainStats {
        BlockchainStats {
            block_height: self.blocks.len() as u64 - 1, // Exclude genesis
            total_transactions: self.blocks.iter().map(|b| b.transactions.len()).sum(),
            active_validators: self.validators.len(),
            total_stake: self.validators.values().map(|v| v.stake).sum(),
            active_jobs: self.state.active_jobs.len(),
            completed_jobs: self.state.completed_jobs.len(),
            current_difficulty: self.current_difficulty,
            pending_transactions: self.pending_transactions.len(),
        }
    }

    /// Get latest block
    pub fn latest_block(&self) -> &Block {
        self.blocks.last().unwrap()
    }

    /// Get block by height
    pub fn get_block(&self, height: u64) -> Option<&Block> {
        self.blocks.get(height as usize)
    }

    /// Calculate block hash
    fn calculate_block_hash(&self, block: &Block) -> String {
        let mut hasher = Sha256::new();
        hasher.update(block.height.to_le_bytes());
        hasher.update(block.previous_hash.as_bytes());
        hasher.update(block.timestamp.to_le_bytes());
        hasher.update(block.merkle_root.as_bytes());
        hasher.update(block.state_root.as_bytes());
        hasher.update(block.validator.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Calculate merkle root of transactions
    fn calculate_merkle_root(&self, transactions: &[Transaction]) -> String {
        if transactions.is_empty() {
            return "0".repeat(64);
        }

        let mut hashes: Vec<String> = transactions
            .iter()
            .map(|tx| {
                let serialized = serde_json::to_string(tx).unwrap();
                format!("{:x}", Sha256::digest(serialized.as_bytes()))
            })
            .collect();

        while hashes.len() > 1 {
            let mut new_hashes = Vec::new();
            for chunk in hashes.chunks(2) {
                let combined = if chunk.len() == 2 {
                    format!("{}{}", chunk[0], chunk[1])
                } else {
                    chunk[0].clone()
                };
                new_hashes.push(format!("{:x}", Sha256::digest(combined.as_bytes())));
            }
            hashes = new_hashes;
        }

        hashes[0].clone()
    }

    /// Calculate state root
    fn calculate_state_root(&self, state: &BlockchainState) -> String {
        // Simplified state root calculation
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}", state.validator_stakes).as_bytes());
        hasher.update(format!("{:?}", state.active_jobs.len()).as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Solve PoUW for block (simplified for demo)
    fn solve_pouw_for_block(&self, task: &Task) -> Result<Solution, BlockchainError> {
        // In production, this would be the actual PoUW solving
        // For demo, we create a valid solution
        Ok(crate::pouw::solve(task, self.current_difficulty))
    }

    /// Adjust difficulty based on block times
    fn adjust_difficulty(&mut self) {
        let recent_blocks = &self.blocks[self.blocks.len().saturating_sub(self.config.difficulty_adjustment_blocks as usize)..];
        
        if recent_blocks.len() >= 2 {
            let time_span = recent_blocks.last().unwrap().timestamp - recent_blocks.first().unwrap().timestamp;
            let target_time = self.config.block_time_secs * recent_blocks.len() as u64;
            
            self.current_difficulty = crate::pouw::calculate_adaptive_difficulty(
                self.current_difficulty,
                target_time,
                time_span,
            );
        }
    }
}

/// Blockchain statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainStats {
    pub block_height: u64,
    pub total_transactions: usize,
    pub active_validators: usize,
    pub total_stake: u64,
    pub active_jobs: usize,
    pub completed_jobs: usize,
    pub current_difficulty: u32,
    pub pending_transactions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blockchain_creation() {
        let config = BlockchainConfig::default();
        let blockchain = Blockchain::new(config);
        
        assert_eq!(blockchain.blocks.len(), 1); // Genesis block
        assert_eq!(blockchain.latest_block().height, 0);
    }

    #[test]
    fn validator_registration() {
        let mut blockchain = Blockchain::new(BlockchainConfig::default());
        
        let result = blockchain.register_validator("validator1".to_string(), 15000);
        assert!(result.is_ok());
        
        let insufficient_stake = blockchain.register_validator("validator2".to_string(), 5000);
        assert!(insufficient_stake.is_err());
    }

    #[test]
    fn transaction_validation() {
        let blockchain = Blockchain::new(BlockchainConfig::default());
        
        let invalid_tx = Transaction::Transfer {
            from: "alice".to_string(),
            to: "bob".to_string(),
            amount: 1000000, // More than possible balance
            fee: 10,
        };
        
        assert!(blockchain.validate_transaction(&invalid_tx).is_err());
    }

    #[test]
    fn block_mining() {
        let mut blockchain = Blockchain::new(BlockchainConfig::default());
        
        // Register validator
        blockchain.register_validator("validator1".to_string(), 15000).unwrap();
        
        // Mine block
        let block = blockchain.mine_block("validator1").unwrap();
        assert_eq!(block.height, 1);
        assert_eq!(block.validator, "validator1");
    }
} 