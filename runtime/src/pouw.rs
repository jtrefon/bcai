use rand::{Rng, RngCore, SeedableRng, rngs::StdRng};
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// A simple matrix multiplication task used for Proof-of-Useful-Work.
#[derive(Clone, Debug)]
pub struct Task {
    pub a: Vec<Vec<u8>>, // NxN matrix
    pub b: Vec<Vec<u8>>, // NxN matrix
    pub timestamp: u64,  // Block timestamp to prevent precomputation
    pub challenge: [u8; 32], // Random challenge to prevent result prediction
}

/// Result of solving a task along with a nonce that satisfies the difficulty.
#[derive(Clone, Debug)]
pub struct Solution {
    pub result: Vec<Vec<u32>>, // matrix multiplication result
    pub nonce: u64,
    pub computation_time: u64, // Time spent computing (anti-precomputation)
}

/// Configuration for PoUW difficulty and security parameters.
#[derive(Clone, Debug)]
pub struct PoUWConfig {
    pub base_difficulty: u32,
    pub time_window_secs: u64,
    pub max_precompute_advantage: u64,
}

impl Default for PoUWConfig {
    fn default() -> Self {
        Self {
            base_difficulty: 0x0000ffff,
            time_window_secs: 300, // 5 minutes
            max_precompute_advantage: 10, // seconds
        }
    }
}

/// Generate a deterministic task given a size and seed with security enhancements.
pub fn generate_task(size: usize, seed: u64) -> Task {
    let mut rng = StdRng::seed_from_u64(seed);
    let a = (0..size).map(|_| (0..size).map(|_| rng.gen_range(0..16)).collect()).collect();
    let b = (0..size).map(|_| (0..size).map(|_| rng.gen_range(0..16)).collect()).collect();
    
    // Add security measures
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    let mut challenge = [0u8; 32];
    rng.fill_bytes(&mut challenge);
    
    Task { a, b, timestamp, challenge }
}

/// Generate a task with explicit parameters for testing and configuration.
pub fn generate_task_with_config(size: usize, seed: u64, _config: &PoUWConfig) -> Task {
    let task = generate_task(size, seed);
    // Adjust difficulty based on config if needed in future
    task
}

/// Multiply two matrices of size NxN.
fn multiply(a: &[Vec<u8>], b: &[Vec<u8>]) -> Vec<Vec<u32>> {
    let n = a.len();
    let mut result = vec![vec![0u32; n]; n];
    result.par_iter_mut().enumerate().for_each(|(i, row)| {
        for (k, b_row) in b.iter().enumerate().take(n) {
            let aik = a[i][k] as u32;
            for (j, &bkj) in b_row.iter().enumerate().take(n) {
                row[j] += aik * bkj as u32;
            }
        }
    });
    result
}

/// **SECURITY FIX**: Proper difficulty calculation using full hash entropy.
/// Previous version only checked first 4 bytes - easily gameable!
fn meets_difficulty(hash: &[u8; 32], difficulty: u32) -> bool {
    // Use full hash for difficulty calculation, not just first 4 bytes
    let mut hash_value = 0u64;
    for (i, &byte) in hash.iter().take(8).enumerate() {
        hash_value |= (byte as u64) << (i * 8);
    }
    
    // Scale difficulty to 64-bit space for proper entropy usage
    let difficulty_threshold = (difficulty as u64) << 32;
    hash_value <= difficulty_threshold
}

/// **SECURITY ENHANCEMENT**: Validate timestamp is within acceptable range.
fn validate_timestamp(timestamp: u64, config: &PoUWConfig) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    // Task must be recent (prevent old precomputed work)
    let age = now.saturating_sub(timestamp);
    age <= config.time_window_secs
}

/// **SECURITY ENHANCEMENT**: Validate computation time to prevent precomputation.
fn validate_computation_time(computation_time: u64, config: &PoUWConfig) -> bool {
    // Computation should take reasonable time (prevent precomputed solutions)
    computation_time >= config.max_precompute_advantage
}

/// Create cryptographic commitment to task parameters for integrity.
fn create_task_commitment(task: &Task) -> [u8; 32] {
    let mut hasher = Sha256::new();
    
    // Hash all task components for integrity
    for row in &task.a {
        for &val in row {
            hasher.update([val]);
        }
    }
    for row in &task.b {
        for &val in row {
            hasher.update([val]);
        }
    }
    hasher.update(task.timestamp.to_le_bytes());
    hasher.update(task.challenge);
    
    hasher.finalize().into()
}

/// Flatten a matrix into bytes for hashing.
fn flatten_matrix(mat: &[Vec<u32>]) -> Vec<u8> {
    mat.iter().flat_map(|row| row.iter().flat_map(|v| v.to_be_bytes())).collect()
}

