use crate::blockchain::{
    block::Block, chain::BlockchainError, transaction::Transaction,
};
use crate::pouw::{PoUWTask, PoUWSolution};
use std::collections::HashMap;

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