#[derive(Debug, Clone)]
pub struct Evaluator {
    node_id: String,
}

impl Evaluator {
    pub fn new(node_id: &str) -> Self {
        Self { node_id: node_id.to_string() }
    }

    pub fn evaluate(&self, _result: &crate::node::TrainingResult) -> bool {
        // Placeholder stub, always accept.
        true
    }
} 