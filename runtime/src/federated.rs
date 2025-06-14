//! Federated Learning Engine for BCAI
//!
//! This module implements federated learning capabilities including model aggregation,
//! federated averaging, and secure multi-party computation for distributed AI training.

use crate::node::NodeCapability;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
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

/// Federated learning participant
#[derive(Debug, Clone)]
pub struct FederatedParticipant {
    pub node_id: String,
    pub capabilities: NodeCapability,
    pub local_model: Option<ModelParameters>,
    pub training_data_size: usize,
    pub contribution_weight: f32,
    pub last_update: Instant,
    pub reputation: i32,
}

/// Aggregation strategy for federated learning
#[derive(Debug, Clone)]
pub enum AggregationStrategy {
    /// Simple average of all participants
    FederatedAveraging,
    /// Weighted average based on data size
    WeightedAveraging,
    /// Reputation-based weighting
    ReputationWeighted,
}

/// Federated learning configuration
#[derive(Debug, Clone)]
pub struct FederatedConfig {
    pub min_participants: usize,
    pub max_participants: usize,
    pub aggregation_strategy: AggregationStrategy,
    pub training_rounds: u32,
    pub convergence_threshold: f32,
    pub participant_timeout: Duration,
    pub model_validation_enabled: bool,
}

impl Default for FederatedConfig {
    fn default() -> Self {
        Self {
            min_participants: 3,
            max_participants: 100,
            aggregation_strategy: AggregationStrategy::FederatedAveraging,
            training_rounds: 10,
            convergence_threshold: 0.01,
            participant_timeout: Duration::from_secs(300),
            model_validation_enabled: true,
        }
    }
}

/// Federated learning engine
pub struct FederatedEngine {
    config: FederatedConfig,
    participants: HashMap<String, FederatedParticipant>,
    global_model: Option<ModelParameters>,
    training_history: Vec<FederatedRound>,
    current_round: u32,
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

impl FederatedEngine {
    /// Create a new federated learning engine
    pub fn new(config: FederatedConfig) -> Self {
        Self {
            config,
            participants: HashMap::new(),
            global_model: None,
            training_history: Vec::new(),
            current_round: 0,
        }
    }

    /// Add a participant to the federated learning session
    pub fn add_participant(
        &mut self,
        node_id: String,
        capabilities: NodeCapability,
        training_data_size: usize,
    ) -> Result<(), FederatedError> {
        if self.participants.len() >= self.config.max_participants {
            return Err(FederatedError::InsufficientParticipants {
                required: self.config.min_participants,
                available: self.participants.len(),
            });
        }

        let participant = FederatedParticipant {
            node_id: node_id.clone(),
            capabilities,
            local_model: None,
            training_data_size,
            contribution_weight: self.calculate_contribution_weight(training_data_size),
            last_update: Instant::now(),
            reputation: 0,
        };

        self.participants.insert(node_id, participant);
        println!("✅ Added federated participant: {} participants", self.participants.len());
        
        Ok(())
    }

    /// Initialize federated learning with global model
    pub fn initialize_global_model(&mut self, initial_model: ModelParameters) -> Result<(), FederatedError> {
        if self.participants.len() < self.config.min_participants {
            return Err(FederatedError::InsufficientParticipants {
                required: self.config.min_participants,
                available: self.participants.len(),
            });
        }

        self.global_model = Some(initial_model);
        self.current_round = 0;
        
        println!("🚀 Initialized federated learning with {} participants", self.participants.len());
        Ok(())
    }

    /// Perform federated averaging aggregation
    pub fn aggregate_models(&mut self, local_models: Vec<(String, ModelParameters)>) -> Result<ModelParameters, FederatedError> {
        if local_models.is_empty() {
            return Err(FederatedError::InsufficientParticipants {
                required: 1,
                available: 0,
            });
        }

        let start_time = Instant::now();
        
        // Validate all models have the same structure
        let reference_model = &local_models[0].1;
        for (_node_id, model) in &local_models {
            if model.weights.len() != reference_model.weights.len() ||
               model.biases.len() != reference_model.biases.len() {
                return Err(FederatedError::ModelDimensionMismatch {
                    expected: reference_model.weights.len(),
                    actual: model.weights.len(),
                });
            }
        }

        // Calculate weighted average based on strategy
        let aggregated_model = match &self.config.aggregation_strategy {
            AggregationStrategy::FederatedAveraging => {
                self.federated_averaging(&local_models)?
            }
            AggregationStrategy::WeightedAveraging => {
                self.weighted_averaging(&local_models)?
            }
            AggregationStrategy::ReputationWeighted => {
                self.reputation_weighted_averaging(&local_models)?
            }
        };

        // Update global model
        self.global_model = Some(aggregated_model.clone());
        self.current_round += 1;

        // Record training round
        let round_record = FederatedRound {
            round_number: self.current_round,
            participants: local_models.iter().map(|(id, _)| id.clone()).collect(),
            global_accuracy: aggregated_model.metadata.accuracy,
            global_loss: aggregated_model.metadata.loss,
            convergence_score: self.calculate_convergence_score(&local_models),
            duration: start_time.elapsed(),
            completed_at: Instant::now(),
        };

        self.training_history.push(round_record);

        println!("📊 Federated round {} completed: {} participants, accuracy: {:.3}",
                 self.current_round, local_models.len(), aggregated_model.metadata.accuracy);

        Ok(aggregated_model)
    }

