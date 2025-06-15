//! Federated Network Coordinator
//!
//! This module coordinates federated learning across the P2P network,
//! integrating FederatedEngine with large data transfer and economic incentives.

use crate::{
    federated::{FederatedEngine, FederatedConfig, ModelParameters, FederatedStats, FederatedError},
    large_data_transfer::{NetworkTransferCoordinator, LargeDataDescriptor, ChunkId},
    network::{NetworkMessage, NetworkCoordinator},
    node::{UnifiedNode, NodeCapability, DistributedJob},
    token::TokenLedger,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use thiserror::Error;

/// Federated network coordination errors
#[derive(Debug, Error)]
pub enum FederatedNetworkError {
    #[error("Federated learning error: {0}")]
    Federated(#[from] FederatedError),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Large data transfer error: {0}")]
    LargeDataTransfer(String),
    #[error("Insufficient participants for job {job_id}: need {required}, have {available}")]
    InsufficientParticipants { job_id: u64, required: usize, available: usize },
    #[error("Job not found: {0}")]
    JobNotFound(u64),
    #[error("Invalid training round: expected {expected}, got {actual}")]
    InvalidRound { expected: u32, actual: u32 },
}

/// Configuration for federated network training
#[derive(Debug, Clone)]
pub struct FederatedNetworkConfig {
    pub federated_config: FederatedConfig,
    pub chunk_size_mb: u32,
    pub max_data_size_gb: u64,
    pub reward_per_participant: u64,
    pub coordinator_fee_percent: f32,
    pub min_reputation: i32,
}

impl Default for FederatedNetworkConfig {
    fn default() -> Self {
        Self {
            federated_config: FederatedConfig::default(),
            chunk_size_mb: 4, // 4MB chunks for LLM data
            max_data_size_gb: 5000, // Support up to 5TB
            reward_per_participant: 1000, // BCAI tokens
            coordinator_fee_percent: 5.0, // 5% coordinator fee
            min_reputation: 10, // Minimum reputation to participate
        }
    }
}

/// Federated training job with network integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedTrainingJob {
    pub job_id: u64,
    pub coordinator_node: String,
    pub participants: HashMap<String, ParticipantInfo>,
    pub data_descriptor: LargeDataDescriptor,
    pub model_architecture: ModelArchitecture,
    pub training_config: FederatedTrainingConfig,
    pub current_round: u32,
    pub status: FederatedJobStatus,
    pub total_reward: u64,
    pub created_at: u64,
    pub deadline: u64,
}

/// Participant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantInfo {
    pub node_id: String,
    pub capability: NodeCapability,
    pub assigned_shards: Vec<String>,
    pub contribution_weight: f32,
    pub reputation: i32,
    pub status: ParticipantStatus,
}

/// Model architecture specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelArchitecture {
    pub model_type: String, // "transformer", "bert", "gpt", etc.
    pub num_layers: u32,
    pub hidden_size: u32,
    pub num_attention_heads: u32,
    pub vocab_size: u32,
    pub max_sequence_length: u32,
    pub parameter_count: u64,
}

/// Federated training configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedTrainingConfig {
    pub local_epochs: u32,
    pub global_rounds: u32,
    pub learning_rate: f32,
    pub batch_size: u32,
    pub convergence_threshold: f32,
    pub max_training_time_hours: u32,
}

/// Job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FederatedJobStatus {
    DataDistribution,
    WaitingForParticipants,
    Training,
    Aggregating,
    Completed,
    Failed,
}

/// Participant status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParticipantStatus {
    DataDownloading,
    Ready,
    Training,
    ModelSubmitted,
    Completed,
    Failed,
}

/// Federated network coordinator
pub struct FederatedNetworkCoordinator {
    config: FederatedNetworkConfig,
    local_node: Arc<RwLock<UnifiedNode>>,
    network_coordinator: Arc<RwLock<NetworkCoordinator>>,
    large_data_coordinator: Option<Arc<NetworkTransferCoordinator>>,
    federated_engines: Arc<RwLock<HashMap<u64, FederatedEngine>>>,
    active_jobs: Arc<RwLock<HashMap<u64, FederatedTrainingJob>>>,
    message_queue: mpsc::UnboundedSender<NetworkMessage>,
}

impl FederatedNetworkCoordinator {
    /// Create new federated network coordinator
    pub fn new(
        config: FederatedNetworkConfig,
        local_node: Arc<RwLock<UnifiedNode>>,
        network_coordinator: Arc<RwLock<NetworkCoordinator>>,
        large_data_coordinator: Option<Arc<NetworkTransferCoordinator>>,
    ) -> Self {
        let (message_sender, _message_receiver) = mpsc::unbounded_channel();

        Self {
            config,
            local_node,
            network_coordinator,
            large_data_coordinator,
            federated_engines: Arc::new(RwLock::new(HashMap::new())),
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
            message_queue: message_sender,
        }
    }

