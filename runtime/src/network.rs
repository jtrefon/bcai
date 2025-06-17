//! Network integration layer bridging P2P communication with unified node architecture
//!
//! This module provides the glue between libp2p networking and BCAI's distributed training system.

use crate::node::{DistributedJob, NodeCapability, TrainingResult, UnifiedNode};
use crate::token::LedgerError;
use crate::federated::{FederatedEngine, ModelParameters, FederatedStats};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use crate::blockchain::{Blockchain, BlockchainConfig, BlockchainError, Transaction, Block};
use crate::wire::WireMessage;
use std::sync::{Arc, Mutex};
use crate::pouw::PoUWTask;
use futures::stream::StreamExt;
use libp2p::gossipsub::IdentTopic as Topic;
use libp2p::swarm::SwarmEvent;
use libp2p::kad::Quorum;
use libp2p::{kad, Swarm};
use log::{error, info};
use std::collections::HashSet;
use tokio::sync::mpsc;
use crate::p2p_service::{GOSSIP_TOPIC, P2PService};

/// Network message types for distributed coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    Ping,
    Pong,
    Data(Vec<u8>),
    Request(String),
    Response(String),
    CapabilityAnnouncement { node_id: String, capability: NodeCapability },
    JobPosted { job: DistributedJob, poster_id: String },
    JobVolunteer { job_id: u64, node_id: String, capability: NodeCapability },
    TrainingResultSubmission { result: TrainingResult, submitter_id: String },
    TrainingEvaluation { job_id: u64, result_hash: String, is_valid: bool, evaluator_id: String },
    JobCompleted { job_id: u64, final_model_hash: String },
    StateSync { requesting_node: String, last_known_block: u64 },
    StateSyncResponse { jobs: Vec<DistributedJob>, current_block: u64 },
    
    // NEW: Federated Learning Messages
    FederatedTrainingStart { 
        job_id: u64, 
        initial_model: ModelParameters,
        participants: Vec<String>,
        coordinator_id: String 
    },
    FederatedModelUpdate { 
        job_id: u64, 
        round: u32,
        local_model: ModelParameters, 
        node_id: String 
    },
    FederatedAggregationResult { 
        job_id: u64, 
        round: u32,
        global_model: ModelParameters,
        stats: FederatedStats,
        coordinator_id: String 
    },
    FederatedTrainingComplete { 
        job_id: u64, 
        final_model: ModelParameters,
        participants_rewards: HashMap<String, u64> 
    },
    LargeDataShardDistribution {
        job_id: u64,
        shard_hash: String,
        target_nodes: Vec<String>,
        shard_size_bytes: u64
    }
}

/// Network-related errors
#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("P2P communication error: {0}")]
    P2pError(String),
    #[error("Failed to serialize message: {0}")]
    SerializationError(String),
    #[error("Blockchain operation failed: {0}")]
    BlockchainError(#[from] BlockchainError),
    #[error("Invalid message received")]
    InvalidMessage,
}

/// The NetworkCoordinator acts as the bridge between the P2P network and the
/// blockchain's state. It listens for messages from the network and applies
/// them to the blockchain, and it can also broadcast messages to the network.
pub struct NetworkCoordinator {
    pub p2p: P2PService,
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub mempool: Arc<Mutex<Vec<Transaction>>>,
    pub rx: mpsc::Receiver<Vec<u8>>,
}

impl NetworkCoordinator {
    /// Create a new network coordinator
    pub async fn new(
        blockchain: Arc<Mutex<Blockchain>>,
        mempool: Arc<Mutex<Vec<Transaction>>>,
    ) -> Self {
        let (tx, rx) = mpsc::channel(100);
        let p2p = P2PService::new(tx).await;
        Self {
            p2p,
            blockchain,
            mempool,
            rx,
        }
    }

