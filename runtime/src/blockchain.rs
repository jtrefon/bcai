use crate::pouw::{Task, Solution};
use crate::token::TokenLedger;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use schnorrkel::{Signature, PublicKey, SecretKey, signing_context};
use hex;

const SIGNING_CONTEXT: &[u8] = b"bcai-transaction";

#[derive(Debug, Error)]
pub enum BlockchainError {
    #[error("Block validation failed: {0}")]
    BlockValidation(String),
    #[error("Transaction validation failed: {0}")]
    TransactionValidation(String),
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Invalid nonce")]
    InvalidNonce,
    #[error("Chain integrity error: {0}")]
    ChainIntegrity(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub max_block_size: usize,
    pub target_block_time: u64,
    pub difficulty_adjustment_window: u64,
    pub max_transactions_per_block: usize,
}

impl Default for BlockchainConfig {
    fn default() -> Self {
        Self {
            max_block_size: 1_000_000, // 1MB
            target_block_time: 60,     // 60 seconds
            difficulty_adjustment_window: 100, // blocks
            max_transactions_per_block: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub height: u64,
    pub timestamp: u64,
    pub previous_hash: String,
    pub merkle_root: String,
    pub transactions: Vec<Transaction>,
    pub pouw_task: Task,
    pub pouw_solution: Solution,
    pub difficulty: u32,
    pub validator: String,
    pub hash: String,
}

impl Block {
    pub fn new(
        height: u64,
        previous_hash: String,
        transactions: Vec<Transaction>,
        difficulty: u32,
        validator: String,
        pouw_task: Task,
        pouw_solution: Solution,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let merkle_root = Self::calculate_merkle_root(&transactions);
        let hash = Self::calculate_hash(height, timestamp, &previous_hash, &merkle_root, difficulty);
        
        Self {
            height,
            timestamp,
            previous_hash,
            merkle_root,
            transactions,
            pouw_task,
            pouw_solution,
            difficulty,
            validator,
            hash,
        }
    }
    
    fn calculate_merkle_root(transactions: &[Transaction]) -> String {
        if transactions.is_empty() {
            return "empty".to_string();
        }
        // Simplified Merkle root calculation
        format!("merkle_{}", transactions.len())
    }
    
    fn calculate_hash(height: u64, timestamp: u64, previous_hash: &str, merkle_root: &str, difficulty: u32) -> String {
        let prev_hash_short = if previous_hash.len() >= 8 { &previous_hash[..8] } else { previous_hash };
        let merkle_short = if merkle_root.len() >= 8 { &merkle_root[..8] } else { merkle_root };
        format!("block_{}_{}_{}_{}_0x{:x}", height, timestamp, prev_hash_short, merkle_short, difficulty)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Transaction {
    Transfer {
        signer: PublicKey,
        to: PublicKey,
        amount: u64,
        fee: u64,
        nonce: u64,
        signature: Signature,
    },
    Stake { 
        validator: String, 
        amount: u64, 
        nonce: u64 
    },
    JobPosting { 
        poster: String, 
        job_spec: String, 
        reward: u64, 
        nonce: u64 
    },
    TrainingSubmission { 
        worker: String, 
        job_id: u64, 
        result_hash: String, 
        pouw_solution: Solution,
        accuracy_claim: f64,
        nonce: u64 
    },
    ValidationVote { 
        validator: String, 
        job_id: u64, 
        vote: bool, 
        nonce: u64 
    },
    RewardDistribution { 
        job_id: u64, 
        recipients: Vec<(String, u64)>, 
        nonce: u64 
    },
}

impl Transaction {
    pub fn new_transfer(
        from_secret: &SecretKey,
        to: PublicKey,
        amount: u64,
        fee: u64,
        nonce: u64,
    ) -> Self {
        let signer = from_secret.public_key();
        let mut tx = Self::Transfer {
            signer,
            to,
            amount,
            fee,
            nonce,
            // Placeholder signature
            signature: from_secret.sign(signing_context(SIGNING_CONTEXT).bytes(b"placeholder")),
        };

        let message = tx.to_signable_bytes();
        let signature = from_secret.sign(signing_context(SIGNING_CONTEXT).bytes(&message));
        
        // Replace placeholder with the real signature
        if let Self::Transfer { signature: s, .. } = &mut tx {
            *s = signature;
        }
        tx
    }

    pub fn to_signable_bytes(&self) -> Vec<u8> {
        match self {
            Self::Transfer { signer, to, amount, fee, nonce, .. } => {
                let mut bytes = Vec::new();
                bytes.extend_from_slice(b"transfer");
                bytes.extend_from_slice(&signer.to_bytes());
                bytes.extend_from_slice(&to.to_bytes());
                bytes.extend_from_slice(&amount.to_le_bytes());
                bytes.extend_from_slice(&fee.to_le_bytes());
                bytes.extend_from_slice(&nonce.to_le_bytes());
                bytes
            }
        }
    }

    pub fn verify_signature(&self) -> bool {
        match self {
            Self::Transfer { signer, signature, .. } => {
                let message = self.to_signable_bytes();
                signer.verify(signing_context(SIGNING_CONTEXT).bytes(&message), signature).is_ok()
            }
        }
    }
    
    pub fn hash(&self) -> String {
        match self {
            Transaction::Transfer { signer, to, amount, nonce, .. } => {
                format!("tx_transfer_{:?}_{:?}_{}_{}", signer, to, amount, nonce)
            }
            Transaction::Stake { validator, amount, nonce } => {
                format!("tx_stake_{}_{}_{}",validator, amount, nonce)
            }
            Transaction::JobPosting { poster, reward, nonce, .. } => {
                format!("tx_job_{}_{}_{}",poster, reward, nonce)
            }
            Transaction::TrainingSubmission { worker, job_id, nonce, .. } => {
                format!("tx_training_{}_{}_{}",worker, job_id, nonce)
            }
            Transaction::ValidationVote { validator, job_id, nonce, .. } => {
                format!("tx_vote_{}_{}_{}",validator, job_id, nonce)
            }
            Transaction::RewardDistribution { job_id, nonce, .. } => {
                format!("tx_reward_{}_{}", job_id, nonce)
            }
        }
    }
    
    pub fn get_sender_pubkey(&self) -> &PublicKey {
        match self {
            Transaction::Transfer { signer, .. } => signer,
        }
    }

    // get_sender now returns a string representation for compatibility with state map keys
    pub fn get_sender(&self) -> String {
        hex::encode(self.get_sender_pubkey().to_bytes())
    }
    
    pub fn get_nonce(&self) -> u64 {
        match self {
            Transaction::Transfer { nonce, .. } => *nonce,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainStats {
    pub block_height: u64,
    pub total_transactions: u64,
    pub pending_transactions: usize,
    pub total_supply: u64,
    pub difficulty: u32,
    pub hash_rate: f64,
}

pub struct Blockchain {
    blocks: Vec<Block>,
    pending_transactions: Vec<Transaction>,
    state: TokenLedger,
    transaction_index: HashMap<String, (u64, usize)>, // hash -> (block_height, tx_index)
    account_nonces: HashMap<String, u64>,
    config: BlockchainConfig,
}

impl Blockchain {
    pub fn new(config: BlockchainConfig) -> Self {
        let mut blockchain = Self {
            blocks: Vec::new(),
            pending_transactions: Vec::new(),
            state: TokenLedger::new(),
            transaction_index: HashMap::new(),
            account_nonces: HashMap::new(),
            config,
        };
        
        // Create genesis block
        blockchain.create_genesis_block();
        blockchain
    }
    
    fn create_genesis_block(&mut self) {
        let genesis_task = Task {
            difficulty: 1,
            data: vec![0; 4],
            target: "genesis".to_string(),
            a: vec![],
            b: vec![],
            timestamp: 0,
            challenge: vec![0; 4],
        };
        
        let genesis_solution = Solution {
            nonce: 0,
            result: vec![],
            computation_time: 0,
        };
        
        let genesis_block = Block::new(
            0,
            "00000000".to_string(), // 8-character genesis hash
            vec![],
            0x0000ffff,
            "genesis".to_string(),
            genesis_task,
            genesis_solution,
        );
        
        self.blocks.push(genesis_block);
    }
    
    pub fn get_tip(&self) -> &Block {
        self.blocks.last().expect("Blockchain should have at least genesis block")
    }
    
    pub fn add_block(&mut self, block: Block) -> Result<bool, BlockchainError> {
        // Basic validation
        if block.height != self.blocks.len() as u64 {
            return Err(BlockchainError::BlockValidation("Invalid block height".to_string()));
        }
        
        // Validate previous hash
        if block.previous_hash != self.get_tip().hash {
            return Err(BlockchainError::ChainIntegrity("Invalid previous hash".to_string()));
        }
        
        // Apply transactions
        for tx in &block.transactions {
            self.apply_transaction(tx)?;
        }
        
        // Index transactions
        for (i, tx) in block.transactions.iter().enumerate() {
            self.transaction_index.insert(tx.hash(), (block.height, i));
        }
        
        self.blocks.push(block);
        Ok(true)
    }
    
    pub fn add_transaction(&mut self, tx: Transaction) -> Result<(), BlockchainError> {
        // Validate nonce
        let sender = tx.get_sender();
        let expected_nonce = self.account_nonces.get(sender).unwrap_or(&0) + 1;
        if tx.get_nonce() != expected_nonce {
            return Err(BlockchainError::InvalidNonce);
        }
        
        // Basic validation (signature verification would go here)
        self.validate_transaction(&tx)?;
        
        self.pending_transactions.push(tx);
        Ok(())
    }
    
    fn validate_transaction(&self, tx: &Transaction) -> Result<(), BlockchainError> {
        if !tx.verify_signature() {
            return Err(BlockchainError::TransactionValidation("Invalid signature".to_string()));
        }

        let sender_address = tx.get_sender();
        let nonce = self.get_nonce(&sender_address);
        if tx.get_nonce() <= nonce {
            return Err(BlockchainError::InvalidNonce);
        }

        if let Transaction::Transfer { amount, fee, .. } = tx {
            let balance = self.get_balance(&sender_address);
            if balance < amount + fee {
                return Err(BlockchainError::InsufficientBalance);
            }
        }
        
        Ok(())
    }
    
    fn apply_transaction(&mut self, tx: &Transaction) -> Result<(), BlockchainError> {
        let sender_address = tx.get_sender();
        self.account_nonces.insert(sender_address.clone(), tx.get_nonce());

        if let Transaction::Transfer { to, amount, fee, .. } = tx {
            self.state.debit(&sender_address, amount + fee)?;
            self.state.credit(&hex::encode(to.to_bytes()), *amount)?;
        }

        Ok(())
    }
    
    pub fn get_pending_transactions(&self, limit: usize) -> Vec<Transaction> {
        self.pending_transactions.iter()
            .take(limit)
            .cloned()
            .collect()
    }
    
    pub fn calculate_next_difficulty(&self) -> u32 {
        // Simplified difficulty adjustment
        if self.blocks.len() < 2 {
            return 0x0000ffff;
        }
        
        // Get last few blocks to calculate average time
        let recent_blocks = if self.blocks.len() >= 10 {
            &self.blocks[self.blocks.len()-10..]
        } else {
            &self.blocks[1..] // Skip genesis
        };
        
        if recent_blocks.len() < 2 {
            return 0x0000ffff;
        }
        
        let time_span = recent_blocks.last().unwrap().timestamp - recent_blocks.first().unwrap().timestamp;
        let avg_block_time = time_span / (recent_blocks.len() - 1) as u64;
        
        let current_difficulty = self.get_tip().difficulty;
        
        // Adjust difficulty based on target block time
        if avg_block_time < self.config.target_block_time {
            // Increase difficulty (lower target)
            current_difficulty / 2
        } else if avg_block_time > self.config.target_block_time * 2 {
            // Decrease difficulty (higher target)
            std::cmp::min(current_difficulty * 2, 0x00ffffff)
        } else {
            current_difficulty
        }
    }
    
    pub fn get_balance(&self, account: &str) -> u64 {
        self.state.balance(account)
    }
    
    pub fn get_nonce(&self, account: &str) -> u64 {
        *self.account_nonces.get(account).unwrap_or(&0)
    }
    
    pub fn credit_balance(&mut self, account: &str, amount: u64) -> Result<(), BlockchainError> {
        self.state.mint(account, amount)
            .map_err(|e| BlockchainError::TransactionValidation(e.to_string()))
    }
    
    pub fn get_stats(&self) -> BlockchainStats {
        let total_transactions = self.blocks.iter()
            .map(|b| b.transactions.len())
            .sum::<usize>() as u64;
            
        BlockchainStats {
            block_height: self.blocks.len() as u64 - 1, // Exclude genesis
            total_transactions,
            pending_transactions: self.pending_transactions.len(),
            total_supply: 0, // TODO: Calculate from state
            difficulty: self.get_tip().difficulty,
            hash_rate: 0.0, // TODO: Calculate from recent blocks
        }
    }
    
    pub fn get_block(&self, height: u64) -> Option<&Block> {
        self.blocks.get(height as usize)
    }
} 