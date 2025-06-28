use crate::pouw::types::{PoUWTask, PoUWSolution};

#[derive(Debug, Clone)]
pub struct Trainer {
    node_id: String,
}

impl Trainer {
    pub fn new(node_id: &str) -> Self { Self { node_id: node_id.to_string() } }

    /// Executes useful work and returns metrics & solution.
    ///
    /// This now records the time spent solving the PoUW task and exposes it in
    /// the returned metrics map so callers can track actual training duration.
    pub fn execute(&self, task: &PoUWTask) -> TrainingOutput {
        let start = std::time::Instant::now();

        // Perform the PoUW solving with a low difficulty for now.
        let solution = crate::pouw::solve(task, 1);

        let duration_ms = start.elapsed().as_millis() as f64;
        let mut metrics = std::collections::HashMap::new();
        metrics.insert("duration_ms".to_string(), duration_ms);

        TrainingOutput { metrics, solution }
    }
}

#[derive(Debug, Clone)]
pub struct TrainingOutput {
    pub metrics: std::collections::HashMap<String, f64>,
    pub solution: PoUWSolution,
} 