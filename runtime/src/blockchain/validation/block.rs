use crate::blockchain::{block::Block, chain::BlockchainError, state::State};
use super::transaction::{validate_transaction_stateless, validate_transaction_stateful};

/// Validate the structural relation between a new block and its predecessor.
pub fn validate_block_structure(block: &Block, prev_block: &Block) -> Result<(), BlockchainError> {
    if block.index != prev_block.index + 1 {
        return Err(BlockchainError::BlockValidationError(format!(
            "Invalid block index. Expected {}, got {}",
            prev_block.index + 1,
            block.index
        )));
    }
    if block.prev_hash != prev_block.hash {
        return Err(BlockchainError::BlockValidationError("Invalid previous hash".into()));
    }
    Ok(())
}

/// Perform full validation of a block (header, PoUW, transactions).
pub fn validate_block(block: &Block, prev_block: &Block, state: &State) -> Result<(), BlockchainError> {
    // Header checks
    validate_block_structure(block, prev_block)?;
    if block.calculate_hash() != block.hash {
        return Err(BlockchainError::InvalidBlock("Block hash is incorrect".into()));
    }

    // PoUW verification
    if !block.task.verify(&block.solution) {
        return Err(BlockchainError::InvalidBlock("Invalid PoUW solution".into()));
    }

    // Transaction checks on a temp state copy
    let mut temp_state = state.clone();
    for tx in &block.transactions {
        validate_transaction_stateless(tx)?;
        validate_transaction_stateful(tx, &temp_state)?;
        temp_state.apply_transaction(tx)?;
    }

    Ok(())
} 