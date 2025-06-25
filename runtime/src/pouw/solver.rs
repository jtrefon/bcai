//! Implements the PoUW solution generation (mining) logic.

use super::{
    types::{PoUWTask, Solution},
    verifier,
};
use ndarray::{Array1, Array2};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use sha2::{Digest, Sha256};

/// Solves a PoUW task by finding a nonce that meets the difficulty requirement.
/// This is the canonical "mining" function.
pub fn solve(task: &PoUWTask, difficulty: u32) -> Solution {
    let start_time = std::time::Instant::now();

    // Execute the useful work (model training). This returns a hash of the
    // trained model parameters and the achieved accuracy.
    let (model_hash, accuracy) = perform_useful_work(task);

    let task_commitment = verifier::create_task_commitment(task);

    // This is the "mining" part: iterate on a nonce until the resulting
    // hash meets the difficulty target.
    for nonce in 0u64.. {
        let mut hasher = Sha256::new();
        hasher.update(&model_hash);
        hasher.update(&task_commitment);
        hasher.update(&nonce.to_le_bytes());
        let hash: [u8; 32] = hasher.finalize().into();

        if verifier::meets_difficulty(&hash, difficulty) {
            let computation_time_ms = start_time.elapsed().as_millis() as u64;
            return Solution {
                trained_model_hash: hex::encode(model_hash),
                accuracy,
                nonce,
                computation_time_ms,
            };
        }
    }

    unreachable!("A solution should always be found");
}

/// A placeholder for the actual "useful work" (e.g., ML model training).
/// The result of this work is then used in the hashing process.
fn perform_useful_work(task: &PoUWTask) -> ([u8; 32], u32) {
    // Generate a deterministic synthetic dataset based on the task parameters.
    let seed = {
        let mut h = Sha256::new();
        h.update(task.model_id.as_bytes());
        h.update(task.dataset_id.as_bytes());
        h.update(task.challenge);
        h.finalize()
    };
    let mut rng = StdRng::from_seed(seed.into());

    let samples = 100;
    let features = 2;
    let mut data = Array2::<f32>::zeros((samples, features));
    let mut labels = Array1::<f32>::zeros(samples);

    // Create a simple linearly separable dataset.
    for i in 0..samples {
        let x = rng.gen_range(-1.0..1.0);
        let y = rng.gen_range(-1.0..1.0);
        data[[i, 0]] = x;
        data[[i, 1]] = y;
        labels[i] = if x + y > 0.0 { 1.0 } else { 0.0 };
    }

    // Train logistic regression via gradient descent.
    let mut weights = Array1::<f32>::zeros(features);
    let lr = 0.5;
    for _ in 0..task.epochs {
        let logits = data.dot(&weights);
        let preds = logits.map(|z| 1.0 / (1.0 + (-z).exp()));
        let gradient = data.t().dot(&(preds.clone() - &labels)) / samples as f32;
        weights -= &(gradient * lr);
    }

    // Compute accuracy on the training data.
    let final_logits = data.dot(&weights);
    let pred_labels = final_logits.map(|z| if *z > 0.0 { 1.0 } else { 0.0 });
    let mut correct = 0u32;
    for (pred, target) in pred_labels.iter().zip(labels.iter()) {
        if (pred - target).abs() < f32::EPSILON { correct += 1; }
    }
    let accuracy = ((correct as f32 / samples as f32) * 10000.0) as u32;

    // Hash the weights to produce the model commitment.
    let mut hasher = Sha256::new();
    for w in weights.iter() { hasher.update(w.to_le_bytes()); }
    (hasher.finalize().into(), accuracy)
}
