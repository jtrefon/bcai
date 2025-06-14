use crate::blockchain::{Blockchain, Block, Transaction, BlockchainError, BlockchainConfig};
use crate::pouw::{generate_task, solve};
use crate::node::NodeCapability;
use crate::neural_network::NeuralNetwork;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, oneshot};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConsensusError {
    #[error("Blockchain error: {0}")]
    Blockchain(#[from] BlockchainError),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Mining error: {0}")]
    Mining(String),
    #[error("Validation error: {0}")]
    Validation(String),
}

pub type ConsensusResult<T> = Result<T, ConsensusError>;

/// Message types for inter-node communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// New block announcement
    NewBlock(Block),
    /// Transaction broadcast
    NewTransaction(Transaction),
    /// Request for block by hash
    BlockRequest(String),
    /// Response with requested block
    BlockResponse(Option<Block>),
    /// Request for blockchain sync from height
    SyncRequest(u64),
    /// Response with multiple blocks for sync
    SyncResponse(Vec<Block>),
    /// Peer discovery and handshake
    Handshake { node_id: String, capabilities: Vec<NodeCapability> },
    /// Heartbeat to maintain connections
    Heartbeat,
}

/// Peer information for network management
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub node_id: String,
    pub capabilities: Vec<NodeCapability>,
    pub last_seen: u64,
    pub reputation: i32,
}

/// Mining status and statistics
#[derive(Debug, Clone)]
pub struct MiningStats {
    pub blocks_mined: u64,
    pub current_difficulty: u32,
    pub hash_rate: f64, // hashes per second
    pub is_mining: bool,
    pub last_block_time: u64,
}

/// Configuration for the consensus node
#[derive(Debug, Clone)]
pub struct ConsensusConfig {
    pub node_id: String,
    pub mining_enabled: bool,
    pub max_peers: usize,
    pub block_time_target: u64, // seconds
    pub max_transactions_per_block: usize,
    pub staking_enabled: bool,
    pub minimum_stake: u64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        ConsensusConfig {
            node_id: format!("node_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()),
            mining_enabled: true,
            max_peers: 50,
            block_time_target: 60, // 1 minute blocks
            max_transactions_per_block: 1000,
            staking_enabled: true,
            minimum_stake: 1000,
        }
    }
}

/// Comprehensive consensus-participating blockchain node
pub struct ConsensusNode {
    /// Node configuration
    config: ConsensusConfig,
    /// Blockchain state
    blockchain: Arc<Mutex<Blockchain>>,
    /// Connected peers
    peers: Arc<Mutex<HashMap<String, PeerInfo>>>,
    /// Mining statistics
    mining_stats: Arc<Mutex<MiningStats>>,
    /// Message channels for networking
    message_sender: mpsc::UnboundedSender<NetworkMessage>,
    /// Shutdown signal
    shutdown_tx: Option<oneshot::Sender<()>>,
    /// Node capabilities
    capabilities: Vec<NodeCapability>,
}

impl ConsensusNode {
    /// Create a new consensus node
    pub fn new(config: ConsensusConfig) -> ConsensusResult<Self> {
        let blockchain_config = BlockchainConfig::default();
        let blockchain = Arc::new(Mutex::new(Blockchain::new(blockchain_config)));
        let (message_sender, _message_receiver) = mpsc::unbounded_channel();
        
        let mining_stats = Arc::new(Mutex::new(MiningStats {
            blocks_mined: 0,
            current_difficulty: 0x0000ffff,
            hash_rate: 0.0,
            is_mining: false,
            last_block_time: 0,
        }));

        let capabilities = vec![
            NodeCapability {
                cpus: 4,
                gpus: 1,
                gpu_memory_gb: 8,
                available_stake: 1000,
                reputation: 100,
            }
        ];

        Ok(ConsensusNode {
            config,
            blockchain,
            peers: Arc::new(Mutex::new(HashMap::new())),
            mining_stats,
            message_sender,
            shutdown_tx: None,
            capabilities,
        })
    }

