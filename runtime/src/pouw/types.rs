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
    /// Optional SHA-256 hash of the ONNX model stored off-chain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_hash: Option<String>,
    /// Optional SHA-256 hash of the validation dataset stored on DFS.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_hash: Option<String>,
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

/// A signed evaluation result from a validator.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedEvaluation {
    /// ID of the evaluated task.
    pub task_id: String,
    /// Accuracy reported by the validator.
    pub accuracy: u32,
    /// Validator ID and signature over (task_id, accuracy).
    pub validator: String,
    pub signature: Vec<u8>,
}

/// Configuration for selecting validators for evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidatorSelectionConfig {
    /// Minimum stake required to be eligible as a validator.
    pub min_stake: u64,
    /// Number of validators to select for each task.
    pub subset_size: usize,
}

impl Default for ValidatorSelectionConfig {
    fn default() -> Self {
        Self {
            min_stake: 1,
            subset_size: 3,
        }
    }
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

/// Simple type alias for backward compatibility.
pub type PoUWSolution = Solution;

impl PoUWTask {
    /// Creates a new PoUWTask with random challenge and current timestamp.
    pub fn new(model_id: String, dataset_id: String, epochs: u32) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut challenge = [0u8; 32];
        rng.fill(&mut challenge);
        Self {
            model_id,
            dataset_id,
            model_hash: None,
            dataset_hash: None,
            epochs,
            timestamp: chrono::Utc::now().timestamp() as u64,
            challenge,
        }
    }

    /// Verifies a solution using the PoUW verifier with default configuration.
    pub fn verify(&self, solution: &PoUWSolution, difficulty: u32) -> bool {
        crate::pouw::verifier::verify(self, solution, difficulty, &PoUWConfig::default())
    }
}
