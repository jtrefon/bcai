// This module will contain the model definitions for federated learning. 

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Federated learning errors
#[derive(Debug, Error)]
pub enum FederatedError {
    #[error("Insufficient participants: need {required}, have {available}")]
    InsufficientParticipants { required: usize, available: usize },
    #[error("Model dimension mismatch: expected {expected}, got {actual}")]
    ModelDimensionMismatch { expected: usize, actual: usize },
    #[error("Aggregation failed: {reason}")]
    AggregationFailed { reason: String },
    #[error("Invalid model parameters")]
    InvalidModelParameters,
    #[error("Timeout waiting for participants")]
    ParticipantTimeout,
}

/// Model parameters for federated learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelParameters {
    pub weights: Vec<f32>,
    pub biases: Vec<f32>,
    pub layer_sizes: Vec<usize>,
    pub metadata: ModelMetadata,
}

/// Model metadata for tracking training progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub model_id: String,
    pub version: u32,
    pub training_rounds: u32,
    pub participant_count: usize,
    pub accuracy: f32,
    pub loss: f32,
    pub created_at: u64,
} 