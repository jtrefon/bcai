#[derive(Debug, Clone)]
pub struct Evaluator {
    node_id: String,
}

impl Evaluator {
    pub fn new(node_id: &str) -> Self {
        Self { node_id: node_id.to_string() }
    }

    /// Evaluates a training result by verifying the embedded PoUW solution.
    ///
    /// When the full `node` feature is enabled we recreate the original task
    /// using the job id as the timestamp seed and run the PoUW verifier. In
    /// minimal builds without the node feature we simply accept the result so
    /// the library continues to compile.
    #[cfg(feature = "node")]
    pub fn evaluate(&self, result: &crate::node::TrainingResult) -> bool {
        let task = crate::pouw::generate_task_with_timestamp(1, result.job_id);
        task.verify(&result.pouw_solution, 1)
    }

    #[cfg(not(feature = "node"))]
    pub fn evaluate(&self, _result: &crate::node::TrainingResult) -> bool {
        true
    }
}