    /// Start a federated LLM training job
    pub async fn start_federated_llm_training(
        &mut self,
        dataset_size_bytes: u64,
        model_architecture: ModelArchitecture,
        training_config: FederatedTrainingConfig,
        total_reward: u64,
        deadline_hours: u32,
    ) -> Result<u64, FederatedNetworkError> {
        let local_node = self.local_node.read().await;
        let job_id = rand::random::<u64>();

        // Calculate data sharding strategy
        let chunk_size = (self.config.chunk_size_mb * 1024 * 1024) as u64;
        let total_chunks = (dataset_size_bytes + chunk_size - 1) / chunk_size;
        let target_participants = std::cmp::min(
            self.config.federated_config.max_participants,
            (total_chunks / 100).max(3) as usize // At least 100 chunks per participant
        );

        println!("🚀 Starting federated LLM training:");
        println!("   • Job ID: {}", job_id);
        println!("   • Dataset: {:.2} GB", dataset_size_bytes as f64 / 1_073_741_824.0);
        println!("   • Model: {} ({} parameters)", model_architecture.model_type, model_architecture.parameter_count);
        println!("   • Target participants: {}", target_participants);
        println!("   • Total chunks: {}", total_chunks);
        println!("   • Reward pool: {} BCAI", total_reward);

        // Create data descriptor (simplified - in production would read actual data)
        let data_descriptor = self.create_data_descriptor(dataset_size_bytes, job_id).await?;

        // Create federated job
        let federated_job = FederatedTrainingJob {
            job_id,
            coordinator_node: local_node.node_id.clone(),
            participants: HashMap::new(),
            data_descriptor,
            model_architecture: model_architecture.clone(),
            training_config: training_config.clone(),
            current_round: 0,
            status: FederatedJobStatus::DataDistribution,
            total_reward,
            created_at: chrono::Utc::now().timestamp() as u64,
            deadline: chrono::Utc::now().timestamp() as u64 + (deadline_hours as u64 * 3600),
        };

        // Initialize federated engine
        let mut fed_config = self.config.federated_config.clone();
        fed_config.min_participants = (target_participants / 2).max(3); // At least half
        fed_config.max_participants = target_participants;
        fed_config.training_rounds = training_config.global_rounds;

        let fed_engine = FederatedEngine::new(fed_config);
        self.federated_engines.write().await.insert(job_id, fed_engine);
        self.active_jobs.write().await.insert(job_id, federated_job);

        // Broadcast job to network for participant recruitment
        self.broadcast_job_announcement(job_id).await?;

        Ok(job_id)
    }

    /// Handle incoming federated network messages
    pub async fn handle_message(
        &mut self,
        message: NetworkMessage,
        sender_id: &str,
    ) -> Result<Vec<NetworkMessage>, FederatedNetworkError> {
        let mut responses = Vec::new();

        match message {
            NetworkMessage::FederatedTrainingStart { job_id, initial_model, participants, coordinator_id } => {
                if coordinator_id != sender_id {
                    return Err(FederatedNetworkError::Network("Invalid coordinator".to_string()));
                }
                responses.extend(self.handle_training_start(job_id, initial_model, participants).await?);
            }

            NetworkMessage::FederatedModelUpdate { job_id, round, local_model, node_id } => {
                if node_id != sender_id {
                    return Err(FederatedNetworkError::Network("Invalid node ID".to_string()));
                }
                responses.extend(self.handle_model_update(job_id, round, local_model, node_id).await?);
            }

            NetworkMessage::LargeDataShardDistribution { job_id, shard_hash, target_nodes, shard_size_bytes } => {
                responses.extend(self.handle_shard_distribution(job_id, shard_hash, target_nodes, shard_size_bytes).await?);
            }

            _ => {} // Handle other message types as needed
        }

        Ok(responses)
    }

