use crate::pouw::{verify_with_solution, Solution, Task};
use serde::{Deserialize, Serialize};

/// Evaluator node responsible for verifying PoUW solutions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evaluator {
    pub node_id: String,
    pub evaluation_history: Vec<EvaluationRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationRecord {
    pub job_id: u64,
    pub accuracy: f64,
    pub timestamp: u64,
    pub is_valid: bool,
}

impl Evaluator {
    /// Create a new evaluator with the given name.
    pub fn new(node_id: &str) -> Self {
        Self {
            node_id: node_id.to_string(),
            evaluation_history: Vec::new(),
        }
    }

    /// Verify a trainer's solution for the given task and difficulty.
    pub fn evaluate(&self, task: &Task, solution: &Solution, difficulty: u32) -> bool {
        verify_with_solution(task, solution, difficulty)
    }

    pub fn evaluate_job(&mut self, job_id: u64, accuracy_claim: f64) -> bool {
        // Simplified evaluation logic
        let is_valid = accuracy_claim > 0.5; // Basic threshold
        
        let record = EvaluationRecord {
            job_id,
            accuracy: accuracy_claim,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            is_valid,
        };
        
        self.evaluation_history.push(record);
        is_valid
    }
}
