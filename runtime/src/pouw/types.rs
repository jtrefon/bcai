//! Defines the core data structures for Proof-of-Useful-Work.

use serde::{Deserialize, Serialize};

/// A PoUW Task, which defines a machine learning job to be completed.
/// This represents the "work" to be done.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PoUWTask {
    /// A unique identifier for the machine learning model architecture.
    pub model_id: String,
    /// An identifier for the dataset to be used for training.
    pub dataset_id: String,
    /// The number of training epochs required.
    pub epochs: u32,
    /// The timestamp when the task was created, to prevent pre-computation.
    pub timestamp: u64,
    /// A random challenge to ensure task uniqueness.
    pub challenge: [u8; 32],
}

/// A PoUW Solution, which provides the result of a completed ML task.
/// This is the "proof" that the work was done.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Solution {
    /// A hash of the resulting trained model weights.
    pub trained_model_hash: String,
    /// The accuracy achieved by the model on a validation set.
    /// Represented as an integer (e.g., 9876 for 98.76% accuracy).
    pub accuracy: u32,
    /// A nonce found by the worker that satisfies the difficulty requirement.
    pub nonce: u64,
    /// The time it took to compute the solution in milliseconds.
    pub computation_time_ms: u64,
}


/// Configuration for PoUW security parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoUWConfig {
    /// The baseline difficulty target. A lower value is more difficult.
    pub base_difficulty: u32,
    /// The time window in which a task is considered valid, in seconds.
    pub time_window_secs: u64,
    /// The minimum time a computation must take, to mitigate pre-computation attacks.
    pub min_computation_ms: u64,
}

impl Default for PoUWConfig {
    fn default() -> Self {
        Self {
            base_difficulty: 0x000FFFFF, // A reasonable starting difficulty
            time_window_secs: 3600,     // 1 hour
            min_computation_ms: 100,    // 100ms
        }
    }
} 