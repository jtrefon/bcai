use crate::pouw::{solve_with_difficulty, Solution, Task};
use serde::{Deserialize, Serialize};

/// Simple trainer node capable of solving Proof-of-Useful-Work tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trainer {
    pub node_id: String,
    pub training_history: Vec<TrainingRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingRecord {
    pub job_id: u64,
    pub task_type: String,
    pub completion_time: u64,
    pub accuracy: f64,
    pub timestamp: u64,
}

impl Trainer {
    /// Create a new trainer with the given node_id.
    pub fn new(node_id: &str) -> Self {
        Self { 
            node_id: node_id.to_string(),
            training_history: Vec::new(),
        }
    }

    /// Solve the provided PoUW task at the given difficulty.
    pub fn train(&self, task: &Task, difficulty: u32) -> Solution {
        solve_with_difficulty(task, difficulty)
    }
    
    /// Record a training job completion
    pub fn record_training(&mut self, job_id: u64, task_type: String, accuracy: f64) {
        let record = TrainingRecord {
            job_id,
            task_type,
            completion_time: 0, // TODO: measure actual time
            accuracy,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        self.training_history.push(record);
    }
}
