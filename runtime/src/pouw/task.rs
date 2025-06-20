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
        epochs: difficulty, // Using difficulty as a proxy for epochs
        timestamp,
        challenge,
    }
} 