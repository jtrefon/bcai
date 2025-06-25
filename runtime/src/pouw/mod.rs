//! Defines the Proof-of-Useful-Work (PoUW) system.
//!
//! PoUW replaces traditional energy-wasting proof-of-work with useful
//! computation, such as training machine learning models. This module provides
//! the data structures and functions for creating, solving, and verifying PoUW tasks.

pub mod difficulty;
pub mod solver;
pub mod task;
pub mod types;
pub mod verifier;
pub mod validator_selection;
pub mod evaluation;
pub mod outlier;
pub mod model;

#[cfg(test)]
mod tests;

pub use difficulty::calculate_adaptive_difficulty;
pub use solver::solve;
pub use task::{generate_task, generate_task_with_timestamp};
pub use types::{PoUWConfig, Solution, PoUWTask, ValidatorSelectionConfig};
pub use types::PoUWTask as Task;
pub use verifier::verify;
pub use validator_selection::select_validators;
pub use evaluation::{sign_evaluation, verify_evaluation, evaluation_hash};
#[cfg(feature = "p2p")]
pub use evaluation::broadcast_evaluation;
pub use outlier::detect_outliers;
pub use model::file_hash as onnx_hash;
