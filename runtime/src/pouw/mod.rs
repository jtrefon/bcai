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

#[cfg(test)]
mod tests;

pub use difficulty::calculate_adaptive_difficulty;
pub use solver::solve;
pub use task::{generate_task, generate_task_with_timestamp};
pub use types::{PoUWConfig, Solution, PoUWTask};
pub use types::PoUWTask as Task;
pub use verifier::verify;