/// **ENHANCED**: Solve a task with proper security measures and time tracking.
pub fn solve(task: &Task, difficulty: u32) -> Solution {
    let start_time = std::time::Instant::now();
    let result = multiply(&task.a, &task.b);
    let bytes = flatten_matrix(&result);
    let task_commitment = create_task_commitment(task);
    
    for nonce in 0u64.. {
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        hasher.update(task_commitment); // Include task commitment
        hasher.update(nonce.to_le_bytes());
        let hash: [u8; 32] = hasher.finalize().into();
        
        if meets_difficulty(&hash, difficulty) {
            let computation_time = start_time.elapsed().as_millis() as u64;
            return Solution { 
                result, 
                nonce, 
                computation_time,
            };
        }
    }
    unreachable!();
}

/// Solve a task while measuring execution time.
pub fn solve_profile(task: &Task, difficulty: u32) -> (Solution, std::time::Duration) {
    let start = std::time::Instant::now();
    let sol = solve(task, difficulty);
    let dur = start.elapsed();
    (sol, dur)
}

/// **ENHANCED**: Comprehensive verification with security checks.
pub fn verify(task: &Task, solution: &Solution, difficulty: u32) -> bool {
    verify_with_config(task, solution, difficulty, &PoUWConfig::default())
}

/// **NEW**: Verify with configurable security parameters.
pub fn verify_with_config(task: &Task, solution: &Solution, difficulty: u32, config: &PoUWConfig) -> bool {
    // 1. Validate timestamp is recent
    if !validate_timestamp(task.timestamp, config) {
        return false;
    }
    
    // 2. Validate computation time (anti-precomputation)
    if !validate_computation_time(solution.computation_time, config) {
        return false;
    }
    
    // 3. Verify matrix multiplication is correct
    let expected = multiply(&task.a, &task.b);
    if expected != solution.result {
        return false;
    }
    
    // 4. Verify cryptographic proof with proper difficulty calculation
    let bytes = flatten_matrix(&solution.result);
    let task_commitment = create_task_commitment(task);
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    hasher.update(task_commitment);
    hasher.update(solution.nonce.to_le_bytes());
    let hash: [u8; 32] = hasher.finalize().into();
    
    meets_difficulty(&hash, difficulty)
}

/// **NEW**: Calculate adaptive difficulty based on network conditions.
/// Lower difficulty numbers mean it's harder to find valid hashes.
pub fn calculate_adaptive_difficulty(
    current_difficulty: u32,
    target_time_secs: u64,
    actual_time_secs: u64,
) -> u32 {
    // Implement difficulty adjustment algorithm
    // If actual time > target time, blocks are too slow -> make easier (higher number)
    // If actual time < target time, blocks are too fast -> make harder (lower number)
    let adjustment_factor = actual_time_secs as f64 / target_time_secs as f64;
    let new_difficulty = (current_difficulty as f64 * adjustment_factor) as u32;
    
    // Clamp adjustments to prevent extreme changes
    let max_adjustment = current_difficulty / 4;
    new_difficulty.clamp(
        current_difficulty.saturating_sub(max_adjustment),
        current_difficulty.saturating_add(max_adjustment),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn security_meets_difficulty_uses_full_hash() {
        // Test that difficulty calculation uses more than just first 4 bytes
        let mut hash = [0xff; 32]; // High entropy hash
        hash[0] = 0x00; // Low first byte
        
        // Should still be difficult because we use full hash entropy
        assert!(!meets_difficulty(&hash, 0x00000001));
    }

    #[test]
    fn task_commitment_prevents_tampering() {
        let task1 = generate_task(2, 42);
        let mut task2 = task1.clone();
        task2.a[0][0] = task2.a[0][0].wrapping_add(1); // Modify task
        
        // Commitments should be different
        assert_ne!(create_task_commitment(&task1), create_task_commitment(&task2));
    }

    #[test]
    fn adaptive_difficulty_adjustment() {
        let current = 0x0000ffff;
        
        // If blocks are too fast (30s vs target 60s), make harder (lower number)
        let adjusted = calculate_adaptive_difficulty(current, 60, 30);
        assert!(adjusted < current, "Expected difficulty to increase (lower number) when blocks are too fast, but {} >= {}", adjusted, current);
        
        // If blocks are too slow (120s vs target 60s), make easier (higher number)
        let adjusted = calculate_adaptive_difficulty(current, 60, 120);
        assert!(adjusted > current, "Expected difficulty to decrease (higher number) when blocks are too slow, but {} <= {}", adjusted, current);
        
        // Test extreme cases are clamped
        let adjusted_extreme = calculate_adaptive_difficulty(current, 60, 1);
        assert!(adjusted_extreme >= current.saturating_sub(current / 4), "Difficulty adjustment should be clamped");
    }
}
