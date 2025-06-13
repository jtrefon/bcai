use rand::{Rng, SeedableRng, rngs::StdRng};
use sha2::{Digest, Sha256};

/// A simple matrix multiplication task used for Proof-of-Useful-Work.
#[derive(Clone, Debug)]
pub struct Task {
    pub a: Vec<Vec<u8>>, // NxN matrix
    pub b: Vec<Vec<u8>>, // NxN matrix
}

/// Result of solving a task along with a nonce that satisfies the difficulty.
#[derive(Clone, Debug)]
pub struct Solution {
    pub result: Vec<Vec<u32>>, // matrix multiplication result
    pub nonce: u64,
}

/// Generate a deterministic task given a size and seed.
pub fn generate_task(size: usize, seed: u64) -> Task {
    let mut rng = StdRng::seed_from_u64(seed);
    let a = (0..size).map(|_| (0..size).map(|_| rng.gen_range(0..16)).collect()).collect();
    let b = (0..size).map(|_| (0..size).map(|_| rng.gen_range(0..16)).collect()).collect();
    Task { a, b }
}

/// Multiply two matrices of size NxN.
fn multiply(a: &[Vec<u8>], b: &[Vec<u8>]) -> Vec<Vec<u32>> {
    let n = a.len();
    let mut result = vec![vec![0u32; n]; n];
    for i in 0..n {
        for k in 0..n {
            let aik = a[i][k] as u32;
            for j in 0..n {
                result[i][j] += aik * b[k][j] as u32;
            }
        }
    }
    result
}

/// Difficulty target for the hash. Lower values are harder.
fn meets_difficulty(hash: &[u8; 32], difficulty: u32) -> bool {
    let value = u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]);
    value <= difficulty
}

/// Flatten a matrix into bytes for hashing.
fn flatten_matrix(mat: &[Vec<u32>]) -> Vec<u8> {
    mat.iter().flat_map(|row| row.iter().flat_map(|v| v.to_be_bytes())).collect()
}

/// Solve a task by computing the matrix multiplication and searching for a nonce
/// whose hash meets the given difficulty. Returns the solution.
pub fn solve(task: &Task, difficulty: u32) -> Solution {
    let result = multiply(&task.a, &task.b);
    let bytes = flatten_matrix(&result);
    for nonce in 0u64.. {
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        hasher.update(nonce.to_le_bytes());
        let hash: [u8; 32] = hasher.finalize().into();
        if meets_difficulty(&hash, difficulty) {
            return Solution { result, nonce };
        }
    }
    unreachable!();
}

/// Verify that a solution is correct for the given task and difficulty.
pub fn verify(task: &Task, solution: &Solution, difficulty: u32) -> bool {
    let expected = multiply(&task.a, &task.b);
    if expected != solution.result {
        return false;
    }
    let bytes = flatten_matrix(&solution.result);
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    hasher.update(solution.nonce.to_le_bytes());
    let hash: [u8; 32] = hasher.finalize().into();
    meets_difficulty(&hash, difficulty)
}
