use crate::blockchain::chain::BlockchainError;
use crate::pouw::{PoUWTask, PoUWSolution};

/// Verify that the provided PoUW solution satisfies the task.
pub fn validate_pow_solution(task: &PoUWTask, solution: &PoUWSolution) -> Result<(), BlockchainError> {
    if task.verify(solution) {
        Ok(())
    } else {
        Err(BlockchainError::BlockValidationError("Invalid PoW solution".into()))
    }
} 