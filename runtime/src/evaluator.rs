use crate::pouw::{Solution, Task, verify};

/// Evaluator node responsible for verifying PoUW solutions.
#[derive(Debug, Clone)]
pub struct Evaluator {
    pub name: String,
}

impl Evaluator {
    /// Create a new evaluator with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Verify a trainer's solution for the given task and difficulty.
    pub fn evaluate(&self, task: &Task, solution: &Solution, difficulty: u32) -> bool {
        verify(task, solution, difficulty)
    }
}
