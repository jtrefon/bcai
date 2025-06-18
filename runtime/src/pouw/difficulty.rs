//! Implements the adaptive difficulty adjustment algorithm.

const DIFFICULTY_ADJUSTMENT_FACTOR: f64 = 0.05; // 5% adjustment factor

/// Adjusts the difficulty based on the time taken to solve the last block or task.
/// The goal is to maintain a consistent average target time.
pub fn calculate_adaptive_difficulty(
    current_difficulty: u32,
    target_time_secs: u64,
    actual_time_secs: u64,
) -> u32 {
    if actual_time_secs == 0 {
        return current_difficulty; // Avoid division by zero
    }

    let time_ratio = actual_time_secs as f64 / target_time_secs as f64;

    // If time is too short, increase difficulty (lower the number).
    // If time is too long, decrease difficulty (raise the number).
    let adjustment = if time_ratio < 1.0 {
        // Faster than target: increase difficulty
        1.0 - (1.0 - time_ratio).min(1.0) * DIFFICULTY_ADJUSTMENT_FACTOR
    } else {
        // Slower than target: decrease difficulty
        1.0 + (time_ratio - 1.0).min(1.0) * DIFFICULTY_ADJUSTMENT_FACTOR
    };

    let new_difficulty_float = current_difficulty as f64 * adjustment;

    // Clamp the new difficulty to be within a reasonable range.
    (new_difficulty_float as u32).max(1)
} 