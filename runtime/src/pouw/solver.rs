//! Implements the PoUW solution generation (mining) logic.

use super::{
    types::{Solution, PoUWTask},
    verifier,
};
use sha2::{Digest, Sha256};

/// Solves a PoUW task by finding a nonce that meets the difficulty requirement.
/// This is the canonical "mining" function.
pub fn solve(task: &PoUWTask, difficulty: u32) -> Solution {
    let start_time = std::time::Instant::now();

    // In a real system, this is where the node would download the model
    // and dataset specified by the task IDs and perform the training.
    // We simulate this with a placeholder computation.
    let useful_work_result = perform_useful_work(task);

    let task_commitment = verifier::create_task_commitment(task);

    // This is the "mining" part: iterate on a nonce until the resulting
    // hash meets the difficulty target.
    for nonce in 0u64.. {
        let mut hasher = Sha256::new();
        hasher.update(&useful_work_result);
        hasher.update(&task_commitment);
        hasher.update(&nonce.to_le_bytes());
        let hash: [u8; 32] = hasher.finalize().into();

        if verifier::meets_difficulty(&hash, difficulty) {
            let computation_time_ms = start_time.elapsed().as_millis() as u64;
            return Solution {
                trained_model_hash: hex::encode(useful_work_result),
                accuracy: 9500, // Placeholder accuracy
                nonce,
                computation_time_ms,
            };
        }
    }

    unreachable!("A solution should always be found");
}

/// A placeholder for the actual "useful work" (e.g., ML model training).
/// The result of this work is then used in the hashing process.
fn perform_useful_work(task: &PoUWTask) -> [u8; 32] {
    // In a real implementation, this would be a complex and time-consuming
    // operation. For this example, we just hash the inputs to get a
    // deterministic result based on the task parameters.
    let mut hasher = Sha256::new();
    hasher.update(&task.model_id);
    hasher.update(&task.dataset_id);
    hasher.update(&task.epochs.to_le_bytes());
    hasher.finalize().into()
} 