//! Network integration layer bridging P2P communication with unified node architecture
//!
//! This module provides the glue between libp2p networking and BCAI's distributed training system.

use crate::node::{DistributedJob, NodeCapability, TrainingResult};
use crate::federated::{ModelParameters, FederatedStats};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use crate::blockchain::{BlockchainError, Transaction, Block};
use crate::pouw::types::SignedEvaluation;

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
    /// Gossip message carrying a signed PoUW evaluation.
    PoUWEvaluation { evaluation: SignedEvaluation },
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub connected_peers: usize,
    pub active_jobs: usize,
    pub completed_jobs: usize,
    pub network_block_height: u64,
    pub pending_transactions: usize,
    pub local_node_stats: crate::node::NodeStats,
}
