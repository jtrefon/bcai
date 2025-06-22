use crate::pouw::types::{PoUWTask, PoUWSolution};

#[derive(Debug, Clone)]
pub struct Trainer {
    node_id: String,
}

impl Trainer {
    pub fn new(node_id: &str) -> Self { Self { node_id: node_id.to_string() } }

    /// Executes useful work and returns dummy metrics & solution for now.
    pub fn execute(&self, task: &PoUWTask) -> TrainingOutput {
        // For compilation we just call solver with low difficulty.
        let solution = task.solve();
        TrainingOutput {
            metrics: std::collections::HashMap::new(),
            solution,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrainingOutput {
    pub metrics: std::collections::HashMap<String, f64>,
    pub solution: PoUWSolution,
} 