    /// The main event loop for the network coordinator.
    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                // Handle messages received from the P2P network
                Some(message) = self.rx.recv() => {
                    if let Ok(wire_message) = bincode::deserialize::<WireMessage>(&message) {
                        if let Err(e) = self.handle_wire_message(wire_message) {
                            error!("Error handling wire message: {}", e);
                        }
                    } else {
                        error!("Failed to deserialize wire message");
                    }
                },
                // Handle direct swarm events
                event = self.p2p.swarm.select_next_some() => {
                    info!("P2P Swarm Event: {:?}", event);
                }
            }
        }
    }

    /// Broadcasts a `WireMessage` to all peers on the gossip topic.
    pub async fn broadcast(&mut self, message: WireMessage) {
         if let Err(e) = self
            .p2p
            .gossipsub_publish(GOSSIP_TOPIC, message)
            .await
        {
            error!("Error broadcasting message: {:?}", e);
        }
    }

    /// Handles an incoming message from the P2P network.
    fn handle_wire_message(&mut self, message: WireMessage) -> Result<(), NetworkError> {
        match message {
            WireMessage::Block(block) => {
                info!("Received new block via gossip: {}", block.hash);
                let included_tx_hashes: HashSet<String> =
                    block.transactions.iter().map(|tx| tx.hash()).collect();

                let mut bc = self.blockchain.lock().unwrap();
                bc.add_block(block)?;

                // Prune mempool
                let mut mempool = self.mempool.lock().unwrap();
                mempool.retain(|tx| !included_tx_hashes.contains(&tx.hash()));
                info!(
                    "Accepted new block. Mempool pruned. Size: {}",
                    mempool.len()
                );
            }
            WireMessage::Transaction(tx) => {
                info!("Received new transaction via gossip: {}", tx.hash());

                // Validate transaction before adding to mempool
                let bc = self.blockchain.lock().unwrap();
                bc.validate_transaction(&tx)?;

                let mut mempool = self.mempool.lock().unwrap();
                if !mempool.iter().any(|mempool_tx| mempool_tx == &tx) {
                    mempool.push(tx);
                    info!(
                        "Added new transaction to mempool. Size: {}",
                        mempool.len()
                    );
                }
            }
        }
        Ok(())
    }

    /// Handle incoming network message
    pub fn handle_message(
        &mut self,
        message: NetworkMessage,
        _sender_id: &str,
    ) -> Result<Vec<NetworkMessage>, NetworkError> {
        let mut responses = Vec::new();

        match message {
            NetworkMessage::CapabilityAnnouncement { node_id, capability } => {
                self.p2p.peer_capabilities.insert(node_id, capability);
            }

            NetworkMessage::JobPosted { job, poster_id: _ } => {
                // Add to global registry
                self.p2p.global_job_registry.insert(job.id, job.clone());

                // Check if local node can handle this job
                if self.can_handle_job(&job) {
                    responses.push(NetworkMessage::JobVolunteer {
                        job_id: job.id,
                        node_id: self.p2p.local_node.node_id.clone(),
                        capability: self.p2p.local_node.capability.clone(),
                    });
                }
            }

            NetworkMessage::JobVolunteer { job_id, node_id, capability: _ } => {
                if let Some(job) = self.p2p.global_job_registry.get_mut(&job_id) {
                    if job.assigned_workers.len() < 3 && !job.assigned_workers.contains(&node_id) {
                        job.assigned_workers.push(node_id);

                        // If we have enough workers, start training
                        if job.assigned_workers.len() >= 3 {
                            job.status = crate::node::JobStatus::WorkersAssigned;
                        }
                    }
                }
            }

            NetworkMessage::TrainingResultSubmission { result, submitter_id: _ } => {
                // Store result for evaluation
                if let Ok(is_valid) =
                    self.p2p.local_node.evaluate_training_result(result.job_id, &result)
                {
                    responses.push(NetworkMessage::TrainingEvaluation {
                        job_id: result.job_id,
                        result_hash: result.model_hash.clone(),
                        is_valid,
                        evaluator_id: self.p2p.local_node.node_id.clone(),
                    });
                }
            }

            NetworkMessage::TrainingEvaluation {
                job_id,
                result_hash: _,
                is_valid,
                evaluator_id,
            } => {
                // Collect evaluations
                let evaluations = self.p2p.pending_evaluations.entry(job_id).or_default();
                evaluations.push((evaluator_id, is_valid));

                // Check if we have consensus (majority agreement)
                if evaluations.len() >= 2 {
                    let valid_count = evaluations.iter().filter(|(_, valid)| *valid).count();
                    let total_count = evaluations.len();

                    if valid_count * 2 > total_count {
                        // Majority agrees it's valid - complete the job
                        if let Ok(()) = self.p2p.local_node.complete_distributed_job(job_id) {
                            if self.p2p.global_job_registry.contains_key(&job_id) {
                                responses.push(NetworkMessage::JobCompleted {
                                    job_id,
                                    final_model_hash: format!("final_model_{}", job_id),
                                });
                            }
                        }
                        self.p2p.pending_evaluations.remove(&job_id);
                    }
                }
            }

            NetworkMessage::JobCompleted { job_id, final_model_hash: _ } => {
                if let Some(job) = self.p2p.global_job_registry.get_mut(&job_id) {
                    job.status = crate::node::JobStatus::Completed;
                }
            }

            NetworkMessage::StateSync { requesting_node: _, last_known_block } => {
                let jobs_to_sync: Vec<DistributedJob> = self
                    .p2p.global_job_registry
                    .values()
                    .filter(|job| job.created_block > last_known_block)
                    .cloned()
                    .collect();

                responses.push(NetworkMessage::StateSyncResponse {
                    jobs: jobs_to_sync,
                    current_block: self.p2p.network_block_height,
                });
            }

            NetworkMessage::StateSyncResponse { jobs, current_block } => {
                // Update network state
                for job in jobs {
                    self.p2p.global_job_registry.insert(job.id, job);
                }
                self.p2p.network_block_height = current_block;
            }

            _ => {} // Handle other basic message types
        }

        Ok(responses)
    }

    /// Check if local node can handle a job
    fn can_handle_job(&self, job: &DistributedJob) -> bool {
        self.p2p.local_node.capability.cpus >= job.required_capability.cpus
            && self.p2p.local_node.capability.gpus >= job.required_capability.gpus
            && self.p2p.local_node.capability.gpu_memory_gb >= job.required_capability.gpu_memory_gb
            && self.p2p.local_node.staked() >= job.required_capability.available_stake
    }

    /// Post a job to the network
    pub fn post_job_to_network(
        &mut self,
        description: String,
        reward: u64,
        required_capability: NodeCapability,
        data_hash: String,
        model_spec: String,
        deadline_blocks: u64,
    ) -> Result<NetworkMessage, crate::node::NodeError> {
        let job_id = self.p2p.local_node.post_distributed_job(
            description,
            reward,
            required_capability,
            data_hash,
            model_spec,
            deadline_blocks,
        )?;

        let job = self.p2p.local_node.distributed_jobs().get(&job_id).unwrap().clone();
        self.p2p.global_job_registry.insert(job_id, job.clone());

        Ok(NetworkMessage::JobPosted { job, poster_id: self.p2p.local_node.node_id.clone() })
    }

    /// Execute training for a job and broadcast results
    pub fn execute_training_and_broadcast(
        &mut self,
        job_id: u64,
    ) -> Result<NetworkMessage, crate::node::NodeError> {
        let result = self.p2p.local_node.execute_training(job_id, 0x0000ffff)?;

        Ok(NetworkMessage::TrainingResultSubmission {
            result,
            submitter_id: self.p2p.local_node.node_id.clone(),
        })
    }

    /// Get network statistics
    pub fn get_network_stats(&self) -> NetworkStats {
        let bc = self.blockchain.lock().unwrap();
        let mempool = self.mempool.lock().unwrap();
        NetworkStats {
            connected_peers: self.p2p.peer_capabilities.len(),
            active_jobs: self
                .p2p.global_job_registry
                .values()
                .filter(|job| {
                    !matches!(
                        job.status,
                        crate::node::JobStatus::Completed | crate::node::JobStatus::Failed
                    )
                })
                .count(),
            completed_jobs: self
                .p2p.global_job_registry
                .values()
                .filter(|job| job.status == crate::node::JobStatus::Completed)
                .count(),
            network_block_height: bc.get_tip().height,
            pending_transactions: mempool.len(),
            local_node_stats: self.p2p.local_node.get_stats(),
        }
    }

    /// Synchronize with network state
    pub fn request_state_sync(&self) -> NetworkMessage {
        NetworkMessage::StateSync {
            requesting_node: self.p2p.local_node.node_id.clone(),
            last_known_block: self.p2p.network_block_height,
        }
    }

    /// Announce capabilities to network
    pub fn announce_capabilities(&self) -> NetworkMessage {
        NetworkMessage::CapabilityAnnouncement {
            node_id: self.p2p.local_node.node_id.clone(),
            capability: self.p2p.local_node.capability.clone(),
        }
    }

    /// Get mutable reference to local node
    pub fn local_node_mut(&mut self) -> &mut UnifiedNode {
        &mut self.p2p.local_node
    }

    /// Get reference to local node
    pub fn local_node(&self) -> &UnifiedNode {
        &self.p2p.local_node
    }

    /// Get global job registry
    pub fn global_jobs(&self) -> &HashMap<u64, DistributedJob> {
        &self.p2p.global_job_registry
    }

    /// Get mutable reference to global job registry (for testing)
    pub fn global_jobs_mut(&mut self) -> &mut HashMap<u64, DistributedJob> {
        &mut self.p2p.global_job_registry
    }
}