    /// Start the consensus node (mining, networking, etc.)
    pub async fn start(&mut self) -> ConsensusResult<()> {
        println!("🚀 Starting consensus node: {}", self.config.node_id);

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        // Start mining if enabled
        if self.config.mining_enabled {
            self.start_mining(shutdown_rx).await?;
        }

        // Start networking
        self.start_networking().await?;

        // Start transaction processing
        self.start_transaction_processor().await?;

        println!("✅ Consensus node started successfully");
        Ok(())
    }

    /// Start the mining process
    async fn start_mining(&self, mut shutdown_rx: oneshot::Receiver<()>) -> ConsensusResult<()> {
        let blockchain_clone = Arc::clone(&self.blockchain);
        let mining_stats_clone = Arc::clone(&self.mining_stats);
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut hash_count = 0u64;
            let mut last_hash_time = SystemTime::now();

            loop {
                // Check for shutdown signal
                if let Ok(()) = shutdown_rx.try_recv() {
                    break;
                }

                // Get current blockchain state
                let (tip_hash, difficulty, pending_transactions, current_height) = {
                    let mut blockchain = blockchain_clone.lock().unwrap();
                    let tip = blockchain.get_tip();
                    let tip_hash = tip.hash.clone();
                    let difficulty = blockchain.calculate_next_difficulty();
                    let pending = blockchain.get_pending_transactions(config.max_transactions_per_block);
                    let current_height = blockchain.get_stats().block_height;
                    (tip_hash, difficulty, pending, current_height)
                };

                // Update mining stats
                {
                    let mut stats = mining_stats_clone.lock().unwrap();
                    stats.is_mining = true;
                    stats.current_difficulty = difficulty;
                    
                    // Calculate hash rate
                    let now = SystemTime::now();
                    if let Ok(elapsed) = now.duration_since(last_hash_time) {
                        if elapsed.as_secs() >= 1 {
                            stats.hash_rate = hash_count as f64 / elapsed.as_secs_f64();
                            hash_count = 0;
                            last_hash_time = now;
                        }
                    }
                }

                // Generate PoUW task
                let task = generate_task(4, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
                
                // Try to solve the task
                let solution = solve(&task, difficulty);
                hash_count += 1;

                // Check if solution meets difficulty
                if crate::pouw::verify_production(&task, &solution, difficulty) {
                    // Create new block
                    let new_block = Block::new(
                        current_height + 1,
                        tip_hash,
                        pending_transactions,
                        difficulty,
                        config.node_id.clone(),
                        task,
                        solution,
                    );

                    // Add block to blockchain
                    match blockchain_clone.lock().unwrap().add_block(new_block.clone()) {
                        Ok(is_main_chain) => {
                            if is_main_chain {
                                let mut stats = mining_stats_clone.lock().unwrap();
                                stats.blocks_mined += 1;
                                stats.last_block_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                                
                                println!("⛏️  Mined new block #{} (difficulty: 0x{:08x})", 
                                         new_block.height, difficulty);
                                
                                // In a real implementation, we would broadcast this block to peers
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to add mined block: {}", e);
                        }
                    }
                }

                // Small delay to prevent busy waiting
                tokio::time::sleep(Duration::from_millis(10)).await;
            }

            // Update mining stats on shutdown
            let mut stats = mining_stats_clone.lock().unwrap();
            stats.is_mining = false;
            println!("⛏️  Mining stopped");
        });

        Ok(())
    }

    /// Start networking components
    async fn start_networking(&self) -> ConsensusResult<()> {
        // In a full implementation, this would:
        // 1. Start TCP/UDP listeners for peer connections
        // 2. Implement peer discovery mechanisms
        // 3. Handle message routing and validation
        // 4. Maintain peer reputation system
        
        println!("🌐 Network layer initialized (simplified mode)");
        Ok(())
    }

    /// Start transaction processing
    async fn start_transaction_processor(&self) -> ConsensusResult<()> {
        // In a full implementation, this would:
        // 1. Validate incoming transactions
        // 2. Manage mempool prioritization
        // 3. Handle transaction relay to peers
        // 4. Implement fee market mechanisms
        
        println!("📤 Transaction processor initialized");
        Ok(())
    }

