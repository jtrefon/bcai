use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::{chain::BlockchainError, transaction::Transaction};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct State {
    /// Mapping from a public key (hex-encoded) to an account balance.
    pub balances: HashMap<String, u64>,
    /// Mapping from a public key (hex-encoded) to the next valid nonce.
    pub nonces: HashMap<String, u64>,
}

impl State {
    /// Creates a new, empty state.
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
            nonces: HashMap::new(),
        }
    }

    /// Applies a transaction to the state, updating balances and nonces.
    /// This function assumes the transaction has already been validated.
    pub fn apply_transaction(&mut self, tx: &Transaction) -> Result<(), BlockchainError> {
        let total_cost = tx.amount.checked_add(tx.fee).ok_or_else(|| {
            BlockchainError::TransactionValidationError("Balance overflow".to_string())
        })?;

        // Debit sender
        let sender_balance = self
            .balances
            .get_mut(&tx.from)
            .ok_or_else(|| BlockchainError::TransactionValidationError("Sender not found".to_string()))?;

        if *sender_balance < total_cost {
            return Err(BlockchainError::TransactionValidationError(
                "Insufficient funds".to_string(),
            ));
        }
        *sender_balance -= total_cost;

        // Credit receiver
        let receiver_balance = self.balances.entry(tx.to.clone()).or_insert(0);
        *receiver_balance = receiver_balance.checked_add(tx.amount).ok_or_else(|| {
            BlockchainError::TransactionValidationError("Balance overflow on receiver".to_string())
        })?;

        // Update sender's nonce
        let nonce = self.nonces.entry(tx.from.clone()).or_insert(0);
        *nonce = tx.nonce; // The new nonce is the one from the applied transaction

        Ok(())
    }

    /// Gets the balance for a given public key.
    pub fn get_balance(&self, pubkey: &str) -> u64 {
        self.balances.get(pubkey).cloned().unwrap_or(0)
    }

    /// Gets the current nonce for a given public key.
    pub fn get_nonce(&self, pubkey: &str) -> u64 {
        self.nonces.get(pubkey).cloned().unwrap_or(0)
    }

    /// A special function to directly set a balance, used for genesis block creation.
    pub fn set_balance(&mut self, pubkey: &str, amount: u64) {
        self.balances.insert(pubkey.to_string(), amount);
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
} 