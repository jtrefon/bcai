//! Defines the structure of a computational job that can be used for PoUW.

use serde::{Deserialize, Serialize};

/// Represents a generic computational job.
///
/// In a real system, this would contain details about the model, dataset,
/// hyperparameters, etc. For now, it's a simple placeholder.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Job {
    pub id: u64,
    pub model_id: String,
    pub dataset_id: String,
    pub iterations: u32,
}

impl Job {
    pub fn new(id: u64, model_id: String, dataset_id: String, iterations: u32) -> Self {
        Self {
            id,
            model_id,
            dataset_id,
            iterations,
        }
    }
} 