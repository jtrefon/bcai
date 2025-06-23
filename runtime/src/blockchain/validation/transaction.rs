use crate::blockchain::{transaction::{Transaction, StorageTx}, chain::BlockchainError, state::State};
use std::collections::HashMap;

/// Stateless checks such as signature validity.
pub fn validate_transaction_stateless(tx: &Transaction) -> Result<(), BlockchainError> {
    if !tx.verify_signature() {
        return Err(BlockchainError::TransactionValidationError("Invalid signature".into()));
    }
    // Additional rule for UpdateMetrics – must come from oracle pub key.
    if let Some(StorageTx::UpdateMetrics { .. }) = &tx.storage {
        if tx.from != crate::blockchain::constants::METRICS_ORACLE_PUB {
            return Err(BlockchainError::TransactionValidationError("Unauthorised metrics submitter".into()));
        }
    }
    Ok(())
}

/// Validate nonce & balance against the current state.
pub fn validate_transaction_stateful(tx: &Transaction, state: &State) -> Result<(), BlockchainError> {
    let expected_nonce = state.get_nonce(&tx.from);
    if tx.nonce != expected_nonce {
        return Err(BlockchainError::TransactionValidationError(format!(
            "Invalid nonce for {}. Expected {}, got {}",
            tx.from, expected_nonce, tx.nonce
        )));
    }

    let balance = state.get_balance(&tx.from);

    // Determine cost depending on storage payload presence
    let total_cost: u128 = match &tx.storage {
        Some(StorageTx::StoreFile { price, .. }) => (*price as u128) + tx.fee as u128,
        Some(StorageTx::RewardHolding { .. }) => tx.fee as u128, // node only pays fee
        Some(StorageTx::UpdateMetrics { .. }) => 0u128, // admin tx no cost
        None => (tx.amount as u128) + tx.fee as u128,
    };

    let total_cost_u64 = total_cost as u64;

    if balance < total_cost_u64 {
        return Err(BlockchainError::TransactionValidationError(format!(
            "Insufficient funds for {}. Have {}, need {}",
            tx.from, balance, total_cost_u64
        )));
    }
    Ok(())
}

/// Combined validation using raw balance/nonce maps (used in mining simulation).
pub fn validate_transaction_with_state(
    tx: &Transaction,
    balances: &HashMap<String, u64>,
    nonces: &HashMap<String, u64>,
) -> Result<(), BlockchainError> {
    if !tx.verify_signature() {
        return Err(BlockchainError::InvalidSignature);
    }

    let expected_nonce = *nonces.get(&tx.from).unwrap_or(&0);
    if tx.nonce != expected_nonce {
        return Err(BlockchainError::InvalidNonce { expected: expected_nonce, got: tx.nonce });
    }
    let sender_balance = *balances.get(&tx.from).unwrap_or(&0);
    let total_cost = tx.amount.saturating_add(tx.fee);
    if sender_balance < total_cost {
        return Err(BlockchainError::InsufficientFunds { required: total_cost, available: sender_balance });
    }
    Ok(())
}

/// Apply a validated transaction to mutable balance/nonce maps.
pub fn apply_transaction_to_state(
    tx: &Transaction,
    balances: &mut HashMap<String, u64>,
    nonces: &mut HashMap<String, u64>,
) -> Result<(), BlockchainError> {
    let total_cost = tx.amount.saturating_add(tx.fee);
    let sender_balance = balances.entry(tx.from.clone()).or_insert(0);
    *sender_balance = sender_balance.saturating_sub(total_cost);

    let recipient_balance = balances.entry(tx.to.clone()).or_insert(0);
    *recipient_balance = recipient_balance.saturating_add(tx.amount);

    let sender_nonce = nonces.entry(tx.from.clone()).or_insert(0);
    *sender_nonce += 1;
    Ok(())
} 