use crate::pouw::{Solution, Task, solve};

/// Simple trainer node capable of solving Proof-of-Useful-Work tasks.
#[derive(Debug, Clone)]
pub struct Trainer {
    pub name: String,
}

impl Trainer {
    /// Create a new trainer with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Solve the provided PoUW task at the given difficulty.
    pub fn train(&self, task: &Task, difficulty: u32) -> Solution {
        solve(task, difficulty)
    }
}
