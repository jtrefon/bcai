// This module will contain the federated learning logic. 

use crate::node::{NodeCapability};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Federated learning participant
#[derive(Debug, Clone)]
pub struct FederatedParticipant {
    pub node_id: String,
    pub capabilities: NodeCapability,
    pub local_model: Option<crate::federated::model::ModelParameters>,
    pub training_data_size: usize,
    pub contribution_weight: f32,
    pub last_update: Instant,
    pub reputation: i32,
}

/// Record of a federated training round
#[derive(Debug, Clone)]
pub struct FederatedRound {
    pub round_number: u32,
    pub participants: Vec<String>,
    pub global_accuracy: f32,
    pub global_loss: f32,
    pub convergence_score: f32,
    pub duration: Duration,
    pub completed_at: Instant,
}

/// Federated learning statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedStats {
    pub current_round: u32,
    pub total_participants: usize,
    pub active_participants: usize,
    pub global_accuracy: f32,
    pub global_loss: f32,
    pub convergence_score: f32,
    pub has_converged: bool,
    pub total_training_time: Duration,
} 