//! Defines functions for generating new PoUW tasks.

use super::types::PoUWTask;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::time::{SystemTime, UNIX_EPOCH};

/// Generates a new PoUW task with a given difficulty proxy.
/// The timestamp is used to seed the random number generator for determinism.
pub fn generate_task(difficulty: u32, seed: u64) -> PoUWTask {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut challenge = [0u8; 32];
    rng.fill(&mut challenge);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    PoUWTask {
        model_id: format!("model_{}", difficulty),
        dataset_id: format!("dataset_{}", difficulty),
        model_hash: None,
        dataset_hash: None,
        epochs: difficulty, // Using difficulty as a proxy for epochs
        timestamp,
        challenge,
    }
}

/// Generates a PoUW task using the given timestamp for determinism.
/// This is a convenience wrapper used by node modules.
pub fn generate_task_with_timestamp(difficulty: u32, timestamp: u64) -> PoUWTask {
    let mut task = generate_task(difficulty, timestamp);
    task.timestamp = timestamp;
    task
}
