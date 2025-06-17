//! Federated Network Coordinator
//!
//! This module coordinates federated learning across the P2P network,
//! integrating FederatedEngine with large data transfer and economic incentives.

use crate::{
    federated::{FederatedConfig, ModelParameters, FederatedStats, FederatedError},
    large_data_transfer::{LargeDataDescriptor, ChunkId},
    node::{NodeCapability, DistributedJob},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

// NOTE: Removed placeholder implementation structs:
// - FederatedNetworkCoordinator
// This file now only defines the data models for the federated network coordinator. 