    /// Submit a new transaction to the network
    pub fn submit_transaction(&self, transaction: Transaction) -> ConsensusResult<String> {
        let tx_hash = transaction.hash();
        
        // Add to blockchain mempool
        self.blockchain.lock().unwrap()
            .add_transaction(transaction.clone())?;

        // In a real implementation, broadcast to peers
        println!("📤 Transaction submitted: {}", &tx_hash[..8]);
        
        Ok(tx_hash)
    }

    /// Get blockchain statistics
    pub fn get_blockchain_stats(&self) -> crate::blockchain::BlockchainStats {
        self.blockchain.lock().unwrap().get_stats()
    }

    /// Get mining statistics
    pub fn get_mining_stats(&self) -> MiningStats {
        self.mining_stats.lock().unwrap().clone()
    }

    /// Get account balance
    pub fn get_balance(&self, account: &str) -> u64 {
        self.blockchain.lock().unwrap().get_balance(account)
    }

    /// Get account nonce
    pub fn get_nonce(&self, account: &str) -> u64 {
        self.blockchain.lock().unwrap().get_nonce(account)
    }

    /// Perform AI training and submit result as transaction
    pub async fn train_and_submit(&self, job_id: u64, data_samples: usize) -> ConsensusResult<String> {
        // Use the neural network for training
        let mut network = NeuralNetwork::new(&[4, 8, 2], 0.01);
        
        // Generate synthetic training data
        let mut inputs = Vec::new();
        let mut targets = Vec::new();
        
        for i in 0..data_samples {
            let input = vec![
                (i as f32 * 0.1) % 1.0,
                ((i + 1) as f32 * 0.1) % 1.0,
                ((i + 2) as f32 * 0.1) % 1.0,
                ((i + 3) as f32 * 0.1) % 1.0,
            ];
            let target = vec![
                if input[0] + input[1] > 1.0 { 1.0 } else { 0.0 },
                if input[2] + input[3] > 1.0 { 1.0 } else { 0.0 },
            ];
            inputs.push(input);
            targets.push(target);
        }

        // Train the network
        let epochs = 5;
        let training_dataset = crate::neural_network::TrainingData { 
            inputs,
            targets
        };
        let metrics = network.train(&training_dataset, epochs);
        
        let final_accuracy = if let Some(last_metric) = metrics.last() {
            last_metric.accuracy
        } else {
            0.0
        };

        println!("Training completed with final accuracy: {:.4}", final_accuracy);

        // Create proof of work for the training
        let task = generate_task(2, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
        let solution = solve(&task, 0x0000ffff);

        // Create AI training submission transaction
        let tx = Transaction::TrainingSubmission {
            worker: self.config.node_id.clone(),
            job_id,
            result_hash: format!("model_hash_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()),
            pouw_solution: solution,
            accuracy_claim: final_accuracy as f64,
        };

        // Submit the transaction
        self.submit_transaction(tx)
    }

    /// Create a token transfer transaction
    pub fn create_transfer(&self, to: &str, amount: u64) -> ConsensusResult<String> {
        let from = &self.config.node_id;

        let tx = Transaction::Transfer {
            from: from.clone(),
            to: to.to_string(),
            amount,
            fee: 1, // Small fee for mining
        };

        self.submit_transaction(tx)
    }

    /// Create a staking transaction
    pub fn create_stake(&self, amount: u64) -> ConsensusResult<String> {
        let validator = &self.config.node_id;

        let tx = Transaction::Stake {
            validator: validator.clone(),
            amount,
        };

        self.submit_transaction(tx)
    }

    /// Get recent blocks
    pub fn get_recent_blocks(&self, count: usize) -> Vec<Block> {
        let blockchain = self.blockchain.lock().unwrap();
        let stats = blockchain.get_stats();
        let mut blocks = Vec::new();
        
        let start_height = if stats.block_height >= count as u64 {
            stats.block_height - count as u64 + 1
        } else {
            0
        };

        // Get recent blocks starting from the calculated height
        for height in start_height..=stats.block_height {
            if let Some(block) = blockchain.get_block(height) {
                blocks.push(block.clone());
            }
        }

        blocks
    }

    /// Get network peer count
    pub fn get_peer_count(&self) -> usize {
        self.peers.lock().unwrap().len()
    }

    /// Stop the consensus node
    pub fn stop(&mut self) -> ConsensusResult<()> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
            println!("🛑 Consensus node stopping...");
        }
        Ok(())
    }
}