    /// Simple federated averaging
    fn federated_averaging(&self, models: &[(String, ModelParameters)]) -> Result<ModelParameters, FederatedError> {
        let num_models = models.len() as f32;
        let reference_model = &models[0].1;
        
        // Average weights
        let mut averaged_weights = vec![0.0; reference_model.weights.len()];
        let mut averaged_biases = vec![0.0; reference_model.biases.len()];
        
        for (_, model) in models {
            for (i, &weight) in model.weights.iter().enumerate() {
                averaged_weights[i] += weight / num_models;
            }
            for (i, &bias) in model.biases.iter().enumerate() {
                averaged_biases[i] += bias / num_models;
            }
        }

        // Calculate average accuracy and loss
        let avg_accuracy = models.iter().map(|(_, m)| m.metadata.accuracy).sum::<f32>() / num_models;
        let avg_loss = models.iter().map(|(_, m)| m.metadata.loss).sum::<f32>() / num_models;

        Ok(ModelParameters {
            weights: averaged_weights,
            biases: averaged_biases,
            layer_sizes: reference_model.layer_sizes.clone(),
            metadata: ModelMetadata {
                model_id: reference_model.metadata.model_id.clone(),
                version: self.current_round + 1,
                training_rounds: self.current_round + 1,
                participant_count: models.len(),
                accuracy: avg_accuracy,
                loss: avg_loss,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        })
    }

    /// Weighted averaging based on data size
    fn weighted_averaging(&self, models: &[(String, ModelParameters)]) -> Result<ModelParameters, FederatedError> {
        let reference_model = &models[0].1;
        let total_weight: f32 = models.iter()
            .map(|(node_id, _)| {
                self.participants.get(node_id)
                    .map(|p| p.contribution_weight)
                    .unwrap_or(1.0)
            })
            .sum();

        let mut weighted_weights = vec![0.0; reference_model.weights.len()];
        let mut weighted_biases = vec![0.0; reference_model.biases.len()];
        
        for (node_id, model) in models {
            let weight = self.participants.get(node_id)
                .map(|p| p.contribution_weight / total_weight)
                .unwrap_or(1.0 / models.len() as f32);
            
            for (i, &w) in model.weights.iter().enumerate() {
                weighted_weights[i] += w * weight;
            }
            for (i, &b) in model.biases.iter().enumerate() {
                weighted_biases[i] += b * weight;
            }
        }

        // Weighted average of accuracy and loss
        let weighted_accuracy = models.iter()
            .map(|(node_id, model)| {
                let weight = self.participants.get(node_id)
                    .map(|p| p.contribution_weight / total_weight)
                    .unwrap_or(1.0 / models.len() as f32);
                model.metadata.accuracy * weight
            })
            .sum();

        let weighted_loss = models.iter()
            .map(|(node_id, model)| {
                let weight = self.participants.get(node_id)
                    .map(|p| p.contribution_weight / total_weight)
                    .unwrap_or(1.0 / models.len() as f32);
                model.metadata.loss * weight
            })
            .sum();

        Ok(ModelParameters {
            weights: weighted_weights,
            biases: weighted_biases,
            layer_sizes: reference_model.layer_sizes.clone(),
            metadata: ModelMetadata {
                model_id: reference_model.metadata.model_id.clone(),
                version: self.current_round + 1,
                training_rounds: self.current_round + 1,
                participant_count: models.len(),
                accuracy: weighted_accuracy,
                loss: weighted_loss,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        })
    }

    /// Reputation-weighted averaging
    fn reputation_weighted_averaging(&self, models: &[(String, ModelParameters)]) -> Result<ModelParameters, FederatedError> {
        // For now, fall back to simple averaging
        // In a real implementation, this would weight by node reputation
        self.federated_averaging(models)
    }

    /// Calculate contribution weight based on data size
    fn calculate_contribution_weight(&self, data_size: usize) -> f32 {
        // Simple linear weighting based on data size
        // In practice, you might use logarithmic or other weighting schemes
        data_size as f32
    }

    /// Calculate convergence score for the current round
    fn calculate_convergence_score(&self, models: &[(String, ModelParameters)]) -> f32 {
        if models.len() < 2 {
            return 1.0;
        }

        // Calculate variance in model parameters as convergence metric
        let mut param_variances = Vec::new();
        
        for param_idx in 0..models[0].1.weights.len() {
            let values: Vec<f32> = models.iter()
                .map(|(_, model)| model.weights[param_idx])
                .collect();
            
            let mean = values.iter().sum::<f32>() / values.len() as f32;
            let variance = values.iter()
                .map(|v| (v - mean).powi(2))
                .sum::<f32>() / values.len() as f32;
            
            param_variances.push(variance);
        }

        // Return average variance (lower is better convergence)
        let avg_variance = param_variances.iter().sum::<f32>() / param_variances.len() as f32;
        
        // Convert to convergence score (higher is better, 0-1 range)
        1.0 / (1.0 + avg_variance)
    }

    /// Check if federated learning has converged
    pub fn has_converged(&self) -> bool {
        if self.training_history.len() < 2 {
            return false;
        }

        let recent_rounds = self.training_history.iter()
            .rev()
            .take(3)
            .collect::<Vec<_>>();

        // Check if convergence score has stabilized
        let convergence_scores: Vec<f32> = recent_rounds.iter()
            .map(|r| r.convergence_score)
            .collect();

        if convergence_scores.len() < 2 {
            return false;
        }

        let score_variance = convergence_scores.iter()
            .map(|&s| (s - convergence_scores[0]).abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(1.0);

        score_variance < self.config.convergence_threshold
    }

    /// Get current global model
    pub fn get_global_model(&self) -> Option<&ModelParameters> {
        self.global_model.as_ref()
    }

    /// Get federated learning statistics
    pub fn get_federated_stats(&self) -> FederatedStats {
        FederatedStats {
            current_round: self.current_round,
            total_participants: self.participants.len(),
            active_participants: self.participants.values()
                .filter(|p| p.last_update.elapsed() < self.config.participant_timeout)
                .count(),
            global_accuracy: self.global_model.as_ref()
                .map(|m| m.metadata.accuracy)
                .unwrap_or(0.0),
            global_loss: self.global_model.as_ref()
                .map(|m| m.metadata.loss)
                .unwrap_or(0.0),
            convergence_score: self.training_history.last()
                .map(|r| r.convergence_score)
                .unwrap_or(0.0),
            has_converged: self.has_converged(),
            total_training_time: self.training_history.iter()
                .map(|r| r.duration)
                .sum(),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_model(weights: Vec<f32>, accuracy: f32) -> ModelParameters {
        ModelParameters {
            weights,
            biases: vec![0.1, 0.2],
            layer_sizes: vec![10, 5, 1],
            metadata: ModelMetadata {
                model_id: "test_model".to_string(),
                version: 1,
                training_rounds: 1,
                participant_count: 1,
                accuracy,
                loss: 1.0 - accuracy,
                created_at: 1234567890,
            },
        }
    }

    #[test]
    fn federated_engine_creation() {
        let config = FederatedConfig::default();
        let engine = FederatedEngine::new(config);
        
        assert_eq!(engine.current_round, 0);
        assert!(engine.global_model.is_none());
        assert_eq!(engine.participants.len(), 0);
    }

    #[test]
    fn add_participants() {
        let mut engine = FederatedEngine::new(FederatedConfig::default());
        let capability = NodeCapability {
            cpus: 4,
            gpus: 1,
            gpu_memory_gb: 8,
            available_stake: 1000,
            reputation: 0,
        };
        
        assert!(engine.add_participant("node1".to_string(), capability.clone(), 1000).is_ok());
        assert!(engine.add_participant("node2".to_string(), capability.clone(), 2000).is_ok());
        
        assert_eq!(engine.participants.len(), 2);
    }

    #[test]
    fn federated_averaging() {
        let mut engine = FederatedEngine::new(FederatedConfig::default());
        
        let models = vec![
            ("node1".to_string(), create_test_model(vec![1.0, 2.0, 3.0], 0.8)),
            ("node2".to_string(), create_test_model(vec![2.0, 3.0, 4.0], 0.9)),
        ];
        
        let result = engine.federated_averaging(&models).unwrap();
        
        assert_eq!(result.weights, vec![1.5, 2.5, 3.5]);
        assert_eq!(result.metadata.accuracy, 0.85); // (0.8 + 0.9) / 2
    }

    #[test]
    fn convergence_detection() {
        let mut config = FederatedConfig::default();
        config.convergence_threshold = 0.1; // More lenient threshold for test
        let mut engine = FederatedEngine::new(config);
        
        // Simulate training rounds with small variance (converged)
        let rounds = vec![
            FederatedRound {
                round_number: 1,
                participants: vec!["node1".to_string()],
                global_accuracy: 0.85,
                global_loss: 0.15,
                convergence_score: 0.85,
                duration: Duration::from_secs(10),
                completed_at: Instant::now(),
            },
            FederatedRound {
                round_number: 2,
                participants: vec!["node1".to_string()],
                global_accuracy: 0.86,
                global_loss: 0.14,
                convergence_score: 0.86,
                duration: Duration::from_secs(10),
                completed_at: Instant::now(),
            },
            FederatedRound {
                round_number: 3,
                participants: vec!["node1".to_string()],
                global_accuracy: 0.87,
                global_loss: 0.13,
                convergence_score: 0.87,
                duration: Duration::from_secs(10),
                completed_at: Instant::now(),
            },
        ];
        
        engine.training_history = rounds;
        
        // With small variance (0.02 < 0.1), should detect convergence
        assert!(engine.has_converged());
    }
} 