    /// Create data descriptor for large dataset
    async fn create_data_descriptor(
        &self,
        dataset_size_bytes: u64,
        job_id: u64,
    ) -> Result<LargeDataDescriptor, FederatedNetworkError> {
        // In production, this would create actual data chunks
        // For now, create a mock descriptor
        use crate::large_data_transfer::{CompressionAlgorithm, EncryptionAlgorithm, TransferMetadata, TransferPriority};

        let chunk_size = (self.config.chunk_size_mb * 1024 * 1024) as u64;
        let chunk_count = (dataset_size_bytes + chunk_size - 1) / chunk_size;

        let chunk_hashes: Vec<String> = (0..chunk_count)
            .map(|i| format!("chunk_{}_{:08x}", job_id, i))
            .collect();

        let content_hash = format!("dataset_{}", job_id);

        let metadata = TransferMetadata {
            name: format!("llm_training_dataset_{}", job_id),
            content_type: "application/octet-stream".to_string(),
            filename: Some(format!("dataset_{}.bin", job_id)),
            priority: TransferPriority::High,
            source_node: Some(self.local_node.read().await.node_id.clone()),
            target_nodes: Vec::new(), // Will be populated with participants
            timeout_seconds: Some(3600 * 24), // 24 hours
            ..Default::default()
        };

        Ok(LargeDataDescriptor {
            content_hash,
            total_size: dataset_size_bytes,
            chunk_count: chunk_count as u32,
            chunk_hashes,
            chunk_size: chunk_size as u32,
            compression: CompressionAlgorithm::Lz4,
            encryption: EncryptionAlgorithm::None,
            metadata,
            redundancy: Default::default(),
            merkle_root: format!("merkle_root_{}", job_id),
        })
    }

    /// Broadcast job announcement to recruit participants
    async fn broadcast_job_announcement(&self, job_id: u64) -> Result<(), FederatedNetworkError> {
        let job = self.active_jobs.read().await
            .get(&job_id)
            .ok_or(FederatedNetworkError::JobNotFound(job_id))?
            .clone();

        // Convert to DistributedJob for network broadcast
        let required_capability = NodeCapability {
            cpus: 8,
            gpus: 1,
            gpu_memory_gb: 16, // Minimum for LLM training
            available_stake: 5000, // Higher stake for large jobs
            reputation: self.config.min_reputation,
            capability_types: vec![
                crate::node::CapabilityType::GpuAccelerated,
                crate::node::CapabilityType::HighMemory,
                crate::node::CapabilityType::Training,
            ],
        };

        let distributed_job = DistributedJob {
            id: job_id,
            description: format!("Federated LLM Training: {} ({} parameters)", 
                               job.model_architecture.model_type, 
                               job.model_architecture.parameter_count),
            reward: job.total_reward,
            required_capability,
            data_hash: job.data_descriptor.content_hash.clone(),
            model_spec: serde_json::to_string(&job.model_architecture).unwrap_or_default(),
            assigned_workers: Vec::new(),
            evaluators: Vec::new(),
            status: crate::node::JobStatus::Posted,
            created_block: 1,
            completion_deadline: job.deadline,
        };

        let message = NetworkMessage::JobPosted {
            job: distributed_job,
            poster_id: self.local_node.read().await.node_id.clone(),
        };

        self.message_queue.send(message)
            .map_err(|e| FederatedNetworkError::Network(format!("Failed to broadcast job: {}", e)))?;

        Ok(())
    }

    /// Handle training start message
    async fn handle_training_start(
        &mut self,
        job_id: u64,
        initial_model: ModelParameters,
        participants: Vec<String>,
    ) -> Result<Vec<NetworkMessage>, FederatedNetworkError> {
        let mut fed_engines = self.federated_engines.write().await;
        let fed_engine = fed_engines.get_mut(&job_id)
            .ok_or(FederatedNetworkError::JobNotFound(job_id))?;

        // Initialize global model
        fed_engine.initialize_global_model(initial_model)?;

        // Start local training on this node if we're a participant
        let local_node_id = self.local_node.read().await.node_id.clone();
        if participants.contains(&local_node_id) {
            println!("🎯 Starting local training for job {} round 1", job_id);
            // In production, this would start actual training
        }

        Ok(Vec::new())
    }

    /// Handle model update from participant
    async fn handle_model_update(
        &mut self,
        job_id: u64,
        round: u32,
        local_model: ModelParameters,
        node_id: String,
    ) -> Result<Vec<NetworkMessage>, FederatedNetworkError> {
        let mut responses = Vec::new();

        // Check if we're the coordinator for this job
        let is_coordinator = {
            let jobs = self.active_jobs.read().await;
            let job = jobs.get(&job_id)
                .ok_or(FederatedNetworkError::JobNotFound(job_id))?;
            job.coordinator_node == self.local_node.read().await.node_id
        };

        if is_coordinator {
            // Collect model updates and aggregate when ready
            responses.extend(self.aggregate_models_if_ready(job_id, round).await?);
        }

        Ok(responses)
    }