/// Network statistics for monitoring
#[derive(Debug, Default, Clone)]
pub struct NetworkStats {
    pub connected_peers: usize,
    pub active_jobs: usize,
    pub completed_jobs: usize,
    pub network_block_height: u64,
    pub pending_transactions: usize,
    pub local_node_stats: crate::node::NodeStats,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeCapability;

    #[test]
    fn network_coordinator_creation() {
        let capability = NodeCapability {
            cpus: 4,
            gpus: 1,
            gpu_memory_gb: 8,
            available_stake: 0,
            reputation: 0,
            capability_types: vec![crate::node::CapabilityType::BasicCompute],
        };

        let node = UnifiedNode::new("test_node".to_string(), capability, 1000);
        let coordinator = NetworkCoordinator::new(Arc::new(Mutex::new(Blockchain::new(BlockchainConfig::default()))), Arc::new(Mutex::new(Vec::new()))).await;

        assert_eq!(coordinator.p2p.peer_capabilities.len(), 0);
        assert_eq!(coordinator.p2p.global_job_registry.len(), 0);
    }

    #[test]
    fn job_posting_and_volunteering() -> Result<(), crate::node::NodeError> {
        let capability = NodeCapability {
            cpus: 4,
            gpus: 1,
            gpu_memory_gb: 8,
            available_stake: 100,
            reputation: 0,
            capability_types: vec![crate::node::CapabilityType::BasicCompute],
        };

        // Create job poster
        let poster_node = UnifiedNode::new("poster".to_string(), capability.clone(), 1000);
        let mut poster_coordinator = NetworkCoordinator::new(Arc::new(Mutex::new(Blockchain::new(BlockchainConfig::default()))), Arc::new(Mutex::new(Vec::new()))).await;

        // Create worker
        let worker_node = UnifiedNode::new("worker".to_string(), capability.clone(), 1000);
        let mut worker_coordinator = NetworkCoordinator::new(Arc::new(Mutex::new(Blockchain::new(BlockchainConfig::default()))), Arc::new(Mutex::new(Vec::new()))).await;
        worker_coordinator.local_node_mut().stake_tokens(150)?;

        // Post job
        let job_message = poster_coordinator.post_job_to_network(
            "Test distributed training".to_string(),
            500,
            capability,
            "data_hash".to_string(),
            "model_spec".to_string(),
            100,
        )?;

        // Worker receives job posting
        let responses = worker_coordinator.handle_message(job_message, "poster")?;
        assert_eq!(responses.len(), 1);

        // Check that worker volunteered
        if let NetworkMessage::JobVolunteer { job_id, node_id, capability: _ } = &responses[0] {
            assert_eq!(*job_id, 1);
            assert_eq!(node_id, "worker");
        } else {
            panic!("Expected JobVolunteer message");
        }

        Ok(())
    }
}
