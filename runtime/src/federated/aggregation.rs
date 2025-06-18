// This module will contain the model aggregation logic. 

use std::time::Duration;

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