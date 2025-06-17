use crate::blockchain::{
    block::Block, chain::BlockchainError, transaction::Transaction,
};
use crate::pouw::{PoUWTask, PoUWSolution};
use std::collections::HashMap;
use crate::state::State;

/// Validates the structural integrity of a new block against the previous one.
pub fn validate_block_structure(
    block: &Block,
    prev_block: &Block,
) -> Result<(), BlockchainError> {
    if block.index as usize != prev_block.index as usize + 1 {
        return Err(BlockchainError::BlockValidationError(format!(
            "Invalid block index. Expected {}, got {}",
            prev_block.index + 1,
            block.index
        )));
    }
    if block.prev_hash != prev_block.hash {
        return Err(BlockchainError::BlockValidationError(
            "Invalid previous hash".to_string(),
        ));
    }
    Ok(())
}

/// Verifies that the PoW solution is valid for the given task.
pub fn validate_pow_solution(
    task: &PoUWTask,
    solution: &PoUWSolution,
) -> Result<(), BlockchainError> {
    if !task.verify(solution) {
        return Err(BlockchainError::BlockValidationError(
            "Invalid PoW solution".to_string(),
        ));
    }
    Ok(())
}

/// Validates a transaction against a given state (balances and nonces).
pub fn validate_transaction_with_state(
    tx: &Transaction,
    balances: &HashMap<String, u64>,
    nonces: &HashMap<String, u64>,
) -> Result<(), BlockchainError> {
    if !tx.verify_signature() {
        return Err(BlockchainError::InvalidSignature);
    }

    let expected_nonce = *nonces.get(&tx.signer).unwrap_or(&0);
    if tx.nonce != expected_nonce {
        return Err(BlockchainError::InvalidNonce {
            expected: expected_nonce,
            got: tx.nonce,
        });
    }

    let sender_balance = *balances.get(&tx.signer).unwrap_or(&0);
    let total_cost = tx.amount.saturating_add(tx.fee);
    if sender_balance < total_cost {
        return Err(BlockchainError::InsufficientFunds {
            required: total_cost,
            available: sender_balance,
        });
    }

    Ok(())
}

/// Applies a transaction to a given state, updating balances and nonces. Assumes prior validation.
pub fn apply_transaction_to_state(
    tx: &Transaction,
    balances: &mut HashMap<String, u64>,
    nonces: &mut HashMap<String, u64>,
) -> Result<(), BlockchainError> {
    let total_cost = tx.amount.saturating_add(tx.fee);

    let sender_balance = balances.entry(tx.signer.clone()).or_insert(0);
    *sender_balance = sender_balance.saturating_sub(total_cost);

    let recipient_balance = balances.entry(tx.recipient.clone()).or_insert(0);
    *recipient_balance = recipient_balance.saturating_add(tx.amount);

    let sender_nonce = nonces.entry(tx.signer.clone()).or_insert(0);
    *sender_nonce += 1;

    Ok(())
}

/// Validates a transaction's internal consistency (e.g., signature).
/// This is a stateless validation.
pub fn validate_transaction_stateless(tx: &Transaction) -> Result<(), BlockchainError> {
    if !tx.verify_signature() {
        return Err(BlockchainError::TransactionValidationError(
            "Invalid signature".to_string(),
        ));
    }
    Ok(())
}

/// Validates a transaction against the current state (e.g., nonce, balance).
/// This is a stateful validation.
pub fn validate_transaction_stateful(
    tx: &Transaction,
    state: &State,
) -> Result<(), BlockchainError> {
    // Check nonce
    let expected_nonce = state.get_nonce(&tx.from);
    if tx.nonce != expected_nonce {
        return Err(BlockchainError::TransactionValidationError(format!(
            "Invalid nonce for {}. Expected {}, got {}",
            tx.from, expected_nonce, tx.nonce
        )));
    }

    // Check balance
    let balance = state.get_balance(&tx.from);
    let total_cost = tx
        .amount
        .checked_add(tx.fee)
        .ok_or_else(|| BlockchainError::TransactionValidationError("Balance overflow".to_string()))?;

    if balance < total_cost {
        return Err(BlockchainError::TransactionValidationError(format!(
            "Insufficient funds for {}. Have {}, need {}",
            tx.from, balance, total_cost
        )));
    }

    Ok(())
}

/// Validates a full block, including all its transactions.
pub fn validate_block(
    block: &Block,
    prev_block: &Block,
    state: &State,
) -> Result<(), BlockchainError> {
    // 1. Validate block header and structure
    if block.index != prev_block.index + 1 {
        return Err(BlockchainError::InvalidBlock(
            "Incorrect block index".to_string(),
        ));
    }
    if block.prev_hash != prev_block.hash {
        return Err(BlockchainError::InvalidBlock(
            "Previous block hash does not match".to_string(),
        ));
    }
    if block.calculate_hash() != block.hash {
        return Err(BlockchainError::InvalidBlock("Block hash is incorrect".to_string()));
    }

    // 2. Validate PoUW solution
    if !block.task.verify(&block.solution) {
        return Err(BlockchainError::InvalidBlock(
            "Invalid PoUW solution".to_string(),
        ));
    }

    // 3. Validate all transactions in the block against the given state
    let mut temp_state = state.clone();
    for tx in &block.transactions {
        validate_transaction_stateless(tx)?;
        validate_transaction_stateful(tx, &temp_state)?;
        temp_state.apply_transaction(tx)?; // Apply to temp state for subsequent tx validation
    }

    Ok(())
} 