impl Drop for ConsensusNode {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Blockchain explorer functionality
pub struct BlockchainExplorer {
    blockchain: Arc<Mutex<Blockchain>>,
}

impl BlockchainExplorer {
    pub fn new(blockchain: Arc<Mutex<Blockchain>>) -> Self {
        BlockchainExplorer { blockchain }
    }

    /// Get detailed block information
    pub fn get_block_details(&self, height: u64) -> Option<BlockDetails> {
        let blockchain = self.blockchain.lock().unwrap();
        blockchain.get_block(height).map(|block| BlockDetails {
            hash: block.hash.clone(),
            height: block.height,
            timestamp: block.timestamp,
            validator: block.validator.clone(),
            transaction_count: block.transactions.len(),
            difficulty: 0x0000ffff, // Simplified - would get from blockchain state
            nonce: 0, // Simplified - would extract from PoUW solution
            previous_hash: block.previous_hash.clone(),
            merkle_root: block.merkle_root.clone(),
        })
    }

    /// Get transaction details
    pub fn get_transaction_details(&self, tx_hash: &str) -> Option<TransactionDetails> {
        let blockchain = self.blockchain.lock().unwrap();
        let stats = blockchain.get_stats();
        
        // Search through all blocks for the transaction
        // In a real implementation, we would have a transaction index
        for height in 0..=stats.block_height {
            if let Some(block) = blockchain.get_block(height) {
                for tx in &block.transactions {
                    if tx.hash() == tx_hash {
                        return Some(TransactionDetails {
                            hash: tx.hash(),
                            block_hash: block.hash.clone(),
                            block_height: block.height,
                            transaction_type: match tx {
                                Transaction::Transfer { .. } => "Transfer".to_string(),
                                Transaction::Stake { .. } => "Stake".to_string(),
                                Transaction::JobPosting { .. } => "JobPosting".to_string(),
                                Transaction::TrainingSubmission { .. } => "TrainingSubmission".to_string(),
                                Transaction::ValidationVote { .. } => "ValidationVote".to_string(),
                                Transaction::RewardDistribution { .. } => "RewardDistribution".to_string(),
                            },
                            from: tx.get_sender().to_string(),
                            nonce: tx.get_nonce(),
                        });
                    }
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDetails {
    pub hash: String,
    pub height: u64,
    pub timestamp: u64,
    pub validator: String,
    pub transaction_count: usize,
    pub difficulty: u32,
    pub nonce: u64,
    pub previous_hash: String,
    pub merkle_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDetails {
    pub hash: String,
    pub block_hash: String,
    pub block_height: u64,
    pub transaction_type: String,
    pub from: String,
    pub nonce: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_consensus_node_creation() {
        let config = ConsensusConfig::default();
        let node = ConsensusNode::new(config).unwrap();
        
        let stats = node.get_blockchain_stats();
        assert_eq!(stats.block_height, 0); // Genesis block
    }

    #[tokio::test]
    async fn test_transaction_submission() {
        let config = ConsensusConfig::default();
        let node = ConsensusNode::new(config).unwrap();

        // Give the node some initial balance for testing
        {
            let mut blockchain = node.blockchain.lock().unwrap();
            blockchain.credit_balance(&node.config.node_id, 1000).ok();
        }

        let tx_hash = node.create_transfer("alice", 100).unwrap();
        assert!(!tx_hash.is_empty());

        let stats = node.get_blockchain_stats();
        assert_eq!(stats.pending_transactions, 1);
    }

    #[test]
    fn test_blockchain_explorer() {
        let blockchain_config = BlockchainConfig::default();
        let blockchain = Arc::new(Mutex::new(Blockchain::new(blockchain_config)));
        let explorer = BlockchainExplorer::new(blockchain.clone());

        // Get genesis block details
        let details = explorer.get_block_details(0).unwrap();
        
        assert_eq!(details.height, 0);
        assert_eq!(details.transaction_count, 0);
    }
} 