    /// Handle data shard distribution
    async fn handle_shard_distribution(
        &mut self,
        job_id: u64,
        shard_hash: String,
        target_nodes: Vec<String>,
        shard_size_bytes: u64,
    ) -> Result<Vec<NetworkMessage>, FederatedNetworkError> {
        let local_node_id = self.local_node.read().await.node_id.clone();
        
        if target_nodes.contains(&local_node_id) {
            println!("📥 Receiving data shard {} ({:.2} MB) for job {}", 
                    shard_hash, shard_size_bytes as f64 / 1_048_576.0, job_id);
            
            // In production, would start downloading the shard
            if let Some(ref coordinator) = self.large_data_coordinator {
                // coordinator.request_chunk(shard_hash).await?;
            }
        }

        Ok(Vec::new())
    }

    /// Aggregate models when ready
    async fn aggregate_models_if_ready(
        &mut self,
        job_id: u64,
        round: u32,
    ) -> Result<Vec<NetworkMessage>, FederatedNetworkError> {
        // Simplified aggregation - in production would collect actual model updates
        let mut responses = Vec::new();

        let fed_engines = self.federated_engines.read().await;
        if let Some(fed_engine) = fed_engines.get(&job_id) {
            if let Some(global_model) = fed_engine.get_global_model() {
                let stats = fed_engine.get_federated_stats();
                
                println!("📊 Aggregation complete for job {} round {}: accuracy {:.3}", 
                        job_id, round, stats.global_accuracy);

                // Clone stats before moving
                let stats_clone = stats.clone();
                
                // Broadcast aggregated model
                responses.push(NetworkMessage::FederatedAggregationResult {
                    job_id,
                    round,
                    global_model: global_model.clone(),
                    stats,
                    coordinator_id: self.local_node.read().await.node_id.clone(),
                });

                // Check if training is complete
                if stats_clone.has_converged || round >= self.config.federated_config.training_rounds {
                    drop(fed_engines); // Drop the read lock before calling mutable method
                    responses.extend(self.complete_federated_training(job_id).await?);
                }
            }
        }

        Ok(responses)
    }

    /// Complete federated training and distribute rewards
    async fn complete_federated_training(
        &mut self,
        job_id: u64,
    ) -> Result<Vec<NetworkMessage>, FederatedNetworkError> {
        let mut responses = Vec::new();

        let job = {
            let mut jobs = self.active_jobs.write().await;
            let job = jobs.get_mut(&job_id)
                .ok_or(FederatedNetworkError::JobNotFound(job_id))?;
            job.status = FederatedJobStatus::Completed;
            job.clone()
        };

        // Calculate participant rewards
        let total_participants = job.participants.len() as u64;
        let coordinator_fee = (job.total_reward as f32 * self.config.coordinator_fee_percent / 100.0) as u64;
        let participant_pool = job.total_reward - coordinator_fee;
        let base_reward = participant_pool / total_participants;

        let mut participants_rewards = HashMap::new();
        for (node_id, participant) in &job.participants {
            let reputation_bonus = (participant.reputation.max(0) as f32 * 0.1) as u64;
            let contribution_bonus = (base_reward as f32 * participant.contribution_weight * 0.2) as u64;
            let total_reward = base_reward + reputation_bonus + contribution_bonus;
            participants_rewards.insert(node_id.clone(), total_reward);
        }

        println!("🎉 Federated training completed for job {}!", job_id);
        println!("   • Total participants: {}", total_participants);
        println!("   • Coordinator fee: {} BCAI", coordinator_fee);
        println!("   • Average participant reward: {} BCAI", base_reward);

        // Get final model
        let final_model = {
            let fed_engines = self.federated_engines.read().await;
            fed_engines.get(&job_id)
                .and_then(|engine| engine.get_global_model())
                .cloned()
                .ok_or(FederatedNetworkError::JobNotFound(job_id))?
        };

        responses.push(NetworkMessage::FederatedTrainingComplete {
            job_id,
            final_model,
            participants_rewards,
        });

        Ok(responses)
    }

    /// Get federated training statistics
    pub async fn get_federated_stats(&self, job_id: u64) -> Option<FederatedStats> {
        self.federated_engines.read().await
            .get(&job_id)
            .map(|engine| engine.get_federated_stats())
    }

    /// List active federated jobs
    pub async fn list_active_jobs(&self) -> Vec<FederatedTrainingJob> {
        self.active_jobs.read().await.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_federated_network_coordinator_creation() {
        let config = FederatedNetworkConfig::default();
        
        // This would normally create actual node and coordinators
        // For test, we'll just verify the config
        assert_eq!(config.chunk_size_mb, 4);
        assert_eq!(config.max_data_size_gb, 5000);
        assert_eq!(config.reward_per_participant, 1000);
    }
} 