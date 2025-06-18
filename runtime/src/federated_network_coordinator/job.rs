use crate::large_data_transfer::LargeDataDescriptor;
use crate::node::NodeCapability;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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