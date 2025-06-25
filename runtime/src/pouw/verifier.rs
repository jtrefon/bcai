//! Implements the PoUW solution verification logic.

use super::types::{PoUWConfig, Solution, PoUWTask};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Verifies a PoUW solution against a task and a set of configuration rules.
/// This is the canonical verification function.
pub fn verify(
    task: &PoUWTask,
    solution: &Solution,
    difficulty: u32,
    config: &PoUWConfig,
) -> bool {
    // 1. Validate timestamp to prevent stale or future-dated work.
    if !validate_timestamp(task.timestamp, config) {
        return false;
    }

    // 2. Validate computation time to mitigate pre-computation attacks.
    if !validate_computation_time(solution.computation_time_ms, config) {
        return false;
    }
    
    // 3. Re-create the hash and check if it meets the difficulty target.
    let task_commitment = create_task_commitment(task);
    let mut hasher = Sha256::new();
    hasher.update(solution.trained_model_hash.as_bytes()); // Assuming this is the core output
    hasher.update(&task_commitment);
    hasher.update(&solution.nonce.to_le_bytes());
    let hash: [u8; 32] = hasher.finalize().into();

    meets_difficulty(&hash, difficulty)
}

/// Checks if a hash meets the given difficulty target.
/// A lower difficulty value means a more difficult target.
pub fn meets_difficulty(hash: &[u8; 32], difficulty: u32) -> bool {
    let hash_prefix = u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]);
    hash_prefix <= difficulty
}

/// Validates that the task's timestamp is within the acceptable window.
fn validate_timestamp(timestamp: u64, config: &PoUWConfig) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Task timestamp cannot be in the future (with a small grace period).
    if timestamp > now + 5 {
        return false;
    }

    // Task must be recent enough to prevent replay attacks with old, pre-computed work.
    let age = now.saturating_sub(timestamp);
    age <= config.time_window_secs
}

/// Validates that the claimed computation time is reasonable.
fn validate_computation_time(computation_time_ms: u64, config: &PoUWConfig) -> bool {
    computation_time_ms >= config.min_computation_ms
}

/// Creates a cryptographic commitment to the task's parameters for integrity.
/// This ensures a solution is tied to a specific, unmodified task.
pub fn create_task_commitment(task: &PoUWTask) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(task.model_id.as_bytes());
    hasher.update(task.dataset_id.as_bytes());
    if let Some(ref h) = task.model_hash {
        hasher.update(h.as_bytes());
    }
    if let Some(ref h) = task.dataset_hash {
        hasher.update(h.as_bytes());
    }
    hasher.update(task.epochs.to_le_bytes());
    hasher.update(task.timestamp.to_le_bytes());
    hasher.update(task.challenge);
    hasher.finalize().into()
} 